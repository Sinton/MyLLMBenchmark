use super::error_to_string;
use crate::models::{
    BenchmarkRequestLogDetail, BenchmarkRequestLogPage, BenchmarkRequestLogPageInput,
    BenchmarkStartInput, BenchmarkTaskSummary, DeleteResult, MetricsTick, StopResult,
};
use crate::services;
use crate::state::AppState;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn start_benchmark(
    app: AppHandle,
    state: State<'_, AppState>,
    input: BenchmarkStartInput,
) -> Result<BenchmarkTaskSummary, String> {
    services::start_benchmark(app, state.inner(), input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn stop_benchmark(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<StopResult, String> {
    services::stop_benchmark(state.inner(), &task_id)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn get_benchmark_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<BenchmarkTaskSummary, String> {
    services::get_benchmark_task(state.inner(), &task_id)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn list_benchmark_ticks(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Vec<MetricsTick>, String> {
    services::list_benchmark_ticks(state.inner(), &task_id)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn list_benchmark_request_logs_page(
    state: State<'_, AppState>,
    input: BenchmarkRequestLogPageInput,
) -> Result<BenchmarkRequestLogPage, String> {
    services::list_benchmark_request_logs_page(state.inner(), input)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn get_benchmark_request_log_detail(
    state: State<'_, AppState>,
    request_id: String,
) -> Result<BenchmarkRequestLogDetail, String> {
    services::get_benchmark_request_log_detail(state.inner(), &request_id)
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn delete_benchmark_request_logs(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<DeleteResult, String> {
    services::delete_benchmark_request_logs(state.inner(), &task_id)
        .await
        .map_err(error_to_string)
}
