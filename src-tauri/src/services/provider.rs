use super::provider_demo;
use crate::benchmark::engines::real::{classify_model, RealProviderClient};
use crate::config::BenchmarkEngineMode;
use crate::error::AppResult;
use crate::models::{
    CreateProviderInput, DeleteResult, ModelSummary, ProviderConnectionResult,
    ProviderDiagnosticsInput, ProviderDiagnosticsResult, ProviderImportInput,
    ProviderImportItemResult, ProviderImportResult, ProviderModelScanResult, ProviderSummary,
    UpdateProviderInput,
};
use crate::state::AppState;
use chrono::Utc;
use reqwest::Url;

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

pub async fn import_providers(
    state: &AppState,
    input: ProviderImportInput,
) -> AppResult<ProviderImportResult> {
    if input.items.is_empty() {
        return Err(crate::error::AppError::validation("导入清单不能为空。"));
    }
    if input.items.len() > 200 {
        return Err(crate::error::AppError::validation(
            "单次最多导入 200 个服务商连接。",
        ));
    }

    let mut result = ProviderImportResult {
        created: 0,
        skipped: 0,
        failed: 0,
        items: Vec::with_capacity(input.items.len()),
    };
    for (index, item) in input.items.into_iter().enumerate() {
        let index = index as i64 + 1;
        if !matches!(
            item.interface_type.trim(),
            "OpenAI"
                | "OpenAI Compatible"
                | "OpenAI-Response"
                | "OpenAI Responses"
                | "Anthropic"
                | "Claude"
                | "Claude Messages"
        ) {
            result.failed += 1;
            result.items.push(ProviderImportItemResult {
                index,
                status: "failed".to_string(),
                provider_id: None,
                message: "接口类型仅支持 OpenAI、OpenAI-Response 或 Anthropic。".to_string(),
            });
            continue;
        }
        let interface_type = match item.interface_type.trim() {
            "OpenAI Compatible" => "OpenAI",
            "OpenAI Responses" => "OpenAI-Response",
            "Claude" | "Claude Messages" => "Anthropic",
            value => value,
        }
        .to_string();
        let normalized_url = match normalize_import_base_url(&item.base_url) {
            Ok(value) => value,
            Err(message) => {
                result.failed += 1;
                result.items.push(ProviderImportItemResult {
                    index,
                    status: "failed".to_string(),
                    provider_id: None,
                    message,
                });
                continue;
            }
        };
        if let Some(existing) = state
            .find_provider_by_endpoint(&normalized_url, &interface_type)
            .await?
        {
            result.skipped += 1;
            result.items.push(ProviderImportItemResult {
                index,
                status: "skipped".to_string(),
                provider_id: Some(existing.id),
                message: "相同 Base URL 和接口类型的服务商已存在。".to_string(),
            });
            continue;
        }

        match state
            .create_provider(CreateProviderInput {
                name: item.name,
                base_url: normalized_url,
                api_key: item.api_key,
                interface_type,
            })
            .await
        {
            Ok(provider) => {
                let mut model_names = item
                    .models
                    .into_iter()
                    .map(|name| name.trim().to_string())
                    .filter(|name| !name.is_empty())
                    .collect::<Vec<_>>();
                model_names.sort_by_cached_key(|name| name.to_ascii_lowercase());
                model_names.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
                if !model_names.is_empty() {
                    let models = model_names
                        .iter()
                        .map(|name| classify_model(name))
                        .collect();
                    if let Err(error) = state
                        .replace_provider_models(
                            &provider.id,
                            models,
                            &chrono::Utc::now().to_rfc3339(),
                        )
                        .await
                    {
                        result.failed += 1;
                        result.items.push(ProviderImportItemResult {
                            index,
                            status: "failed".to_string(),
                            provider_id: Some(provider.id),
                            message: format!("服务商已创建，但模型写入失败：{error}"),
                        });
                        continue;
                    }
                }
                result.created += 1;
                result.items.push(ProviderImportItemResult {
                    index,
                    status: "created".to_string(),
                    provider_id: Some(provider.id),
                    message: if model_names.is_empty() {
                        "已导入为待检查服务商。".to_string()
                    } else {
                        format!("已导入服务商和 {} 个模型。", model_names.len())
                    },
                });
            }
            Err(error) => {
                result.failed += 1;
                result.items.push(ProviderImportItemResult {
                    index,
                    status: "failed".to_string(),
                    provider_id: None,
                    message: error.to_string(),
                });
            }
        }
    }
    Ok(result)
}

fn normalize_import_base_url(value: &str) -> Result<String, String> {
    let mut parsed = Url::parse(value.trim())
        .map_err(|_| "Base URL 必须是有效的 http:// 或 https:// 地址。".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("Base URL 只支持 http:// 或 https://。".to_string());
    }
    if parsed.query().is_some() || parsed.fragment().is_some() {
        return Err("Base URL 不能包含 query 或 fragment。".to_string());
    }
    parsed.set_query(None);
    parsed.set_fragment(None);
    Ok(parsed.to_string().trim_end_matches('/').to_string())
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
    use super::{import_providers, scan_provider_models, test_provider_connection};
    use crate::config::{AppConfig, BenchmarkEngineMode, DataMode};
    use crate::models::{CreateProviderInput, ProviderImportInput, ProviderImportItem};
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

    #[tokio::test]
    async fn provider_import_supports_partial_success_and_never_returns_keys() {
        let root = std::env::temp_dir().join(format!(
            "my-llm-benchmark-provider-import-{}",
            Uuid::new_v4()
        ));
        let state = AppState::initialize(root.join("config"), root.join("data"))
            .await
            .unwrap();
        let secret = "sk-import-secret";
        let result = import_providers(
            &state,
            ProviderImportInput {
                items: vec![
                    ProviderImportItem {
                        name: "Imported gateway".to_string(),
                        base_url: "https://import.example.com/v1/".to_string(),
                        api_key: Some(secret.to_string()),
                        interface_type: "OpenAI".to_string(),
                        models: vec!["model-a".to_string()],
                    },
                    ProviderImportItem {
                        name: "Duplicate gateway".to_string(),
                        base_url: "https://import.example.com/v1".to_string(),
                        api_key: Some("different-secret".to_string()),
                        interface_type: "OpenAI".to_string(),
                        models: vec![],
                    },
                    ProviderImportItem {
                        name: "Unsupported gateway".to_string(),
                        base_url: "https://gemini.example.com".to_string(),
                        api_key: Some("gemini-secret".to_string()),
                        interface_type: "Gemini".to_string(),
                        models: vec![],
                    },
                ],
            },
        )
        .await
        .unwrap();

        assert_eq!(result.created, 1);
        assert_eq!(result.skipped, 1);
        assert_eq!(result.failed, 1);
        assert_eq!(result.items[0].status, "created");
        assert_eq!(result.items[1].status, "skipped");
        assert_eq!(result.items[2].status, "failed");
        let serialized = serde_json::to_string(&result).unwrap();
        assert!(!serialized.contains(secret));
        assert!(!serialized.contains("different-secret"));
        assert!(!serialized.contains("gemini-secret"));

        let provider_id = result.items[0].provider_id.as_deref().unwrap();
        let provider = state
            .list_providers()
            .await
            .unwrap()
            .into_iter()
            .find(|provider| provider.id == provider_id)
            .unwrap();
        assert_eq!(provider.status, "unchecked");
        assert_eq!(
            state.list_provider_models(provider_id).await.unwrap().len(),
            1
        );

        let _ = std::fs::remove_dir_all(root);
    }
}
