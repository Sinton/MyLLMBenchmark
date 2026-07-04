use crate::config::{AppConfig, ConfigStore, ConfigUpdateResult};
use crate::data::AppDataSource;
use crate::db::Database;
use crate::domain::benchmark_sample::StageSample;
use crate::mock::MockDataStore;
use crate::models::{
    BenchmarkErrorRecord, BenchmarkStartInput, BenchmarkTaskSummary, CreateProviderInput,
    DashboardSummary, DatasetAppendInput, DatasetExportInput, DatasetExportResult,
    DatasetImportInput, DatasetSample, DatasetSampleBatchDeleteInput, DatasetSampleCreateInput,
    DatasetSamplePage, DatasetSamplePageInput, DatasetSamplePreview, DatasetSampleUpdateInput,
    DatasetSummary, DatasetUpdateInput, DatasetValidationResult, DeleteResult, DiscoveredModel,
    MetricsTick, ModelSummary, ProviderConnectionConfig, ProviderConnectionResult,
    ProviderDiagnosticsResult, ProviderModelScanResult, ProviderSummary, ReportDetail,
    ReportSummary, UpdateProviderInput,
};
use crate::tasks::TaskManager;
use std::path::PathBuf;
use tokio::sync::watch;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    config_store: ConfigStore,
    data: AppDataSource,
    tasks: TaskManager,
}

impl AppState {
    pub async fn initialize(config_dir: PathBuf, data_dir: PathBuf) -> anyhow::Result<Self> {
        let config_store = ConfigStore::new(config_dir);
        let config = config_store.load_or_create()?;
        let data = if config.uses_mock_data() {
            AppDataSource::Mock(MockDataStore::new())
        } else {
            AppDataSource::Sqlite(Database::initialize(&data_dir).await?)
        };
        Ok(Self {
            config,
            config_store,
            data,
            tasks: TaskManager::default(),
        })
    }

    pub fn current_config(&self) -> anyhow::Result<AppConfig> {
        self.config_store.load_or_create()
    }

    pub fn save_config(&self, config: AppConfig) -> anyhow::Result<ConfigUpdateResult> {
        self.config_store.save(&config)?;
        Ok(ConfigUpdateResult {
            config,
            restart_required: true,
        })
    }

    pub async fn dashboard_summary(&self) -> anyhow::Result<DashboardSummary> {
        self.data.dashboard_summary().await
    }

    pub async fn list_providers(&self) -> anyhow::Result<Vec<ProviderSummary>> {
        self.data.list_providers().await
    }

    pub async fn create_provider(
        &self,
        input: CreateProviderInput,
    ) -> anyhow::Result<ProviderSummary> {
        self.data.create_provider(input).await
    }

    pub async fn update_provider(
        &self,
        provider_id: &str,
        input: UpdateProviderInput,
    ) -> anyhow::Result<ProviderSummary> {
        self.data.update_provider(provider_id, input).await
    }

    pub async fn delete_provider(&self, provider_id: &str) -> anyhow::Result<DeleteResult> {
        self.data.delete_provider(provider_id).await
    }

    pub async fn test_provider_connection(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderConnectionResult> {
        self.data.test_provider_connection(provider_id).await
    }

    pub async fn list_provider_models(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<Vec<ModelSummary>> {
        self.data.list_provider_models(provider_id).await
    }

    pub async fn scan_provider_models(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderModelScanResult> {
        self.data.scan_provider_models(provider_id).await
    }

    pub async fn provider_connection_config(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderConnectionConfig> {
        self.data.get_provider_connection_config(provider_id).await
    }

    pub async fn update_provider_connection_status(
        &self,
        provider_id: &str,
        status: &str,
        checked_at: &str,
    ) -> anyhow::Result<()> {
        self.data
            .update_provider_connection_status(provider_id, status, checked_at)
            .await
    }

    pub async fn replace_provider_models(
        &self,
        provider_id: &str,
        models: Vec<DiscoveredModel>,
        scanned_at: &str,
    ) -> anyhow::Result<Vec<ModelSummary>> {
        self.data
            .replace_provider_models(provider_id, models, scanned_at)
            .await
    }

    pub async fn save_provider_diagnostics(
        &self,
        result: &ProviderDiagnosticsResult,
    ) -> anyhow::Result<()> {
        self.data.save_provider_diagnostics(result).await
    }

    pub async fn get_provider_diagnostics(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<Option<ProviderDiagnosticsResult>> {
        self.data.get_provider_diagnostics(provider_id).await
    }

    pub async fn list_datasets(&self) -> anyhow::Result<Vec<DatasetSummary>> {
        self.data.list_datasets().await
    }

    pub async fn import_dataset(
        &self,
        input: DatasetImportInput,
    ) -> anyhow::Result<DatasetSummary> {
        self.data.import_dataset(input).await
    }

    pub async fn update_dataset(
        &self,
        input: DatasetUpdateInput,
    ) -> anyhow::Result<DatasetSummary> {
        self.data.update_dataset(input).await
    }

    pub async fn delete_dataset(&self, dataset_id: &str) -> anyhow::Result<DeleteResult> {
        self.data.delete_dataset(dataset_id).await
    }

    pub async fn preview_dataset_samples(
        &self,
        dataset_id: &str,
        limit: i64,
    ) -> anyhow::Result<Vec<DatasetSamplePreview>> {
        self.data.preview_dataset_samples(dataset_id, limit).await
    }

    pub async fn list_dataset_samples_page(
        &self,
        input: DatasetSamplePageInput,
    ) -> anyhow::Result<DatasetSamplePage> {
        self.data.list_dataset_samples_page(input).await
    }

    pub async fn list_dataset_samples(
        &self,
        dataset_id: &str,
    ) -> anyhow::Result<Vec<DatasetSample>> {
        self.data.list_dataset_samples(dataset_id).await
    }

    pub async fn create_dataset_sample(
        &self,
        input: DatasetSampleCreateInput,
    ) -> anyhow::Result<DatasetSamplePreview> {
        self.data.create_dataset_sample(input).await
    }

    pub async fn update_dataset_sample(
        &self,
        input: DatasetSampleUpdateInput,
    ) -> anyhow::Result<DatasetSamplePreview> {
        self.data.update_dataset_sample(input).await
    }

    pub async fn delete_dataset_sample(&self, sample_id: &str) -> anyhow::Result<DeleteResult> {
        self.data.delete_dataset_sample(sample_id).await
    }

    pub async fn append_dataset_samples(
        &self,
        input: DatasetAppendInput,
    ) -> anyhow::Result<DatasetSummary> {
        self.data.append_dataset_samples(input).await
    }

    pub async fn delete_dataset_samples_batch(
        &self,
        input: DatasetSampleBatchDeleteInput,
    ) -> anyhow::Result<DeleteResult> {
        self.data.delete_dataset_samples_batch(input).await
    }

    pub async fn export_dataset(
        &self,
        input: DatasetExportInput,
    ) -> anyhow::Result<DatasetExportResult> {
        self.data.export_dataset(input).await
    }

    pub async fn validate_dataset_samples(
        &self,
        dataset_id: &str,
    ) -> anyhow::Result<DatasetValidationResult> {
        self.data.validate_dataset_samples(dataset_id).await
    }

    pub async fn create_task(
        &self,
        input: &BenchmarkStartInput,
    ) -> anyhow::Result<BenchmarkTaskSummary> {
        self.data.create_task(input).await
    }

    pub async fn update_task_finished(
        &self,
        task_id: &str,
        status: &str,
        success_rate: f64,
        p95_latency_ms: i64,
        goodput_qps: f64,
    ) -> anyhow::Result<()> {
        self.data
            .update_task_finished(task_id, status, success_rate, p95_latency_ms, goodput_qps)
            .await
    }

    pub async fn insert_stage(&self, sample: &StageSample) -> anyhow::Result<()> {
        self.data.insert_stage(sample).await
    }

    pub async fn insert_tick(&self, tick: &MetricsTick) -> anyhow::Result<()> {
        self.data.insert_tick(tick).await
    }

    pub async fn insert_benchmark_error(&self, error: &BenchmarkErrorRecord) -> anyhow::Result<()> {
        self.data.insert_benchmark_error(error).await
    }

    pub async fn get_task_summary(&self, task_id: &str) -> anyhow::Result<BenchmarkTaskSummary> {
        self.data.get_task_summary(task_id).await
    }

    pub async fn list_ticks(&self, task_id: &str) -> anyhow::Result<Vec<MetricsTick>> {
        self.data.list_ticks(task_id).await
    }

    pub async fn update_task_engine_mode(
        &self,
        task_id: &str,
        engine_mode: &str,
    ) -> anyhow::Result<()> {
        self.data
            .update_task_engine_mode(task_id, engine_mode)
            .await
    }

    pub async fn update_task_preflight(
        &self,
        task_id: &str,
        preflight_result: Option<serde_json::Value>,
        diagnostics_snapshot: Option<ProviderDiagnosticsResult>,
    ) -> anyhow::Result<()> {
        self.data
            .update_task_preflight(task_id, preflight_result, diagnostics_snapshot)
            .await
    }

    pub async fn generate_report(&self, task_id: &str) -> anyhow::Result<ReportSummary> {
        self.data.generate_report(task_id).await
    }

    pub async fn list_reports(&self) -> anyhow::Result<Vec<ReportSummary>> {
        self.data.list_reports().await
    }

    pub async fn get_report_detail(&self, report_id: &str) -> anyhow::Result<ReportDetail> {
        self.data.get_report_detail(report_id).await
    }

    pub async fn register_task(&self, task_id: String, tx: watch::Sender<bool>) {
        self.tasks.register(task_id, tx).await;
    }

    pub async fn stop_task(&self, task_id: &str) -> bool {
        self.tasks.stop(task_id).await
    }

    pub async fn remove_task(&self, task_id: &str) {
        self.tasks.remove(task_id).await;
    }
}
