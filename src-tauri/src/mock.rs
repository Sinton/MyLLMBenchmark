use crate::models::{ModelSummary, ProviderSummary};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;

mod benchmarks;
mod dashboard;
mod datasets;
mod providers;
mod reports;
mod seed;
mod site_probe;
mod types;

use seed::seed_mock_data;
use types::MockData;

#[derive(Clone)]
pub struct MockDataStore {
    pub(in crate::mock) inner: Arc<RwLock<MockData>>,
}

impl MockDataStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(seed_mock_data())),
        }
    }
}

pub(in crate::mock) fn with_model_count(
    provider: &ProviderSummary,
    models: &[ModelSummary],
) -> ProviderSummary {
    let mut provider = provider.clone();
    provider.model_count = models
        .iter()
        .filter(|model| model.provider_id == provider.id)
        .count() as i64;
    provider
}

pub(in crate::mock) fn resolve_model(
    models: &[ModelSummary],
    provider_id: &str,
    model_id: Option<&str>,
) -> Option<ModelSummary> {
    if let Some(model_id) = model_id.filter(|id| !id.is_empty()) {
        return models.iter().find(|model| model.id == model_id).cloned();
    }
    models
        .iter()
        .find(|model| model.provider_id == provider_id)
        .cloned()
}

pub(in crate::mock) fn now() -> String {
    Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::MockDataStore;
    use crate::domain::model_catalog::{model_templates_for_interface, CatalogFlavor};
    use crate::domain::model_type::default_capabilities;
    use crate::error::AppError;
    use crate::models::{
        BenchmarkStartInput, CreateProviderInput, DatasetSampleCreateInput, DatasetSamplePageInput,
        DatasetSampleUpdateInput, DatasetUpdateInput, DiscoveredModel, ModelSummary,
        UpdateProviderInput,
    };

    fn benchmark_input() -> BenchmarkStartInput {
        BenchmarkStartInput {
            provider_id: "mock-provider-openai".to_string(),
            model_id: None,
            dataset_id: "mock-dataset-chat".to_string(),
            mode: "fixed".to_string(),
            concurrency: 8,
            duration_seconds: 20,
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

    async fn replace_demo_models(
        store: &MockDataStore,
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
        store
            .replace_provider_models(provider_id, discovered, &chrono::Utc::now().to_rfc3339())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn provider_create_and_delete_are_available_after_split() {
        let store = MockDataStore::new();
        let provider = store
            .create_provider(CreateProviderInput {
                name: "Local vLLM".to_string(),
                base_url: "http://127.0.0.1:8000/v1".to_string(),
                api_key: Some("secret".to_string()),
                interface_type: "OpenAI".to_string(),
            })
            .await
            .unwrap();

        assert!(store
            .list_providers()
            .await
            .unwrap()
            .iter()
            .any(|item| item.id == provider.id));

        let deleted = store.delete_provider(&provider.id).await.unwrap();
        assert!(deleted.deleted);
        assert!(!store
            .list_providers()
            .await
            .unwrap()
            .iter()
            .any(|item| item.id == provider.id));
    }

    #[tokio::test]
    async fn provider_name_update_keeps_connection_state_key_and_models() {
        let store = MockDataStore::new();
        let provider = store
            .list_providers()
            .await
            .unwrap()
            .into_iter()
            .find(|item| item.id == "mock-provider-openai")
            .unwrap();
        let scanned = replace_demo_models(&store, &provider.id, &provider.interface_type).await;
        assert!(!scanned.is_empty());
        let provider = store
            .list_providers()
            .await
            .unwrap()
            .into_iter()
            .find(|item| item.id == provider.id)
            .unwrap();

        let updated = store
            .update_provider(
                &provider.id,
                UpdateProviderInput {
                    name: "Renamed Mock Provider".to_string(),
                    base_url: provider.base_url_masked.clone(),
                    api_key: Some(provider.api_key_masked.clone()),
                    interface_type: provider.interface_type.clone(),
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.name, "Renamed Mock Provider");
        assert_eq!(updated.status, provider.status);
        assert_eq!(updated.last_checked_at, provider.last_checked_at);
        assert_eq!(updated.api_key_masked, provider.api_key_masked);
        assert_eq!(updated.model_count, scanned.len() as i64);
        assert_eq!(
            store
                .list_provider_models(&provider.id)
                .await
                .unwrap()
                .len(),
            scanned.len()
        );
    }

    #[tokio::test]
    async fn provider_config_update_resets_state_and_clears_models() {
        let store = MockDataStore::new();
        let provider = store
            .create_provider(CreateProviderInput {
                name: "Configurable Provider".to_string(),
                base_url: "http://127.0.0.1:8000/v1".to_string(),
                api_key: Some("secret".to_string()),
                interface_type: "OpenAI".to_string(),
            })
            .await
            .unwrap();
        store
            .update_provider_connection_status(
                &provider.id,
                "online",
                &chrono::Utc::now().to_rfc3339(),
            )
            .await
            .unwrap();
        let scanned = replace_demo_models(&store, &provider.id, &provider.interface_type).await;
        assert!(!scanned.is_empty());

        let updated = store
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
        assert!(store
            .list_provider_models(&provider.id)
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn completed_task_can_generate_report() {
        let store = MockDataStore::new();
        let task = store.create_task(&benchmark_input()).await.unwrap();
        store
            .update_task_finished(&task.id, "completed", 99.9, 1800, 12.4)
            .await
            .unwrap();

        let report = store.generate_report(&task.id).await.unwrap();
        let detail = store.get_report_detail(&report.id).await.unwrap();

        assert_eq!(detail.summary.id, report.id);
        assert_eq!(detail.summary.task_id, task.id);
    }

    #[tokio::test]
    async fn running_task_report_generation_is_invalid_state() {
        let store = MockDataStore::new();
        let task = store.create_task(&benchmark_input()).await.unwrap();

        let error = store.generate_report(&task.id).await.unwrap_err();
        let app_error = error.downcast_ref::<AppError>().unwrap();

        assert!(matches!(app_error, AppError::InvalidTaskState(_)));
    }

    #[tokio::test]
    async fn missing_task_and_report_are_not_found() {
        let store = MockDataStore::new();

        let task_error = store.get_task_summary("missing-task").await.unwrap_err();
        let report_error = store.get_report_detail("missing-report").await.unwrap_err();

        assert!(matches!(
            task_error.downcast_ref::<AppError>().unwrap(),
            AppError::NotFound(resource) if resource == "task"
        ));
        assert!(matches!(
            report_error.downcast_ref::<AppError>().unwrap(),
            AppError::NotFound(resource) if resource == "report"
        ));
    }

    #[tokio::test]
    async fn dataset_editing_lifecycle_updates_mock_stats_and_keeps_task_snapshot() {
        let store = MockDataStore::new();
        let task = store.create_task(&benchmark_input()).await.unwrap();
        assert_eq!(
            store
                .preview_dataset_samples("mock-dataset-chat", 50)
                .await
                .unwrap()
                .len(),
            50
        );
        assert_eq!(
            store
                .preview_dataset_samples("mock-dataset-chat", 0)
                .await
                .unwrap()
                .len(),
            128
        );

        let updated = store
            .update_dataset(DatasetUpdateInput {
                id: "mock-dataset-chat".to_string(),
                name: "Renamed Chat Dataset".to_string(),
                dataset_type: "Chat".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(updated.name, "Renamed Chat Dataset");

        let created = store
            .create_dataset_sample(DatasetSampleCreateInput {
                dataset_id: "mock-dataset-chat".to_string(),
                prompt: "新增一条政企售前压测 Prompt".to_string(),
            })
            .await
            .unwrap();
        assert!(created.prompt.contains("政企售前"));

        let edited = store
            .update_dataset_sample(DatasetSampleUpdateInput {
                sample_id: created.id.clone(),
                prompt: "修改后的 Prompt 样本".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(edited.prompt, "修改后的 Prompt 样本");

        let deleted_sample = store.delete_dataset_sample(&created.id).await.unwrap();
        assert!(deleted_sample.deleted);

        let deleted_dataset = store.delete_dataset("mock-dataset-chat").await.unwrap();
        assert!(deleted_dataset.deleted);
        assert!(!store
            .list_datasets()
            .await
            .unwrap()
            .iter()
            .any(|dataset| dataset.id == "mock-dataset-chat"));
        assert_eq!(
            store.get_task_summary(&task.id).await.unwrap().dataset_name,
            "Chat Prompt Mock Set"
        );
    }

    #[tokio::test]
    async fn dataset_samples_page_filters_and_paginates_mock_samples() {
        let store = MockDataStore::new();

        let first_page = store
            .list_dataset_samples_page(DatasetSamplePageInput {
                dataset_id: "mock-dataset-chat".to_string(),
                page: 1,
                page_size: 50,
                keyword: None,
            })
            .await
            .unwrap();
        assert_eq!(first_page.total, 128);
        assert_eq!(first_page.items.len(), 50);
        assert_eq!(first_page.items[0].sample_index, 0);

        let third_page = store
            .list_dataset_samples_page(DatasetSamplePageInput {
                dataset_id: "mock-dataset-chat".to_string(),
                page: 3,
                page_size: 50,
                keyword: None,
            })
            .await
            .unwrap();
        assert_eq!(third_page.items.len(), 28);
        assert_eq!(third_page.items[0].sample_index, 100);

        let filtered = store
            .list_dataset_samples_page(DatasetSamplePageInput {
                dataset_id: "mock-dataset-chat".to_string(),
                page: 1,
                page_size: 20,
                keyword: Some("容量".to_string()),
            })
            .await
            .unwrap();
        assert!(filtered.total > 0);
        assert!(filtered
            .items
            .iter()
            .all(|sample| sample.prompt.contains("容量")));
    }

    #[tokio::test]
    async fn seeded_mock_dataset_summaries_match_materialized_samples() {
        let store = MockDataStore::new();

        for dataset in store.list_datasets().await.unwrap() {
            let samples = store.list_dataset_samples(&dataset.id).await.unwrap();
            assert!(
                !samples.is_empty(),
                "{} should contain samples",
                dataset.name
            );
            assert_eq!(dataset.sample_count, samples.len() as i64);
        }

        let embedding = store
            .list_dataset_samples("mock-dataset-embedding")
            .await
            .unwrap();
        assert!(embedding
            .iter()
            .all(|sample| sample.prompt.contains("知识库")));
    }
}
