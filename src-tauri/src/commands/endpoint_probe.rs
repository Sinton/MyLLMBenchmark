use super::error_to_string;
use crate::models::{
    DeleteResult, EndpointProbeBatchDetail, EndpointProbeBatchSummary, EndpointProbeHistoryPage,
    EndpointProbeHistoryPageInput, EndpointProbeModelScanInput, EndpointProbeModelScanResult,
    EndpointProbePromotionInput, EndpointProbePromotionResult, EndpointProbeRunDetail,
    EndpointProbeStartInput, EndpointProbeStopResult,
};
use crate::services;
use crate::state::AppState;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn start_endpoint_probe(
    app: AppHandle,
    state: State<'_, AppState>,
    input: EndpointProbeStartInput,
) -> Result<EndpointProbeBatchSummary, String> {
    services::start_endpoint_probe(app, state.inner(), input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn stop_endpoint_probe(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<EndpointProbeStopResult, String> {
    services::stop_endpoint_probe(state.inner(), &batch_id)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn scan_endpoint_probe_models(
    state: State<'_, AppState>,
    input: EndpointProbeModelScanInput,
) -> Result<EndpointProbeModelScanResult, String> {
    services::scan_endpoint_probe_models(state.inner(), input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn promote_endpoint_probe_target(
    state: State<'_, AppState>,
    input: EndpointProbePromotionInput,
) -> Result<EndpointProbePromotionResult, String> {
    services::promote_endpoint_probe_target(state.inner(), input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn list_endpoint_probe_batches_page(
    state: State<'_, AppState>,
    input: EndpointProbeHistoryPageInput,
) -> Result<EndpointProbeHistoryPage, String> {
    services::list_endpoint_probe_batches_page(state.inner(), input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn get_endpoint_probe_batch_detail(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<EndpointProbeBatchDetail, String> {
    services::get_endpoint_probe_batch_detail(state.inner(), &batch_id)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn get_endpoint_probe_run_detail(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<EndpointProbeRunDetail, String> {
    services::get_endpoint_probe_run_detail(state.inner(), &run_id)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn delete_endpoint_probe_batch(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<DeleteResult, String> {
    services::delete_endpoint_probe_batch(state.inner(), &batch_id)
        .await
        .map_err(error_to_string)
}
