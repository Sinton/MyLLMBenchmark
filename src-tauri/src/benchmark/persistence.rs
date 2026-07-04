use crate::domain::benchmark_sample::StageSample;
use crate::models::{BenchmarkErrorRecord, BenchmarkTaskSummary, MetricsTick};
use crate::state::AppState;

#[derive(Clone)]
pub struct BenchmarkPersistence {
    state: AppState,
}

impl BenchmarkPersistence {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn insert_tick(&self, tick: &MetricsTick) -> anyhow::Result<()> {
        self.state.insert_tick(tick).await
    }

    pub async fn insert_error(&self, error: &BenchmarkErrorRecord) -> anyhow::Result<()> {
        self.state.insert_benchmark_error(error).await
    }

    pub async fn mark_engine_mode(&self, task_id: &str, engine_mode: &str) -> anyhow::Result<()> {
        self.state
            .update_task_engine_mode(task_id, engine_mode)
            .await
    }

    pub async fn insert_stage(&self, sample: &StageSample) -> anyhow::Result<()> {
        self.state.insert_stage(sample).await
    }

    pub async fn finish_task(
        &self,
        task_id: &str,
        status: &str,
        success_rate: f64,
        p95_latency_ms: i64,
        goodput_qps: f64,
    ) -> anyhow::Result<()> {
        self.state
            .update_task_finished(task_id, status, success_rate, p95_latency_ms, goodput_qps)
            .await
    }

    pub async fn task_summary(&self, task_id: &str) -> anyhow::Result<BenchmarkTaskSummary> {
        self.state.get_task_summary(task_id).await
    }

    pub async fn remove_task(&self, task_id: &str) {
        self.state.remove_task(task_id).await;
    }
}
