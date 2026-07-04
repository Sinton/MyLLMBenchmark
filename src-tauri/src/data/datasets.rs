use super::{AppDataSource, DatasetRepository};
use crate::db::Database;
use crate::mock::MockDataStore;
use crate::models::{
    DatasetAppendInput, DatasetExportInput, DatasetExportResult, DatasetImportInput, DatasetSample,
    DatasetSampleBatchDeleteInput, DatasetSampleCreateInput, DatasetSamplePage,
    DatasetSamplePageInput, DatasetSamplePreview, DatasetSampleUpdateInput, DatasetSummary,
    DatasetUpdateInput, DatasetValidationResult, DeleteResult,
};

impl DatasetRepository for MockDataStore {
    async fn list_datasets(&self) -> anyhow::Result<Vec<DatasetSummary>> {
        MockDataStore::list_datasets(self).await
    }

    async fn import_dataset(&self, input: DatasetImportInput) -> anyhow::Result<DatasetSummary> {
        MockDataStore::import_dataset(self, input).await
    }

    async fn update_dataset(&self, input: DatasetUpdateInput) -> anyhow::Result<DatasetSummary> {
        MockDataStore::update_dataset(self, input).await
    }

    async fn delete_dataset(&self, dataset_id: &str) -> anyhow::Result<DeleteResult> {
        MockDataStore::delete_dataset(self, dataset_id).await
    }

    async fn preview_dataset_samples(
        &self,
        dataset_id: &str,
        limit: i64,
    ) -> anyhow::Result<Vec<DatasetSamplePreview>> {
        MockDataStore::preview_dataset_samples(self, dataset_id, limit).await
    }

    async fn list_dataset_samples_page(
        &self,
        input: DatasetSamplePageInput,
    ) -> anyhow::Result<DatasetSamplePage> {
        MockDataStore::list_dataset_samples_page(self, input).await
    }

    async fn list_dataset_samples(&self, dataset_id: &str) -> anyhow::Result<Vec<DatasetSample>> {
        MockDataStore::list_dataset_samples(self, dataset_id).await
    }

    async fn create_dataset_sample(
        &self,
        input: DatasetSampleCreateInput,
    ) -> anyhow::Result<DatasetSamplePreview> {
        MockDataStore::create_dataset_sample(self, input).await
    }

    async fn update_dataset_sample(
        &self,
        input: DatasetSampleUpdateInput,
    ) -> anyhow::Result<DatasetSamplePreview> {
        MockDataStore::update_dataset_sample(self, input).await
    }

    async fn delete_dataset_sample(&self, sample_id: &str) -> anyhow::Result<DeleteResult> {
        MockDataStore::delete_dataset_sample(self, sample_id).await
    }

    async fn append_dataset_samples(
        &self,
        input: DatasetAppendInput,
    ) -> anyhow::Result<DatasetSummary> {
        MockDataStore::append_dataset_samples(self, input).await
    }

    async fn delete_dataset_samples_batch(
        &self,
        input: DatasetSampleBatchDeleteInput,
    ) -> anyhow::Result<DeleteResult> {
        MockDataStore::delete_dataset_samples_batch(self, input).await
    }

    async fn export_dataset(
        &self,
        input: DatasetExportInput,
    ) -> anyhow::Result<DatasetExportResult> {
        MockDataStore::export_dataset(self, input).await
    }

    async fn validate_dataset_samples(
        &self,
        dataset_id: &str,
    ) -> anyhow::Result<DatasetValidationResult> {
        MockDataStore::validate_dataset_samples(self, dataset_id).await
    }
}

impl DatasetRepository for Database {
    async fn list_datasets(&self) -> anyhow::Result<Vec<DatasetSummary>> {
        Database::list_datasets(self).await
    }

    async fn import_dataset(&self, input: DatasetImportInput) -> anyhow::Result<DatasetSummary> {
        Database::import_dataset(self, input).await
    }

    async fn update_dataset(&self, input: DatasetUpdateInput) -> anyhow::Result<DatasetSummary> {
        Database::update_dataset(self, input).await
    }

    async fn delete_dataset(&self, dataset_id: &str) -> anyhow::Result<DeleteResult> {
        Database::delete_dataset(self, dataset_id).await
    }

    async fn preview_dataset_samples(
        &self,
        dataset_id: &str,
        limit: i64,
    ) -> anyhow::Result<Vec<DatasetSamplePreview>> {
        Database::preview_dataset_samples(self, dataset_id, limit).await
    }

    async fn list_dataset_samples_page(
        &self,
        input: DatasetSamplePageInput,
    ) -> anyhow::Result<DatasetSamplePage> {
        Database::list_dataset_samples_page(self, input).await
    }

    async fn list_dataset_samples(&self, dataset_id: &str) -> anyhow::Result<Vec<DatasetSample>> {
        Database::list_dataset_samples(self, dataset_id).await
    }

    async fn create_dataset_sample(
        &self,
        input: DatasetSampleCreateInput,
    ) -> anyhow::Result<DatasetSamplePreview> {
        Database::create_dataset_sample(self, input).await
    }

    async fn update_dataset_sample(
        &self,
        input: DatasetSampleUpdateInput,
    ) -> anyhow::Result<DatasetSamplePreview> {
        Database::update_dataset_sample(self, input).await
    }

    async fn delete_dataset_sample(&self, sample_id: &str) -> anyhow::Result<DeleteResult> {
        Database::delete_dataset_sample(self, sample_id).await
    }

    async fn append_dataset_samples(
        &self,
        input: DatasetAppendInput,
    ) -> anyhow::Result<DatasetSummary> {
        Database::append_dataset_samples(self, input).await
    }

    async fn delete_dataset_samples_batch(
        &self,
        input: DatasetSampleBatchDeleteInput,
    ) -> anyhow::Result<DeleteResult> {
        Database::delete_dataset_samples_batch(self, input).await
    }

    async fn export_dataset(
        &self,
        input: DatasetExportInput,
    ) -> anyhow::Result<DatasetExportResult> {
        Database::export_dataset(self, input).await
    }

    async fn validate_dataset_samples(
        &self,
        dataset_id: &str,
    ) -> anyhow::Result<DatasetValidationResult> {
        Database::validate_dataset_samples(self, dataset_id).await
    }
}

impl AppDataSource {
    pub async fn list_datasets(&self) -> anyhow::Result<Vec<DatasetSummary>> {
        match self {
            Self::Mock(source) => DatasetRepository::list_datasets(source).await,
            Self::Sqlite(source) => DatasetRepository::list_datasets(source).await,
        }
    }

    pub async fn import_dataset(
        &self,
        input: DatasetImportInput,
    ) -> anyhow::Result<DatasetSummary> {
        match self {
            Self::Mock(source) => DatasetRepository::import_dataset(source, input).await,
            Self::Sqlite(source) => DatasetRepository::import_dataset(source, input).await,
        }
    }

    pub async fn update_dataset(
        &self,
        input: DatasetUpdateInput,
    ) -> anyhow::Result<DatasetSummary> {
        match self {
            Self::Mock(source) => DatasetRepository::update_dataset(source, input).await,
            Self::Sqlite(source) => DatasetRepository::update_dataset(source, input).await,
        }
    }

    pub async fn delete_dataset(&self, dataset_id: &str) -> anyhow::Result<DeleteResult> {
        match self {
            Self::Mock(source) => DatasetRepository::delete_dataset(source, dataset_id).await,
            Self::Sqlite(source) => DatasetRepository::delete_dataset(source, dataset_id).await,
        }
    }

    pub async fn preview_dataset_samples(
        &self,
        dataset_id: &str,
        limit: i64,
    ) -> anyhow::Result<Vec<DatasetSamplePreview>> {
        match self {
            Self::Mock(source) => {
                DatasetRepository::preview_dataset_samples(source, dataset_id, limit).await
            }
            Self::Sqlite(source) => {
                DatasetRepository::preview_dataset_samples(source, dataset_id, limit).await
            }
        }
    }

    pub async fn list_dataset_samples_page(
        &self,
        input: DatasetSamplePageInput,
    ) -> anyhow::Result<DatasetSamplePage> {
        match self {
            Self::Mock(source) => DatasetRepository::list_dataset_samples_page(source, input).await,
            Self::Sqlite(source) => {
                DatasetRepository::list_dataset_samples_page(source, input).await
            }
        }
    }

    pub async fn list_dataset_samples(
        &self,
        dataset_id: &str,
    ) -> anyhow::Result<Vec<DatasetSample>> {
        match self {
            Self::Mock(source) => DatasetRepository::list_dataset_samples(source, dataset_id).await,
            Self::Sqlite(source) => {
                DatasetRepository::list_dataset_samples(source, dataset_id).await
            }
        }
    }

    pub async fn create_dataset_sample(
        &self,
        input: DatasetSampleCreateInput,
    ) -> anyhow::Result<DatasetSamplePreview> {
        match self {
            Self::Mock(source) => DatasetRepository::create_dataset_sample(source, input).await,
            Self::Sqlite(source) => DatasetRepository::create_dataset_sample(source, input).await,
        }
    }

    pub async fn update_dataset_sample(
        &self,
        input: DatasetSampleUpdateInput,
    ) -> anyhow::Result<DatasetSamplePreview> {
        match self {
            Self::Mock(source) => DatasetRepository::update_dataset_sample(source, input).await,
            Self::Sqlite(source) => DatasetRepository::update_dataset_sample(source, input).await,
        }
    }

    pub async fn delete_dataset_sample(&self, sample_id: &str) -> anyhow::Result<DeleteResult> {
        match self {
            Self::Mock(source) => DatasetRepository::delete_dataset_sample(source, sample_id).await,
            Self::Sqlite(source) => {
                DatasetRepository::delete_dataset_sample(source, sample_id).await
            }
        }
    }

    pub async fn append_dataset_samples(
        &self,
        input: DatasetAppendInput,
    ) -> anyhow::Result<DatasetSummary> {
        match self {
            Self::Mock(source) => DatasetRepository::append_dataset_samples(source, input).await,
            Self::Sqlite(source) => DatasetRepository::append_dataset_samples(source, input).await,
        }
    }

    pub async fn delete_dataset_samples_batch(
        &self,
        input: DatasetSampleBatchDeleteInput,
    ) -> anyhow::Result<DeleteResult> {
        match self {
            Self::Mock(source) => {
                DatasetRepository::delete_dataset_samples_batch(source, input).await
            }
            Self::Sqlite(source) => {
                DatasetRepository::delete_dataset_samples_batch(source, input).await
            }
        }
    }

    pub async fn export_dataset(
        &self,
        input: DatasetExportInput,
    ) -> anyhow::Result<DatasetExportResult> {
        match self {
            Self::Mock(source) => DatasetRepository::export_dataset(source, input).await,
            Self::Sqlite(source) => DatasetRepository::export_dataset(source, input).await,
        }
    }

    pub async fn validate_dataset_samples(
        &self,
        dataset_id: &str,
    ) -> anyhow::Result<DatasetValidationResult> {
        match self {
            Self::Mock(source) => {
                DatasetRepository::validate_dataset_samples(source, dataset_id).await
            }
            Self::Sqlite(source) => {
                DatasetRepository::validate_dataset_samples(source, dataset_id).await
            }
        }
    }
}
