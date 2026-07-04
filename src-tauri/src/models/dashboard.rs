use super::{BenchmarkTaskSummary, ReportSummary};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSummary {
    pub providers: i64,
    pub models: i64,
    pub tasks: i64,
    pub reports: i64,
    pub recent_tasks: Vec<BenchmarkTaskSummary>,
    pub recent_reports: Vec<ReportSummary>,
}
