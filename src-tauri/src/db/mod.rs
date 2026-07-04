use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;

mod benchmarks;
mod dashboard;
mod datasets;
mod migrations;
mod providers;
mod reports;
mod rows;
mod seed;

#[derive(Clone)]
pub struct Database {
    pub(in crate::db) pool: SqlitePool,
}

impl Database {
    pub async fn initialize(data_dir: &Path) -> anyhow::Result<Self> {
        std::fs::create_dir_all(data_dir)?;

        let db_path = data_dir.join("llmbench.db");
        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        let db = Self { pool };
        db.configure().await?;
        db.migrate().await?;
        db.seed_defaults().await?;
        Ok(db)
    }
}

pub(in crate::db) fn now() -> String {
    Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::Database;
    use crate::models::{
        BenchmarkStartInput, CreateProviderInput, DatasetImportInput, DatasetSampleCreateInput,
        DatasetSamplePageInput, DatasetSampleUpdateInput, DatasetUpdateInput, UpdateProviderInput,
    };
    use base64::prelude::*;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn temp_data_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("llmbench-{name}-{}", Uuid::new_v4()))
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

    #[tokio::test]
    async fn migrations_are_recorded_once_for_existing_database() {
        let data_dir = temp_data_dir("migration");
        let db = Database::initialize(&data_dir).await.unwrap();
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM schema_migrations;")
            .fetch_one(&db.pool)
            .await
            .unwrap();
        assert_eq!(count, 3);
        let latest: i64 = sqlx::query_scalar("SELECT MAX(version) FROM schema_migrations;")
            .fetch_one(&db.pool)
            .await
            .unwrap();
        assert_eq!(latest, 3);
        drop(db);

        let db = Database::initialize(&data_dir).await.unwrap();
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM schema_migrations;")
            .fetch_one(&db.pool)
            .await
            .unwrap();
        assert_eq!(count, 3);
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
    async fn provider_name_update_keeps_sqlite_state_key_and_models() {
        let data_dir = temp_data_dir("provider-name");
        let db = Database::initialize(&data_dir).await.unwrap();
        let provider = db.create_provider(provider_input()).await.unwrap();
        db.test_provider_connection(&provider.id).await.unwrap();
        let scanned = db.scan_provider_models(&provider.id).await.unwrap();
        assert!(!scanned.models.is_empty());

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
        assert_eq!(updated.model_count, scanned.models.len() as i64);
        assert_eq!(
            db.list_provider_models(&provider.id).await.unwrap().len(),
            scanned.models.len()
        );
        drop(db);

        let _ = std::fs::remove_dir_all(data_dir);
    }

    #[tokio::test]
    async fn provider_config_update_resets_sqlite_state_and_clears_models() {
        let data_dir = temp_data_dir("provider-config");
        let db = Database::initialize(&data_dir).await.unwrap();
        let provider = db.create_provider(provider_input()).await.unwrap();
        db.test_provider_connection(&provider.id).await.unwrap();
        let scanned = db.scan_provider_models(&provider.id).await.unwrap();
        assert!(!scanned.models.is_empty());

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
        let scanned = db.scan_provider_models(&provider.id).await.unwrap();
        let dataset = db.import_dataset(dataset_input()).await.unwrap();

        let task = db
            .create_task(&BenchmarkStartInput {
                provider_id: provider.id.clone(),
                model_id: Some(scanned.models[0].id.clone()),
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
}
