use crate::db::Database;
use crate::domain::benchmark_sample::StageSample;
use crate::mock::MockDataStore;
use crate::models::{
    BenchmarkErrorRecord, BenchmarkRequestLogDetail, BenchmarkRequestLogPage,
    BenchmarkRequestLogPageInput, BenchmarkRequestLogRecord, BenchmarkStartInput,
    BenchmarkTaskSummary, CreateProviderInput, DashboardSummary, DatasetAppendInput,
    DatasetExportInput, DatasetExportResult, DatasetImportInput, DatasetSample,
    DatasetSampleBatchDeleteInput, DatasetSampleCreateInput, DatasetSamplePage,
    DatasetSamplePageInput, DatasetSamplePreview, DatasetSampleUpdateInput, DatasetSummary,
    DatasetUpdateInput, DatasetValidationResult, DeleteResult, DiscoveredModel,
    EndpointProbeBatchDetail, EndpointProbeBatchRecord, EndpointProbeBatchSummary,
    EndpointProbeHistoryPage, EndpointProbeHistoryPageInput, EndpointProbeRunDetail,
    EndpointProbeRunRecord, EndpointProbeRunSummary, MetricsTick, ModelSummary,
    ProviderConnectionConfig, ProviderDiagnosticsResult, ProviderSummary, ReportDetail,
    ReportSummary, UpdateProviderInput,
};

mod benchmarks;
mod dashboard;
mod datasets;
mod endpoint_probe;
mod providers;
mod reports;

#[derive(Clone)]
pub enum AppDataSource {
    Mock(MockDataStore),
    Sqlite(Database),
}

#[allow(async_fn_in_trait)]
pub(crate) trait DashboardRepository {
    async fn dashboard_summary(&self) -> anyhow::Result<DashboardSummary>;
}

#[allow(async_fn_in_trait)]
pub(crate) trait DatasetRepository {
    async fn list_datasets(&self) -> anyhow::Result<Vec<DatasetSummary>>;
    async fn import_dataset(&self, input: DatasetImportInput) -> anyhow::Result<DatasetSummary>;
    async fn update_dataset(&self, input: DatasetUpdateInput) -> anyhow::Result<DatasetSummary>;
    async fn delete_dataset(&self, dataset_id: &str) -> anyhow::Result<DeleteResult>;
    async fn preview_dataset_samples(
        &self,
        dataset_id: &str,
        limit: i64,
    ) -> anyhow::Result<Vec<DatasetSamplePreview>>;
    async fn list_dataset_samples_page(
        &self,
        input: DatasetSamplePageInput,
    ) -> anyhow::Result<DatasetSamplePage>;
    async fn list_dataset_samples(&self, dataset_id: &str) -> anyhow::Result<Vec<DatasetSample>>;
    async fn create_dataset_sample(
        &self,
        input: DatasetSampleCreateInput,
    ) -> anyhow::Result<DatasetSamplePreview>;
    async fn update_dataset_sample(
        &self,
        input: DatasetSampleUpdateInput,
    ) -> anyhow::Result<DatasetSamplePreview>;
    async fn delete_dataset_sample(&self, sample_id: &str) -> anyhow::Result<DeleteResult>;
    async fn append_dataset_samples(
        &self,
        input: DatasetAppendInput,
    ) -> anyhow::Result<DatasetSummary>;
    async fn delete_dataset_samples_batch(
        &self,
        input: DatasetSampleBatchDeleteInput,
    ) -> anyhow::Result<DeleteResult>;
    async fn export_dataset(
        &self,
        input: DatasetExportInput,
    ) -> anyhow::Result<DatasetExportResult>;
    async fn validate_dataset_samples(
        &self,
        dataset_id: &str,
    ) -> anyhow::Result<DatasetValidationResult>;
}

#[allow(async_fn_in_trait)]
pub(crate) trait ProviderRepository {
    async fn list_providers(&self) -> anyhow::Result<Vec<ProviderSummary>>;
    async fn create_provider(&self, input: CreateProviderInput) -> anyhow::Result<ProviderSummary>;
    async fn update_provider(
        &self,
        provider_id: &str,
        input: UpdateProviderInput,
    ) -> anyhow::Result<ProviderSummary>;
    async fn delete_provider(&self, provider_id: &str) -> anyhow::Result<DeleteResult>;
    async fn list_provider_models(&self, provider_id: &str) -> anyhow::Result<Vec<ModelSummary>>;
    async fn get_provider_connection_config(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderConnectionConfig>;
    async fn find_provider_by_endpoint(
        &self,
        base_url: &str,
        interface_type: &str,
    ) -> anyhow::Result<Option<ProviderSummary>>;
    async fn update_provider_connection_status(
        &self,
        provider_id: &str,
        status: &str,
        checked_at: &str,
    ) -> anyhow::Result<()>;
    async fn replace_provider_models(
        &self,
        provider_id: &str,
        models: Vec<DiscoveredModel>,
        scanned_at: &str,
    ) -> anyhow::Result<Vec<ModelSummary>>;
    async fn save_provider_diagnostics(
        &self,
        result: &ProviderDiagnosticsResult,
    ) -> anyhow::Result<()>;
    async fn get_provider_diagnostics(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<Option<ProviderDiagnosticsResult>>;
}

#[allow(async_fn_in_trait)]
pub(crate) trait BenchmarkRepository {
    async fn create_task(
        &self,
        input: &BenchmarkStartInput,
    ) -> anyhow::Result<BenchmarkTaskSummary>;
    async fn update_task_finished(
        &self,
        task_id: &str,
        status: &str,
        success_rate: f64,
        p95_latency_ms: i64,
        goodput_qps: f64,
    ) -> anyhow::Result<()>;
    async fn insert_stage(&self, sample: &StageSample) -> anyhow::Result<()>;
    async fn insert_tick(&self, tick: &MetricsTick) -> anyhow::Result<()>;
    async fn insert_benchmark_error(&self, error: &BenchmarkErrorRecord) -> anyhow::Result<()>;
    async fn insert_request_log(&self, log: &BenchmarkRequestLogRecord) -> anyhow::Result<()>;
    async fn list_request_logs_page(
        &self,
        input: BenchmarkRequestLogPageInput,
    ) -> anyhow::Result<BenchmarkRequestLogPage>;
    async fn get_request_log_detail(
        &self,
        request_id: &str,
    ) -> anyhow::Result<BenchmarkRequestLogDetail>;
    async fn delete_request_logs(&self, task_id: &str) -> anyhow::Result<DeleteResult>;
    async fn get_task_summary(&self, task_id: &str) -> anyhow::Result<BenchmarkTaskSummary>;
    async fn list_running_tasks(&self) -> anyhow::Result<Vec<BenchmarkTaskSummary>>;
    async fn list_ticks(&self, task_id: &str) -> anyhow::Result<Vec<MetricsTick>>;
    async fn update_task_engine_mode(&self, task_id: &str, engine_mode: &str)
        -> anyhow::Result<()>;
    async fn update_task_preflight(
        &self,
        task_id: &str,
        preflight_result: Option<serde_json::Value>,
        diagnostics_snapshot: Option<ProviderDiagnosticsResult>,
    ) -> anyhow::Result<()>;
}

#[allow(async_fn_in_trait)]
pub(crate) trait ReportRepository {
    async fn generate_report(&self, task_id: &str) -> anyhow::Result<ReportSummary>;
    async fn list_reports(&self) -> anyhow::Result<Vec<ReportSummary>>;
    async fn get_report_detail(&self, report_id: &str) -> anyhow::Result<ReportDetail>;
}

#[allow(async_fn_in_trait)]
pub(crate) trait EndpointProbeRepository {
    async fn create_endpoint_probe_batch(
        &self,
        batch: EndpointProbeBatchRecord,
        runs: Vec<EndpointProbeRunRecord>,
    ) -> anyhow::Result<EndpointProbeBatchSummary>;
    async fn mark_endpoint_probe_run_started(&self, run_id: &str) -> anyhow::Result<()>;
    async fn finish_endpoint_probe_run(
        &self,
        record: EndpointProbeRunRecord,
    ) -> anyhow::Result<EndpointProbeRunSummary>;
    async fn finish_endpoint_probe_batch(
        &self,
        batch_id: &str,
        status: &str,
        finished_at: &str,
    ) -> anyhow::Result<EndpointProbeBatchSummary>;
    async fn list_endpoint_probe_batches_page(
        &self,
        input: EndpointProbeHistoryPageInput,
    ) -> anyhow::Result<EndpointProbeHistoryPage>;
    async fn get_endpoint_probe_batch_detail(
        &self,
        batch_id: &str,
    ) -> anyhow::Result<EndpointProbeBatchDetail>;
    async fn get_endpoint_probe_run_detail(
        &self,
        run_id: &str,
    ) -> anyhow::Result<EndpointProbeRunDetail>;
    async fn delete_endpoint_probe_batch(&self, batch_id: &str) -> anyhow::Result<DeleteResult>;
    async fn recover_endpoint_probe_batches(&self, message: &str) -> anyhow::Result<()>;
}
