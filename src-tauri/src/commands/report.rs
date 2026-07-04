use super::error_to_string;
use crate::models::{ReportDetail, ReportExportInput, ReportExportResult, ReportSummary};
use crate::services;
use crate::state::AppState;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn generate_report(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: String,
) -> Result<ReportSummary, String> {
    services::generate_report(app, state.inner(), &task_id)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn list_reports(state: State<'_, AppState>) -> Result<Vec<ReportSummary>, String> {
    services::list_reports(state.inner())
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn get_report_detail(
    state: State<'_, AppState>,
    report_id: String,
) -> Result<ReportDetail, String> {
    services::get_report_detail(state.inner(), &report_id)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn export_report(
    app: AppHandle,
    state: State<'_, AppState>,
    input: ReportExportInput,
) -> Result<ReportExportResult, String> {
    services::export_report(app, state.inner(), input)
        .await
        .map_err(error_to_string)
}
