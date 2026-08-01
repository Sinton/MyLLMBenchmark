use super::{AppDataSource, SiteProbeRepository};
use crate::db::Database;
use crate::mock::MockDataStore;
use crate::models::{
    DeleteResult, SiteProbeHistoryPage, SiteProbeHistoryPageInput, SiteProbeRunDetail,
    SiteProbeRunRecord, SiteProbeRunSummary,
};

impl SiteProbeRepository for MockDataStore {
    async fn insert_site_probe_run(
        &self,
        record: SiteProbeRunRecord,
    ) -> anyhow::Result<SiteProbeRunSummary> {
        MockDataStore::insert_site_probe_run(self, record).await
    }

    async fn list_site_probe_runs_page(
        &self,
        input: SiteProbeHistoryPageInput,
    ) -> anyhow::Result<SiteProbeHistoryPage> {
        MockDataStore::list_site_probe_runs_page(self, input).await
    }

    async fn get_site_probe_run_detail(&self, run_id: &str) -> anyhow::Result<SiteProbeRunDetail> {
        MockDataStore::get_site_probe_run_detail(self, run_id).await
    }

    async fn delete_site_probe_run(&self, run_id: &str) -> anyhow::Result<DeleteResult> {
        MockDataStore::delete_site_probe_run(self, run_id).await
    }
}

impl SiteProbeRepository for Database {
    async fn insert_site_probe_run(
        &self,
        record: SiteProbeRunRecord,
    ) -> anyhow::Result<SiteProbeRunSummary> {
        Database::insert_site_probe_run(self, &record).await
    }

    async fn list_site_probe_runs_page(
        &self,
        input: SiteProbeHistoryPageInput,
    ) -> anyhow::Result<SiteProbeHistoryPage> {
        Database::list_site_probe_runs_page(self, input).await
    }

    async fn get_site_probe_run_detail(&self, run_id: &str) -> anyhow::Result<SiteProbeRunDetail> {
        Database::get_site_probe_run_detail(self, run_id).await
    }

    async fn delete_site_probe_run(&self, run_id: &str) -> anyhow::Result<DeleteResult> {
        Database::delete_site_probe_run(self, run_id).await
    }
}

impl AppDataSource {
    pub async fn insert_site_probe_run(
        &self,
        record: SiteProbeRunRecord,
    ) -> anyhow::Result<SiteProbeRunSummary> {
        match self {
            Self::Mock(source) => SiteProbeRepository::insert_site_probe_run(source, record).await,
            Self::Sqlite(source) => {
                SiteProbeRepository::insert_site_probe_run(source, record).await
            }
        }
    }

    pub async fn list_site_probe_runs_page(
        &self,
        input: SiteProbeHistoryPageInput,
    ) -> anyhow::Result<SiteProbeHistoryPage> {
        match self {
            Self::Mock(source) => SiteProbeRepository::list_site_probe_runs_page(source, input).await,
            Self::Sqlite(source) => {
                SiteProbeRepository::list_site_probe_runs_page(source, input).await
            }
        }
    }

    pub async fn get_site_probe_run_detail(
        &self,
        run_id: &str,
    ) -> anyhow::Result<SiteProbeRunDetail> {
        match self {
            Self::Mock(source) => SiteProbeRepository::get_site_probe_run_detail(source, run_id).await,
            Self::Sqlite(source) => {
                SiteProbeRepository::get_site_probe_run_detail(source, run_id).await
            }
        }
    }

    pub async fn delete_site_probe_run(&self, run_id: &str) -> anyhow::Result<DeleteResult> {
        match self {
            Self::Mock(source) => SiteProbeRepository::delete_site_probe_run(source, run_id).await,
            Self::Sqlite(source) => SiteProbeRepository::delete_site_probe_run(source, run_id).await,
        }
    }
}
