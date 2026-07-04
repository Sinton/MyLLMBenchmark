use super::{AppDataSource, ProviderRepository};
use crate::db::Database;
use crate::mock::MockDataStore;
use crate::models::{
    CreateProviderInput, DeleteResult, DiscoveredModel, ModelSummary, ProviderConnectionConfig,
    ProviderConnectionResult, ProviderDiagnosticsResult, ProviderModelScanResult, ProviderSummary,
    UpdateProviderInput,
};

impl ProviderRepository for MockDataStore {
    async fn list_providers(&self) -> anyhow::Result<Vec<ProviderSummary>> {
        MockDataStore::list_providers(self).await
    }

    async fn create_provider(&self, input: CreateProviderInput) -> anyhow::Result<ProviderSummary> {
        MockDataStore::create_provider(self, input).await
    }

    async fn update_provider(
        &self,
        provider_id: &str,
        input: UpdateProviderInput,
    ) -> anyhow::Result<ProviderSummary> {
        MockDataStore::update_provider(self, provider_id, input).await
    }

    async fn delete_provider(&self, provider_id: &str) -> anyhow::Result<DeleteResult> {
        MockDataStore::delete_provider(self, provider_id).await
    }

    async fn test_provider_connection(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderConnectionResult> {
        MockDataStore::test_provider_connection(self, provider_id).await
    }

    async fn list_provider_models(&self, provider_id: &str) -> anyhow::Result<Vec<ModelSummary>> {
        MockDataStore::list_provider_models(self, provider_id).await
    }

    async fn scan_provider_models(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderModelScanResult> {
        MockDataStore::scan_provider_models(self, provider_id).await
    }

    async fn get_provider_connection_config(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderConnectionConfig> {
        MockDataStore::get_provider_connection_config(self, provider_id).await
    }

    async fn update_provider_connection_status(
        &self,
        provider_id: &str,
        status: &str,
        checked_at: &str,
    ) -> anyhow::Result<()> {
        MockDataStore::update_provider_connection_status(self, provider_id, status, checked_at)
            .await
    }

    async fn replace_provider_models(
        &self,
        provider_id: &str,
        models: Vec<DiscoveredModel>,
        scanned_at: &str,
    ) -> anyhow::Result<Vec<ModelSummary>> {
        MockDataStore::replace_provider_models(self, provider_id, models, scanned_at).await
    }

    async fn save_provider_diagnostics(
        &self,
        result: &ProviderDiagnosticsResult,
    ) -> anyhow::Result<()> {
        MockDataStore::save_provider_diagnostics(self, result).await
    }

    async fn get_provider_diagnostics(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<Option<ProviderDiagnosticsResult>> {
        MockDataStore::get_provider_diagnostics(self, provider_id).await
    }
}

impl ProviderRepository for Database {
    async fn list_providers(&self) -> anyhow::Result<Vec<ProviderSummary>> {
        Database::list_providers(self).await
    }

    async fn create_provider(&self, input: CreateProviderInput) -> anyhow::Result<ProviderSummary> {
        Database::create_provider(self, input).await
    }

    async fn update_provider(
        &self,
        provider_id: &str,
        input: UpdateProviderInput,
    ) -> anyhow::Result<ProviderSummary> {
        Database::update_provider(self, provider_id, input).await
    }

    async fn delete_provider(&self, provider_id: &str) -> anyhow::Result<DeleteResult> {
        let deleted = Database::delete_provider(self, provider_id).await?;
        Ok(DeleteResult {
            id: provider_id.to_string(),
            deleted,
        })
    }

    async fn test_provider_connection(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderConnectionResult> {
        Database::test_provider_connection(self, provider_id).await
    }

    async fn list_provider_models(&self, provider_id: &str) -> anyhow::Result<Vec<ModelSummary>> {
        Database::list_provider_models(self, provider_id).await
    }

    async fn scan_provider_models(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderModelScanResult> {
        Database::scan_provider_models(self, provider_id).await
    }

    async fn get_provider_connection_config(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderConnectionConfig> {
        Database::get_provider_connection_config(self, provider_id).await
    }

    async fn update_provider_connection_status(
        &self,
        provider_id: &str,
        status: &str,
        checked_at: &str,
    ) -> anyhow::Result<()> {
        Database::update_provider_connection_status(self, provider_id, status, checked_at).await
    }

    async fn replace_provider_models(
        &self,
        provider_id: &str,
        models: Vec<DiscoveredModel>,
        scanned_at: &str,
    ) -> anyhow::Result<Vec<ModelSummary>> {
        Database::replace_provider_models(self, provider_id, models, scanned_at).await
    }

    async fn save_provider_diagnostics(
        &self,
        result: &ProviderDiagnosticsResult,
    ) -> anyhow::Result<()> {
        Database::save_provider_diagnostics(self, result).await
    }

    async fn get_provider_diagnostics(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<Option<ProviderDiagnosticsResult>> {
        Database::get_provider_diagnostics(self, provider_id).await
    }
}

impl AppDataSource {
    pub async fn list_providers(&self) -> anyhow::Result<Vec<ProviderSummary>> {
        match self {
            Self::Mock(source) => ProviderRepository::list_providers(source).await,
            Self::Sqlite(source) => ProviderRepository::list_providers(source).await,
        }
    }

    pub async fn create_provider(
        &self,
        input: CreateProviderInput,
    ) -> anyhow::Result<ProviderSummary> {
        match self {
            Self::Mock(source) => ProviderRepository::create_provider(source, input).await,
            Self::Sqlite(source) => ProviderRepository::create_provider(source, input).await,
        }
    }

    pub async fn update_provider(
        &self,
        provider_id: &str,
        input: UpdateProviderInput,
    ) -> anyhow::Result<ProviderSummary> {
        match self {
            Self::Mock(source) => {
                ProviderRepository::update_provider(source, provider_id, input).await
            }
            Self::Sqlite(source) => {
                ProviderRepository::update_provider(source, provider_id, input).await
            }
        }
    }

    pub async fn delete_provider(&self, provider_id: &str) -> anyhow::Result<DeleteResult> {
        match self {
            Self::Mock(source) => ProviderRepository::delete_provider(source, provider_id).await,
            Self::Sqlite(source) => ProviderRepository::delete_provider(source, provider_id).await,
        }
    }

    pub async fn test_provider_connection(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderConnectionResult> {
        match self {
            Self::Mock(source) => {
                ProviderRepository::test_provider_connection(source, provider_id).await
            }
            Self::Sqlite(source) => {
                ProviderRepository::test_provider_connection(source, provider_id).await
            }
        }
    }

    pub async fn list_provider_models(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<Vec<ModelSummary>> {
        match self {
            Self::Mock(source) => {
                ProviderRepository::list_provider_models(source, provider_id).await
            }
            Self::Sqlite(source) => {
                ProviderRepository::list_provider_models(source, provider_id).await
            }
        }
    }

    pub async fn scan_provider_models(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderModelScanResult> {
        match self {
            Self::Mock(source) => {
                ProviderRepository::scan_provider_models(source, provider_id).await
            }
            Self::Sqlite(source) => {
                ProviderRepository::scan_provider_models(source, provider_id).await
            }
        }
    }

    pub async fn get_provider_connection_config(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderConnectionConfig> {
        match self {
            Self::Mock(source) => {
                ProviderRepository::get_provider_connection_config(source, provider_id).await
            }
            Self::Sqlite(source) => {
                ProviderRepository::get_provider_connection_config(source, provider_id).await
            }
        }
    }

    pub async fn update_provider_connection_status(
        &self,
        provider_id: &str,
        status: &str,
        checked_at: &str,
    ) -> anyhow::Result<()> {
        match self {
            Self::Mock(source) => {
                ProviderRepository::update_provider_connection_status(
                    source,
                    provider_id,
                    status,
                    checked_at,
                )
                .await
            }
            Self::Sqlite(source) => {
                ProviderRepository::update_provider_connection_status(
                    source,
                    provider_id,
                    status,
                    checked_at,
                )
                .await
            }
        }
    }

    pub async fn replace_provider_models(
        &self,
        provider_id: &str,
        models: Vec<DiscoveredModel>,
        scanned_at: &str,
    ) -> anyhow::Result<Vec<ModelSummary>> {
        match self {
            Self::Mock(source) => {
                ProviderRepository::replace_provider_models(source, provider_id, models, scanned_at)
                    .await
            }
            Self::Sqlite(source) => {
                ProviderRepository::replace_provider_models(source, provider_id, models, scanned_at)
                    .await
            }
        }
    }

    pub async fn save_provider_diagnostics(
        &self,
        result: &ProviderDiagnosticsResult,
    ) -> anyhow::Result<()> {
        match self {
            Self::Mock(source) => {
                ProviderRepository::save_provider_diagnostics(source, result).await
            }
            Self::Sqlite(source) => {
                ProviderRepository::save_provider_diagnostics(source, result).await
            }
        }
    }

    pub async fn get_provider_diagnostics(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<Option<ProviderDiagnosticsResult>> {
        match self {
            Self::Mock(source) => {
                ProviderRepository::get_provider_diagnostics(source, provider_id).await
            }
            Self::Sqlite(source) => {
                ProviderRepository::get_provider_diagnostics(source, provider_id).await
            }
        }
    }
}
