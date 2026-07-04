use super::{AppDataSource, BenchmarkRepository};
use crate::db::Database;
use crate::domain::benchmark_sample::StageSample;
use crate::mock::MockDataStore;
use crate::models::{
    BenchmarkErrorRecord, BenchmarkStartInput, BenchmarkTaskSummary, MetricsTick,
    ProviderDiagnosticsResult,
};

impl BenchmarkRepository for MockDataStore {
    async fn create_task(
        &self,
        input: &BenchmarkStartInput,
    ) -> anyhow::Result<BenchmarkTaskSummary> {
        MockDataStore::create_task(self, input).await
    }

    async fn update_task_finished(
        &self,
        task_id: &str,
        status: &str,
        success_rate: f64,
        p95_latency_ms: i64,
        goodput_qps: f64,
    ) -> anyhow::Result<()> {
        MockDataStore::update_task_finished(
            self,
            task_id,
            status,
            success_rate,
            p95_latency_ms,
            goodput_qps,
        )
        .await
    }

    async fn insert_stage(&self, sample: &StageSample) -> anyhow::Result<()> {
        MockDataStore::insert_stage(self, sample).await
    }

    async fn insert_tick(&self, tick: &MetricsTick) -> anyhow::Result<()> {
        MockDataStore::insert_tick(self, tick).await
    }

    async fn insert_benchmark_error(&self, error: &BenchmarkErrorRecord) -> anyhow::Result<()> {
        MockDataStore::insert_benchmark_error(self, error).await
    }

    async fn get_task_summary(&self, task_id: &str) -> anyhow::Result<BenchmarkTaskSummary> {
        MockDataStore::get_task_summary(self, task_id).await
    }

    async fn list_ticks(&self, task_id: &str) -> anyhow::Result<Vec<MetricsTick>> {
        MockDataStore::list_ticks(self, task_id).await
    }

    async fn update_task_engine_mode(
        &self,
        task_id: &str,
        engine_mode: &str,
    ) -> anyhow::Result<()> {
        MockDataStore::update_task_engine_mode(self, task_id, engine_mode).await
    }

    async fn update_task_preflight(
        &self,
        task_id: &str,
        preflight_result: Option<serde_json::Value>,
        diagnostics_snapshot: Option<ProviderDiagnosticsResult>,
    ) -> anyhow::Result<()> {
        MockDataStore::update_task_preflight(self, task_id, preflight_result, diagnostics_snapshot)
            .await
    }
}

impl BenchmarkRepository for Database {
    async fn create_task(
        &self,
        input: &BenchmarkStartInput,
    ) -> anyhow::Result<BenchmarkTaskSummary> {
        Database::create_task(self, input).await
    }

    async fn update_task_finished(
        &self,
        task_id: &str,
        status: &str,
        success_rate: f64,
        p95_latency_ms: i64,
        goodput_qps: f64,
    ) -> anyhow::Result<()> {
        Database::update_task_finished(
            self,
            task_id,
            status,
            success_rate,
            p95_latency_ms,
            goodput_qps,
        )
        .await
    }

    async fn insert_stage(&self, sample: &StageSample) -> anyhow::Result<()> {
        Database::insert_stage(self, sample).await
    }

    async fn insert_tick(&self, tick: &MetricsTick) -> anyhow::Result<()> {
        Database::insert_tick(self, tick).await
    }

    async fn insert_benchmark_error(&self, error: &BenchmarkErrorRecord) -> anyhow::Result<()> {
        Database::insert_benchmark_error(self, error).await
    }

    async fn get_task_summary(&self, task_id: &str) -> anyhow::Result<BenchmarkTaskSummary> {
        Database::get_task_summary(self, task_id).await
    }

    async fn list_ticks(&self, task_id: &str) -> anyhow::Result<Vec<MetricsTick>> {
        Database::list_ticks(self, task_id).await
    }

    async fn update_task_engine_mode(
        &self,
        task_id: &str,
        engine_mode: &str,
    ) -> anyhow::Result<()> {
        Database::update_task_engine_mode(self, task_id, engine_mode).await
    }

    async fn update_task_preflight(
        &self,
        task_id: &str,
        preflight_result: Option<serde_json::Value>,
        diagnostics_snapshot: Option<ProviderDiagnosticsResult>,
    ) -> anyhow::Result<()> {
        Database::update_task_preflight(self, task_id, preflight_result, diagnostics_snapshot).await
    }
}

impl AppDataSource {
    pub async fn create_task(
        &self,
        input: &BenchmarkStartInput,
    ) -> anyhow::Result<BenchmarkTaskSummary> {
        match self {
            Self::Mock(source) => BenchmarkRepository::create_task(source, input).await,
            Self::Sqlite(source) => BenchmarkRepository::create_task(source, input).await,
        }
    }

    pub async fn update_task_finished(
        &self,
        task_id: &str,
        status: &str,
        success_rate: f64,
        p95_latency_ms: i64,
        goodput_qps: f64,
    ) -> anyhow::Result<()> {
        match self {
            Self::Mock(source) => {
                BenchmarkRepository::update_task_finished(
                    source,
                    task_id,
                    status,
                    success_rate,
                    p95_latency_ms,
                    goodput_qps,
                )
                .await
            }
            Self::Sqlite(source) => {
                BenchmarkRepository::update_task_finished(
                    source,
                    task_id,
                    status,
                    success_rate,
                    p95_latency_ms,
                    goodput_qps,
                )
                .await
            }
        }
    }

    pub async fn insert_stage(&self, sample: &StageSample) -> anyhow::Result<()> {
        match self {
            Self::Mock(source) => BenchmarkRepository::insert_stage(source, sample).await,
            Self::Sqlite(source) => BenchmarkRepository::insert_stage(source, sample).await,
        }
    }

    pub async fn insert_tick(&self, tick: &MetricsTick) -> anyhow::Result<()> {
        match self {
            Self::Mock(source) => BenchmarkRepository::insert_tick(source, tick).await,
            Self::Sqlite(source) => BenchmarkRepository::insert_tick(source, tick).await,
        }
    }

    pub async fn insert_benchmark_error(&self, error: &BenchmarkErrorRecord) -> anyhow::Result<()> {
        match self {
            Self::Mock(source) => BenchmarkRepository::insert_benchmark_error(source, error).await,
            Self::Sqlite(source) => {
                BenchmarkRepository::insert_benchmark_error(source, error).await
            }
        }
    }

    pub async fn get_task_summary(&self, task_id: &str) -> anyhow::Result<BenchmarkTaskSummary> {
        match self {
            Self::Mock(source) => BenchmarkRepository::get_task_summary(source, task_id).await,
            Self::Sqlite(source) => BenchmarkRepository::get_task_summary(source, task_id).await,
        }
    }

    pub async fn list_ticks(&self, task_id: &str) -> anyhow::Result<Vec<MetricsTick>> {
        match self {
            Self::Mock(source) => BenchmarkRepository::list_ticks(source, task_id).await,
            Self::Sqlite(source) => BenchmarkRepository::list_ticks(source, task_id).await,
        }
    }

    pub async fn update_task_engine_mode(
        &self,
        task_id: &str,
        engine_mode: &str,
    ) -> anyhow::Result<()> {
        match self {
            Self::Mock(source) => {
                BenchmarkRepository::update_task_engine_mode(source, task_id, engine_mode).await
            }
            Self::Sqlite(source) => {
                BenchmarkRepository::update_task_engine_mode(source, task_id, engine_mode).await
            }
        }
    }

    pub async fn update_task_preflight(
        &self,
        task_id: &str,
        preflight_result: Option<serde_json::Value>,
        diagnostics_snapshot: Option<ProviderDiagnosticsResult>,
    ) -> anyhow::Result<()> {
        match self {
            Self::Mock(source) => {
                BenchmarkRepository::update_task_preflight(
                    source,
                    task_id,
                    preflight_result,
                    diagnostics_snapshot,
                )
                .await
            }
            Self::Sqlite(source) => {
                BenchmarkRepository::update_task_preflight(
                    source,
                    task_id,
                    preflight_result,
                    diagnostics_snapshot,
                )
                .await
            }
        }
    }
}
