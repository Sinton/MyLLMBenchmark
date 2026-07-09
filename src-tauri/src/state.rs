use crate::config::{AppConfig, ConfigStore, ConfigUpdateResult};
use crate::data::AppDataSource;
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
    DatasetUpdateInput, DatasetValidationResult, DeleteResult, DiscoveredModel, MetricsTick,
    ModelSummary, ProviderConnectionConfig, ProviderConnectionResult, ProviderDiagnosticsResult,
    ProviderModelScanResult, ProviderSummary, ReportDetail, ReportSummary, UpdateProviderInput,
};
use crate::tasks::TaskManager;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::{watch, RwLock};

#[derive(Clone)]
pub struct AppState {
    config: Arc<RwLock<AppConfig>>,
    config_store: ConfigStore,
    data: Arc<RwLock<AppDataSource>>,
    data_dir: PathBuf,
    tasks: TaskManager,
}

impl AppState {
    pub async fn initialize(config_dir: PathBuf, data_dir: PathBuf) -> anyhow::Result<Self> {
        let config_store = ConfigStore::new(config_dir);
        let config = config_store.load_or_create()?;
        let data = create_data_source(&config, &data_dir).await?;
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_store,
            data: Arc::new(RwLock::new(data)),
            data_dir,
            tasks: TaskManager::default(),
        })
    }

    pub async fn current_config(&self) -> anyhow::Result<AppConfig> {
        Ok(self.config.read().await.clone())
    }

    pub async fn save_config(&self, config: AppConfig) -> anyhow::Result<ConfigUpdateResult> {
        let current = self.config.read().await.clone();
        self.config_store.save(&config)?;

        if current.data_mode != config.data_mode {
            let data = create_data_source(&config, &self.data_dir).await?;
            *self.data.write().await = data;
        }

        *self.config.write().await = config.clone();
        Ok(ConfigUpdateResult {
            config,
            restart_required: false,
        })
    }

    async fn data_source(&self) -> AppDataSource {
        self.data.read().await.clone()
    }

    pub async fn dashboard_summary(&self) -> anyhow::Result<DashboardSummary> {
        self.data_source().await.dashboard_summary().await
    }

    pub async fn list_providers(&self) -> anyhow::Result<Vec<ProviderSummary>> {
        self.data_source().await.list_providers().await
    }

    pub async fn create_provider(
        &self,
        input: CreateProviderInput,
    ) -> anyhow::Result<ProviderSummary> {
        self.data_source().await.create_provider(input).await
    }

    pub async fn update_provider(
        &self,
        provider_id: &str,
        input: UpdateProviderInput,
    ) -> anyhow::Result<ProviderSummary> {
        self.data_source()
            .await
            .update_provider(provider_id, input)
            .await
    }

    pub async fn delete_provider(&self, provider_id: &str) -> anyhow::Result<DeleteResult> {
        self.data_source().await.delete_provider(provider_id).await
    }

    pub async fn test_provider_connection(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderConnectionResult> {
        self.data_source()
            .await
            .test_provider_connection(provider_id)
            .await
    }

    pub async fn list_provider_models(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<Vec<ModelSummary>> {
        self.data_source()
            .await
            .list_provider_models(provider_id)
            .await
    }

    pub async fn scan_provider_models(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderModelScanResult> {
        self.data_source()
            .await
            .scan_provider_models(provider_id)
            .await
    }

    pub async fn provider_connection_config(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderConnectionConfig> {
        self.data_source()
            .await
            .get_provider_connection_config(provider_id)
            .await
    }

    pub async fn update_provider_connection_status(
        &self,
        provider_id: &str,
        status: &str,
        checked_at: &str,
    ) -> anyhow::Result<()> {
        self.data_source()
            .await
            .update_provider_connection_status(provider_id, status, checked_at)
            .await
    }

    pub async fn replace_provider_models(
        &self,
        provider_id: &str,
        models: Vec<DiscoveredModel>,
        scanned_at: &str,
    ) -> anyhow::Result<Vec<ModelSummary>> {
        self.data_source()
            .await
            .replace_provider_models(provider_id, models, scanned_at)
            .await
    }

    pub async fn save_provider_diagnostics(
        &self,
        result: &ProviderDiagnosticsResult,
    ) -> anyhow::Result<()> {
        self.data_source()
            .await
            .save_provider_diagnostics(result)
            .await
    }

    pub async fn get_provider_diagnostics(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<Option<ProviderDiagnosticsResult>> {
        self.data_source()
            .await
            .get_provider_diagnostics(provider_id)
            .await
    }

    pub async fn list_datasets(&self) -> anyhow::Result<Vec<DatasetSummary>> {
        self.data_source().await.list_datasets().await
    }

    pub async fn import_dataset(
        &self,
        input: DatasetImportInput,
    ) -> anyhow::Result<DatasetSummary> {
        self.data_source().await.import_dataset(input).await
    }

    pub async fn update_dataset(
        &self,
        input: DatasetUpdateInput,
    ) -> anyhow::Result<DatasetSummary> {
        self.data_source().await.update_dataset(input).await
    }

    pub async fn delete_dataset(&self, dataset_id: &str) -> anyhow::Result<DeleteResult> {
        self.data_source().await.delete_dataset(dataset_id).await
    }

    pub async fn preview_dataset_samples(
        &self,
        dataset_id: &str,
        limit: i64,
    ) -> anyhow::Result<Vec<DatasetSamplePreview>> {
        self.data_source()
            .await
            .preview_dataset_samples(dataset_id, limit)
            .await
    }

    pub async fn list_dataset_samples_page(
        &self,
        input: DatasetSamplePageInput,
    ) -> anyhow::Result<DatasetSamplePage> {
        self.data_source()
            .await
            .list_dataset_samples_page(input)
            .await
    }

    pub async fn list_dataset_samples(
        &self,
        dataset_id: &str,
    ) -> anyhow::Result<Vec<DatasetSample>> {
        self.data_source()
            .await
            .list_dataset_samples(dataset_id)
            .await
    }

    pub async fn create_dataset_sample(
        &self,
        input: DatasetSampleCreateInput,
    ) -> anyhow::Result<DatasetSamplePreview> {
        self.data_source().await.create_dataset_sample(input).await
    }

    pub async fn update_dataset_sample(
        &self,
        input: DatasetSampleUpdateInput,
    ) -> anyhow::Result<DatasetSamplePreview> {
        self.data_source().await.update_dataset_sample(input).await
    }

    pub async fn delete_dataset_sample(&self, sample_id: &str) -> anyhow::Result<DeleteResult> {
        self.data_source()
            .await
            .delete_dataset_sample(sample_id)
            .await
    }

    pub async fn append_dataset_samples(
        &self,
        input: DatasetAppendInput,
    ) -> anyhow::Result<DatasetSummary> {
        self.data_source().await.append_dataset_samples(input).await
    }

    pub async fn delete_dataset_samples_batch(
        &self,
        input: DatasetSampleBatchDeleteInput,
    ) -> anyhow::Result<DeleteResult> {
        self.data_source()
            .await
            .delete_dataset_samples_batch(input)
            .await
    }

    pub async fn export_dataset(
        &self,
        input: DatasetExportInput,
    ) -> anyhow::Result<DatasetExportResult> {
        self.data_source().await.export_dataset(input).await
    }

    pub async fn validate_dataset_samples(
        &self,
        dataset_id: &str,
    ) -> anyhow::Result<DatasetValidationResult> {
        self.data_source()
            .await
            .validate_dataset_samples(dataset_id)
            .await
    }

    pub async fn create_task(
        &self,
        input: &BenchmarkStartInput,
    ) -> anyhow::Result<BenchmarkTaskSummary> {
        self.data_source().await.create_task(input).await
    }

    pub async fn update_task_finished(
        &self,
        task_id: &str,
        status: &str,
        success_rate: f64,
        p95_latency_ms: i64,
        goodput_qps: f64,
    ) -> anyhow::Result<()> {
        self.data_source()
            .await
            .update_task_finished(task_id, status, success_rate, p95_latency_ms, goodput_qps)
            .await
    }

    pub async fn insert_stage(&self, sample: &StageSample) -> anyhow::Result<()> {
        self.data_source().await.insert_stage(sample).await
    }

    pub async fn insert_tick(&self, tick: &MetricsTick) -> anyhow::Result<()> {
        self.data_source().await.insert_tick(tick).await
    }

    pub async fn insert_benchmark_error(&self, error: &BenchmarkErrorRecord) -> anyhow::Result<()> {
        self.data_source().await.insert_benchmark_error(error).await
    }

    pub async fn insert_request_log(
        &self,
        mut log: BenchmarkRequestLogRecord,
    ) -> anyhow::Result<()> {
        let data = self.data_source().await;
        if matches!(&data, AppDataSource::Sqlite(_)) && request_log_has_body(&log) {
            self.append_request_log_body(&log).await?;
            log.body_ref = Some(request_log_body_ref(&log.summary.task_id));
        }
        data.insert_request_log(&log).await
    }

    pub async fn list_request_logs_page(
        &self,
        input: BenchmarkRequestLogPageInput,
    ) -> anyhow::Result<BenchmarkRequestLogPage> {
        self.data_source().await.list_request_logs_page(input).await
    }

    pub async fn get_request_log_detail(
        &self,
        request_id: &str,
    ) -> anyhow::Result<BenchmarkRequestLogDetail> {
        let data = self.data_source().await;
        let mut detail = data.get_request_log_detail(request_id).await?;
        if detail.body_available
            && detail.prompt.is_none()
            && matches!(&data, AppDataSource::Sqlite(_))
        {
            if let Some(body) = self
                .read_request_log_body(&detail.summary.task_id, &detail.summary.id)
                .await?
            {
                detail.prompt = body.prompt;
                detail.response_text = body.response_text;
                detail.raw_error = body.raw_error.or(detail.raw_error);
                detail.raw_usage = body.raw_usage;
            } else {
                detail.body_available = false;
            }
        }
        Ok(detail)
    }

    pub async fn delete_request_logs(&self, task_id: &str) -> anyhow::Result<DeleteResult> {
        let result = self
            .data_source()
            .await
            .delete_request_logs(task_id)
            .await?;
        let path = self.request_log_body_path(task_id);
        match tokio::fs::remove_file(path).await {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        Ok(result)
    }

    pub async fn get_task_summary(&self, task_id: &str) -> anyhow::Result<BenchmarkTaskSummary> {
        self.data_source().await.get_task_summary(task_id).await
    }

    pub async fn list_ticks(&self, task_id: &str) -> anyhow::Result<Vec<MetricsTick>> {
        self.data_source().await.list_ticks(task_id).await
    }

    pub async fn update_task_engine_mode(
        &self,
        task_id: &str,
        engine_mode: &str,
    ) -> anyhow::Result<()> {
        self.data_source()
            .await
            .update_task_engine_mode(task_id, engine_mode)
            .await
    }

    pub async fn update_task_preflight(
        &self,
        task_id: &str,
        preflight_result: Option<serde_json::Value>,
        diagnostics_snapshot: Option<ProviderDiagnosticsResult>,
    ) -> anyhow::Result<()> {
        self.data_source()
            .await
            .update_task_preflight(task_id, preflight_result, diagnostics_snapshot)
            .await
    }

    pub async fn generate_report(&self, task_id: &str) -> anyhow::Result<ReportSummary> {
        self.data_source().await.generate_report(task_id).await
    }

    pub async fn list_reports(&self) -> anyhow::Result<Vec<ReportSummary>> {
        self.data_source().await.list_reports().await
    }

    pub async fn get_report_detail(&self, report_id: &str) -> anyhow::Result<ReportDetail> {
        self.data_source().await.get_report_detail(report_id).await
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

    async fn append_request_log_body(&self, log: &BenchmarkRequestLogRecord) -> anyhow::Result<()> {
        let dir = self.data_dir.join("request_logs");
        tokio::fs::create_dir_all(&dir).await?;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.request_log_body_path(&log.summary.task_id))
            .await?;
        let line = serde_json::to_string(&RequestLogBodyLine {
            id: log.summary.id.clone(),
            prompt: log.prompt.clone(),
            response_text: log.response_text.clone(),
            raw_error: log.raw_error.clone(),
            raw_usage: log.raw_usage.clone(),
        })?;
        file.write_all(line.as_bytes()).await?;
        file.write_all(b"\n").await?;
        Ok(())
    }

    async fn read_request_log_body(
        &self,
        task_id: &str,
        request_id: &str,
    ) -> anyhow::Result<Option<RequestLogBodyLine>> {
        let path = self.request_log_body_path(task_id);
        let content = match tokio::fs::read_to_string(path).await {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        for line in content.lines() {
            let Ok(body) = serde_json::from_str::<RequestLogBodyLine>(line) else {
                continue;
            };
            if body.id == request_id {
                return Ok(Some(body));
            }
        }
        Ok(None)
    }

    fn request_log_body_path(&self, task_id: &str) -> PathBuf {
        self.data_dir
            .join("request_logs")
            .join(format!("{task_id}.jsonl"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RequestLogBodyLine {
    id: String,
    prompt: Option<String>,
    response_text: Option<String>,
    raw_error: Option<String>,
    raw_usage: Option<serde_json::Value>,
}

fn request_log_has_body(log: &BenchmarkRequestLogRecord) -> bool {
    log.prompt.is_some()
        || log.response_text.is_some()
        || log.raw_error.is_some()
        || log.raw_usage.is_some()
}

fn request_log_body_ref(task_id: &str) -> String {
    format!("request_logs/{task_id}.jsonl")
}

async fn create_data_source(
    config: &AppConfig,
    data_dir: &PathBuf,
) -> anyhow::Result<AppDataSource> {
    if config.uses_mock_data() {
        Ok(AppDataSource::Mock(MockDataStore::new()))
    } else {
        Ok(AppDataSource::Sqlite(Database::initialize(data_dir).await?))
    }
}

#[cfg(test)]
mod tests {
    use super::AppState;
    use crate::config::{AppConfig, BenchmarkEngineMode, DataMode};
    use crate::models::CreateProviderInput;
    use uuid::Uuid;

    #[tokio::test]
    async fn config_update_switches_runtime_data_source_to_sqlite() {
        let root =
            std::env::temp_dir().join(format!("my-llm-benchmark-state-switch-{}", Uuid::new_v4()));
        let config_dir = root.join("config");
        let data_dir = root.join("data");
        let state = AppState::initialize(config_dir.clone(), data_dir.clone())
            .await
            .unwrap();

        assert_eq!(
            state.current_config().await.unwrap().data_mode,
            DataMode::Mock
        );

        let result = state
            .save_config(AppConfig {
                data_mode: DataMode::Sqlite,
                benchmark_engine: BenchmarkEngineMode::Mock,
            })
            .await
            .unwrap();
        assert!(!result.restart_required);
        assert_eq!(
            state.current_config().await.unwrap().data_mode,
            DataMode::Sqlite
        );

        let provider = state
            .create_provider(CreateProviderInput {
                name: "Runtime SQLite Provider".to_string(),
                base_url: "http://127.0.0.1:9000/v1".to_string(),
                api_key: Some("runtime-key".to_string()),
                interface_type: "OpenAI".to_string(),
            })
            .await
            .unwrap();

        let reloaded = AppState::initialize(config_dir, data_dir).await.unwrap();
        assert!(reloaded
            .list_providers()
            .await
            .unwrap()
            .iter()
            .any(|item| item.id == provider.id));

        let _ = std::fs::remove_dir_all(root);
    }
}
