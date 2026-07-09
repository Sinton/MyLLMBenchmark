use crate::benchmark::engines::openai::OpenAICompatibleClient;
use crate::config::BenchmarkEngineMode;
use crate::error::AppResult;
use crate::models::{
    CreateProviderInput, DeleteResult, ModelSummary, ProviderConnectionResult,
    ProviderDiagnosticsInput, ProviderDiagnosticsResult, ProviderModelScanResult, ProviderSummary,
    UpdateProviderInput,
};
use crate::state::AppState;
use chrono::Utc;

pub async fn list_providers(state: &AppState) -> AppResult<Vec<ProviderSummary>> {
    Ok(state.list_providers().await?)
}

pub async fn create_provider(
    state: &AppState,
    input: CreateProviderInput,
) -> AppResult<ProviderSummary> {
    Ok(state.create_provider(input).await?)
}

pub async fn update_provider(
    state: &AppState,
    provider_id: &str,
    input: UpdateProviderInput,
) -> AppResult<ProviderSummary> {
    Ok(state.update_provider(provider_id, input).await?)
}

pub async fn delete_provider(state: &AppState, provider_id: &str) -> AppResult<DeleteResult> {
    Ok(state.delete_provider(provider_id).await?)
}

pub async fn test_provider_connection(
    state: &AppState,
    provider_id: &str,
) -> AppResult<ProviderConnectionResult> {
    let engine_mode = state.current_config().await?.benchmark_engine;
    if engine_mode == BenchmarkEngineMode::Mock {
        return Ok(state.test_provider_connection(provider_id).await?);
    }

    let checked_at = Utc::now().to_rfc3339();
    let config = state.provider_connection_config(provider_id).await?;
    let client = OpenAICompatibleClient::new()?;
    match client.list_models(&config).await {
        Ok(models) => {
            state
                .update_provider_connection_status(provider_id, "online", &checked_at)
                .await?;
            Ok(ProviderConnectionResult {
                provider_id: provider_id.to_string(),
                ok: true,
                status: "online".to_string(),
                message: format!(
                    "连接成功，已从 {} 获取到 {} 个模型。",
                    config.name,
                    models.len()
                ),
                checked_at,
            })
        }
        Err(error) => {
            let _ = state
                .update_provider_connection_status(provider_id, "offline", &checked_at)
                .await;
            Ok(ProviderConnectionResult {
                provider_id: provider_id.to_string(),
                ok: false,
                status: "offline".to_string(),
                message: format!("连接失败：{}", error),
                checked_at,
            })
        }
    }
}

pub async fn list_provider_models(
    state: &AppState,
    provider_id: &str,
) -> AppResult<Vec<ModelSummary>> {
    Ok(state.list_provider_models(provider_id).await?)
}

pub async fn scan_provider_models(
    state: &AppState,
    provider_id: &str,
) -> AppResult<ProviderModelScanResult> {
    let engine_mode = state.current_config().await?.benchmark_engine;
    if engine_mode == BenchmarkEngineMode::Mock {
        return Ok(state.scan_provider_models(provider_id).await?);
    }

    let scanned_at = Utc::now().to_rfc3339();
    let config = state.provider_connection_config(provider_id).await?;
    let client = OpenAICompatibleClient::new()?;
    let discovered = client.list_models(&config).await?;
    let models = state
        .replace_provider_models(provider_id, discovered, &scanned_at)
        .await?;
    state
        .update_provider_connection_status(provider_id, "online", &scanned_at)
        .await?;

    Ok(ProviderModelScanResult {
        provider_id: provider_id.to_string(),
        message: format!("已扫描到 {} 个真实模型。", models.len()),
        models,
        scanned_at,
    })
}

pub async fn diagnose_provider(
    state: &AppState,
    input: ProviderDiagnosticsInput,
) -> AppResult<ProviderDiagnosticsResult> {
    let checked_at = Utc::now().to_rfc3339();
    let config = state.provider_connection_config(&input.provider_id).await?;
    let models = state
        .list_provider_models(&input.provider_id)
        .await
        .unwrap_or_default();
    let samples = if let Some(dataset_id) = input
        .dataset_id
        .as_deref()
        .filter(|dataset_id| !dataset_id.trim().is_empty())
    {
        state
            .list_dataset_samples(dataset_id)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let client = OpenAICompatibleClient::new()?;
    let result = client
        .diagnose_provider(
            &config,
            &input,
            &models,
            &samples,
            state.current_config().await?.benchmark_engine,
            checked_at,
        )
        .await;
    state.save_provider_diagnostics(&result).await?;
    Ok(result)
}

pub async fn get_provider_diagnostics(
    state: &AppState,
    provider_id: &str,
) -> AppResult<Option<ProviderDiagnosticsResult>> {
    Ok(state.get_provider_diagnostics(provider_id).await?)
}
