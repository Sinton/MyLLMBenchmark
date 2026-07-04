use super::common::{build_common_load, build_tick_payload, success_rate, TickPayload};
use super::{BenchmarkAdapter, TickInput};
use crate::models::MetricsTick;

impl BenchmarkAdapter {
    pub(super) fn rerank_tick(&self, input: TickInput<'_>) -> MetricsTick {
        let base = build_common_load(input);
        let docs = self.config.documents_per_query.max(1);
        let docs_factor = (docs as f64 / 30.0).clamp(0.35, 5.0);
        let qps = (input.concurrency as f64 * (0.62 + base.progress * 0.18)
            / (1.0 + base.saturation * 0.22 + docs_factor * 0.2))
            .max(0.8);
        let latency_ms = (360.0
            + base.pressure * 150.0
            + docs_factor * 260.0
            + base.saturation.powf(2.0) * 1100.0) as i64;
        let pair_count = (qps * docs as f64).round() as i64;
        let input_tokens = (pair_count as f64 * 260.0).round() as i64;
        let errors = if input.concurrency >= 96 {
            5
        } else if input.concurrency >= 64 && base.progress > 0.7 {
            2
        } else {
            0
        };
        let success_rate = success_rate(
            errors,
            base.progress,
            base.stage_pressure,
            base.saturation * 0.8,
        );
        build_tick_payload(
            input,
            TickPayload {
                qps,
                latency_ms,
                ttft_ms: 0,
                tps: pair_count as f64,
                success_rate,
                errors,
                input_tokens,
                output_tokens: 0,
                batch_size: 0,
                text_count: 0,
                documents_per_query: docs,
                pair_count,
                image_count: 0,
            },
        )
    }
}
