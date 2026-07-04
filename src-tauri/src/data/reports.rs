use super::{AppDataSource, ReportRepository};
use crate::db::Database;
use crate::mock::MockDataStore;
use crate::models::{ReportDetail, ReportSummary};

impl ReportRepository for MockDataStore {
    async fn generate_report(&self, task_id: &str) -> anyhow::Result<ReportSummary> {
        MockDataStore::generate_report(self, task_id).await
    }

    async fn list_reports(&self) -> anyhow::Result<Vec<ReportSummary>> {
        MockDataStore::list_reports(self).await
    }

    async fn get_report_detail(&self, report_id: &str) -> anyhow::Result<ReportDetail> {
        MockDataStore::get_report_detail(self, report_id).await
    }
}

impl ReportRepository for Database {
    async fn generate_report(&self, task_id: &str) -> anyhow::Result<ReportSummary> {
        Database::generate_report(self, task_id).await
    }

    async fn list_reports(&self) -> anyhow::Result<Vec<ReportSummary>> {
        Database::list_reports(self).await
    }

    async fn get_report_detail(&self, report_id: &str) -> anyhow::Result<ReportDetail> {
        Database::get_report_detail(self, report_id).await
    }
}

impl AppDataSource {
    pub async fn generate_report(&self, task_id: &str) -> anyhow::Result<ReportSummary> {
        match self {
            Self::Mock(source) => ReportRepository::generate_report(source, task_id).await,
            Self::Sqlite(source) => ReportRepository::generate_report(source, task_id).await,
        }
    }

    pub async fn list_reports(&self) -> anyhow::Result<Vec<ReportSummary>> {
        match self {
            Self::Mock(source) => ReportRepository::list_reports(source).await,
            Self::Sqlite(source) => ReportRepository::list_reports(source).await,
        }
    }

    pub async fn get_report_detail(&self, report_id: &str) -> anyhow::Result<ReportDetail> {
        match self {
            Self::Mock(source) => ReportRepository::get_report_detail(source, report_id).await,
            Self::Sqlite(source) => ReportRepository::get_report_detail(source, report_id).await,
        }
    }
}
