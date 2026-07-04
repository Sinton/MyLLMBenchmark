use super::error_to_string;
use crate::models::DashboardSummary;
use crate::services;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_dashboard_summary(state: State<'_, AppState>) -> Result<DashboardSummary, String> {
    services::get_dashboard_summary(state.inner())
        .await
        .map_err(error_to_string)
}
