use super::error_to_string;
use crate::models::{
    DatasetAppendInput, DatasetExportInput, DatasetExportResult, DatasetImportInput,
    DatasetSampleBatchDeleteInput, DatasetSampleCreateInput, DatasetSamplePage,
    DatasetSamplePageInput, DatasetSamplePreview, DatasetSampleUpdateInput, DatasetSummary,
    DatasetUpdateInput, DatasetValidationResult, DeleteResult,
};
use crate::services;
use crate::state::AppState;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn list_datasets(state: State<'_, AppState>) -> Result<Vec<DatasetSummary>, String> {
    services::list_datasets(state.inner())
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn import_dataset(
    state: State<'_, AppState>,
    input: DatasetImportInput,
) -> Result<DatasetSummary, String> {
    services::import_dataset(state.inner(), input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn update_dataset(
    state: State<'_, AppState>,
    input: DatasetUpdateInput,
) -> Result<DatasetSummary, String> {
    services::update_dataset(state.inner(), input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn delete_dataset(
    state: State<'_, AppState>,
    dataset_id: String,
) -> Result<DeleteResult, String> {
    services::delete_dataset(state.inner(), &dataset_id)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn preview_dataset_samples(
    state: State<'_, AppState>,
    dataset_id: String,
    limit: Option<i64>,
) -> Result<Vec<DatasetSamplePreview>, String> {
    services::preview_dataset_samples(state.inner(), &dataset_id, limit)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn list_dataset_samples_page(
    state: State<'_, AppState>,
    input: DatasetSamplePageInput,
) -> Result<DatasetSamplePage, String> {
    services::list_dataset_samples_page(state.inner(), input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn create_dataset_sample(
    state: State<'_, AppState>,
    input: DatasetSampleCreateInput,
) -> Result<DatasetSamplePreview, String> {
    services::create_dataset_sample(state.inner(), input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn update_dataset_sample(
    state: State<'_, AppState>,
    input: DatasetSampleUpdateInput,
) -> Result<DatasetSamplePreview, String> {
    services::update_dataset_sample(state.inner(), input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn delete_dataset_sample(
    state: State<'_, AppState>,
    sample_id: String,
) -> Result<DeleteResult, String> {
    services::delete_dataset_sample(state.inner(), &sample_id)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn append_dataset_samples(
    state: State<'_, AppState>,
    input: DatasetAppendInput,
) -> Result<DatasetSummary, String> {
    services::append_dataset_samples(state.inner(), input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn delete_dataset_samples_batch(
    state: State<'_, AppState>,
    input: DatasetSampleBatchDeleteInput,
) -> Result<DeleteResult, String> {
    services::delete_dataset_samples_batch(state.inner(), input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn export_dataset(
    app: AppHandle,
    state: State<'_, AppState>,
    input: DatasetExportInput,
) -> Result<DatasetExportResult, String> {
    services::export_dataset(app, state.inner(), input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn validate_dataset_samples(
    state: State<'_, AppState>,
    dataset_id: String,
) -> Result<DatasetValidationResult, String> {
    services::validate_dataset_samples(state.inner(), &dataset_id)
        .await
        .map_err(error_to_string)
}
