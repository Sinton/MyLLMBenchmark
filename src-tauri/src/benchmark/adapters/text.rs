use super::common::{
    build_common_load, build_tick_payload, error_count, prompt_tokens_for, success_rate,
    TickPayload,
};
use super::{BenchmarkAdapter, TickInput};
use crate::models::MetricsTick;

impl BenchmarkAdapter {
    pub(super) fn text_generation_tick(&self, input: TickInput<'_>) -> MetricsTick {
        let base = build_common_load(input);
        let output_factor = (self.config.max_output_tokens as f64 / 512.0).clamp(0.65, 2.0);
        let latency_ms = (620.0
            + base.pressure * 340.0
            + base.progress * 420.0
            + base.saturation.powf(2.0) * 1700.0 * output_factor) as i64;
        let ttft_ms = (220.0
            + base.pressure * 110.0
            + base.progress * 150.0
            + base.saturation.powf(2.0) * 720.0) as i64;
        let tps = (64.0 - base.pressure * 3.6 - base.stage_pressure * 3.5 - base.saturation * 9.0)
            .max(10.0)
            / output_factor.sqrt();
        let qps = (input.concurrency as f64 * (0.38 + base.progress * 0.2)
            / (1.0 + base.saturation * 0.18))
            .max(0.8);
        let errors = error_count(input.concurrency, base.progress);
        let success_rate =
            success_rate(errors, base.progress, base.stage_pressure, base.saturation);
        let input_tokens = (qps * prompt_tokens_for(&self.config.prompt_profile)).round() as i64;
        let output_tokens = tps.round() as i64;
        build_tick_payload(
            input,
            TickPayload {
                qps,
                latency_ms,
                ttft_ms,
                tps,
                success_rate,
                errors,
                input_tokens,
                output_tokens,
                batch_size: 0,
                text_count: 0,
                documents_per_query: 0,
                pair_count: 0,
                image_count: 0,
            },
        )
    }
}
