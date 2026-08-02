use crate::config::{AppConfig, ConfigStore, ConfigUpdateResult};
use crate::data::AppDataSource;
use crate::db::Database;
use crate::domain::benchmark_sample::StageSample;
use crate::error::AppError;
use crate::mock::MockDataStore;
use crate::models::{
    BenchmarkErrorRecord, BenchmarkRequestLogDetail, BenchmarkRequestLogPage,
    BenchmarkRequestLogPageInput, BenchmarkRequestLogRecord, BenchmarkStartInput,
    BenchmarkTaskSummary, CreateProviderInput, DashboardSummary, DatasetAppendInput,
    DatasetExportInput, DatasetExportResult, DatasetImportInput, DatasetSample,
    DatasetSampleBatchDeleteInput, DatasetSampleCreateInput, DatasetSamplePage,
    DatasetSamplePageInput, DatasetSamplePreview, DatasetSampleUpdateInput, DatasetSummary,
    DatasetUpdateInput, DatasetValidationResult, DeleteResult, DiscoveredModel, MetricsTick,
    ModelSummary, ProviderConnectionConfig, ProviderDiagnosticsResult, ProviderSummary,
    ReportDetail, ReportSummary, SiteProbeHistoryPage, SiteProbeHistoryPageInput,
    SiteProbeRunDetail, SiteProbeRunRecord, SiteProbeRunSummary, UpdateProviderInput,
};
use crate::storage::{
    RequestLogBodyLine, RequestLogBodyStore, SiteProbeBodyLine, SiteProbeBodyStore,
};
use crate::tasks::TaskManager;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{watch, RwLock};

#[derive(Clone)]
pub struct AppState {
    config: Arc<RwLock<AppConfig>>,
    config_store: ConfigStore,
    data: Arc<RwLock<AppDataSource>>,
    request_log_bodies: RequestLogBodyStore,
    site_probe_bodies: SiteProbeBodyStore,
    tasks: TaskManager,
}

impl AppState {
    pub async fn initialize(config_dir: PathBuf, data_dir: PathBuf) -> anyhow::Result<Self> {
        let config_store = ConfigStore::new(config_dir);
        let config = config_store.load_or_create()?;
        let data = create_data_source(&config, &data_dir).await?;
        let state = Self {
            config: Arc::new(RwLock::new(config)),
            config_store,
            data: Arc::new(RwLock::new(data)),
            request_log_bodies: RequestLogBodyStore::new(data_dir.clone()),
            site_probe_bodies: SiteProbeBodyStore::new(data_dir),
            tasks: TaskManager::default(),
        };
        state.recover_orphaned_running_tasks().await?;
        Ok(state)
    }

    pub async fn current_config(&self) -> anyhow::Result<AppConfig> {
        Ok(self.config.read().await.clone())
    }

    pub async fn save_config(&self, config: AppConfig) -> anyhow::Result<ConfigUpdateResult> {
        let current = self.config.read().await.clone();
        let switching_data_mode = current.data_mode != config.data_mode;
        let switching_engine = current.benchmark_engine != config.benchmark_engine;

        if (switching_data_mode || switching_engine) && self.has_running_tasks().await {
            let running = self.running_task_ids().await;
            return Err(AppError::validation(format!(
                "当前仍有 {} 个压测任务在运行（{}），请先停止后再切换数据源或压测引擎。",
                running.len(),
                running.join(", ")
            ))
            .into());
        }

        self.config_store.save(&config)?;

        if switching_data_mode {
            let data = create_data_source(&config, self.request_log_bodies.data_dir()).await?;
            *self.data.write().await = data;
            self.recover_orphaned_running_tasks().await?;
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

    pub async fn list_provider_models(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<Vec<ModelSummary>> {
        self.data_source()
            .await
            .list_provider_models(provider_id)
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
            self.request_log_bodies
                .append_body(
                    &log.summary.task_id,
                    &RequestLogBodyLine {
                        id: log.summary.id.clone(),
                        prompt: log.prompt.clone(),
                        response_text: log.response_text.clone(),
                        raw_error: log.raw_error.clone(),
                        raw_usage: log.raw_usage.clone(),
                    },
                )
                .await?;
            log.body_ref = Some(RequestLogBodyStore::body_ref(&log.summary.task_id));
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
                .request_log_bodies
                .read_body(&detail.summary.task_id, &detail.summary.id)
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
        self.request_log_bodies.delete_task_bodies(task_id).await?;
        Ok(result)
    }

    pub async fn insert_site_probe_run(
        &self,
        mut record: SiteProbeRunRecord,
    ) -> anyhow::Result<SiteProbeRunSummary> {
        let data = self.data_source().await;
        if matches!(&data, AppDataSource::Sqlite(_)) && site_probe_has_body(&record) {
            self.site_probe_bodies
                .write_body(
                    &record.summary.id,
                    &SiteProbeBodyLine {
                        id: record.summary.id.clone(),
                        prompt: record.prompt.clone(),
                        response_text: record.response_text.clone(),
                        request_payload: record.request_payload.clone(),
                        raw_error: record.raw_error.clone(),
                        raw_usage: record.raw_usage.clone(),
                    },
                )
                .await?;
            record.body_ref = Some(SiteProbeBodyStore::body_ref(&record.summary.id));
            record.summary.body_available = true;
        }
        data.insert_site_probe_run(record).await
    }

    pub async fn list_site_probe_runs_page(
        &self,
        input: SiteProbeHistoryPageInput,
    ) -> anyhow::Result<SiteProbeHistoryPage> {
        self.data_source()
            .await
            .list_site_probe_runs_page(input)
            .await
    }

    pub async fn get_site_probe_run_detail(
        &self,
        run_id: &str,
    ) -> anyhow::Result<SiteProbeRunDetail> {
        let data = self.data_source().await;
        let mut detail = data.get_site_probe_run_detail(run_id).await?;
        if detail.summary.body_available
            && detail.prompt.is_none()
            && matches!(&data, AppDataSource::Sqlite(_))
        {
            if let Some(body) = self.site_probe_bodies.read_body(&detail.summary.id).await? {
                detail.prompt = body.prompt;
                detail.response_text = body.response_text;
                detail.request_payload = body.request_payload;
                detail.raw_error = body.raw_error.or(detail.raw_error);
                detail.raw_usage = body.raw_usage;
            } else {
                detail.summary.body_available = false;
            }
        }
        Ok(detail)
    }

    pub async fn delete_site_probe_run(&self, run_id: &str) -> anyhow::Result<DeleteResult> {
        let result = self
            .data_source()
            .await
            .delete_site_probe_run(run_id)
            .await?;
        self.site_probe_bodies.delete_body(run_id).await?;
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

    pub async fn has_running_tasks(&self) -> bool {
        self.tasks.has_running_tasks().await
    }

    pub async fn running_task_ids(&self) -> Vec<String> {
        self.tasks.running_task_ids().await
    }

    async fn recover_orphaned_running_tasks(&self) -> anyhow::Result<()> {
        let data = self.data_source().await;
        let running = data.list_running_tasks().await?;
        for task in running {
            // After process restart, in-memory task registry is empty, so any
            // persisted "running" task is orphaned and must be closed.
            data.insert_benchmark_error(&BenchmarkErrorRecord {
                task_id: task.id.clone(),
                error_kind: "orphaned".to_string(),
                message: "应用重启后清理未完成任务".to_string(),
                count: 1,
            })
            .await?;
            data.update_task_finished(
                &task.id,
                "failed",
                task.success_rate,
                task.p95_latency_ms,
                task.goodput_qps,
            )
            .await?;
        }
        Ok(())
    }
}

fn request_log_has_body(log: &BenchmarkRequestLogRecord) -> bool {
    log.prompt.is_some()
        || log.response_text.is_some()
        || log.raw_error.is_some()
        || log.raw_usage.is_some()
}

fn site_probe_has_body(record: &SiteProbeRunRecord) -> bool {
    record.prompt.is_some()
        || record.response_text.is_some()
        || record.request_payload.is_some()
        || record.raw_error.is_some()
        || record.raw_usage.is_some()
}

async fn create_data_source(
    config: &AppConfig,
    data_dir: &std::path::Path,
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
    use crate::models::{BenchmarkStartInput, CreateProviderInput, DatasetImportInput};
    use base64::prelude::*;
    use tokio::sync::watch;
    use uuid::Uuid;

    fn temp_root(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("my-llm-benchmark-state-{name}-{}", Uuid::new_v4()))
    }

    #[tokio::test]
    async fn config_update_switches_runtime_data_source_to_sqlite() {
        let root = temp_root("switch");
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

    #[tokio::test]
    async fn initialize_marks_orphaned_running_tasks_as_failed() {
        let root = temp_root("recover");
        let config_dir = root.join("config");
        let data_dir = root.join("data");
        let state = AppState::initialize(config_dir.clone(), data_dir.clone())
            .await
            .unwrap();
        state
            .save_config(AppConfig {
                data_mode: DataMode::Sqlite,
                benchmark_engine: BenchmarkEngineMode::Mock,
            })
            .await
            .unwrap();

        let provider = state
            .create_provider(CreateProviderInput {
                name: "Recover Provider".to_string(),
                base_url: "http://127.0.0.1:8000/v1".to_string(),
                api_key: Some("key".to_string()),
                interface_type: "OpenAI".to_string(),
            })
            .await
            .unwrap();
        let dataset = state
            .import_dataset(DatasetImportInput {
                name: "Recover Dataset".to_string(),
                dataset_type: "Chat".to_string(),
                format: "JSONL".to_string(),
                file_name: "chat.jsonl".to_string(),
                content_base64: BASE64_STANDARD.encode("{\"prompt\":\"hello\"}"),
            })
            .await
            .unwrap();
        let task = state
            .create_task(&BenchmarkStartInput {
                provider_id: provider.id,
                model_id: None,
                dataset_id: dataset.id,
                mode: "fixed".to_string(),
                concurrency: 1,
                duration_seconds: 5,
                start_concurrency: None,
                end_concurrency: None,
                step_strategy: None,
                step_value: None,
                stage_sample_rounds: Some(1),
                stage_duration_seconds: None,
                warmup_rounds: Some(0),
                warmup_seconds: None,
                request_timeout_seconds: Some(30),
                sla_p95_ms: Some(5000),
                min_success_rate: Some(99.0),
                sla_stop_policy: None,
                workload_config: None,
                request_log_config: None,
            })
            .await
            .unwrap();
        assert_eq!(task.status, "running");

        let reloaded = AppState::initialize(config_dir, data_dir).await.unwrap();
        let recovered = reloaded.get_task_summary(&task.id).await.unwrap();
        assert_eq!(recovered.status, "failed");

        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn save_config_blocks_engine_switch_while_tasks_are_running() {
        let root = temp_root("block-switch");
        let state = AppState::initialize(root.join("config"), root.join("data"))
            .await
            .unwrap();
        let (tx, _rx) = watch::channel(false);
        state.register_task("task-running".to_string(), tx).await;

        let error = state
            .save_config(AppConfig {
                data_mode: DataMode::Mock,
                benchmark_engine: BenchmarkEngineMode::OpenaiCompatible,
            })
            .await
            .unwrap_err();
        let message = error.to_string();
        assert!(message.contains("仍有"));
        assert!(message.contains("压测任务在运行"));

        let _ = std::fs::remove_dir_all(root);
    }
}
