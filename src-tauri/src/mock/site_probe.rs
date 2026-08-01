use super::MockDataStore;
use crate::error::AppError;
use crate::models::{
    DeleteResult, SiteProbeHistoryPage, SiteProbeHistoryPageInput, SiteProbeRunDetail,
    SiteProbeRunRecord, SiteProbeRunSummary,
};

impl MockDataStore {
    pub async fn insert_site_probe_run(
        &self,
        record: SiteProbeRunRecord,
    ) -> anyhow::Result<SiteProbeRunSummary> {
        let mut data = self.inner.write().await;
        let mut summary = record.summary;
        summary.body_available = record.prompt.is_some()
            || record.response_text.is_some()
            || record.request_payload.is_some()
            || record.raw_error.is_some()
            || record.raw_usage.is_some();
        let detail = SiteProbeRunDetail {
            summary: summary.clone(),
            prompt: record.prompt,
            response_text: record.response_text,
            request_payload: record.request_payload,
            raw_error: record.raw_error,
            raw_usage: record.raw_usage,
        };
        data.site_probe_runs.push(detail);
        Ok(summary)
    }

    pub async fn list_site_probe_runs_page(
        &self,
        input: SiteProbeHistoryPageInput,
    ) -> anyhow::Result<SiteProbeHistoryPage> {
        let input = input.normalized();
        let keyword = input.keyword.as_ref().map(|value| value.to_lowercase());
        let data = self.inner.read().await;
        let filtered = data
            .site_probe_runs
            .iter()
            .map(|detail| detail.summary.clone())
            .filter(|summary| {
                input
                    .status
                    .as_ref()
                    .map(|status| summary.status == *status)
                    .unwrap_or(true)
            })
            .filter(|summary| {
                keyword
                    .as_ref()
                    .map(|keyword| {
                        [
                            summary.name.as_str(),
                            summary.base_url.as_str(),
                            summary.model.as_str(),
                            summary.prompt_preview.as_deref().unwrap_or(""),
                            summary.response_preview.as_deref().unwrap_or(""),
                            summary.error_message.as_deref().unwrap_or(""),
                        ]
                        .iter()
                        .any(|value| value.to_lowercase().contains(keyword))
                    })
                    .unwrap_or(true)
            })
            .rev()
            .collect::<Vec<_>>();
        let total = filtered.len() as i64;
        let start = ((input.page - 1) * input.page_size).max(0) as usize;
        Ok(SiteProbeHistoryPage {
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

    pub async fn get_site_probe_run_detail(
        &self,
        run_id: &str,
    ) -> anyhow::Result<SiteProbeRunDetail> {
        let data = self.inner.read().await;
        data.site_probe_runs
            .iter()
            .find(|detail| detail.summary.id == run_id)
            .cloned()
            .ok_or_else(|| AppError::not_found("site_probe_run").into())
    }

    pub async fn delete_site_probe_run(&self, run_id: &str) -> anyhow::Result<DeleteResult> {
        let mut data = self.inner.write().await;
        let before = data.site_probe_runs.len();
        data.site_probe_runs
            .retain(|detail| detail.summary.id != run_id);
        Ok(DeleteResult {
            id: run_id.to_string(),
            deleted: data.site_probe_runs.len() != before,
        })
    }
}
