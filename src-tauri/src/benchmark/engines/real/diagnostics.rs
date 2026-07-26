use super::providers::{
    anthropic, embedding_openai as embedding, gemini, jina_rerank as rerank,
    openai_compatible as chat, openai_responses as responses, vision_openai as vision,
};
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

use super::client::RealProviderClient;
use super::helpers::{
    api_url, classify_model, duration_ms, parse_vision_sample, request_timeout, VisionSample,
};
use super::protocol::{map_reqwest_error, ModelsResponse, RealProviderProtocol};

impl RealProviderClient {
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

        let protocol = RealProviderProtocol::from_interface_type(&config.interface_type)
            .unwrap_or(RealProviderProtocol::OpenAICompatible);

        let mut endpoints = Vec::new();
        let (models_endpoint, discovered_models) = self.probe_models(config, protocol).await;
        endpoints.push(models_endpoint);

        let selected = select_model(input, stored_models, &discovered_models);
        if let Some((model_name, model_type)) = selected {
            let workload = WorkloadConfig::for_model_type(model_type.as_str());
            match model_type {
                ModelType::Embedding => {
                    if protocol == RealProviderProtocol::OpenAICompatible {
                        endpoints.push(
                            self.probe_json_post(
                                config,
                                protocol,
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
                    } else {
                        endpoints.push(unsupported_model_type_endpoint(
                            protocol,
                            "Embedding 最小请求",
                            model_type,
                        ));
                    }
                }
                ModelType::Rerank => {
                    if protocol == RealProviderProtocol::OpenAICompatible {
                        endpoints.push(
                            self.probe_json_post(
                                config,
                                protocol,
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
                    } else {
                        endpoints.push(unsupported_model_type_endpoint(
                            protocol,
                            "Rerank 最小请求",
                            model_type,
                        ));
                    }
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
                                protocol,
                                "Vision 最小请求",
                                vision_probe_path(protocol, &model_name),
                                vision_probe_body(protocol, &model_name, &sample, &workload),
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
                            protocol,
                            text_probe_name(protocol),
                            text_probe_path(protocol, &model_name),
                            text_probe_body(
                                protocol,
                                &model_name,
                                sample_prompt(samples).unwrap_or_else(|| {
                                    diagnostic_prompt_for_protocol(protocol).to_string()
                                }),
                                &workload,
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
        protocol: RealProviderProtocol,
    ) -> (DiagnosticEndpoint, Vec<DiscoveredModel>) {
        let started = Instant::now();
        let mut request = self.with_protocol_auth(
            self.client.get(api_url(&config.base_url, "models")),
            config,
            protocol,
        );
        if protocol == RealProviderProtocol::Anthropic {
            request = request.header("anthropic-version", "2023-06-01");
        }
        let response = match request.timeout(request_timeout(20)).send().await {
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

        if protocol == RealProviderProtocol::Gemini {
            return match response.json::<serde_json::Value>().await {
                Ok(payload) => {
                    let models = payload
                        .get("models")
                        .and_then(|value| value.as_array())
                        .into_iter()
                        .flatten()
                        .filter_map(|model| {
                            model
                                .get("name")
                                .and_then(|value| value.as_str())
                                .map(|name| name.trim_start_matches("models/").to_string())
                        })
                        .map(|name| classify_model(&name))
                        .collect::<Vec<_>>();
                    (
                        DiagnosticEndpoint {
                            name: "模型列表".to_string(),
                            method: "GET".to_string(),
                            path: "/models".to_string(),
                            ok: true,
                            latency_ms,
                            http_status: Some(status.as_u16() as i64),
                            message: format!("读取成功，发现 {} 个 Gemini 模型。", models.len()),
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
            };
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
        protocol: RealProviderProtocol,
        name: &str,
        path: impl AsRef<str>,
        body: serde_json::Value,
        timeout_seconds: i64,
    ) -> DiagnosticEndpoint {
        let started = Instant::now();
        let path = path.as_ref();
        let mut request = self.with_protocol_auth(
            self.client.post(api_url(&config.base_url, path)),
            config,
            protocol,
        );
        if protocol == RealProviderProtocol::Anthropic {
            request = request.header("anthropic-version", "2023-06-01");
        }
        let response = match request
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

fn text_probe_name(protocol: RealProviderProtocol) -> &'static str {
    match protocol {
        RealProviderProtocol::OpenAICompatible => "Chat 最小请求",
        RealProviderProtocol::OpenAIResponses => "Responses 最小请求",
        RealProviderProtocol::Anthropic => "Anthropic Messages 最小请求",
        RealProviderProtocol::Gemini => "Gemini GenerateContent 最小请求",
    }
}

fn text_probe_path(protocol: RealProviderProtocol, model_name: &str) -> String {
    match protocol {
        RealProviderProtocol::OpenAICompatible => "chat/completions".to_string(),
        RealProviderProtocol::OpenAIResponses => "responses".to_string(),
        RealProviderProtocol::Anthropic => "messages".to_string(),
        RealProviderProtocol::Gemini => {
            format!(
                "models/{}:generateContent",
                model_name.trim_start_matches("models/")
            )
        }
    }
}

fn vision_probe_path(protocol: RealProviderProtocol, model_name: &str) -> String {
    text_probe_path(protocol, model_name)
}

fn text_probe_body(
    protocol: RealProviderProtocol,
    model_name: &str,
    prompt: String,
    workload: &WorkloadConfig,
) -> serde_json::Value {
    match protocol {
        RealProviderProtocol::OpenAICompatible => {
            chat::completion_body(model_name, &prompt, workload, false, 0.2)
        }
        RealProviderProtocol::OpenAIResponses => {
            responses::response_body(model_name, &prompt, workload)
        }
        RealProviderProtocol::Anthropic => anthropic::messages_body(model_name, &prompt, workload),
        RealProviderProtocol::Gemini => gemini::generate_content_body(&prompt, workload),
    }
}

fn vision_probe_body(
    protocol: RealProviderProtocol,
    model_name: &str,
    sample: &VisionSample,
    workload: &WorkloadConfig,
) -> serde_json::Value {
    match protocol {
        RealProviderProtocol::OpenAICompatible => {
            vision::vision_completion_body(model_name, sample, workload)
        }
        RealProviderProtocol::OpenAIResponses => {
            responses::vision_response_body(model_name, sample, workload)
        }
        RealProviderProtocol::Anthropic => {
            anthropic::vision_messages_body(model_name, sample, workload)
        }
        RealProviderProtocol::Gemini => gemini::vision_generate_content_body(sample, workload),
    }
}

fn diagnostic_prompt_for_protocol(protocol: RealProviderProtocol) -> &'static str {
    match protocol {
        RealProviderProtocol::OpenAICompatible => chat::diagnostic_prompt(),
        RealProviderProtocol::OpenAIResponses => responses::diagnostic_prompt(),
        RealProviderProtocol::Anthropic => anthropic::diagnostic_prompt(),
        RealProviderProtocol::Gemini => gemini::diagnostic_prompt(),
    }
}

fn unsupported_model_type_endpoint(
    protocol: RealProviderProtocol,
    name: &str,
    model_type: ModelType,
) -> DiagnosticEndpoint {
    DiagnosticEndpoint {
        name: name.to_string(),
        method: "LOCAL".to_string(),
        path: protocol.label().to_string(),
        ok: false,
        latency_ms: None,
        http_status: None,
        message: format!(
            "{} 当前真实压测不支持 {} 模型。",
            protocol.label(),
            model_type.as_str()
        ),
        error_kind: Some("unsupported_model_type".to_string()),
    }
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
