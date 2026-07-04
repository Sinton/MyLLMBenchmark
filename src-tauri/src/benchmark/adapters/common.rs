use super::TickInput;
use crate::models::MetricsTick;

#[derive(Clone, Copy)]
pub(super) struct CommonLoad {
    pub(super) progress: f64,
    pub(super) pressure: f64,
    pub(super) stage_pressure: f64,
    pub(super) saturation: f64,
}

pub(super) struct TickPayload {
    pub(super) qps: f64,
    pub(super) latency_ms: i64,
    pub(super) ttft_ms: i64,
    pub(super) tps: f64,
    pub(super) success_rate: f64,
    pub(super) errors: i64,
    pub(super) input_tokens: i64,
    pub(super) output_tokens: i64,
    pub(super) batch_size: i64,
    pub(super) text_count: i64,
    pub(super) documents_per_query: i64,
    pub(super) pair_count: i64,
    pub(super) image_count: i64,
}

pub(super) fn build_common_load(input: TickInput<'_>) -> CommonLoad {
    let progress = input.elapsed_in_stage as f64 / input.stage_total as f64;
    let pressure = input.concurrency as f64 / 32.0;
    let stage_pressure = input.stage_index as f64 / input.stage_count.max(1) as f64;
    let saturation = (pressure - 1.0).max(0.0);
    CommonLoad {
        progress,
        pressure,
        stage_pressure,
        saturation,
    }
}

pub(super) fn build_tick_payload(input: TickInput<'_>, payload: TickPayload) -> MetricsTick {
    let request_count = input.concurrency.max(1);
    let failure_count = payload.errors.clamp(0, request_count);
    let success_count = (request_count - failure_count).max(0);

    MetricsTick {
        task_id: input.task_id.to_string(),
        elapsed_seconds: input.elapsed,
        qps: round2(payload.qps),
        latency_ms: payload.latency_ms,
        ttft_ms: payload.ttft_ms,
        tps: round2(payload.tps),
        success_rate: round2(payload.success_rate),
        errors: payload.errors,
        in_flight: input.concurrency,
        request_count,
        success_count,
        failure_count,
        input_tokens: payload.input_tokens,
        output_tokens: payload.output_tokens,
        total_tokens: payload.input_tokens + payload.output_tokens,
        batch_size: payload.batch_size,
        text_count: payload.text_count,
        documents_per_query: payload.documents_per_query,
        pair_count: payload.pair_count,
        image_count: payload.image_count,
    }
}

pub(super) fn error_count(concurrency: i64, progress: f64) -> i64 {
    if concurrency >= 64 {
        8
    } else if concurrency >= 48 && progress > 0.6 {
        3
    } else if concurrency >= 32 && progress > 0.85 {
        1
    } else {
        0
    }
}

pub(super) fn success_rate(
    errors: i64,
    progress: f64,
    stage_pressure: f64,
    saturation: f64,
) -> f64 {
    if errors > 0 {
        (99.7 - errors as f64 * 0.55 - saturation * 1.2).max(92.0)
    } else {
        99.98 - progress * 0.08 - stage_pressure * 0.05
    }
}

pub(super) fn prompt_tokens_for(profile: &str) -> f64 {
    match profile {
        "short" => 160.0,
        "long" => 1200.0,
        _ => 320.0,
    }
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
