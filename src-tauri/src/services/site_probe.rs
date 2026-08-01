use crate::benchmark::engines::real::{api_url, RealProviderClient, RealProviderProtocol, RequestOutcome};
use crate::domain::workload::WorkloadConfig;
use crate::error::{AppError, AppResult};
use crate::models::{
    DeleteResult, ProviderConnectionConfig, SiteProbeHistoryPage, SiteProbeHistoryPageInput,
    SiteProbeRunDetail, SiteProbeRunInput, SiteProbeRunRecord, SiteProbeRunSummary,
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
            RealProviderProtocol::OpenAICompatible,
            &prepared.model,
            &prepared.prompt,
            &prepared.workload,
            prepared.timeout_seconds,
        )
        .await;
    let record = build_record(&prepared, outcome);
    let mut detail = detail_from_record(&record);
    let summary = state.insert_site_probe_run(record).await?;
    detail.summary.body_available = summary.body_available;
    Ok(detail)
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
    config: ProviderConnectionConfig,
    request_payload: serde_json::Value,
}

fn prepare_input(input: SiteProbeRunInput) -> AppResult<PreparedProbeInput> {
    let base_url = input.base_url.trim().trim_end_matches('/').to_string();
    if base_url.is_empty() {
        return Err(AppError::validation("请填写中转站 Base URL"));
    }
    let parsed = Url::parse(&base_url)
        .map_err(|_| AppError::validation("Base URL 必须是有效的 http:// 或 https:// 地址"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(AppError::validation("Base URL 只支持 http:// 或 https://"));
    }

    let interface_type = normalize_interface_type(&input.interface_type)?;
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
    let request_payload = build_openai_chat_payload(&model, &prompt, &workload);
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

fn normalize_interface_type(value: &str) -> AppResult<String> {
    match value.trim() {
        "OpenAI" | "OpenAI Compatible" | "" => Ok("OpenAI".to_string()),
        other => Err(AppError::validation(format!(
            "站点测活 v1 仅支持 OpenAI Compatible Chat，当前接口类型 {other} 暂不支持。"
        ))),
    }
}

fn build_openai_chat_payload(
    model: &str,
    prompt: &str,
    workload: &WorkloadConfig,
) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "stream": workload.streaming,
        "max_tokens": workload.max_output_tokens,
        "temperature": 0.7
    });
    if workload.streaming {
        payload["stream_options"] = serde_json::json!({"include_usage": true});
    }
    payload
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
        raw_usage: prepared.save_body.then(|| outcome.raw_usage.clone()).flatten(),
    }
}

fn detail_from_record(record: &SiteProbeRunRecord) -> SiteProbeRunDetail {
    SiteProbeRunDetail {
        summary: record.summary.clone(),
        prompt: record.prompt.clone(),
        response_text: record.response_text.clone(),
        request_payload: record.request_payload.clone(),
        raw_error: record.raw_error.clone().or(record.summary.error_message.clone()),
        raw_usage: record.raw_usage.clone(),
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

#[allow(dead_code)]
fn _chat_url_for_debug(base_url: &str) -> String {
    api_url(base_url, "chat/completions")
}
