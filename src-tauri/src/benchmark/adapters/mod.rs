use crate::domain::workload::WorkloadConfig;
use crate::models::{BenchmarkStartInput, MetricsTick};

mod common;
mod embedding;
mod rerank;
mod text;
mod vision;

#[derive(Clone)]
pub struct BenchmarkAdapter {
    pub(super) model_type: String,
    pub(super) config: WorkloadConfig,
}

impl BenchmarkAdapter {
    pub fn from_input(model_type: &str, input: &BenchmarkStartInput) -> Self {
        Self {
            model_type: model_type.to_string(),
            config: WorkloadConfig::from_value(model_type, input.workload_config.as_ref()),
        }
    }

    pub fn build_tick(&self, input: TickInput<'_>) -> MetricsTick {
        match self.model_type.as_str() {
            "embedding" => self.embedding_tick(input),
            "rerank" => self.rerank_tick(input),
            "multimodal" => self.vision_tick(input),
            _ => self.text_generation_tick(input),
        }
    }
}

#[derive(Clone, Copy)]
pub struct TickInput<'a> {
    pub task_id: &'a str,
    pub elapsed: i64,
    pub elapsed_in_stage: i64,
    pub stage_total: i64,
    pub concurrency: i64,
    pub stage_index: i64,
    pub stage_count: i64,
}

#[cfg(test)]
mod tests {
    use super::{BenchmarkAdapter, TickInput};
    use crate::models::BenchmarkStartInput;

    fn input() -> BenchmarkStartInput {
        BenchmarkStartInput {
            provider_id: "provider".to_string(),
            model_id: None,
            dataset_id: "dataset".to_string(),
            mode: "fixed".to_string(),
            concurrency: 16,
            duration_seconds: 30,
            start_concurrency: None,
            end_concurrency: None,
            step_strategy: None,
            step_value: None,
            stage_sample_rounds: None,
            stage_duration_seconds: None,
            warmup_rounds: None,
            warmup_seconds: None,
            request_timeout_seconds: None,
            sla_p95_ms: None,
            min_success_rate: None,
            sla_stop_policy: None,
            workload_config: None,
            request_log_config: None,
        }
    }

    fn tick_input() -> TickInput<'static> {
        TickInput {
            task_id: "task",
            elapsed: 10,
            elapsed_in_stage: 5,
            stage_total: 10,
            concurrency: 16,
            stage_index: 1,
            stage_count: 3,
        }
    }

    #[test]
    fn text_generation_tick_contains_generation_metrics() {
        let tick =
            BenchmarkAdapter::from_input("text_generation", &input()).build_tick(tick_input());

        assert!(tick.ttft_ms > 0);
        assert!(tick.output_tokens > 0);
        assert!(tick.tps > 0.0);
    }

    #[test]
    fn embedding_tick_contains_batch_and_input_throughput() {
        let tick = BenchmarkAdapter::from_input("embedding", &input()).build_tick(tick_input());

        assert_eq!(tick.ttft_ms, 0);
        assert!(tick.batch_size > 0);
        assert!(tick.text_count > 0);
        assert!(tick.input_tokens > 0);
    }

    #[test]
    fn rerank_tick_contains_pair_throughput() {
        let tick = BenchmarkAdapter::from_input("rerank", &input()).build_tick(tick_input());

        assert_eq!(tick.ttft_ms, 0);
        assert!(tick.documents_per_query > 0);
        assert!(tick.pair_count > 0);
    }

    #[test]
    fn vision_tick_contains_multimodal_metrics() {
        let tick = BenchmarkAdapter::from_input("multimodal", &input()).build_tick(tick_input());

        assert!(tick.ttft_ms > 0);
        assert!(tick.image_count > 0);
        assert!(tick.input_tokens > 0);
    }
}
