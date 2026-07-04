use super::MockDataStore;
use crate::models::DashboardSummary;

impl MockDataStore {
    pub async fn dashboard_summary(&self) -> anyhow::Result<DashboardSummary> {
        let data = self.inner.read().await;
        Ok(DashboardSummary {
            providers: data.providers.len() as i64,
            models: data.models.len() as i64,
            tasks: data.tasks.len() as i64,
            reports: data.reports.len() as i64,
            recent_tasks: data
                .tasks
                .iter()
                .rev()
                .take(4)
                .map(|task| task.summary.clone())
                .collect(),
            recent_reports: data.reports.iter().rev().take(4).cloned().collect(),
        })
    }
}
