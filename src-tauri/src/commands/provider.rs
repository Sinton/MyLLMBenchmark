use super::error_to_string;
use crate::models::{
    CreateProviderInput, DeleteResult, ModelSummary, ProviderConnectionResult,
    ProviderDiagnosticsInput, ProviderDiagnosticsResult, ProviderModelScanResult, ProviderSummary,
    UpdateProviderInput,
};
use crate::services;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_providers(state: State<'_, AppState>) -> Result<Vec<ProviderSummary>, String> {
    services::list_providers(state.inner())
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn create_provider(
    state: State<'_, AppState>,
    input: CreateProviderInput,
) -> Result<ProviderSummary, String> {
    services::create_provider(state.inner(), input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn update_provider(
    state: State<'_, AppState>,
    provider_id: String,
    input: UpdateProviderInput,
) -> Result<ProviderSummary, String> {
    services::update_provider(state.inner(), &provider_id, input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn delete_provider(
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<DeleteResult, String> {
    services::delete_provider(state.inner(), &provider_id)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn test_provider_connection(
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<ProviderConnectionResult, String> {
    services::test_provider_connection(state.inner(), &provider_id)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn list_provider_models(
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<Vec<ModelSummary>, String> {
    services::list_provider_models(state.inner(), &provider_id)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn scan_provider_models(
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<ProviderModelScanResult, String> {
    services::scan_provider_models(state.inner(), &provider_id)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn diagnose_provider(
    state: State<'_, AppState>,
    input: ProviderDiagnosticsInput,
) -> Result<ProviderDiagnosticsResult, String> {
    services::diagnose_provider(state.inner(), input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn get_provider_diagnostics(
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<Option<ProviderDiagnosticsResult>, String> {
    services::get_provider_diagnostics(state.inner(), &provider_id)
        .await
        .map_err(error_to_string)
}
