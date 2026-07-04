use super::common::{build_common_load, build_tick_payload, success_rate, TickPayload};
use super::{BenchmarkAdapter, TickInput};
use crate::models::MetricsTick;

impl BenchmarkAdapter {
    pub(super) fn embedding_tick(&self, input: TickInput<'_>) -> MetricsTick {
        let base = build_common_load(input);
        let batch = self.config.batch_size.max(1);
        let text_count = self.config.text_count_per_request.max(1);
        let batch_factor = (batch as f64 / 16.0).clamp(0.5, 4.0);
        let qps = (input.concurrency as f64 * (0.9 + base.progress * 0.25)
            / (1.0 + base.saturation * 0.28 + batch_factor * 0.08))
            .max(1.0);
        let latency_ms = (210.0
            + base.pressure * 90.0
            + batch_factor * 120.0
            + base.saturation.powf(2.0) * 900.0) as i64;
        let input_tokens = (qps * text_count as f64 * 180.0).round() as i64;
        let text_per_second = (qps * text_count as f64).round() as i64;
        let tps = input_tokens as f64;
        let errors = if input.concurrency >= 160 {
            5
        } else if input.concurrency >= 96 && base.progress > 0.75 {
            2
        } else {
            0
        };
        let success_rate = success_rate(
            errors,
            base.progress,
            base.stage_pressure,
            base.saturation * 0.6,
        );
        build_tick_payload(
            input,
            TickPayload {
                qps,
                latency_ms,
                ttft_ms: 0,
                tps,
                success_rate,
                errors,
                input_tokens,
                output_tokens: 0,
                batch_size: batch,
                text_count: text_per_second,
                documents_per_query: 0,
                pair_count: 0,
                image_count: 0,
            },
        )
    }
}
