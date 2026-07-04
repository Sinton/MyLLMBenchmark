use crate::{
    config::BenchmarkEngineMode,
    domain::{model_type::ModelType, workload::WorkloadConfig},
    models::{
        DatasetSample, DiagnosticEndpoint, DiscoveredModel, ModelSummary, ProviderConnectionConfig,
        ProviderDiagnosticsInput, ProviderDiagnosticsResult,
    },
};
use reqwest::StatusCode;
use tokio::time::Instant;

use super::{
    api_url, chat, classify_model, duration_ms, embedding, map_reqwest_error, parse_vision_sample,
    request_timeout, rerank, vision, ModelsResponse, OpenAICompatibleClient,
};

impl OpenAICompatibleClient {
    pub async fn diagnose_provider(
        &self,
        config: &ProviderConnectionConfig,
        input: &ProviderDiagnosticsInput,
        stored_models: &[ModelSummary],
        samples: &[DatasetSample],
        engine_mode: BenchmarkEngineMode,
        checked_at: String,
    ) -> ProviderDiagnosticsResult {
        let engine_mode_label = match engine_mode {
            BenchmarkEngineMode::Mock => "mock",
            BenchmarkEngineMode::OpenaiCompatible => "openai_compatible",
        }
        .to_string();

        if is_unsupported_interface(&config.interface_type) {
            return unsupported_result(config, checked_at, engine_mode_label);
        }

        let mut endpoints = Vec::new();
        let (models_endpoint, discovered_models) = self.probe_models(config).await;
        endpoints.push(models_endpoint);

        let selected = select_model(input, stored_models, &discovered_models);
        if let Some((model_name, model_type)) = selected {
            let workload = WorkloadConfig::for_model_type(model_type.as_str());
            match model_type {
                ModelType::Embedding => {
                    endpoints.push(
                        self.probe_json_post(
                            config,
                            "Embedding 最小请求",
                            "embeddings",
                            embedding::embeddings_body(
                                &model_name,
                                sample_prompts(samples, 2, embedding::diagnostic_inputs()),
                            ),
                            30,
                        )
                        .await,
                    );
                }
                ModelType::Rerank => {
                    endpoints.push(
                        self.probe_json_post(
                            config,
                            "Rerank 最小请求",
                            "rerank",
                            rerank::rerank_body(
                                &model_name,
                                sample_prompt(samples).unwrap_or_else(rerank::diagnostic_query),
                                sample_prompts(samples, 3, rerank::diagnostic_documents()),
                                &workload,
                            ),
                            30,
                        )
                        .await,
                    );
                }
                ModelType::Multimodal => {
                    if let Some(sample) = samples
                        .iter()
                        .map(|sample| parse_vision_sample(&sample.prompt, workload.image_count))
                        .find(|sample| !sample.image_urls.is_empty())
                    {
                        endpoints.push(
                            self.probe_json_post(
                                config,
                                "Vision 最小请求",
                                "chat/completions",
                                vision::vision_completion_body(&model_name, &sample, &workload),
                                45,
                            )
                            .await,
                        );
                    } else {
                        endpoints.push(DiagnosticEndpoint {
                            name: "Vision 样本检查".to_string(),
                            method: "LOCAL".to_string(),
                            path: "dataset_samples".to_string(),
                            ok: false,
                            latency_ms: None,
                            http_status: None,
                            message:
                                "当前数据集没有可用图片 URL；Vision 诊断需要 image_url / image_urls / images 字段。"
                                    .to_string(),
                            error_kind: Some("missing_image_sample".to_string()),
                        });
                    }
                }
                ModelType::TextGeneration => {
                    endpoints.push(
                        self.probe_json_post(
                            config,
                            "Chat 最小请求",
                            "chat/completions",
                            chat::completion_body(
                                &model_name,
                                sample_prompt(samples)
                                    .unwrap_or_else(|| chat::diagnostic_prompt().to_string())
                                    .as_str(),
                                &workload,
                                false,
                                0.2,
                            ),
                            45,
                        )
                        .await,
                    );
                }
            }
        } else {
            endpoints.push(DiagnosticEndpoint {
                name: "模型选择".to_string(),
                method: "LOCAL".to_string(),
                path: "models".to_string(),
                ok: false,
                latency_ms: None,
                http_status: None,
                message: "没有可用于最小请求探测的模型，请先测试连接并扫描模型。".to_string(),
                error_kind: Some("missing_model".to_string()),
            });
        }

        let status = summarize_status(&endpoints);
        let recommendations = build_recommendations(&status, &engine_mode_label, &endpoints);
        ProviderDiagnosticsResult {
            provider_id: config.id.clone(),
            status,
            checked_at,
            engine_mode: engine_mode_label,
            endpoints,
            recommendations,
        }
    }

    async fn probe_models(
        &self,
        config: &ProviderConnectionConfig,
    ) -> (DiagnosticEndpoint, Vec<DiscoveredModel>) {
        let started = Instant::now();
        let response = match self
            .with_auth(self.client.get(api_url(&config.base_url, "models")), config)
            .timeout(request_timeout(20))
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                let mapped = map_reqwest_error(error);
                return (
                    DiagnosticEndpoint {
                        name: "模型列表".to_string(),
                        method: "GET".to_string(),
                        path: "/models".to_string(),
                        ok: false,
                        latency_ms: Some(duration_ms(started.elapsed())),
                        http_status: None,
                        message: mapped.to_string(),
                        error_kind: Some("request".to_string()),
                    },
                    Vec::new(),
                );
            }
        };
        let status = response.status();
        let latency_ms = Some(duration_ms(started.elapsed()));
        if !status.is_success() {
            return (
                endpoint_from_status("模型列表", "GET", "/models", status, latency_ms),
                Vec::new(),
            );
        }

        match response.json::<ModelsResponse>().await {
            Ok(payload) => {
                let models = payload
                    .data
                    .into_iter()
                    .map(|model| classify_model(&model.id))
                    .collect::<Vec<_>>();
                (
                    DiagnosticEndpoint {
                        name: "模型列表".to_string(),
                        method: "GET".to_string(),
                        path: "/models".to_string(),
                        ok: true,
                        latency_ms,
                        http_status: Some(status.as_u16() as i64),
                        message: format!("读取成功，发现 {} 个模型。", models.len()),
                        error_kind: None,
                    },
                    models,
                )
            }
            Err(error) => (
                DiagnosticEndpoint {
                    name: "模型列表".to_string(),
                    method: "GET".to_string(),
                    path: "/models".to_string(),
                    ok: false,
                    latency_ms,
                    http_status: Some(status.as_u16() as i64),
                    message: format!("响应解析失败：{error}"),
                    error_kind: Some("parse".to_string()),
                },
                Vec::new(),
            ),
        }
    }

    async fn probe_json_post(
        &self,
        config: &ProviderConnectionConfig,
        name: &str,
        path: &str,
        body: serde_json::Value,
        timeout_seconds: i64,
    ) -> DiagnosticEndpoint {
        let started = Instant::now();
        let response = match self
            .with_auth(self.client.post(api_url(&config.base_url, path)), config)
            .timeout(request_timeout(timeout_seconds))
            .json(&body)
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                let kind = if error.is_timeout() {
                    "timeout"
                } else if error.is_connect() {
                    "connection"
                } else {
                    "request"
                };
                return DiagnosticEndpoint {
                    name: name.to_string(),
                    method: "POST".to_string(),
                    path: format!("/{}", path.trim_start_matches('/')),
                    ok: false,
                    latency_ms: Some(duration_ms(started.elapsed())),
                    http_status: None,
                    message: error.to_string(),
                    error_kind: Some(kind.to_string()),
                };
            }
        };
        let status = response.status();
        let latency_ms = Some(duration_ms(started.elapsed()));
        if !status.is_success() {
            return endpoint_from_status(
                name,
                "POST",
                &format!("/{}", path.trim_start_matches('/')),
                status,
                latency_ms,
            );
        }

        match response.json::<serde_json::Value>().await {
            Ok(_) => DiagnosticEndpoint {
                name: name.to_string(),
                method: "POST".to_string(),
                path: format!("/{}", path.trim_start_matches('/')),
                ok: true,
                latency_ms,
                http_status: Some(status.as_u16() as i64),
                message: "最小请求成功，响应可解析。".to_string(),
                error_kind: None,
            },
            Err(error) => DiagnosticEndpoint {
                name: name.to_string(),
                method: "POST".to_string(),
                path: format!("/{}", path.trim_start_matches('/')),
                ok: false,
                latency_ms,
                http_status: Some(status.as_u16() as i64),
                message: format!("响应解析失败：{error}"),
                error_kind: Some("parse".to_string()),
            },
        }
    }
}

fn unsupported_result(
    config: &ProviderConnectionConfig,
    checked_at: String,
    engine_mode: String,
) -> ProviderDiagnosticsResult {
    ProviderDiagnosticsResult {
        provider_id: config.id.clone(),
        status: "unsupported".to_string(),
        checked_at,
        engine_mode,
        endpoints: vec![DiagnosticEndpoint {
            name: "真实引擎支持状态".to_string(),
            method: "LOCAL".to_string(),
            path: config.interface_type.clone(),
            ok: false,
            latency_ms: None,
            http_status: None,
            message: format!(
                "{} 当前版本未启用真实压测引擎；不会按 OpenAI 协议误发请求。",
                config.interface_type
            ),
            error_kind: Some("unsupported".to_string()),
        }],
        recommendations: vec![
            "当前版本真实压测优先支持 OpenAI Compatible 与 Jina Rerank。".to_string(),
            "该服务商可继续作为配置记录，真实引擎适配放在后续版本。".to_string(),
        ],
    }
}

fn is_unsupported_interface(interface_type: &str) -> bool {
    matches!(interface_type, "OpenAI-Response" | "Anthropic" | "Gemini")
}

fn select_model(
    input: &ProviderDiagnosticsInput,
    stored_models: &[ModelSummary],
    discovered_models: &[DiscoveredModel],
) -> Option<(String, ModelType)> {
    input
        .model_id
        .as_ref()
        .and_then(|model_id| stored_models.iter().find(|model| &model.id == model_id))
        .or_else(|| stored_models.first())
        .map(|model| (model.name.clone(), ModelType::normalize(&model.model_type)))
        .or_else(|| {
            discovered_models
                .first()
                .map(|model| (model.name.clone(), ModelType::normalize(&model.model_type)))
        })
}

fn sample_prompt(samples: &[DatasetSample]) -> Option<String> {
    samples
        .iter()
        .map(|sample| sample.prompt.trim())
        .find(|prompt| !prompt.is_empty())
        .map(ToString::to_string)
}

fn sample_prompts(samples: &[DatasetSample], count: usize, fallback: Vec<String>) -> Vec<String> {
    let prompts = samples
        .iter()
        .map(|sample| sample.prompt.trim())
        .filter(|prompt| !prompt.is_empty())
        .take(count)
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    if prompts.is_empty() {
        fallback
    } else {
        prompts
    }
}

fn endpoint_from_status(
    name: &str,
    method: &str,
    path: &str,
    status: StatusCode,
    latency_ms: Option<i64>,
) -> DiagnosticEndpoint {
    let kind = if status.is_client_error() {
        "http_4xx"
    } else if status.is_server_error() {
        "http_5xx"
    } else {
        "http"
    };
    DiagnosticEndpoint {
        name: name.to_string(),
        method: method.to_string(),
        path: path.to_string(),
        ok: false,
        latency_ms,
        http_status: Some(status.as_u16() as i64),
        message: format!("HTTP {status}；请检查 Base URL、API Key、模型权限和接口路径。"),
        error_kind: Some(kind.to_string()),
    }
}

fn summarize_status(endpoints: &[DiagnosticEndpoint]) -> String {
    if endpoints
        .iter()
        .any(|endpoint| endpoint.error_kind.as_deref() == Some("unsupported"))
    {
        return "unsupported".to_string();
    }
    if endpoints.iter().any(|endpoint| {
        !endpoint.ok && endpoint.error_kind.as_deref() != Some("missing_image_sample")
    }) {
        return "failed".to_string();
    }
    if endpoints.iter().any(|endpoint| !endpoint.ok) {
        return "warning".to_string();
    }
    "passed".to_string()
}

fn build_recommendations(
    status: &str,
    engine_mode: &str,
    endpoints: &[DiagnosticEndpoint],
) -> Vec<String> {
    let mut items = Vec::new();
    if engine_mode != "openai_compatible" {
        items.push(
            "当前压测引擎仍是 Mock；诊断只验证真实端点，启动真实压测需要在设置中切换 OpenAI Compatible 并重启。"
                .to_string(),
        );
    }
    if endpoints
        .iter()
        .any(|endpoint| endpoint.http_status == Some(401) || endpoint.http_status == Some(403))
    {
        items.push("鉴权失败，请检查 API Key、网关权限和模型访问范围。".to_string());
    }
    if endpoints
        .iter()
        .any(|endpoint| endpoint.http_status == Some(404))
    {
        items.push(
            "路径不存在，请确认 Base URL 是否填写到 /v1，或目标网关是否支持该 endpoint。"
                .to_string(),
        );
    }
    if endpoints
        .iter()
        .any(|endpoint| endpoint.error_kind.as_deref() == Some("missing_image_sample"))
    {
        items.push(
            "Vision 诊断需要带图片 URL 的数据集样本，建议使用 JSON 样本维护 image_url。"
                .to_string(),
        );
    }
    if status == "passed" {
        items.push("端点诊断通过，可以进入压测工作台进行小并发试跑。".to_string());
    }
    if items.is_empty() {
        items.push("请根据失败 endpoint 的 HTTP 状态和错误类型修复配置后重试诊断。".to_string());
    }
    items
}
