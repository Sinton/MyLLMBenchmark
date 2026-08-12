use super::Database;
use crate::domain::benchmark_sample::StageSample;
use crate::domain::model_catalog::{model_templates_for_interface, CatalogFlavor};
use crate::domain::model_type::default_capabilities;
use crate::models::{
    BenchmarkRequestLogRecord, BenchmarkRequestLogSummary, BenchmarkStartInput,
    CreateProviderInput, DatasetImportInput, DatasetSampleCreateInput, DatasetSamplePageInput,
    DatasetSampleUpdateInput, DatasetUpdateInput, DiscoveredModel, EndpointProbeBatchRecord,
    EndpointProbeBatchSummary, EndpointProbeHistoryPageInput, EndpointProbeRunRecord,
    EndpointProbeRunSummary, MetricsTick, ModelSummary, UpdateProviderInput,
};
use base64::prelude::*;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_data_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("my-llm-benchmark-{name}-{}", Uuid::new_v4()))
}

fn provider_input() -> CreateProviderInput {
    CreateProviderInput {
        name: "SQLite Provider".to_string(),
        base_url: "http://127.0.0.1:8000/v1".to_string(),
        api_key: Some("secret".to_string()),
        interface_type: "OpenAI".to_string(),
    }
}

fn dataset_input() -> DatasetImportInput {
    DatasetImportInput {
        name: "SQLite Chat Dataset".to_string(),
        dataset_type: "Chat".to_string(),
        format: "JSONL".to_string(),
        file_name: "chat.jsonl".to_string(),
        content_base64: BASE64_STANDARD
            .encode("{\"prompt\":\"介绍杭州\"}\n{\"prompt\":\"解释 Transformer\"}"),
    }
}

async fn replace_demo_models(
    db: &Database,
    provider_id: &str,
    interface_type: &str,
) -> Vec<ModelSummary> {
    let discovered = model_templates_for_interface(interface_type, CatalogFlavor::Demo)
        .into_iter()
        .map(|template| DiscoveredModel {
            name: template.name,
            model_type: template.model_type.clone(),
            capabilities: if template.capabilities.is_empty() {
                default_capabilities(&template.model_type)
            } else {
                template.capabilities
            },
            supports_streaming: template.supports_streaming,
            recommended_concurrency: template.recommended_concurrency,
        })
        .collect();
    db.replace_provider_models(provider_id, discovered, &chrono::Utc::now().to_rfc3339())
        .await
        .unwrap()
}

#[tokio::test]
async fn sqlite_initialization_creates_data_db_for_new_install() {
    let data_dir = temp_data_dir("db-name");
    let db = Database::initialize(&data_dir).await.unwrap();
    drop(db);

    assert!(data_dir.join("data.db").exists());
    assert!(!data_dir.join("llmbench.db").exists());

    let _ = std::fs::remove_dir_all(data_dir);
}

#[tokio::test]
async fn sqlite_initialization_migrates_legacy_database_filename() {
    let data_dir = temp_data_dir("db-name-migration");
    fs::create_dir_all(&data_dir).unwrap();
    let legacy_path = data_dir.join("llmbench.db");
    fs::write(&legacy_path, "").unwrap();
    fs::write(data_dir.join("llmbench.db-wal"), "").unwrap();
    fs::write(data_dir.join("llmbench.db-shm"), "").unwrap();

    let db = Database::initialize(&data_dir).await.unwrap();

    assert!(data_dir.join("data.db").exists());
    assert!(!data_dir.join("llmbench.db").exists());
    assert!(data_dir.join("data.db-wal").exists());
    assert!(data_dir.join("data.db-shm").exists());
    assert!(!data_dir.join("llmbench.db-wal").exists());
    assert!(!data_dir.join("llmbench.db-shm").exists());
    drop(db);

    let _ = std::fs::remove_dir_all(data_dir);
}

#[tokio::test]
async fn migrations_are_recorded_once_for_existing_database() {
    let data_dir = temp_data_dir("migration");
    let db = Database::initialize(&data_dir).await.unwrap();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM schema_migrations;")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(count, 5);
    let latest: i64 = sqlx::query_scalar("SELECT MAX(version) FROM schema_migrations;")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(latest, 6);
    drop(db);

    let db = Database::initialize(&data_dir).await.unwrap();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM schema_migrations;")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(count, 5);
    drop(db);

    let _ = std::fs::remove_dir_all(data_dir);
}

#[tokio::test]
async fn sqlite_seed_backfills_chat_samples_for_existing_database() {
    let data_dir = temp_data_dir("seed-backfill");
    let db = Database::initialize(&data_dir).await.unwrap();
    let dataset = db
        .list_datasets()
        .await
        .unwrap()
        .into_iter()
        .find(|item| item.name == "文本生成标准问答样本")
        .unwrap();
    let page = db
        .list_dataset_samples_page(DatasetSamplePageInput {
            dataset_id: dataset.id.clone(),
            page: 1,
            page_size: 50,
            keyword: None,
        })
        .await
        .unwrap();
    assert_eq!(page.total, 128);
    assert_eq!(page.items.len(), 50);

    sqlx::query("DELETE FROM dataset_samples WHERE dataset_id = ?;")
        .bind(&dataset.id)
        .execute(&db.pool)
        .await
        .unwrap();
    drop(db);

    let db = Database::initialize(&data_dir).await.unwrap();
    let page = db
        .list_dataset_samples_page(DatasetSamplePageInput {
            dataset_id: dataset.id.clone(),
            page: 1,
            page_size: 50,
            keyword: Some("容量".to_string()),
        })
        .await
        .unwrap();
    assert!(page.total > 0);
    assert!(page.items.iter().all(|item| item.prompt.contains("容量")));

    drop(db);
    let _ = std::fs::remove_dir_all(data_dir);
}

#[tokio::test]
async fn sqlite_seed_backfills_model_specific_samples_and_repairs_counts() {
    let data_dir = temp_data_dir("model-specific-seed-backfill");
    let db = Database::initialize(&data_dir).await.unwrap();

    for (dataset_type, legacy_count) in [("Embedding", 2048_i64), ("Reranker", 512), ("Vision", 96)]
    {
        let dataset = db
            .list_datasets()
            .await
            .unwrap()
            .into_iter()
            .find(|item| item.dataset_type == dataset_type)
            .unwrap();
        sqlx::query("DELETE FROM dataset_samples WHERE dataset_id = ?;")
            .bind(&dataset.id)
            .execute(&db.pool)
            .await
            .unwrap();
        sqlx::query("UPDATE datasets SET sample_count = ? WHERE id = ?;")
            .bind(legacy_count)
            .bind(&dataset.id)
            .execute(&db.pool)
            .await
            .unwrap();
    }
    drop(db);

    let db = Database::initialize(&data_dir).await.unwrap();
    for dataset in db
        .list_datasets()
        .await
        .unwrap()
        .into_iter()
        .filter(|item| {
            matches!(
                item.dataset_type.as_str(),
                "Embedding" | "Reranker" | "Vision"
            )
        })
    {
        let samples = db.list_dataset_samples(&dataset.id).await.unwrap();
        assert!(
            !samples.is_empty(),
            "{} should contain samples",
            dataset.name
        );
        assert_eq!(dataset.sample_count, samples.len() as i64);
    }

    drop(db);
    let _ = std::fs::remove_dir_all(data_dir);
}

#[tokio::test]
async fn sqlite_initialization_does_not_seed_demo_provider() {
    let data_dir = temp_data_dir("no-demo-provider");
    let db = Database::initialize(&data_dir).await.unwrap();

    let providers = db.list_providers().await.unwrap();
    assert!(providers.is_empty());

    drop(db);
    let _ = std::fs::remove_dir_all(data_dir);
}

#[tokio::test]
async fn provider_name_update_keeps_sqlite_state_key_and_models() {
    let data_dir = temp_data_dir("provider-name");
    let db = Database::initialize(&data_dir).await.unwrap();
    let provider = db.create_provider(provider_input()).await.unwrap();
    assert!(!serde_json::to_string(&provider).unwrap().contains("secret"));
    assert_eq!(
        db.get_provider_connection_config(&provider.id)
            .await
            .unwrap()
            .api_key_plaintext,
        "secret"
    );
    db.update_provider_connection_status(&provider.id, "online", &chrono::Utc::now().to_rfc3339())
        .await
        .unwrap();
    let scanned = replace_demo_models(&db, &provider.id, &provider.interface_type).await;
    assert!(!scanned.is_empty());

    let provider = db
        .list_providers()
        .await
        .unwrap()
        .into_iter()
        .find(|item| item.id == provider.id)
        .unwrap();
    let updated = db
        .update_provider(
            &provider.id,
            UpdateProviderInput {
                name: "Renamed SQLite Provider".to_string(),
                base_url: provider.base_url_masked.clone(),
                api_key: None,
                interface_type: provider.interface_type.clone(),
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.name, "Renamed SQLite Provider");
    assert_eq!(updated.status, provider.status);
    assert_eq!(updated.last_checked_at, provider.last_checked_at);
    assert_eq!(updated.api_key_masked, provider.api_key_masked);
    assert_eq!(updated.model_count, scanned.len() as i64);
    assert_eq!(
        db.list_provider_models(&provider.id).await.unwrap().len(),
        scanned.len()
    );
    drop(db);

    let _ = std::fs::remove_dir_all(data_dir);
}

#[tokio::test]
async fn provider_config_update_resets_sqlite_state_and_clears_models() {
    let data_dir = temp_data_dir("provider-config");
    let db = Database::initialize(&data_dir).await.unwrap();
    let provider = db.create_provider(provider_input()).await.unwrap();
    db.update_provider_connection_status(&provider.id, "online", &chrono::Utc::now().to_rfc3339())
        .await
        .unwrap();
    let scanned = replace_demo_models(&db, &provider.id, &provider.interface_type).await;
    assert!(!scanned.is_empty());

    let provider = db
        .list_providers()
        .await
        .unwrap()
        .into_iter()
        .find(|item| item.id == provider.id)
        .unwrap();
    let updated = db
        .update_provider(
            &provider.id,
            UpdateProviderInput {
                name: provider.name.clone(),
                base_url: provider.base_url_masked.clone(),
                api_key: Some(String::new()),
                interface_type: "Gemini".to_string(),
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.status, "unchecked");
    assert_eq!(updated.last_checked_at, None);
    assert_eq!(updated.api_key_masked, "未配置");
    assert_eq!(updated.model_count, 0);
    assert!(db
        .list_provider_models(&provider.id)
        .await
        .unwrap()
        .is_empty());
    drop(db);

    let _ = std::fs::remove_dir_all(data_dir);
}

#[tokio::test]
async fn provider_config_update_detaches_historical_task_models_before_clearing_cache() {
    let data_dir = temp_data_dir("provider-config-task-fk");
    let db = Database::initialize(&data_dir).await.unwrap();
    let provider = db.create_provider(provider_input()).await.unwrap();
    let scanned = replace_demo_models(&db, &provider.id, &provider.interface_type).await;
    let dataset = db.import_dataset(dataset_input()).await.unwrap();

    let task = db
        .create_task(&BenchmarkStartInput {
            provider_id: provider.id.clone(),
            model_id: Some(scanned[0].id.clone()),
            dataset_id: dataset.id.clone(),
            mode: "fixed".to_string(),
            concurrency: 1,
            duration_seconds: 5,
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
        })
        .await
        .unwrap();

    let updated = db
        .update_provider(
            &provider.id,
            UpdateProviderInput {
                name: provider.name.clone(),
                base_url: "http://127.0.0.1:9000/v1".to_string(),
                api_key: None,
                interface_type: provider.interface_type.clone(),
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.status, "unchecked");
    assert_eq!(updated.model_count, 0);
    assert!(db
        .list_provider_models(&provider.id)
        .await
        .unwrap()
        .is_empty());
    assert!(db.get_task_summary(&task.id).await.is_ok());

    let task_model_id: Option<String> =
        sqlx::query_scalar("SELECT model_id FROM benchmark_tasks WHERE id = ?;")
            .bind(&task.id)
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert_eq!(task_model_id, None);

    drop(db);
    let _ = std::fs::remove_dir_all(data_dir);
}

#[tokio::test]
async fn dataset_editing_lifecycle_updates_sqlite_stats_and_keeps_task_join() {
    let data_dir = temp_data_dir("dataset-editing");
    let db = Database::initialize(&data_dir).await.unwrap();
    let provider = db.create_provider(provider_input()).await.unwrap();
    let dataset = db.import_dataset(dataset_input()).await.unwrap();

    let preview = db.preview_dataset_samples(&dataset.id, 10).await.unwrap();
    assert_eq!(preview.len(), 2);
    assert_eq!(preview[0].prompt, "介绍杭州");

    let first_page = db
        .list_dataset_samples_page(DatasetSamplePageInput {
            dataset_id: dataset.id.clone(),
            page: 1,
            page_size: 20,
            keyword: None,
        })
        .await
        .unwrap();
    assert_eq!(first_page.total, 2);
    assert_eq!(first_page.items.len(), 2);

    let filtered_page = db
        .list_dataset_samples_page(DatasetSamplePageInput {
            dataset_id: dataset.id.clone(),
            page: 1,
            page_size: 20,
            keyword: Some("Transformer".to_string()),
        })
        .await
        .unwrap();
    assert_eq!(filtered_page.total, 1);
    assert!(filtered_page.items[0].prompt.contains("Transformer"));

    let empty_page = db
        .list_dataset_samples_page(DatasetSamplePageInput {
            dataset_id: dataset.id.clone(),
            page: 99,
            page_size: 20,
            keyword: None,
        })
        .await
        .unwrap();
    assert_eq!(empty_page.total, 2);
    assert!(empty_page.items.is_empty());

    let updated = db
        .update_dataset(DatasetUpdateInput {
            id: dataset.id.clone(),
            name: "Renamed SQLite Dataset".to_string(),
            dataset_type: "Chat".to_string(),
        })
        .await
        .unwrap();
    assert_eq!(updated.name, "Renamed SQLite Dataset");

    let created = db
        .create_dataset_sample(DatasetSampleCreateInput {
            dataset_id: dataset.id.clone(),
            prompt: "新增一条真实压测 Prompt".to_string(),
        })
        .await
        .unwrap();
    assert_eq!(
        db.list_datasets()
            .await
            .unwrap()
            .into_iter()
            .find(|item| item.id == dataset.id)
            .unwrap()
            .sample_count,
        3
    );

    let edited = db
        .update_dataset_sample(DatasetSampleUpdateInput {
            sample_id: created.id.clone(),
            prompt: "修改后的真实 Prompt".to_string(),
        })
        .await
        .unwrap();
    assert_eq!(edited.prompt, "修改后的真实 Prompt");

    let task = db
        .create_task(&BenchmarkStartInput {
            provider_id: provider.id.clone(),
            model_id: None,
            dataset_id: dataset.id.clone(),
            mode: "fixed".to_string(),
            concurrency: 1,
            duration_seconds: 5,
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
        })
        .await
        .unwrap();

    assert!(db.delete_dataset_sample(&created.id).await.unwrap().deleted);
    assert!(db.delete_dataset(&dataset.id).await.unwrap().deleted);
    assert!(!db
        .list_datasets()
        .await
        .unwrap()
        .iter()
        .any(|item| item.id == dataset.id));
    assert_eq!(
        db.get_task_summary(&task.id).await.unwrap().dataset_name,
        "Renamed SQLite Dataset"
    );

    drop(db);
    let _ = std::fs::remove_dir_all(data_dir);
}

#[tokio::test]
async fn request_logs_page_and_report_meta_are_available_in_sqlite() {
    let data_dir = temp_data_dir("request-logs");
    let db = Database::initialize(&data_dir).await.unwrap();
    let provider = db.create_provider(provider_input()).await.unwrap();
    let dataset = db.import_dataset(dataset_input()).await.unwrap();
    let task = db
        .create_task(&BenchmarkStartInput {
            provider_id: provider.id.clone(),
            model_id: None,
            dataset_id: dataset.id.clone(),
            mode: "fixed".to_string(),
            concurrency: 2,
            duration_seconds: 5,
            start_concurrency: None,
            end_concurrency: None,
            step_strategy: None,
            step_value: None,
            stage_sample_rounds: Some(2),
            stage_duration_seconds: None,
            warmup_rounds: Some(0),
            warmup_seconds: None,
            request_timeout_seconds: Some(120),
            sla_p95_ms: Some(5000),
            min_success_rate: Some(99.0),
            sla_stop_policy: None,
            workload_config: None,
            request_log_config: None,
        })
        .await
        .unwrap();
    let tick = MetricsTick {
        task_id: task.id.clone(),
        elapsed_seconds: 1,
        qps: 2.0,
        latency_ms: 820,
        ttft_ms: 180,
        tps: 24.0,
        success_rate: 50.0,
        errors: 1,
        in_flight: 2,
        request_count: 2,
        success_count: 1,
        failure_count: 1,
        input_tokens: 32,
        output_tokens: 48,
        total_tokens: 80,
        batch_size: 0,
        text_count: 0,
        documents_per_query: 0,
        pair_count: 0,
        image_count: 0,
    };
    db.insert_tick(&tick).await.unwrap();
    db.insert_stage(&StageSample::from_tick_with_evidence(
        1,
        2,
        &tick,
        2,
        0,
        false,
        Some("成功率未达到 SLA".to_string()),
    ))
    .await
    .unwrap();
    db.insert_request_log(&request_log_record(
        &task.id,
        1,
        "success",
        Some("介绍杭州的核心产业"),
        Some("杭州核心产业包括数字经济与先进制造。"),
        None,
        Some("request_logs/test.jsonl"),
    ))
    .await
    .unwrap();
    db.insert_request_log(&request_log_record(
        &task.id,
        2,
        "failed",
        Some("解释 Transformer"),
        None,
        Some("timeout"),
        None,
    ))
    .await
    .unwrap();

    let failed_page = db
        .list_request_logs_page(crate::models::BenchmarkRequestLogPageInput {
            task_id: task.id.clone(),
            page: 1,
            page_size: 20,
            stage_index: Some(1),
            status: Some("failed".to_string()),
            keyword: Some("timeout".to_string()),
        })
        .await
        .unwrap();
    assert_eq!(failed_page.total, 1);
    assert_eq!(failed_page.items[0].request_index, 2);

    db.update_task_finished(&task.id, "completed", 50.0, 820, 2.0)
        .await
        .unwrap();
    let report = db.generate_report(&task.id).await.unwrap();
    let detail = db.get_report_detail(&report.id).await.unwrap();
    assert!(detail.request_log_meta.enabled);
    assert_eq!(detail.request_log_meta.total_records, 2);
    assert_eq!(detail.request_log_meta.body_records, 1);

    drop(db);
    let _ = std::fs::remove_dir_all(data_dir);
}

#[tokio::test]
async fn endpoint_probe_batches_and_runs_are_available_in_sqlite() {
    let data_dir = temp_data_dir("endpoint-probe");
    let db = Database::initialize(&data_dir).await.unwrap();
    let (batch, runs) = endpoint_probe_batch_fixture();
    let batch_id = batch.summary.id.clone();
    let passed_id = runs[0].summary.id.clone();
    let failed_id = runs[1].summary.id.clone();

    let created = db.create_endpoint_probe_batch(&batch, &runs).await.unwrap();
    assert_eq!(created.total_runs, 2);
    assert_eq!(created.pending_runs, 2);
    assert!(
        !db.delete_endpoint_probe_batch(&batch_id)
            .await
            .unwrap()
            .deleted
    );

    db.mark_endpoint_probe_run_started(&passed_id)
        .await
        .unwrap();
    let mut passed = runs[0].clone();
    passed.summary.status = "passed".to_string();
    passed.summary.latency_ms = 320;
    passed.summary.ttft_ms = 80;
    passed.summary.input_tokens = 8;
    passed.summary.output_tokens = 12;
    passed.summary.total_tokens = 20;
    passed.summary.response_preview = Some("测活成功".to_string());
    passed.summary.finished_at = Some(chrono::Utc::now().to_rfc3339());
    passed.body_ref = Some("endpoint_probe_bodies/passed.jsonl".to_string());
    db.finish_endpoint_probe_run(&passed).await.unwrap();

    db.mark_endpoint_probe_run_started(&failed_id)
        .await
        .unwrap();
    let mut failed = runs[1].clone();
    failed.summary.status = "failed".to_string();
    failed.summary.error_kind = Some("http_4xx".to_string());
    failed.summary.error_message = Some("HTTP 401".to_string());
    failed.summary.response_preview = Some("HTTP 401".to_string());
    failed.summary.finished_at = Some(chrono::Utc::now().to_rfc3339());
    db.finish_endpoint_probe_run(&failed).await.unwrap();

    db.finish_endpoint_probe_batch(&batch_id, "completed", &chrono::Utc::now().to_rfc3339())
        .await
        .unwrap();

    let failed_page = db
        .list_endpoint_probe_batches_page(EndpointProbeHistoryPageInput {
            page: 1,
            page_size: 20,
            status: Some("completed".to_string()),
            keyword: Some("401".to_string()),
        })
        .await
        .unwrap();
    assert_eq!(failed_page.total, 1);
    assert_eq!(failed_page.items[0].id, batch_id);
    assert_eq!(failed_page.items[0].passed_runs, 1);
    assert_eq!(failed_page.items[0].failed_runs, 1);

    let detail = db.get_endpoint_probe_run_detail(&passed_id).await.unwrap();
    assert!(detail.summary.body_available);
    assert_eq!(detail.summary.response_preview.as_deref(), Some("测活成功"));

    let deleted = db.delete_endpoint_probe_batch(&batch_id).await.unwrap();
    assert!(deleted.deleted);
    assert!(db.get_endpoint_probe_run_detail(&failed_id).await.is_err());

    drop(db);
    let _ = std::fs::remove_dir_all(data_dir);
}

#[tokio::test]
async fn endpoint_probe_recovery_marks_orphaned_batches_and_runs_failed() {
    let data_dir = temp_data_dir("endpoint-probe-recovery");
    let db = Database::initialize(&data_dir).await.unwrap();
    let (batch, runs) = endpoint_probe_batch_fixture();
    let batch_id = batch.summary.id.clone();
    db.create_endpoint_probe_batch(&batch, &runs).await.unwrap();
    db.mark_endpoint_probe_run_started(&runs[0].summary.id)
        .await
        .unwrap();

    db.recover_endpoint_probe_batches("restart cleanup")
        .await
        .unwrap();

    let detail = db.get_endpoint_probe_batch_detail(&batch_id).await.unwrap();
    assert_eq!(detail.summary.status, "failed");
    assert_eq!(detail.summary.failed_runs, 2);
    assert!(detail.runs.iter().all(|run| run.status == "failed"));
    assert!(detail
        .runs
        .iter()
        .all(|run| run.error_kind.as_deref() == Some("orphaned")));

    drop(db);
    let _ = std::fs::remove_dir_all(data_dir);
}

#[tokio::test]
async fn legacy_site_probe_rows_migrate_into_single_run_batches() {
    let data_dir = temp_data_dir("endpoint-probe-migration");
    let db = Database::initialize(&data_dir).await.unwrap();
    sqlx::query(
        "CREATE TABLE site_probe_runs (
            id TEXT PRIMARY KEY, name TEXT NOT NULL, base_url TEXT NOT NULL,
            interface_type TEXT NOT NULL, model TEXT NOT NULL, status TEXT NOT NULL,
            latency_ms INTEGER NOT NULL, ttft_ms INTEGER NOT NULL,
            input_tokens INTEGER NOT NULL, output_tokens INTEGER NOT NULL,
            total_tokens INTEGER NOT NULL, error_kind TEXT, error_message TEXT,
            prompt_preview TEXT, response_preview TEXT, body_ref TEXT,
            created_at TEXT NOT NULL
        );",
    )
    .execute(&db.pool)
    .await
    .unwrap();
    let legacy_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO site_probe_runs
         (id, name, base_url, interface_type, model, status, latency_ms, ttft_ms,
          input_tokens, output_tokens, total_tokens, prompt_preview, response_preview,
          body_ref, created_at)
         VALUES (?, 'Legacy gateway', 'http://127.0.0.1:3000/v1', 'OpenAI',
                 'legacy-model', 'passed', 300, 70, 8, 12, 20, 'hello', 'ok',
                 'site_probe_bodies/legacy.jsonl', '2026-08-05T00:00:00Z');",
    )
    .bind(&legacy_id)
    .execute(&db.pool)
    .await
    .unwrap();
    drop(db);

    let db = Database::initialize(&data_dir).await.unwrap();
    let detail = db
        .get_endpoint_probe_batch_detail(&legacy_id)
        .await
        .unwrap();
    assert_eq!(detail.summary.id, legacy_id);
    assert_eq!(detail.summary.total_runs, 1);
    assert_eq!(detail.summary.status, "completed");
    assert_eq!(detail.runs[0].source_type, "temporary");
    assert_eq!(detail.runs[0].status, "passed");
    assert!(detail.runs[0].body_available);
    let legacy_table: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'site_probe_runs';",
    )
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(legacy_table, 0);

    drop(db);
    let _ = std::fs::remove_dir_all(data_dir);
}

fn request_log_record(
    task_id: &str,
    request_index: i64,
    status: &str,
    prompt: Option<&str>,
    response: Option<&str>,
    error_kind: Option<&str>,
    body_ref: Option<&str>,
) -> BenchmarkRequestLogRecord {
    BenchmarkRequestLogRecord {
        summary: BenchmarkRequestLogSummary {
            id: Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            stage_index: 1,
            request_index,
            sample_index: request_index,
            status: status.to_string(),
            latency_ms: 820,
            ttft_ms: if status == "success" { 180 } else { 0 },
            input_tokens: 16,
            output_tokens: if status == "success" { 24 } else { 0 },
            total_tokens: if status == "success" { 40 } else { 16 },
            error_kind: error_kind.map(ToString::to_string),
            prompt_preview: prompt.map(ToString::to_string),
            response_preview: response.map(ToString::to_string),
            created_at: chrono::Utc::now().to_rfc3339(),
        },
        body_ref: body_ref.map(ToString::to_string),
        prompt: prompt.map(ToString::to_string),
        response_text: response.map(ToString::to_string),
        raw_error: error_kind.map(ToString::to_string),
        raw_usage: None,
    }
}

fn endpoint_probe_batch_fixture() -> (EndpointProbeBatchRecord, Vec<EndpointProbeRunRecord>) {
    let batch_id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();
    let batch = EndpointProbeBatchRecord {
        summary: EndpointProbeBatchSummary {
            id: batch_id.clone(),
            name: "Batch probe".to_string(),
            status: "running".to_string(),
            total_runs: 2,
            pending_runs: 2,
            running_runs: 0,
            passed_runs: 0,
            failed_runs: 0,
            cancelled_runs: 0,
            streaming: true,
            max_output_tokens: 1024,
            timeout_seconds: 60,
            save_body: true,
            concurrency: 2,
            prompt_preview: Some("请回复测活成功".to_string()),
            created_at: created_at.clone(),
            finished_at: None,
        },
    };
    let runs = ["test-model", "failing-model"]
        .into_iter()
        .map(|model| EndpointProbeRunRecord {
            summary: EndpointProbeRunSummary {
                id: Uuid::new_v4().to_string(),
                batch_id: batch_id.clone(),
                source_type: "temporary".to_string(),
                provider_id: None,
                name: "new-api gateway".to_string(),
                base_url: "http://127.0.0.1:3000/v1".to_string(),
                interface_type: "OpenAI".to_string(),
                model: model.to_string(),
                status: "pending".to_string(),
                latency_ms: 0,
                ttft_ms: 0,
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 0,
                error_kind: None,
                error_message: None,
                prompt_preview: Some("请回复测活成功".to_string()),
                response_preview: None,
                body_available: false,
                created_at: created_at.clone(),
                finished_at: None,
            },
            body_ref: None,
            prompt: None,
            response_text: None,
            request_payload: None,
            raw_error: None,
            raw_usage: None,
        })
        .collect();
    (batch, runs)
}
