use super::error_to_string;
use crate::models::{
    DeleteResult, SiteProbeHistoryPage, SiteProbeHistoryPageInput, SiteProbeModelScanInput,
    SiteProbeModelScanResult, SiteProbeRunDetail, SiteProbeRunInput,
};
use crate::services;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn run_site_probe(
    state: State<'_, AppState>,
    input: SiteProbeRunInput,
) -> Result<SiteProbeRunDetail, String> {
    services::run_site_probe(state.inner(), input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn scan_site_probe_models(
    input: SiteProbeModelScanInput,
) -> Result<SiteProbeModelScanResult, String> {
    services::scan_site_probe_models(input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn list_site_probe_runs_page(
    state: State<'_, AppState>,
    input: SiteProbeHistoryPageInput,
) -> Result<SiteProbeHistoryPage, String> {
    services::list_site_probe_runs_page(state.inner(), input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn get_site_probe_run_detail(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<SiteProbeRunDetail, String> {
    services::get_site_probe_run_detail(state.inner(), &run_id)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn delete_site_probe_run(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<DeleteResult, String> {
    services::delete_site_probe_run(state.inner(), &run_id)
        .await
        .map_err(error_to_string)
}
