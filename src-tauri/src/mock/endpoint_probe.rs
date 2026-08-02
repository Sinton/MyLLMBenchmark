use super::MockDataStore;
use crate::error::AppError;
use crate::models::{
    DeleteResult, EndpointProbeBatchDetail, EndpointProbeBatchRecord, EndpointProbeBatchSummary,
    EndpointProbeHistoryPage, EndpointProbeHistoryPageInput, EndpointProbeRunDetail,
    EndpointProbeRunRecord, EndpointProbeRunSummary,
};

impl MockDataStore {
    pub async fn create_endpoint_probe_batch(
        &self,
        batch: &EndpointProbeBatchRecord,
        runs: &[EndpointProbeRunRecord],
    ) -> anyhow::Result<EndpointProbeBatchSummary> {
        let mut data = self.inner.write().await;
        data.endpoint_probe_batches.push(batch.summary.clone());
        data.endpoint_probe_runs
            .extend(runs.iter().map(detail_from_record));
        Ok(with_counts(&data, batch.summary.clone()))
    }

    pub async fn mark_endpoint_probe_run_started(&self, run_id: &str) -> anyhow::Result<()> {
        let mut data = self.inner.write().await;
        let run = data
            .endpoint_probe_runs
            .iter_mut()
            .find(|run| run.summary.id == run_id)
            .ok_or_else(|| AppError::not_found("endpoint_probe_run"))?;
        if run.summary.status == "pending" {
            run.summary.status = "running".to_string();
        }
        Ok(())
    }

    pub async fn finish_endpoint_probe_run(
        &self,
        record: &EndpointProbeRunRecord,
    ) -> anyhow::Result<EndpointProbeRunSummary> {
        let mut data = self.inner.write().await;
        let run = data
            .endpoint_probe_runs
            .iter_mut()
            .find(|run| run.summary.id == record.summary.id)
            .ok_or_else(|| AppError::not_found("endpoint_probe_run"))?;
        *run = detail_from_record(record);
        Ok(run.summary.clone())
    }

    pub async fn finish_endpoint_probe_batch(
        &self,
        batch_id: &str,
        status: &str,
        finished_at: &str,
    ) -> anyhow::Result<EndpointProbeBatchSummary> {
        let mut data = self.inner.write().await;
        let index = data
            .endpoint_probe_batches
            .iter()
            .position(|batch| batch.id == batch_id)
            .ok_or_else(|| AppError::not_found("endpoint_probe_batch"))?;
        data.endpoint_probe_batches[index].status = status.to_string();
        data.endpoint_probe_batches[index].finished_at = Some(finished_at.to_string());
        Ok(with_counts(
            &data,
            data.endpoint_probe_batches[index].clone(),
        ))
    }

    pub async fn list_endpoint_probe_batches_page(
        &self,
        input: EndpointProbeHistoryPageInput,
    ) -> anyhow::Result<EndpointProbeHistoryPage> {
        let input = input.normalized();
        let keyword = input.keyword.as_ref().map(|value| value.to_lowercase());
        let data = self.inner.read().await;
        let filtered = data
            .endpoint_probe_batches
            .iter()
            .filter(|batch| {
                input
                    .status
                    .as_ref()
                    .map(|status| batch.status == *status)
                    .unwrap_or(true)
            })
            .filter(|batch| {
                keyword
                    .as_ref()
                    .map(|keyword| {
                        batch.name.to_lowercase().contains(keyword)
                            || batch
                                .prompt_preview
                                .as_ref()
                                .is_some_and(|value| value.to_lowercase().contains(keyword))
                            || data.endpoint_probe_runs.iter().any(|run| {
                                run.summary.batch_id == batch.id
                                    && [
                                        run.summary.name.as_str(),
                                        run.summary.base_url.as_str(),
                                        run.summary.model.as_str(),
                                        run.summary.error_message.as_deref().unwrap_or_default(),
                                        run.summary.response_preview.as_deref().unwrap_or_default(),
                                    ]
                                    .iter()
                                    .any(|value| value.to_lowercase().contains(keyword))
                            })
                    })
                    .unwrap_or(true)
            })
            .rev()
            .map(|batch| with_counts(&data, batch.clone()))
            .collect::<Vec<_>>();
        let total = filtered.len() as i64;
        let start = ((input.page - 1) * input.page_size).max(0) as usize;
        Ok(EndpointProbeHistoryPage {
            items: filtered
                .into_iter()
                .skip(start)
                .take(input.page_size as usize)
                .collect(),
            total,
            page: input.page,
            page_size: input.page_size,
        })
    }

    pub async fn get_endpoint_probe_batch_detail(
        &self,
        batch_id: &str,
    ) -> anyhow::Result<EndpointProbeBatchDetail> {
        let data = self.inner.read().await;
        let batch = data
            .endpoint_probe_batches
            .iter()
            .find(|batch| batch.id == batch_id)
            .cloned()
            .ok_or_else(|| AppError::not_found("endpoint_probe_batch"))?;
        Ok(EndpointProbeBatchDetail {
            summary: with_counts(&data, batch),
            runs: data
                .endpoint_probe_runs
                .iter()
                .filter(|run| run.summary.batch_id == batch_id)
                .map(|run| run.summary.clone())
                .collect(),
        })
    }

    pub async fn get_endpoint_probe_run_detail(
        &self,
        run_id: &str,
    ) -> anyhow::Result<EndpointProbeRunDetail> {
        let data = self.inner.read().await;
        data.endpoint_probe_runs
            .iter()
            .find(|run| run.summary.id == run_id)
            .cloned()
            .ok_or_else(|| AppError::not_found("endpoint_probe_run").into())
    }

    pub async fn delete_endpoint_probe_batch(
        &self,
        batch_id: &str,
    ) -> anyhow::Result<DeleteResult> {
        let mut data = self.inner.write().await;
        if data.endpoint_probe_batches.iter().any(|batch| {
            batch.id == batch_id && matches!(batch.status.as_str(), "pending" | "running")
        }) {
            return Ok(DeleteResult {
                id: batch_id.to_string(),
                deleted: false,
            });
        }
        let before = data.endpoint_probe_batches.len();
        data.endpoint_probe_batches
            .retain(|batch| batch.id != batch_id);
        data.endpoint_probe_runs
            .retain(|run| run.summary.batch_id != batch_id);
        Ok(DeleteResult {
            id: batch_id.to_string(),
            deleted: data.endpoint_probe_batches.len() != before,
        })
    }

    pub async fn recover_endpoint_probe_batches(&self, message: &str) -> anyhow::Result<()> {
        let mut data = self.inner.write().await;
        let finished_at = chrono::Utc::now().to_rfc3339();
        for run in &mut data.endpoint_probe_runs {
            if matches!(run.summary.status.as_str(), "pending" | "running") {
                run.summary.status = "failed".to_string();
                run.summary.error_kind = Some("orphaned".to_string());
                run.summary.error_message = Some(message.to_string());
                run.summary.finished_at = Some(finished_at.clone());
            }
        }
        for batch in &mut data.endpoint_probe_batches {
            if matches!(batch.status.as_str(), "pending" | "running") {
                batch.status = "failed".to_string();
                batch.finished_at = Some(finished_at.clone());
            }
        }
        Ok(())
    }
}

fn detail_from_record(record: &EndpointProbeRunRecord) -> EndpointProbeRunDetail {
    let mut summary = record.summary.clone();
    summary.body_available = record.body_ref.is_some()
        || record.prompt.is_some()
        || record.response_text.is_some()
        || record.request_payload.is_some()
        || record.raw_error.is_some()
        || record.raw_usage.is_some();
    EndpointProbeRunDetail {
        summary,
        prompt: record.prompt.clone(),
        response_text: record.response_text.clone(),
        request_payload: record.request_payload.clone(),
        raw_error: record.raw_error.clone(),
        raw_usage: record.raw_usage.clone(),
    }
}

fn with_counts(
    data: &super::types::MockData,
    mut batch: EndpointProbeBatchSummary,
) -> EndpointProbeBatchSummary {
    let runs = data
        .endpoint_probe_runs
        .iter()
        .filter(|run| run.summary.batch_id == batch.id)
        .collect::<Vec<_>>();
    batch.total_runs = runs.len() as i64;
    batch.pending_runs = count_status(&runs, "pending");
    batch.running_runs = count_status(&runs, "running");
    batch.passed_runs = count_status(&runs, "passed");
    batch.failed_runs = count_status(&runs, "failed");
    batch.cancelled_runs = count_status(&runs, "cancelled");
    batch
}

fn count_status(runs: &[&EndpointProbeRunDetail], status: &str) -> i64 {
    runs.iter()
        .filter(|run| run.summary.status == status)
        .count() as i64
}
