use super::{AppDataSource, EndpointProbeRepository};
use crate::db::Database;
use crate::mock::MockDataStore;
use crate::models::{
    DeleteResult, EndpointProbeBatchDetail, EndpointProbeBatchRecord, EndpointProbeBatchSummary,
    EndpointProbeHistoryPage, EndpointProbeHistoryPageInput, EndpointProbeRunDetail,
    EndpointProbeRunRecord, EndpointProbeRunSummary,
};

macro_rules! impl_repository {
    ($source:ty) => {
        impl EndpointProbeRepository for $source {
            async fn create_endpoint_probe_batch(
                &self,
                batch: EndpointProbeBatchRecord,
                runs: Vec<EndpointProbeRunRecord>,
            ) -> anyhow::Result<EndpointProbeBatchSummary> {
                <$source>::create_endpoint_probe_batch(self, &batch, &runs).await
            }

            async fn mark_endpoint_probe_run_started(&self, run_id: &str) -> anyhow::Result<()> {
                <$source>::mark_endpoint_probe_run_started(self, run_id).await
            }

            async fn finish_endpoint_probe_run(
                &self,
                record: EndpointProbeRunRecord,
            ) -> anyhow::Result<EndpointProbeRunSummary> {
                <$source>::finish_endpoint_probe_run(self, &record).await
            }

            async fn finish_endpoint_probe_batch(
                &self,
                batch_id: &str,
                status: &str,
                finished_at: &str,
            ) -> anyhow::Result<EndpointProbeBatchSummary> {
                <$source>::finish_endpoint_probe_batch(self, batch_id, status, finished_at).await
            }

            async fn list_endpoint_probe_batches_page(
                &self,
                input: EndpointProbeHistoryPageInput,
            ) -> anyhow::Result<EndpointProbeHistoryPage> {
                <$source>::list_endpoint_probe_batches_page(self, input).await
            }

            async fn get_endpoint_probe_batch_detail(
                &self,
                batch_id: &str,
            ) -> anyhow::Result<EndpointProbeBatchDetail> {
                <$source>::get_endpoint_probe_batch_detail(self, batch_id).await
            }

            async fn get_endpoint_probe_run_detail(
                &self,
                run_id: &str,
            ) -> anyhow::Result<EndpointProbeRunDetail> {
                <$source>::get_endpoint_probe_run_detail(self, run_id).await
            }

            async fn delete_endpoint_probe_batch(
                &self,
                batch_id: &str,
            ) -> anyhow::Result<DeleteResult> {
                <$source>::delete_endpoint_probe_batch(self, batch_id).await
            }

            async fn recover_endpoint_probe_batches(&self, message: &str) -> anyhow::Result<()> {
                <$source>::recover_endpoint_probe_batches(self, message).await
            }
        }
    };
}

impl_repository!(MockDataStore);
impl_repository!(Database);

impl AppDataSource {
    pub async fn create_endpoint_probe_batch(
        &self,
        batch: EndpointProbeBatchRecord,
        runs: Vec<EndpointProbeRunRecord>,
    ) -> anyhow::Result<EndpointProbeBatchSummary> {
        match self {
            Self::Mock(source) => {
                EndpointProbeRepository::create_endpoint_probe_batch(source, batch, runs).await
            }
            Self::Sqlite(source) => {
                EndpointProbeRepository::create_endpoint_probe_batch(source, batch, runs).await
            }
        }
    }

    pub async fn mark_endpoint_probe_run_started(&self, run_id: &str) -> anyhow::Result<()> {
        match self {
            Self::Mock(source) => {
                EndpointProbeRepository::mark_endpoint_probe_run_started(source, run_id).await
            }
            Self::Sqlite(source) => {
                EndpointProbeRepository::mark_endpoint_probe_run_started(source, run_id).await
            }
        }
    }

    pub async fn finish_endpoint_probe_run(
        &self,
        record: EndpointProbeRunRecord,
    ) -> anyhow::Result<EndpointProbeRunSummary> {
        match self {
            Self::Mock(source) => {
                EndpointProbeRepository::finish_endpoint_probe_run(source, record).await
            }
            Self::Sqlite(source) => {
                EndpointProbeRepository::finish_endpoint_probe_run(source, record).await
            }
        }
    }

    pub async fn finish_endpoint_probe_batch(
        &self,
        batch_id: &str,
        status: &str,
        finished_at: &str,
    ) -> anyhow::Result<EndpointProbeBatchSummary> {
        match self {
            Self::Mock(source) => {
                EndpointProbeRepository::finish_endpoint_probe_batch(
                    source,
                    batch_id,
                    status,
                    finished_at,
                )
                .await
            }
            Self::Sqlite(source) => {
                EndpointProbeRepository::finish_endpoint_probe_batch(
                    source,
                    batch_id,
                    status,
                    finished_at,
                )
                .await
            }
        }
    }

    pub async fn list_endpoint_probe_batches_page(
        &self,
        input: EndpointProbeHistoryPageInput,
    ) -> anyhow::Result<EndpointProbeHistoryPage> {
        match self {
            Self::Mock(source) => {
                EndpointProbeRepository::list_endpoint_probe_batches_page(source, input).await
            }
            Self::Sqlite(source) => {
                EndpointProbeRepository::list_endpoint_probe_batches_page(source, input).await
            }
        }
    }

    pub async fn get_endpoint_probe_batch_detail(
        &self,
        batch_id: &str,
    ) -> anyhow::Result<EndpointProbeBatchDetail> {
        match self {
            Self::Mock(source) => {
                EndpointProbeRepository::get_endpoint_probe_batch_detail(source, batch_id).await
            }
            Self::Sqlite(source) => {
                EndpointProbeRepository::get_endpoint_probe_batch_detail(source, batch_id).await
            }
        }
    }

    pub async fn get_endpoint_probe_run_detail(
        &self,
        run_id: &str,
    ) -> anyhow::Result<EndpointProbeRunDetail> {
        match self {
            Self::Mock(source) => {
                EndpointProbeRepository::get_endpoint_probe_run_detail(source, run_id).await
            }
            Self::Sqlite(source) => {
                EndpointProbeRepository::get_endpoint_probe_run_detail(source, run_id).await
            }
        }
    }

    pub async fn delete_endpoint_probe_batch(
        &self,
        batch_id: &str,
    ) -> anyhow::Result<DeleteResult> {
        match self {
            Self::Mock(source) => {
                EndpointProbeRepository::delete_endpoint_probe_batch(source, batch_id).await
            }
            Self::Sqlite(source) => {
                EndpointProbeRepository::delete_endpoint_probe_batch(source, batch_id).await
            }
        }
    }

    pub async fn recover_endpoint_probe_batches(&self, message: &str) -> anyhow::Result<()> {
        match self {
            Self::Mock(source) => {
                EndpointProbeRepository::recover_endpoint_probe_batches(source, message).await
            }
            Self::Sqlite(source) => {
                EndpointProbeRepository::recover_endpoint_probe_batches(source, message).await
            }
        }
    }
}
