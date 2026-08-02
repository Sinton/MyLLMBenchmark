use crate::benchmark::engines::real::{
    build_text_generation_request_body, RealProviderClient, RealProviderProtocol, RequestOutcome,
};
use crate::domain::workload::WorkloadConfig;
use crate::error::{AppError, AppResult};
use crate::models::{
    DeleteResult, ProviderConnectionConfig, SiteProbeHistoryPage, SiteProbeHistoryPageInput,
    SiteProbeModelOption, SiteProbeModelScanInput, SiteProbeModelScanResult, SiteProbeRunDetail,
    SiteProbeRunInput, SiteProbeRunRecord, SiteProbeRunSummary,
};
use crate::state::AppState;
use chrono::Utc;
use reqwest::Url;
use uuid::Uuid;

pub async fn run_site_probe(
    state: &AppState,
    input: SiteProbeRunInput,
) -> AppResult<SiteProbeRunDetail> {
    let prepared = prepare_input(input)?;
    let client = RealProviderClient::new()?;
    let outcome = client
        .text_generation(
            &prepared.config,
            prepared.protocol,
            &prepared.model,
            &prepared.prompt,
            &prepared.workload,
            prepared.timeout_seconds,
        )
        .await;
    let mut detail = detail_from_outcome(&prepared, &outcome);
    let record = build_record(&prepared, outcome);
    let summary = state.insert_site_probe_run(record).await?;
    detail.summary.body_available = summary.body_available;
    Ok(detail)
}

pub async fn scan_site_probe_models(
    input: SiteProbeModelScanInput,
) -> AppResult<SiteProbeModelScanResult> {
    let base_url = normalize_base_url(&input.base_url)?;
    let (interface_type, _) = normalize_interface_type(&input.interface_type)?;
    let config = ProviderConnectionConfig {
        id: "site-probe-model-scan".to_string(),
        name: "Site Probe Model Scan".to_string(),
        base_url,
        api_key_plaintext: input.api_key.unwrap_or_default(),
        interface_type,
    };
    let client = RealProviderClient::new()?;
    let mut models = client
        .list_models(&config)
        .await
        .map_err(|error| AppError::Unexpected(anyhow::anyhow!("获取模型列表失败：{error}")))?
        .into_iter()
        .map(|model| SiteProbeModelOption {
            name: model.name,
            model_type: model.model_type,
            capabilities: model.capabilities,
            supports_streaming: model.supports_streaming,
        })
        .collect::<Vec<_>>();
    models.sort_by_cached_key(|model| model.name.to_ascii_lowercase());
    models.dedup_by(|left, right| left.name.eq_ignore_ascii_case(&right.name));

    Ok(SiteProbeModelScanResult {
        message: if models.is_empty() {
            "模型接口已响应，但没有返回可用模型。可切换为手动填写。".to_string()
        } else {
            format!("已从 /models 获取到 {} 个模型。", models.len())
        },
        models,
        scanned_at: Utc::now().to_rfc3339(),
    })
}

pub async fn list_site_probe_runs_page(
    state: &AppState,
    input: SiteProbeHistoryPageInput,
) -> AppResult<SiteProbeHistoryPage> {
    Ok(state.list_site_probe_runs_page(input).await?)
}

pub async fn get_site_probe_run_detail(
    state: &AppState,
    run_id: &str,
) -> AppResult<SiteProbeRunDetail> {
    Ok(state.get_site_probe_run_detail(run_id).await?)
}

pub async fn delete_site_probe_run(state: &AppState, run_id: &str) -> AppResult<DeleteResult> {
    Ok(state.delete_site_probe_run(run_id).await?)
}

struct PreparedProbeInput {
    id: String,
    name: String,
    base_url: String,
    interface_type: String,
    model: String,
    prompt: String,
    timeout_seconds: i64,
    save_body: bool,
    workload: WorkloadConfig,
    protocol: RealProviderProtocol,
    config: ProviderConnectionConfig,
    request_payload: serde_json::Value,
}

fn prepare_input(input: SiteProbeRunInput) -> AppResult<PreparedProbeInput> {
    let base_url = normalize_base_url(&input.base_url)?;
    let parsed = Url::parse(&base_url)
        .map_err(|_| AppError::validation("Base URL 必须是有效的 http:// 或 https:// 地址"))?;
    let (interface_type, protocol) = normalize_interface_type(&input.interface_type)?;
    let model = input.model.trim().to_string();
    if model.is_empty() {
        return Err(AppError::validation("请填写要测活的模型名称"));
    }
    let prompt = input.prompt.trim().to_string();
    if prompt.is_empty() {
        return Err(AppError::validation("请填写自定义测试 Prompt"));
    }

    let mut workload = WorkloadConfig::for_model_type("text_generation");
    workload.streaming = input.streaming;
    workload.max_output_tokens = input.max_output_tokens.unwrap_or(512).clamp(1, 8192);
    let timeout_seconds = input.timeout_seconds.unwrap_or(60).clamp(5, 600);
    let name = input
        .name
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| parsed.host_str().unwrap_or("OpenAI Compatible").to_string());
    let request_payload = build_text_generation_request_body(protocol, &model, &prompt, &workload);
    let id = Uuid::new_v4().to_string();

    Ok(PreparedProbeInput {
        id: id.clone(),
        name,
        base_url: base_url.clone(),
        interface_type: interface_type.clone(),
        model: model.clone(),
        prompt,
        timeout_seconds,
        save_body: input.save_body,
        workload,
        protocol,
        config: ProviderConnectionConfig {
            id,
            name: "Site Probe".to_string(),
            base_url,
            api_key_plaintext: input.api_key.unwrap_or_default(),
            interface_type,
        },
        request_payload,
    })
}

fn normalize_base_url(value: &str) -> AppResult<String> {
    let base_url = value.trim().trim_end_matches('/').to_string();
    if base_url.is_empty() {
        return Err(AppError::validation("请填写中转站 Base URL"));
    }
    let parsed = Url::parse(&base_url)
        .map_err(|_| AppError::validation("Base URL 必须是有效的 http:// 或 https:// 地址"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(AppError::validation("Base URL 只支持 http:// 或 https://"));
    }
    Ok(base_url)
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

fn build_record(prepared: &PreparedProbeInput, outcome: RequestOutcome) -> SiteProbeRunRecord {
    let status = if outcome.ok { "passed" } else { "failed" }.to_string();
    let raw_error = outcome.error_message.clone();
    let summary = SiteProbeRunSummary {
        id: prepared.id.clone(),
        name: prepared.name.clone(),
        base_url: prepared.base_url.clone(),
        interface_type: prepared.interface_type.clone(),
        model: prepared.model.clone(),
        status,
        latency_ms: outcome.latency_ms,
        ttft_ms: outcome.ttft_ms,
        input_tokens: outcome.usage.input_tokens,
        output_tokens: outcome.usage.output_tokens,
        total_tokens: outcome.usage.total_tokens,
        error_kind: outcome.error_kind.map(str::to_string),
        error_message: raw_error.clone(),
        prompt_preview: Some(preview_text(&prepared.prompt)),
        response_preview: outcome
            .response_text
            .as_deref()
            .map(preview_text)
            .or_else(|| raw_error.as_deref().map(preview_text)),
        body_available: false,
        created_at: Utc::now().to_rfc3339(),
    };

    SiteProbeRunRecord {
        summary,
        body_ref: None,
        prompt: prepared.save_body.then(|| prepared.prompt.clone()),
        response_text: prepared
            .save_body
            .then(|| outcome.response_text.clone())
            .flatten(),
        request_payload: prepared.save_body.then(|| prepared.request_payload.clone()),
        raw_error: prepared.save_body.then(|| raw_error.clone()).flatten(),
        raw_usage: prepared
            .save_body
            .then(|| outcome.raw_usage.clone())
            .flatten(),
    }
}

fn detail_from_outcome(
    prepared: &PreparedProbeInput,
    outcome: &RequestOutcome,
) -> SiteProbeRunDetail {
    SiteProbeRunDetail {
        summary: SiteProbeRunSummary {
            id: prepared.id.clone(),
            name: prepared.name.clone(),
            base_url: prepared.base_url.clone(),
            interface_type: prepared.interface_type.clone(),
            model: prepared.model.clone(),
            status: if outcome.ok { "passed" } else { "failed" }.to_string(),
            latency_ms: outcome.latency_ms,
            ttft_ms: outcome.ttft_ms,
            input_tokens: outcome.usage.input_tokens,
            output_tokens: outcome.usage.output_tokens,
            total_tokens: outcome.usage.total_tokens,
            error_kind: outcome.error_kind.map(str::to_string),
            error_message: outcome.error_message.clone(),
            prompt_preview: Some(preview_text(&prepared.prompt)),
            response_preview: outcome
                .response_text
                .as_deref()
                .map(preview_text)
                .or_else(|| outcome.error_message.as_deref().map(preview_text)),
            body_available: true,
            created_at: Utc::now().to_rfc3339(),
        },
        prompt: Some(prepared.prompt.clone()),
        response_text: outcome.response_text.clone(),
        request_payload: Some(prepared.request_payload.clone()),
        raw_error: outcome.error_message.clone(),
        raw_usage: outcome.raw_usage.clone(),
    }
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
    use super::{normalize_interface_type, prepare_input, scan_site_probe_models};
    use crate::benchmark::engines::real::RealProviderProtocol;
    use crate::models::{SiteProbeModelScanInput, SiteProbeRunInput};
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn supports_all_site_probe_text_protocols() {
        assert_eq!(
            normalize_interface_type("OpenAI-Response").unwrap().1,
            RealProviderProtocol::OpenAIResponses
        );
        assert_eq!(
            normalize_interface_type("Anthropic").unwrap().1,
            RealProviderProtocol::Anthropic
        );
        assert!(normalize_interface_type("Gemini").is_err());
    }

    #[test]
    fn builds_protocol_specific_probe_payloads() {
        let responses = prepared_input("OpenAI-Response");
        assert_eq!(responses.request_payload["input"], "ping");
        assert_eq!(responses.request_payload["max_output_tokens"], 128);
        assert!(responses.request_payload.get("messages").is_none());

        let anthropic = prepared_input("Anthropic");
        assert_eq!(anthropic.request_payload["messages"][0]["content"], "ping");
        assert_eq!(anthropic.request_payload["max_tokens"], 128);
        assert!(anthropic.request_payload.get("stream_options").is_none());
    }

    #[tokio::test]
    async fn scans_models_with_protocol_specific_auth() {
        let (responses_url, responses_handle) = spawn_models_server(|request| {
            assert!(request.starts_with("GET /v1/models "));
            assert!(request
                .to_ascii_lowercase()
                .contains("authorization: bearer probe-secret"));
        });
        let responses = scan_site_probe_models(SiteProbeModelScanInput {
            base_url: responses_url,
            api_key: Some("probe-secret".to_string()),
            interface_type: "OpenAI-Response".to_string(),
        })
        .await
        .unwrap();
        responses_handle.join().unwrap();
        assert_eq!(
            responses
                .models
                .iter()
                .map(|model| model.name.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha-model", "zeta-model"]
        );

        let (anthropic_url, anthropic_handle) = spawn_models_server(|request| {
            let request = request.to_ascii_lowercase();
            assert!(request.starts_with("get /v1/models "));
            assert!(request.contains("x-api-key: probe-secret"));
            assert!(request.contains("anthropic-version: 2023-06-01"));
        });
        let anthropic = scan_site_probe_models(SiteProbeModelScanInput {
            base_url: anthropic_url,
            api_key: Some("probe-secret".to_string()),
            interface_type: "Anthropic".to_string(),
        })
        .await
        .unwrap();
        anthropic_handle.join().unwrap();
        assert_eq!(anthropic.models.len(), 2);
    }

    fn prepared_input(interface_type: &str) -> super::PreparedProbeInput {
        prepare_input(SiteProbeRunInput {
            name: None,
            base_url: "https://gateway.example.com/v1".to_string(),
            api_key: Some("secret".to_string()),
            interface_type: interface_type.to_string(),
            model: "test-model".to_string(),
            prompt: "ping".to_string(),
            streaming: true,
            max_output_tokens: Some(128),
            timeout_seconds: Some(30),
            save_body: false,
        })
        .unwrap()
    }

    fn spawn_models_server(
        inspect: impl FnOnce(&str) + Send + 'static,
    ) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0_u8; 8192];
            let size = stream.read(&mut buffer).unwrap();
            let request = String::from_utf8_lossy(&buffer[..size]);
            inspect(&request);
            let body = r#"{"data":[{"id":"zeta-model"},{"id":"alpha-model"}]}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        });
        (format!("http://{address}/v1"), handle)
    }
}
