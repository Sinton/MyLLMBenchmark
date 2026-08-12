use crate::benchmark::engines::real::{
    build_text_generation_request_body, classify_model, RealProviderClient, RealProviderProtocol,
};
use crate::domain::workload::WorkloadConfig;
use crate::endpoint_probe::{spawn_endpoint_probe_batch, EndpointProbeExecution};
use crate::error::{AppError, AppResult};
use crate::models::{
    CreateProviderInput, DeleteResult, EndpointProbeBatchDetail, EndpointProbeBatchRecord,
    EndpointProbeBatchSummary, EndpointProbeHistoryPage, EndpointProbeHistoryPageInput,
    EndpointProbeModelOption, EndpointProbeModelScanInput, EndpointProbeModelScanResult,
    EndpointProbePromotionInput, EndpointProbePromotionResult, EndpointProbeRunDetail,
    EndpointProbeRunRecord, EndpointProbeRunSummary, EndpointProbeStartInput,
    EndpointProbeStopResult, EndpointProbeTargetInput, ProviderConnectionConfig,
};
use crate::state::AppState;
use chrono::Utc;
use reqwest::Url;
use std::collections::HashSet;
use tauri::AppHandle;
use tokio::sync::watch;
use uuid::Uuid;

pub async fn start_endpoint_probe(
    app: AppHandle,
    state: &AppState,
    input: EndpointProbeStartInput,
) -> AppResult<EndpointProbeBatchSummary> {
    let prepared = prepare_batch(state, input).await?;
    let batch = state
        .create_endpoint_probe_batch(prepared.batch, prepared.records)
        .await?;
    let (tx, rx) = watch::channel(false);
    state
        .register_endpoint_probe_batch(batch.id.clone(), tx)
        .await;
    spawn_endpoint_probe_batch(app, state.clone(), batch.clone(), prepared.executions, rx);
    Ok(batch)
}

pub async fn stop_endpoint_probe(
    state: &AppState,
    batch_id: &str,
) -> AppResult<EndpointProbeStopResult> {
    Ok(EndpointProbeStopResult {
        batch_id: batch_id.to_string(),
        stopped: state.stop_endpoint_probe_batch(batch_id).await,
    })
}

pub async fn scan_endpoint_probe_models(
    state: &AppState,
    input: EndpointProbeModelScanInput,
) -> AppResult<EndpointProbeModelScanResult> {
    let (provider_id, config) = match input {
        EndpointProbeModelScanInput::Provider { provider_id } => {
            let config = state.provider_connection_config(&provider_id).await?;
            ensure_supported_protocol(&config.interface_type)?;
            (Some(provider_id), config)
        }
        EndpointProbeModelScanInput::Temporary {
            base_url,
            api_key,
            interface_type,
        } => {
            let base_url = normalize_base_url(&base_url)?;
            let (interface_type, _) = normalize_interface_type(&interface_type)?;
            (
                None,
                ProviderConnectionConfig {
                    id: "temporary-endpoint-model-scan".to_string(),
                    name: "Temporary endpoint".to_string(),
                    base_url,
                    api_key_plaintext: api_key.unwrap_or_default(),
                    interface_type,
                },
            )
        }
    };

    let client = RealProviderClient::new()?;
    let mut discovered = client
        .list_models(&config)
        .await
        .map_err(|error| AppError::Unexpected(anyhow::anyhow!("获取模型列表失败：{error}")))?;
    discovered.sort_by_cached_key(|model| model.name.to_ascii_lowercase());
    discovered.dedup_by(|left, right| left.name.eq_ignore_ascii_case(&right.name));
    let scanned_at = Utc::now().to_rfc3339();
    if let Some(provider_id) = provider_id.as_deref() {
        state
            .replace_provider_models(provider_id, discovered.clone(), &scanned_at)
            .await?;
        state
            .update_provider_connection_status(provider_id, "online", &scanned_at)
            .await?;
    }
    let models = discovered
        .into_iter()
        .map(EndpointProbeModelOption::from)
        .collect::<Vec<_>>();
    Ok(EndpointProbeModelScanResult {
        provider_id,
        message: if models.is_empty() {
            "模型接口已响应，但没有返回可用模型；可以手动填写模型名称。".to_string()
        } else {
            format!("已从 /models 获取到 {} 个模型。", models.len())
        },
        models,
        scanned_at,
    })
}

pub async fn promote_endpoint_probe_target(
    state: &AppState,
    input: EndpointProbePromotionInput,
) -> AppResult<EndpointProbePromotionResult> {
    let run = state.get_endpoint_probe_run_detail(&input.run_id).await?;
    if run.summary.source_type != "temporary" {
        return Err(AppError::validation(
            "只有临时站点测活记录可以保存为服务商。",
        ));
    }
    if let Some(provider) = state
        .find_provider_by_endpoint(&run.summary.base_url, &run.summary.interface_type)
        .await?
    {
        return Ok(EndpointProbePromotionResult {
            status: "already_exists".to_string(),
            provider,
            models_synced: false,
            warning: Some("相同 Base URL 和接口类型的服务商已经存在，未覆盖原配置。".to_string()),
        });
    }

    let provider = state
        .create_provider(CreateProviderInput {
            name: input
                .name
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| run.summary.name.clone()),
            base_url: run.summary.base_url.clone(),
            api_key: input.api_key,
            interface_type: run.summary.interface_type.clone(),
        })
        .await?;
    if !input.sync_models {
        return Ok(EndpointProbePromotionResult {
            status: "created".to_string(),
            provider,
            models_synced: false,
            warning: None,
        });
    }

    let config = state.provider_connection_config(&provider.id).await?;
    let client = RealProviderClient::new()?;
    let scanned_at = Utc::now().to_rfc3339();
    match client.list_models(&config).await {
        Ok(models) => {
            state
                .replace_provider_models(&provider.id, models, &scanned_at)
                .await?;
            state
                .update_provider_connection_status(&provider.id, "online", &scanned_at)
                .await?;
            let provider = find_provider(state, &provider.id).await?;
            Ok(EndpointProbePromotionResult {
                status: "created".to_string(),
                provider,
                models_synced: true,
                warning: None,
            })
        }
        Err(error) if run.summary.status == "passed" => {
            state
                .replace_provider_models(
                    &provider.id,
                    vec![classify_model(&run.summary.model)],
                    &scanned_at,
                )
                .await?;
            state
                .update_provider_connection_status(&provider.id, "online", &scanned_at)
                .await?;
            let provider = find_provider(state, &provider.id).await?;
            Ok(EndpointProbePromotionResult {
                status: "created".to_string(),
                provider,
                models_synced: false,
                warning: Some(format!(
                    "/models 同步失败，已保存本次验证通过的模型 {}：{error}",
                    run.summary.model
                )),
            })
        }
        Err(error) => Ok(EndpointProbePromotionResult {
            status: "created".to_string(),
            provider,
            models_synced: false,
            warning: Some(format!("服务商已保存，但 /models 同步失败：{error}")),
        }),
    }
}

pub async fn list_endpoint_probe_batches_page(
    state: &AppState,
    input: EndpointProbeHistoryPageInput,
) -> AppResult<EndpointProbeHistoryPage> {
    Ok(state.list_endpoint_probe_batches_page(input).await?)
}

pub async fn get_endpoint_probe_batch_detail(
    state: &AppState,
    batch_id: &str,
) -> AppResult<EndpointProbeBatchDetail> {
    Ok(state.get_endpoint_probe_batch_detail(batch_id).await?)
}

pub async fn get_endpoint_probe_run_detail(
    state: &AppState,
    run_id: &str,
) -> AppResult<EndpointProbeRunDetail> {
    Ok(state.get_endpoint_probe_run_detail(run_id).await?)
}

pub async fn delete_endpoint_probe_batch(
    state: &AppState,
    batch_id: &str,
) -> AppResult<DeleteResult> {
    if state
        .running_endpoint_probe_batch_ids()
        .await
        .iter()
        .any(|id| id == batch_id)
    {
        return Err(AppError::validation("运行中的测活批次不能删除，请先停止。"));
    }
    Ok(state.delete_endpoint_probe_batch(batch_id).await?)
}

struct PreparedBatch {
    batch: EndpointProbeBatchRecord,
    records: Vec<EndpointProbeRunRecord>,
    executions: Vec<EndpointProbeExecution>,
}

async fn prepare_batch(
    state: &AppState,
    input: EndpointProbeStartInput,
) -> AppResult<PreparedBatch> {
    if input.targets.is_empty() {
        return Err(AppError::validation("请至少选择一个测活目标。"));
    }
    let prompt = input.prompt.trim().to_string();
    if prompt.is_empty() {
        return Err(AppError::validation("请填写自定义测试 Prompt。"));
    }
    let concurrency = input.concurrency.unwrap_or(3);
    if !(1..=10).contains(&concurrency) {
        return Err(AppError::validation("批量并发必须在 1 到 10 之间。"));
    }
    let max_output_tokens = input.max_output_tokens.unwrap_or(1024).clamp(1, 8192);
    let timeout_seconds = input.timeout_seconds.unwrap_or(60).clamp(5, 600);
    let mut workload = WorkloadConfig::for_model_type("text_generation");
    workload.streaming = input.streaming;
    workload.max_output_tokens = max_output_tokens;
    let batch_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    let prompt_preview = Some(preview_text(&prompt));
    let mut records = Vec::new();
    let mut executions = Vec::new();
    let mut identities = HashSet::new();
    let mut station_ids = HashSet::new();

    for target in input.targets {
        let prepared = prepare_target(state, target).await?;
        station_ids.insert(prepared.identity.clone());
        for model in prepared.models {
            let identity = format!("{}\n{}", prepared.identity, model.to_ascii_lowercase());
            if !identities.insert(identity) {
                continue;
            }
            if executions.len() >= 200 {
                return Err(AppError::validation(
                    "单个测活批次最多包含 200 个“服务商 + 模型”项目。",
                ));
            }
            let id = Uuid::new_v4().to_string();
            let summary = EndpointProbeRunSummary {
                id: id.clone(),
                batch_id: batch_id.clone(),
                source_type: prepared.source_type.clone(),
                provider_id: prepared.provider_id.clone(),
                name: prepared.config.name.clone(),
                base_url: prepared.config.base_url.clone(),
                interface_type: prepared.config.interface_type.clone(),
                model: model.clone(),
                status: "pending".to_string(),
                latency_ms: 0,
                ttft_ms: 0,
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 0,
                error_kind: None,
                error_message: None,
                prompt_preview: prompt_preview.clone(),
                response_preview: None,
                body_available: false,
                created_at: created_at.clone(),
                finished_at: None,
            };
            let request_payload =
                build_text_generation_request_body(prepared.protocol, &model, &prompt, &workload);
            records.push(EndpointProbeRunRecord {
                summary: summary.clone(),
                body_ref: None,
                prompt: None,
                response_text: None,
                request_payload: None,
                raw_error: None,
                raw_usage: None,
            });
            executions.push(EndpointProbeExecution {
                summary,
                config: prepared.config.clone(),
                protocol: prepared.protocol,
                prompt: prompt.clone(),
                workload: workload.clone(),
                timeout_seconds,
                save_body: input.save_body,
                request_payload,
            });
        }
    }
    if executions.is_empty() {
        return Err(AppError::validation("请至少为一个站点选择一个模型。"));
    }

    let name = if executions.len() == 1 {
        format!(
            "{} / {}",
            executions[0].summary.name, executions[0].summary.model
        )
    } else {
        format!(
            "批量测活 · {} 个站点 / {} 个模型",
            station_ids.len(),
            executions.len()
        )
    };
    let total_runs = executions.len() as i64;
    let summary = EndpointProbeBatchSummary {
        id: batch_id,
        name,
        status: "running".to_string(),
        total_runs,
        pending_runs: total_runs,
        running_runs: 0,
        passed_runs: 0,
        failed_runs: 0,
        cancelled_runs: 0,
        streaming: input.streaming,
        max_output_tokens,
        timeout_seconds,
        save_body: input.save_body,
        concurrency,
        prompt_preview,
        created_at,
        finished_at: None,
    };
    Ok(PreparedBatch {
        batch: EndpointProbeBatchRecord { summary },
        records,
        executions,
    })
}

struct PreparedTarget {
    source_type: String,
    provider_id: Option<String>,
    identity: String,
    config: ProviderConnectionConfig,
    protocol: RealProviderProtocol,
    models: Vec<String>,
}

async fn prepare_target(
    state: &AppState,
    target: EndpointProbeTargetInput,
) -> AppResult<PreparedTarget> {
    match target {
        EndpointProbeTargetInput::Provider {
            provider_id,
            models,
        } => {
            let config = state.provider_connection_config(&provider_id).await?;
            let protocol = ensure_supported_protocol(&config.interface_type)?;
            Ok(PreparedTarget {
                source_type: "provider".to_string(),
                identity: provider_id.clone(),
                provider_id: Some(provider_id),
                config,
                protocol,
                models: normalize_models(models)?,
            })
        }
        EndpointProbeTargetInput::Temporary {
            name,
            base_url,
            api_key,
            interface_type,
            models,
        } => {
            let base_url = normalize_base_url(&base_url)?;
            let parsed = Url::parse(&base_url).map_err(|_| {
                AppError::validation("Base URL 必须是有效的 http:// 或 https:// 地址。")
            })?;
            let (interface_type, protocol) = normalize_interface_type(&interface_type)?;
            let name = name
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| {
                    parsed
                        .host_str()
                        .unwrap_or("Temporary endpoint")
                        .to_string()
                });
            let identity = format!("{}\n{}", base_url.to_ascii_lowercase(), interface_type);
            Ok(PreparedTarget {
                source_type: "temporary".to_string(),
                provider_id: None,
                identity,
                config: ProviderConnectionConfig {
                    id: Uuid::new_v4().to_string(),
                    name,
                    base_url,
                    api_key_plaintext: api_key.unwrap_or_default(),
                    interface_type,
                },
                protocol,
                models: normalize_models(models)?,
            })
        }
    }
}

fn normalize_models(models: Vec<String>) -> AppResult<Vec<String>> {
    let mut normalized = models
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    normalized.sort_by_cached_key(|value| value.to_ascii_lowercase());
    normalized.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    if normalized.is_empty() {
        return Err(AppError::validation("每个测活目标至少需要选择一个模型。"));
    }
    Ok(normalized)
}

fn ensure_supported_protocol(interface_type: &str) -> AppResult<RealProviderProtocol> {
    normalize_interface_type(interface_type).map(|(_, protocol)| protocol)
}

fn normalize_interface_type(value: &str) -> AppResult<(String, RealProviderProtocol)> {
    match value.trim() {
        "OpenAI" | "OpenAI Compatible" | "" => Ok((
            "OpenAI".to_string(),
            RealProviderProtocol::OpenAICompatible,
        )),
        "OpenAI-Response" | "OpenAI Responses" => Ok((
            "OpenAI-Response".to_string(),
            RealProviderProtocol::OpenAIResponses,
        )),
        "Anthropic" | "Claude" | "Claude Messages" => Ok((
            "Anthropic".to_string(),
            RealProviderProtocol::Anthropic,
        )),
        other => Err(AppError::validation(format!(
            "站点测活仅支持 OpenAI Chat Completions、OpenAI Responses 和 Anthropic Messages，当前接口类型 {other} 暂不支持。"
        ))),
    }
}

fn normalize_base_url(value: &str) -> AppResult<String> {
    let mut parsed = Url::parse(value.trim())
        .map_err(|_| AppError::validation("Base URL 必须是有效的 http:// 或 https:// 地址。"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(AppError::validation(
            "Base URL 只支持 http:// 或 https://。",
        ));
    }
    if parsed.query().is_some() || parsed.fragment().is_some() {
        return Err(AppError::validation(
            "Base URL 不能包含 query 或 fragment。",
        ));
    }
    parsed.set_query(None);
    parsed.set_fragment(None);
    Ok(parsed.to_string().trim_end_matches('/').to_string())
}

async fn find_provider(
    state: &AppState,
    provider_id: &str,
) -> AppResult<crate::models::ProviderSummary> {
    state
        .list_providers()
        .await?
        .into_iter()
        .find(|provider| provider.id == provider_id)
        .ok_or_else(|| AppError::not_found("provider"))
}

fn preview_text(value: &str) -> String {
    const MAX_CHARS: usize = 120;
    let mut preview = value.chars().take(MAX_CHARS).collect::<String>();
    if value.chars().count() > MAX_CHARS {
        preview.push_str("...");
    }
    preview
}

#[cfg(test)]
mod tests {
    use super::promote_endpoint_probe_target;
    use crate::models::{
        EndpointProbeBatchRecord, EndpointProbeBatchSummary, EndpointProbePromotionInput,
        EndpointProbeRunRecord, EndpointProbeRunSummary,
    };
    use crate::state::AppState;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;
    use uuid::Uuid;

    #[tokio::test]
    async fn temporary_probe_can_be_promoted_without_leaking_or_overwriting_key() {
        let root = std::env::temp_dir().join(format!(
            "my-llm-benchmark-probe-promotion-{}",
            Uuid::new_v4()
        ));
        let state = AppState::initialize(root.join("config"), root.join("data"))
            .await
            .unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 4096];
            let _ = stream.read(&mut request).unwrap();
            let body = r#"{"error":"models unavailable"}"#;
            let response = format!(
                "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        });
        let batch_id = Uuid::new_v4().to_string();
        let run_id = Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();
        let run = EndpointProbeRunRecord {
            summary: EndpointProbeRunSummary {
                id: run_id.clone(),
                batch_id: batch_id.clone(),
                source_type: "temporary".to_string(),
                provider_id: None,
                name: "Temporary gateway".to_string(),
                base_url: format!("http://{address}/v1"),
                interface_type: "OpenAI".to_string(),
                model: "verified-model".to_string(),
                status: "passed".to_string(),
                latency_ms: 300,
                ttft_ms: 80,
                input_tokens: 8,
                output_tokens: 12,
                total_tokens: 20,
                error_kind: None,
                error_message: None,
                prompt_preview: Some("hello".to_string()),
                response_preview: Some("ok".to_string()),
                body_available: false,
                created_at: created_at.clone(),
                finished_at: Some(created_at.clone()),
            },
            body_ref: None,
            prompt: None,
            response_text: None,
            request_payload: None,
            raw_error: None,
            raw_usage: None,
        };
        state
            .create_endpoint_probe_batch(
                EndpointProbeBatchRecord {
                    summary: EndpointProbeBatchSummary {
                        id: batch_id,
                        name: "Temporary gateway / verified-model".to_string(),
                        status: "completed".to_string(),
                        total_runs: 1,
                        pending_runs: 0,
                        running_runs: 0,
                        passed_runs: 1,
                        failed_runs: 0,
                        cancelled_runs: 0,
                        streaming: true,
                        max_output_tokens: 1024,
                        timeout_seconds: 60,
                        save_body: false,
                        concurrency: 1,
                        prompt_preview: Some("hello".to_string()),
                        created_at: created_at.clone(),
                        finished_at: Some(created_at),
                    },
                },
                vec![run],
            )
            .await
            .unwrap();

        let secret = "sk-promotion-secret";
        let created = promote_endpoint_probe_target(
            &state,
            EndpointProbePromotionInput {
                run_id: run_id.clone(),
                name: Some("Saved gateway".to_string()),
                api_key: Some(secret.to_string()),
                sync_models: true,
            },
        )
        .await
        .unwrap();
        assert_eq!(created.status, "created");
        assert_eq!(created.provider.name, "Saved gateway");
        assert!(!created.models_synced);
        assert!(created
            .warning
            .as_deref()
            .unwrap()
            .contains("已保存本次验证通过的模型"));
        assert!(!serde_json::to_string(&created).unwrap().contains(secret));
        assert_eq!(
            state
                .list_provider_models(&created.provider.id)
                .await
                .unwrap()[0]
                .name,
            "verified-model"
        );
        server.join().unwrap();

        let duplicate = promote_endpoint_probe_target(
            &state,
            EndpointProbePromotionInput {
                run_id,
                name: Some("Should not overwrite".to_string()),
                api_key: Some("replacement-secret".to_string()),
                sync_models: false,
            },
        )
        .await
        .unwrap();
        assert_eq!(duplicate.status, "already_exists");
        assert_eq!(duplicate.provider.id, created.provider.id);
        assert_eq!(duplicate.provider.name, "Saved gateway");
        let config = state
            .provider_connection_config(&created.provider.id)
            .await
            .unwrap();
        assert_eq!(config.api_key_plaintext, secret);

        let _ = std::fs::remove_dir_all(root);
    }
}
