use crate::benchmark::engines::real::RealProviderProtocol;
use crate::benchmark::runner::{spawn_mock_benchmark, spawn_real_benchmark};
use crate::config::BenchmarkEngineMode;
use crate::domain::model_type::ModelType;
use crate::domain::workload::WorkloadConfig;
use crate::error::{AppError, AppResult};
use crate::models::{
    BenchmarkRequestLogDetail, BenchmarkRequestLogPage, BenchmarkRequestLogPageInput,
    BenchmarkStartInput, BenchmarkTaskSummary, DatasetSample, DatasetValidationResult,
    DeleteResult, MetricsTick, ModelSummary, ProviderConnectionConfig, ProviderDiagnosticsResult,
    StopResult,
};
use crate::state::AppState;
use tauri::AppHandle;
use tokio::sync::watch;

pub async fn start_benchmark(
    app: AppHandle,
    state: &AppState,
    input: BenchmarkStartInput,
) -> AppResult<BenchmarkTaskSummary> {
    let engine_mode = state.current_config().await?.benchmark_engine;
    let real_context = if engine_mode == BenchmarkEngineMode::OpenaiCompatible {
        Some(prepare_openai_compatible_context(state, &input).await?)
    } else {
        None
    };

    let task = state.create_task(&input).await?;
    if engine_mode == BenchmarkEngineMode::OpenaiCompatible {
        state
            .update_task_engine_mode(&task.id, "openai_compatible")
            .await?;
        if let Some(context) = real_context.as_ref() {
            state
                .update_task_preflight(
                    &task.id,
                    Some(context.preflight_result.clone()),
                    context.diagnostics_snapshot.clone(),
                )
                .await?;
        }
    }

    let (tx, rx) = watch::channel(false);
    state.register_task(task.id.clone(), tx).await;
    match engine_mode {
        BenchmarkEngineMode::Mock => {
            spawn_mock_benchmark(app, state.clone(), task.clone(), input, rx);
        }
        BenchmarkEngineMode::OpenaiCompatible => {
            let context = real_context.expect("real context is prepared");
            spawn_real_benchmark(
                app,
                state.clone(),
                task.clone(),
                input,
                context.provider,
                context.samples,
                rx,
            );
        }
    }
    Ok(task)
}

pub async fn stop_benchmark(state: &AppState, task_id: &str) -> AppResult<StopResult> {
    let stopped = state.stop_task(task_id).await;
    Ok(StopResult {
        task_id: task_id.to_string(),
        stopped,
    })
}

pub async fn get_benchmark_task(
    state: &AppState,
    task_id: &str,
) -> AppResult<BenchmarkTaskSummary> {
    Ok(state.get_task_summary(task_id).await?)
}

pub async fn list_benchmark_ticks(state: &AppState, task_id: &str) -> AppResult<Vec<MetricsTick>> {
    Ok(state.list_ticks(task_id).await?)
}

pub async fn list_benchmark_request_logs_page(
    state: &AppState,
    input: BenchmarkRequestLogPageInput,
) -> AppResult<BenchmarkRequestLogPage> {
    Ok(state.list_request_logs_page(input).await?)
}

pub async fn get_benchmark_request_log_detail(
    state: &AppState,
    request_id: &str,
) -> AppResult<BenchmarkRequestLogDetail> {
    Ok(state.get_request_log_detail(request_id).await?)
}

pub async fn delete_benchmark_request_logs(
    state: &AppState,
    task_id: &str,
) -> AppResult<DeleteResult> {
    Ok(state.delete_request_logs(task_id).await?)
}

async fn prepare_openai_compatible_context(
    state: &AppState,
    input: &BenchmarkStartInput,
) -> AppResult<RealBenchmarkContext> {
    let provider = state.provider_connection_config(&input.provider_id).await?;
    if provider.base_url.trim().is_empty() {
        return Err(AppError::validation("真实压测需要配置 Base URL。"));
    }
    if provider.api_key_plaintext.trim().is_empty() {
        return Err(AppError::validation("真实压测需要配置 API Key。"));
    }
    let selected_model = resolve_selected_model(state, input).await?;
    let normalized_type = ModelType::normalize(&selected_model.model_type);
    validate_provider_model_protocol(&provider, normalized_type)?;
    validate_dataset_type(state, input, normalized_type).await?;

    let samples = state.list_dataset_samples(&input.dataset_id).await?;
    if samples.is_empty() {
        return Err(AppError::validation(format!(
            "当前数据集没有可用样本，请先导入适用于 {} 的测试数据集。",
            normalized_type.as_str()
        )));
    }

    let workload =
        WorkloadConfig::from_value(normalized_type.as_str(), input.workload_config.as_ref());
    validate_workload_config(normalized_type, &workload)?;
    let dataset_quality = state.validate_dataset_samples(&input.dataset_id).await?;
    validate_dataset_quality(normalized_type, &dataset_quality, &workload, samples.len())?;
    let diagnostics_snapshot = state
        .get_provider_diagnostics(&input.provider_id)
        .await
        .ok()
        .flatten();
    let mut warnings = Vec::new();
    match diagnostics_snapshot
        .as_ref()
        .map(|result| result.status.as_str())
    {
        None => warnings.push("未找到最近兼容性诊断，建议先在服务商页执行诊断。".to_string()),
        Some("failed") | Some("unsupported") => {
            warnings.push("最近兼容性诊断未通过，本次启动可能失败。".to_string())
        }
        Some("warning") => warnings.push("最近兼容性诊断存在警告，请关注报告附录。".to_string()),
        _ => {}
    }

    Ok(RealBenchmarkContext {
        provider,
        samples,
        diagnostics_snapshot,
        preflight_result: serde_json::json!({
            "status": if warnings.is_empty() { "passed" } else { "warning" },
            "warnings": warnings,
            "model_type": normalized_type.as_str(),
            "dataset_quality": dataset_quality,
            "checked_at": chrono::Utc::now().to_rfc3339(),
        }),
    })
}

async fn resolve_selected_model(
    state: &AppState,
    input: &BenchmarkStartInput,
) -> AppResult<ModelSummary> {
    let models = state.list_provider_models(&input.provider_id).await?;
    let selected = input
        .model_id
        .as_ref()
        .filter(|id| !id.trim().is_empty())
        .and_then(|model_id| models.iter().find(|model| model.id == *model_id))
        .or_else(|| models.first());

    selected
        .cloned()
        .ok_or_else(|| AppError::validation("真实压测需要先连接服务商并扫描出一个可用模型。"))
}

struct RealBenchmarkContext {
    provider: ProviderConnectionConfig,
    samples: Vec<DatasetSample>,
    diagnostics_snapshot: Option<ProviderDiagnosticsResult>,
    preflight_result: serde_json::Value,
}

async fn validate_dataset_type(
    state: &AppState,
    input: &BenchmarkStartInput,
    model_type: ModelType,
) -> AppResult<()> {
    let dataset = state
        .list_datasets()
        .await?
        .into_iter()
        .find(|dataset| dataset.id == input.dataset_id)
        .ok_or_else(|| AppError::not_found("dataset"))?;
    let actual = ModelType::normalize(&dataset.dataset_type);
    if actual != model_type {
        return Err(AppError::validation(format!(
            "数据集类型与模型类型不匹配：当前模型是 {}，但数据集是 {}。",
            model_type.as_str(),
            dataset.dataset_type
        )));
    }
    Ok(())
}

fn validate_provider_model_protocol(
    provider: &ProviderConnectionConfig,
    model_type: ModelType,
) -> AppResult<()> {
    let protocol = RealProviderProtocol::from_interface_type(&provider.interface_type)
        .unwrap_or(RealProviderProtocol::OpenAICompatible);
    if provider.interface_type == "Jina Rerank" && model_type != ModelType::Rerank {
        return Err(AppError::validation(
            "Jina Rerank 服务商只能启动 Rerank 模型压测。",
        ));
    }
    if provider.interface_type != "Jina Rerank" && model_type == ModelType::Rerank {
        return Err(AppError::validation(
            "Rerank 真实压测当前仅支持 Jina Rerank 接口类型。",
        ));
    }
    if matches!(
        protocol,
        RealProviderProtocol::Anthropic | RealProviderProtocol::Gemini
    ) && matches!(model_type, ModelType::Embedding | ModelType::Rerank)
    {
        return Err(AppError::validation(format!(
            "{} 当前真实压测仅支持文本生成和 Vision 模型。",
            provider.interface_type
        )));
    }
    if protocol == RealProviderProtocol::OpenAIResponses && model_type == ModelType::Rerank {
        return Err(AppError::validation(
            "OpenAI Responses 真实压测不支持 Rerank，请选择 OpenAI Compatible / Jina Rerank。",
        ));
    }
    Ok(())
}

fn validate_workload_config(model_type: ModelType, workload: &WorkloadConfig) -> AppResult<()> {
    match model_type {
        ModelType::TextGeneration => {
            if !(1..=8192).contains(&workload.max_output_tokens) {
                return Err(AppError::validation(
                    "文本生成 max_output_tokens 必须在 1-8192 之间。",
                ));
            }
        }
        ModelType::Embedding => {
            if !(1..=512).contains(&workload.batch_size)
                || !(1..=512).contains(&workload.text_count_per_request)
            {
                return Err(AppError::validation(
                    "Embedding batch_size 和 text_count_per_request 必须在 1-512 之间。",
                ));
            }
        }
        ModelType::Rerank => {
            if !(1..=1000).contains(&workload.documents_per_query) {
                return Err(AppError::validation(
                    "Rerank documents_per_query 必须在 1-1000 之间。",
                ));
            }
            if workload.top_k < 1 || workload.top_k > workload.documents_per_query {
                return Err(AppError::validation(
                    "Rerank top_k 必须大于 0 且不能超过 documents_per_query。",
                ));
            }
        }
        ModelType::Multimodal => {
            if !(1..=8).contains(&workload.image_count) {
                return Err(AppError::validation("Vision image_count 必须在 1-8 之间。"));
            }
            if !(1..=8192).contains(&workload.max_output_tokens) {
                return Err(AppError::validation(
                    "Vision max_output_tokens 必须在 1-8192 之间。",
                ));
            }
        }
    }
    Ok(())
}

fn validate_dataset_quality(
    model_type: ModelType,
    quality: &DatasetValidationResult,
    workload: &WorkloadConfig,
    sample_count: usize,
) -> AppResult<()> {
    if quality
        .issues
        .iter()
        .any(|issue| issue.kind == "empty_prompt")
    {
        return Err(AppError::validation(
            "数据集包含空 Prompt，请清理后再启动真实压测。",
        ));
    }
    if model_type == ModelType::Multimodal
        && quality
            .issues
            .iter()
            .any(|issue| issue.kind == "vision_missing_image")
    {
        return Err(AppError::validation(
            "Vision 真实压测需要包含 image_url / image_urls / images 的图片样本。",
        ));
    }
    if model_type == ModelType::Rerank {
        let required = workload.documents_per_query.max(1) as usize + 1;
        if sample_count < required {
            return Err(AppError::validation(format!(
                "Rerank 数据集样本不足：当前 {} 条，至少需要 {} 条用于 query + documents。",
                sample_count, required
            )));
        }
    }
    Ok(())
}
