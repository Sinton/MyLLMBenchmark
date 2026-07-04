use super::{AppDataSource, DashboardRepository};
use crate::db::Database;
use crate::mock::MockDataStore;
use crate::models::DashboardSummary;

impl DashboardRepository for MockDataStore {
    async fn dashboard_summary(&self) -> anyhow::Result<DashboardSummary> {
        MockDataStore::dashboard_summary(self).await
    }
}

impl DashboardRepository for Database {
    async fn dashboard_summary(&self) -> anyhow::Result<DashboardSummary> {
        Database::dashboard_summary(self).await
    }
}

impl AppDataSource {
    pub async fn dashboard_summary(&self) -> anyhow::Result<DashboardSummary> {
        match self {
            Self::Mock(source) => DashboardRepository::dashboard_summary(source).await,
            Self::Sqlite(source) => DashboardRepository::dashboard_summary(source).await,
        }
    }
}
