use super::common::{build_common_load, build_tick_payload, success_rate, TickPayload};
use super::{BenchmarkAdapter, TickInput};
use crate::models::MetricsTick;

impl BenchmarkAdapter {
    pub(super) fn vision_tick(&self, input: TickInput<'_>) -> MetricsTick {
        let base = build_common_load(input);
        let image_count = self.config.image_count.max(1);
        let image_factor = match self.config.image_profile.as_str() {
            "small" => 0.72,
            "large" => 1.55,
            _ => 1.0,
        } * image_count as f64;
        let qps = (input.concurrency as f64 * (0.28 + base.progress * 0.12)
            / (1.0 + base.saturation * 0.25 + image_factor * 0.14))
            .max(0.4);
        let latency_ms = (900.0
            + base.pressure * 420.0
            + image_factor * 520.0
            + base.saturation.powf(2.0) * 2100.0) as i64;
        let ttft_ms = (360.0
            + base.pressure * 160.0
            + image_factor * 210.0
            + base.saturation.powf(2.0) * 840.0) as i64;
        let tps =
            (44.0 - base.pressure * 2.8 - image_factor * 1.6 - base.saturation * 7.0).max(8.0);
        let input_tokens = (qps * 420.0 * image_factor).round() as i64;
        let output_tokens = (tps * 0.75).round() as i64;
        let errors = if input.concurrency >= 48 {
            6
        } else if input.concurrency >= 24 && base.progress > 0.78 {
            2
        } else {
            0
        };
        let success_rate =
            success_rate(errors, base.progress, base.stage_pressure, base.saturation);
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
                image_count,
            },
        )
    }
}
