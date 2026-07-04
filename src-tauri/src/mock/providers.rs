use super::{now, types::MockData, with_model_count, MockDataStore};
use crate::domain::model_catalog::{model_summaries_for_interface, CatalogFlavor};
use crate::domain::provider::{
    prepare_provider_create, prepare_provider_update, ExistingProviderConfig,
};
use crate::error::AppError;
use crate::models::{
    CreateProviderInput, DeleteResult, DiscoveredModel, ModelSummary, ProviderConnectionConfig,
    ProviderConnectionResult, ProviderDiagnosticsResult, ProviderModelScanResult, ProviderSummary,
    UpdateProviderInput,
};
use uuid::Uuid;

impl MockDataStore {
    pub async fn list_providers(&self) -> anyhow::Result<Vec<ProviderSummary>> {
        let data = self.inner.read().await;
        Ok(data
            .providers
            .iter()
            .map(|provider| display_provider(&data, provider))
            .rev()
            .collect())
    }

    pub async fn create_provider(
        &self,
        input: CreateProviderInput,
    ) -> anyhow::Result<ProviderSummary> {
        let prepared = prepare_provider_create(input)?;
        let mut data = self.inner.write().await;
        let id = Uuid::new_v4().to_string();
        let now = now();
        let provider = ProviderSummary {
            id: id.clone(),
            name: prepared.name,
            base_url_masked: prepared.base_url_masked,
            api_key_masked: prepared.api_key_masked,
            interface_type: prepared.interface_type,
            status: "unchecked".to_string(),
            model_count: 0,
            last_checked_at: None,
            created_at: now,
        };
        data.provider_base_urls
            .insert(id.clone(), prepared.base_url);
        data.provider_api_keys
            .insert(id.clone(), prepared.api_key_plaintext);
        data.providers.push(provider.clone());
        Ok(provider)
    }

    pub async fn update_provider(
        &self,
        provider_id: &str,
        input: UpdateProviderInput,
    ) -> anyhow::Result<ProviderSummary> {
        let mut data = self.inner.write().await;
        let index = data
            .providers
            .iter()
            .position(|provider| provider.id == provider_id)
            .ok_or_else(|| AppError::not_found("provider"))?;
        let current = data.providers[index].clone();
        let current_base_url = data
            .provider_base_urls
            .get(provider_id)
            .cloned()
            .unwrap_or_else(|| current.base_url_masked.clone());
        let current_api_key = data
            .provider_api_keys
            .get(provider_id)
            .cloned()
            .unwrap_or_default();
        let prepared = prepare_provider_update(
            input,
            ExistingProviderConfig {
                base_url: current_base_url,
                base_url_masked: current.base_url_masked,
                api_key_masked: current.api_key_masked,
                api_key_plaintext: current_api_key,
                interface_type: current.interface_type,
                status: current.status,
                last_checked_at: current.last_checked_at,
            },
        )?;

        {
            let provider = &mut data.providers[index];
            provider.name = prepared.name;
            provider.base_url_masked = prepared.base_url_masked;
            provider.api_key_masked = prepared.api_key_masked;
            provider.interface_type = prepared.interface_type;
            provider.status = prepared.status;
            provider.last_checked_at = prepared.last_checked_at;
        }

        data.provider_base_urls
            .insert(provider_id.to_string(), prepared.base_url);
        data.provider_api_keys
            .insert(provider_id.to_string(), prepared.api_key_plaintext);
        if prepared.config_changed {
            data.models.retain(|model| model.provider_id != provider_id);
        }

        Ok(display_provider(&data, &data.providers[index]))
    }

    pub async fn delete_provider(&self, provider_id: &str) -> anyhow::Result<DeleteResult> {
        let mut data = self.inner.write().await;
        let before = data.providers.len();
        data.providers.retain(|provider| provider.id != provider_id);
        data.provider_base_urls.remove(provider_id);
        data.provider_api_keys.remove(provider_id);
        data.provider_diagnostics.remove(provider_id);
        data.models.retain(|model| model.provider_id != provider_id);
        let task_ids: Vec<String> = data
            .tasks
            .iter()
            .filter(|task| task.provider_id == provider_id)
            .map(|task| task.summary.id.clone())
            .collect();
        data.tasks.retain(|task| task.provider_id != provider_id);
        data.reports
            .retain(|report| !task_ids.iter().any(|task_id| task_id == &report.task_id));
        for task_id in task_ids {
            data.stages.remove(&task_id);
            data.ticks.remove(&task_id);
        }
        Ok(DeleteResult {
            id: provider_id.to_string(),
            deleted: data.providers.len() != before,
        })
    }

    pub async fn test_provider_connection(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderConnectionResult> {
        let mut data = self.inner.write().await;
        let checked_at = now();
        if let Some(provider) = data
            .providers
            .iter_mut()
            .find(|item| item.id == provider_id)
        {
            provider.status = "online".to_string();
            provider.last_checked_at = Some(checked_at.clone());
            return Ok(ProviderConnectionResult {
                provider_id: provider_id.to_string(),
                ok: true,
                status: "online".to_string(),
                message: "Mock connection check passed in Rust backend.".to_string(),
                checked_at,
            });
        }
        Ok(ProviderConnectionResult {
            provider_id: provider_id.to_string(),
            ok: false,
            status: "offline".to_string(),
            message: "Provider not found.".to_string(),
            checked_at,
        })
    }

    pub async fn list_provider_models(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<Vec<ModelSummary>> {
        let data = self.inner.read().await;
        Ok(data
            .models
            .iter()
            .filter(|model| model.provider_id == provider_id)
            .cloned()
            .collect())
    }

    pub async fn scan_provider_models(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderModelScanResult> {
        let mut data = self.inner.write().await;
        let interface_type = data
            .providers
            .iter()
            .find(|provider| provider.id == provider_id)
            .map(|provider| provider.interface_type.clone())
            .ok_or_else(|| AppError::not_found("provider"))?;
        let scanned_at = now();
        data.models.retain(|model| model.provider_id != provider_id);
        let models =
            model_summaries_for_interface(provider_id, &interface_type, CatalogFlavor::Mock);
        data.models.extend(models.clone());
        Ok(ProviderModelScanResult {
            provider_id: provider_id.to_string(),
            models: models.clone(),
            message: format!("Rust backend mock scanned {} models.", models.len()),
            scanned_at,
        })
    }

    pub async fn get_provider_connection_config(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderConnectionConfig> {
        let data = self.inner.read().await;
        let provider = data
            .providers
            .iter()
            .find(|provider| provider.id == provider_id)
            .ok_or_else(|| AppError::not_found("provider"))?;
        Ok(ProviderConnectionConfig {
            id: provider.id.clone(),
            name: provider.name.clone(),
            base_url: data
                .provider_base_urls
                .get(provider_id)
                .cloned()
                .unwrap_or_else(|| provider.base_url_masked.clone()),
            api_key_plaintext: data
                .provider_api_keys
                .get(provider_id)
                .cloned()
                .unwrap_or_default(),
            interface_type: provider.interface_type.clone(),
        })
    }

    pub async fn update_provider_connection_status(
        &self,
        provider_id: &str,
        status: &str,
        checked_at: &str,
    ) -> anyhow::Result<()> {
        let mut data = self.inner.write().await;
        let provider = data
            .providers
            .iter_mut()
            .find(|provider| provider.id == provider_id)
            .ok_or_else(|| AppError::not_found("provider"))?;
        provider.status = status.to_string();
        provider.last_checked_at = Some(checked_at.to_string());
        Ok(())
    }

    pub async fn replace_provider_models(
        &self,
        provider_id: &str,
        models: Vec<DiscoveredModel>,
        scanned_at: &str,
    ) -> anyhow::Result<Vec<ModelSummary>> {
        let mut data = self.inner.write().await;
        if !data
            .providers
            .iter()
            .any(|provider| provider.id == provider_id)
        {
            return Err(AppError::not_found("provider").into());
        }

        data.models.retain(|model| model.provider_id != provider_id);
        let summaries: Vec<ModelSummary> = models
            .into_iter()
            .map(|model| ModelSummary {
                id: Uuid::new_v4().to_string(),
                provider_id: provider_id.to_string(),
                name: model.name,
                model_type: model.model_type,
                capabilities: model.capabilities,
                supports_streaming: model.supports_streaming,
                recommended_concurrency: model.recommended_concurrency,
            })
            .collect();
        data.models.extend(summaries.clone());
        for provider in &mut data.providers {
            if provider.id == provider_id {
                provider.last_checked_at = Some(scanned_at.to_string());
                provider.status = "online".to_string();
                provider.model_count = summaries.len() as i64;
            }
        }
        Ok(summaries)
    }

    pub async fn save_provider_diagnostics(
        &self,
        result: &ProviderDiagnosticsResult,
    ) -> anyhow::Result<()> {
        let mut data = self.inner.write().await;
        data.provider_diagnostics
            .insert(result.provider_id.clone(), result.clone());
        Ok(())
    }

    pub async fn get_provider_diagnostics(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<Option<ProviderDiagnosticsResult>> {
        let data = self.inner.read().await;
        Ok(data.provider_diagnostics.get(provider_id).cloned())
    }
}

fn display_provider(data: &MockData, provider: &ProviderSummary) -> ProviderSummary {
    let mut provider = with_model_count(provider, &data.models);
    if let Some(base_url) = data.provider_base_urls.get(&provider.id) {
        provider.base_url_masked = base_url.clone();
    }
    if let Some(api_key) = data.provider_api_keys.get(&provider.id) {
        provider.api_key_masked = if api_key.trim().is_empty() {
            "未配置".to_string()
        } else {
            api_key.clone()
        };
    }
    provider
}
