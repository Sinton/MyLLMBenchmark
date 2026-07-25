use super::provider_demo;
use crate::benchmark::engines::real::RealProviderClient;
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
        let checked_at = Utc::now().to_rfc3339();
        let config = state.provider_connection_config(provider_id).await?;
        let result = provider_demo::test_connection(&config, checked_at.clone());
        state
            .update_provider_connection_status(provider_id, "online", &checked_at)
            .await?;
        return Ok(result);
    }

    let checked_at = Utc::now().to_rfc3339();
    let config = state.provider_connection_config(provider_id).await?;
    let client = RealProviderClient::new()?;
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
                message: format!("连接失败：{error}"),
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
        let scanned_at = Utc::now().to_rfc3339();
        let config = state.provider_connection_config(provider_id).await?;
        let discovered = provider_demo::discover_models(&config);
        let models = state
            .replace_provider_models(provider_id, discovered, &scanned_at)
            .await?;
        state
            .update_provider_connection_status(provider_id, "online", &scanned_at)
            .await?;
        return Ok(ProviderModelScanResult {
            provider_id: provider_id.to_string(),
            message: format!("已扫描到 {} 个演示模型（Mock 引擎）。", models.len()),
            models,
            scanned_at,
        });
    }

    let scanned_at = Utc::now().to_rfc3339();
    let config = state.provider_connection_config(provider_id).await?;
    let client = RealProviderClient::new()?;
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
    let client = RealProviderClient::new()?;
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

#[cfg(test)]
mod tests {
    use super::{scan_provider_models, test_provider_connection};
    use crate::config::{AppConfig, BenchmarkEngineMode, DataMode};
    use crate::models::CreateProviderInput;
    use crate::state::AppState;
    use uuid::Uuid;

    #[tokio::test]
    async fn mock_engine_uses_demo_gateway_with_sqlite_data_source() {
        let root = std::env::temp_dir().join(format!(
            "my-llm-benchmark-provider-demo-sqlite-{}",
            Uuid::new_v4()
        ));
        let state = AppState::initialize(root.join("config"), root.join("data"))
            .await
            .unwrap();
        state
            .save_config(AppConfig {
                data_mode: DataMode::Sqlite,
                benchmark_engine: BenchmarkEngineMode::Mock,
            })
            .await
            .unwrap();
        let provider = state
            .create_provider(CreateProviderInput {
                name: "SQLite Demo Gateway Provider".to_string(),
                base_url: "http://127.0.0.1:8000/v1".to_string(),
                api_key: Some("demo-key".to_string()),
                interface_type: "OpenAI".to_string(),
            })
            .await
            .unwrap();

        let connection = test_provider_connection(&state, &provider.id)
            .await
            .unwrap();
        assert!(connection.ok);
        assert_eq!(connection.status, "online");
        assert!(connection.message.contains("Mock 引擎"));

        let scanned = scan_provider_models(&state, &provider.id).await.unwrap();
        assert!(!scanned.models.is_empty());
        assert!(scanned
            .models
            .iter()
            .any(|model| model.name.contains("Demo")));
        assert_eq!(
            state
                .list_provider_models(&provider.id)
                .await
                .unwrap()
                .len(),
            scanned.models.len()
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn mock_engine_uses_demo_gateway_with_mock_data_source() {
        let root = std::env::temp_dir().join(format!(
            "my-llm-benchmark-provider-demo-mock-{}",
            Uuid::new_v4()
        ));
        let state = AppState::initialize(root.join("config"), root.join("data"))
            .await
            .unwrap();

        let connection = test_provider_connection(&state, "mock-provider-openai")
            .await
            .unwrap();
        assert!(connection.ok);
        let scanned = scan_provider_models(&state, "mock-provider-openai")
            .await
            .unwrap();
        assert!(!scanned.models.is_empty());
        assert!(scanned
            .models
            .iter()
            .any(|model| model.name.contains("Demo")));

        let _ = std::fs::remove_dir_all(root);
    }
}
