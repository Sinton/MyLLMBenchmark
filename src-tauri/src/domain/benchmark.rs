use crate::error::{AppError, AppResult};
use crate::models::BenchmarkStartInput;

pub fn validate_benchmark_start(input: &BenchmarkStartInput) -> AppResult<()> {
    if input.provider_id.trim().is_empty() {
        return Err(AppError::validation("provider_id is required"));
    }
    if input.dataset_id.trim().is_empty() {
        return Err(AppError::validation("dataset_id is required"));
    }
    if input.concurrency < 1 {
        return Err(AppError::validation("concurrency must be at least 1"));
    }
    if input.duration_seconds < 1 {
        return Err(AppError::validation("duration_seconds must be at least 1"));
    }
    Ok(())
}
