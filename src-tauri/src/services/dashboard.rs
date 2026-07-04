use crate::error::AppResult;
use crate::models::DashboardSummary;
use crate::state::AppState;

pub async fn get_dashboard_summary(state: &AppState) -> AppResult<DashboardSummary> {
    Ok(state.dashboard_summary().await?)
}
