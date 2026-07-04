use crate::error::AppResult;
use crate::models::{
    DatasetAppendInput, DatasetExportInput, DatasetExportResult, DatasetImportInput,
    DatasetSampleBatchDeleteInput, DatasetSampleCreateInput, DatasetSamplePage,
    DatasetSamplePageInput, DatasetSamplePreview, DatasetSampleUpdateInput, DatasetSummary,
    DatasetUpdateInput, DatasetValidationResult, DeleteResult,
};
use crate::state::AppState;
use crate::{domain::dataset_tools, error::AppError};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub async fn list_datasets(state: &AppState) -> AppResult<Vec<DatasetSummary>> {
    Ok(state.list_datasets().await?)
}

pub async fn import_dataset(
    state: &AppState,
    input: DatasetImportInput,
) -> AppResult<DatasetSummary> {
    Ok(state.import_dataset(input).await?)
}

pub async fn update_dataset(
    state: &AppState,
    input: DatasetUpdateInput,
) -> AppResult<DatasetSummary> {
    Ok(state.update_dataset(input).await?)
}

pub async fn delete_dataset(state: &AppState, dataset_id: &str) -> AppResult<DeleteResult> {
    Ok(state.delete_dataset(dataset_id).await?)
}

pub async fn preview_dataset_samples(
    state: &AppState,
    dataset_id: &str,
    limit: Option<i64>,
) -> AppResult<Vec<DatasetSamplePreview>> {
    Ok(state
        .preview_dataset_samples(dataset_id, limit.unwrap_or(10))
        .await?)
}

pub async fn list_dataset_samples_page(
    state: &AppState,
    input: DatasetSamplePageInput,
) -> AppResult<DatasetSamplePage> {
    Ok(state.list_dataset_samples_page(input).await?)
}

pub async fn create_dataset_sample(
    state: &AppState,
    input: DatasetSampleCreateInput,
) -> AppResult<DatasetSamplePreview> {
    Ok(state.create_dataset_sample(input).await?)
}

pub async fn update_dataset_sample(
    state: &AppState,
    input: DatasetSampleUpdateInput,
) -> AppResult<DatasetSamplePreview> {
    Ok(state.update_dataset_sample(input).await?)
}

pub async fn delete_dataset_sample(state: &AppState, sample_id: &str) -> AppResult<DeleteResult> {
    Ok(state.delete_dataset_sample(sample_id).await?)
}

pub async fn append_dataset_samples(
    state: &AppState,
    input: DatasetAppendInput,
) -> AppResult<DatasetSummary> {
    Ok(state.append_dataset_samples(input).await?)
}

pub async fn delete_dataset_samples_batch(
    state: &AppState,
    input: DatasetSampleBatchDeleteInput,
) -> AppResult<DeleteResult> {
    Ok(state.delete_dataset_samples_batch(input).await?)
}

pub async fn validate_dataset_samples(
    state: &AppState,
    dataset_id: &str,
) -> AppResult<DatasetValidationResult> {
    Ok(state.validate_dataset_samples(dataset_id).await?)
}

pub async fn export_dataset(
    app: AppHandle,
    state: &AppState,
    input: DatasetExportInput,
) -> AppResult<DatasetExportResult> {
    let datasets = state.list_datasets().await?;
    let dataset = datasets
        .into_iter()
        .find(|dataset| dataset.id == input.dataset_id)
        .ok_or_else(|| AppError::not_found("dataset"))?;
    let samples = state.list_dataset_samples(&input.dataset_id).await?;
    let payload = dataset_tools::render_dataset_export(&samples, &input.format);
    let file_name = format!(
        "{}-{}.{}",
        sanitize_file_part(&dataset.name),
        chrono::Local::now().format("%Y%m%d%H%M%S"),
        payload.file_extension
    );
    let export_dir = app
        .path()
        .app_data_dir()
        .map_err(anyhow::Error::from)?
        .join("exports")
        .join("datasets");
    tokio::fs::create_dir_all(&export_dir)
        .await
        .map_err(anyhow::Error::from)?;
    let file_path = export_dir.join(&file_name);
    tokio::fs::write(&file_path, &payload.bytes)
        .await
        .map_err(anyhow::Error::from)?;
    Ok(dataset_tools::dataset_export_result(
        &dataset,
        &payload,
        file_name,
        path_to_string(file_path),
    ))
}

fn sanitize_file_part(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| match ch {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            _ => ch,
        })
        .collect::<String>();
    let sanitized = sanitized.trim().trim_matches('-');
    if sanitized.is_empty() {
        "dataset".to_string()
    } else {
        sanitized.chars().take(48).collect()
    }
}

fn path_to_string(path: PathBuf) -> String {
    path.to_string_lossy().to_string()
}
