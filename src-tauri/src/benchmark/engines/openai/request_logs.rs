use super::{preview_text, RequestOutcome};
use crate::benchmark::persistence::BenchmarkPersistence;
use crate::models::{BenchmarkRequestLogRecord, BenchmarkRequestLogSummary, RequestLogConfig};
use uuid::Uuid;

pub(super) async fn record_request_logs(
    persistence: &BenchmarkPersistence,
    task_id: &str,
    stage_index: i64,
    config: &RequestLogConfig,
    results: &[RequestOutcome],
) -> anyhow::Result<()> {
    if !config.enabled {
        return Ok(());
    }

    for result in results
        .iter()
        .filter(|result| result.request_index <= config.max_records_per_stage)
    {
        let status = if result.ok { "success" } else { "failed" }.to_string();
        let prompt_preview = result.prompt.as_deref().map(preview_text);
        let response_preview = result.response_text.as_deref().map(preview_text);
        let summary = BenchmarkRequestLogSummary {
            id: Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            stage_index,
            request_index: result.request_index,
            sample_index: result.sample_index,
            status,
            latency_ms: result.latency_ms,
            ttft_ms: result.ttft_ms,
            input_tokens: result.usage.input_tokens,
            output_tokens: result.usage.output_tokens,
            total_tokens: result.usage.total_tokens,
            error_kind: result.error_kind.map(str::to_string),
            prompt_preview,
            response_preview,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        persistence
            .insert_request_log(BenchmarkRequestLogRecord {
                summary,
                body_ref: None,
                prompt: config.capture_body.then(|| result.prompt.clone()).flatten(),
                response_text: config
                    .capture_body
                    .then(|| result.response_text.clone())
                    .flatten(),
                raw_error: config
                    .capture_body
                    .then(|| result.error_message.clone())
                    .flatten(),
                raw_usage: config
                    .capture_body
                    .then(|| result.raw_usage.clone())
                    .flatten(),
            })
            .await?;
    }
    Ok(())
}
