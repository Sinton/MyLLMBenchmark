use crate::models::MetricsTick;

#[derive(Debug, Clone)]
pub struct StageSample {
    pub task_id: String,
    pub stage_index: i64,
    pub concurrency: i64,
    pub sample_rounds: i64,
    pub warmup_rounds: i64,
    pub request_count: i64,
    pub success_count: i64,
    pub failure_count: i64,
    pub goodput_qps: f64,
    pub p95_latency_ms: i64,
    pub ttft_ms: i64,
    pub tps: f64,
    pub success_rate: f64,
    pub error_rate: f64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub batch_size: i64,
    pub text_count: i64,
    pub documents_per_query: i64,
    pub pair_count: i64,
    pub image_count: i64,
    pub sla_passed: bool,
    pub stop_reason: Option<String>,
}

impl StageSample {
    pub fn from_tick(stage_index: i64, concurrency: i64, tick: &MetricsTick) -> Self {
        Self::from_tick_with_evidence(stage_index, concurrency, tick, 1, 0, true, None)
    }

    pub fn from_tick_with_evidence(
        stage_index: i64,
        concurrency: i64,
        tick: &MetricsTick,
        sample_rounds: i64,
        warmup_rounds: i64,
        sla_passed: bool,
        stop_reason: Option<String>,
    ) -> Self {
        Self {
            task_id: tick.task_id.clone(),
            stage_index,
            concurrency,
            sample_rounds,
            warmup_rounds,
            request_count: tick.request_count,
            success_count: tick.success_count,
            failure_count: tick.failure_count,
            goodput_qps: tick.qps,
            p95_latency_ms: tick.latency_ms,
            ttft_ms: tick.ttft_ms,
            tps: tick.tps,
            success_rate: tick.success_rate,
            error_rate: (100.0 - tick.success_rate).clamp(0.0, 100.0),
            input_tokens: tick.input_tokens,
            output_tokens: tick.output_tokens,
            total_tokens: tick.total_tokens,
            batch_size: tick.batch_size,
            text_count: tick.text_count,
            documents_per_query: tick.documents_per_query,
            pair_count: tick.pair_count,
            image_count: tick.image_count,
            sla_passed,
            stop_reason,
        }
    }
}
