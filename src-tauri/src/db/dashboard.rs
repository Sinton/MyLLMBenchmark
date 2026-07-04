use super::rows::count;
use super::Database;
use crate::models::DashboardSummary;

impl Database {
    pub async fn dashboard_summary(&self) -> anyhow::Result<DashboardSummary> {
        let providers = count(&self.pool, "providers").await?;
        let models = count(&self.pool, "models").await?;
        let tasks = count(&self.pool, "benchmark_tasks").await?;
        let reports = count(&self.pool, "reports").await?;

        Ok(DashboardSummary {
            providers,
            models,
            tasks,
            reports,
            recent_tasks: self.list_recent_tasks(4).await?,
            recent_reports: self.list_reports_limit(4).await?,
        })
    }
}
