use super::{now, Database};
use crate::domain::provider::{
    prepare_provider_create, prepare_provider_update, ExistingProviderConfig,
};
use crate::error::AppError;
use crate::models::{
    CreateProviderInput, DiscoveredModel, ModelSummary, ProviderConnectionConfig,
    ProviderDiagnosticsResult, ProviderSummary, UpdateProviderInput,
};
use crate::security::mask_secret;
use sqlx::Row;
use uuid::Uuid;

impl Database {
    pub async fn list_providers(&self) -> anyhow::Result<Vec<ProviderSummary>> {
        let rows = sqlx::query(
            "SELECT p.id, p.name, p.base_url, p.api_key_masked, p.api_key_plaintext,
                    p.interface_type, p.status,
                    p.last_checked_at, p.created_at, COUNT(m.id) AS model_count
             FROM providers p
             LEFT JOIN models m ON m.provider_id = p.id
             GROUP BY p.id
             ORDER BY p.created_at DESC;",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| ProviderSummary {
                id: row.get("id"),
                name: row.get("name"),
                base_url_masked: row.get("base_url"),
                api_key_masked: display_api_key(&row),
                interface_type: row.get("interface_type"),
                status: row.get("status"),
                model_count: row.get("model_count"),
                last_checked_at: row.get("last_checked_at"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    pub async fn create_provider(
        &self,
        input: CreateProviderInput,
    ) -> anyhow::Result<ProviderSummary> {
        let prepared = prepare_provider_create(input)?;
        let id = Uuid::new_v4().to_string();
        let now = now();

        sqlx::query(
            "INSERT INTO providers
             (id, name, base_url, api_key_masked, api_key_plaintext, interface_type, status, last_checked_at, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);",
        )
        .bind(&id)
        .bind(&prepared.name)
        .bind(&prepared.base_url)
        .bind(&prepared.api_key_masked)
        .bind(&prepared.api_key_plaintext)
        .bind(&prepared.interface_type)
        .bind("unchecked")
        .bind(Option::<String>::None)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        let providers = self.list_providers().await?;
        providers
            .into_iter()
            .find(|provider| provider.id == id)
            .ok_or_else(|| AppError::storage("provider was not created").into())
    }

    pub async fn update_provider(
        &self,
        provider_id: &str,
        input: UpdateProviderInput,
    ) -> anyhow::Result<ProviderSummary> {
        let row = sqlx::query(
            "SELECT base_url, api_key_masked, api_key_plaintext, interface_type, status, last_checked_at
             FROM providers
             WHERE id = ?;",
        )
        .bind(provider_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::not_found("provider"))?;

        let current_base_url: String = row.get("base_url");
        let prepared = prepare_provider_update(
            input,
            ExistingProviderConfig {
                base_url_masked: current_base_url.clone(),
                base_url: current_base_url,
                api_key_masked: display_api_key(&row),
                api_key_plaintext: row.get("api_key_plaintext"),
                interface_type: row.get("interface_type"),
                status: row.get("status"),
                last_checked_at: row.get("last_checked_at"),
            },
        )?;

        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "UPDATE providers
             SET name = ?, base_url = ?, api_key_masked = ?, api_key_plaintext = ?, interface_type = ?, status = ?, last_checked_at = ?
             WHERE id = ?;",
        )
        .bind(&prepared.name)
        .bind(&prepared.base_url)
        .bind(&prepared.api_key_masked)
        .bind(&prepared.api_key_plaintext)
        .bind(&prepared.interface_type)
        .bind(&prepared.status)
        .bind(&prepared.last_checked_at)
        .bind(provider_id)
        .execute(&mut *tx)
        .await?;

        if prepared.config_changed {
            clear_provider_model_cache(&mut tx, provider_id).await?;
        }

        tx.commit().await?;

        let providers = self.list_providers().await?;
        providers
            .into_iter()
            .find(|provider| provider.id == provider_id)
            .ok_or_else(|| AppError::storage("provider was not updated").into())
    }

    pub async fn delete_provider(&self, provider_id: &str) -> anyhow::Result<bool> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            "DELETE FROM reports
             WHERE task_id IN (SELECT id FROM benchmark_tasks WHERE provider_id = ?);",
        )
        .bind(provider_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "DELETE FROM benchmark_stages
             WHERE task_id IN (SELECT id FROM benchmark_tasks WHERE provider_id = ?);",
        )
        .bind(provider_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query("DELETE FROM benchmark_tasks WHERE provider_id = ?;")
            .bind(provider_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM models WHERE provider_id = ?;")
            .bind(provider_id)
            .execute(&mut *tx)
            .await?;

        let result = sqlx::query("DELETE FROM providers WHERE id = ?;")
            .bind(provider_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn list_provider_models(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<Vec<ModelSummary>> {
        let rows = sqlx::query(
            "SELECT id, provider_id, name, model_type, supports_streaming,
                    capabilities, recommended_concurrency
             FROM models
             WHERE provider_id = ?
             ORDER BY created_at ASC;",
        )
        .bind(provider_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(super::rows::model_from_row).collect())
    }

    pub async fn get_provider_connection_config(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<ProviderConnectionConfig> {
        let row = sqlx::query(
            "SELECT id, name, base_url, api_key_plaintext, interface_type
             FROM providers
             WHERE id = ?;",
        )
        .bind(provider_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::not_found("provider"))?;

        Ok(ProviderConnectionConfig {
            id: row.get("id"),
            name: row.get("name"),
            base_url: row.get("base_url"),
            api_key_plaintext: row.get("api_key_plaintext"),
            interface_type: row.get("interface_type"),
        })
    }

    pub async fn find_provider_by_endpoint(
        &self,
        base_url: &str,
        interface_type: &str,
    ) -> anyhow::Result<Option<ProviderSummary>> {
        let row = sqlx::query(
            "SELECT p.id, p.name, p.base_url, p.api_key_masked, p.api_key_plaintext,
                    p.interface_type, p.status, p.last_checked_at, p.created_at,
                    COUNT(m.id) AS model_count
             FROM providers p
             LEFT JOIN models m ON m.provider_id = p.id
             WHERE LOWER(RTRIM(p.base_url, '/')) = LOWER(?)
               AND LOWER(p.interface_type) = LOWER(?)
             GROUP BY p.id
             LIMIT 1;",
        )
        .bind(base_url.trim().trim_end_matches('/'))
        .bind(interface_type.trim())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| ProviderSummary {
            id: row.get("id"),
            name: row.get("name"),
            base_url_masked: row.get("base_url"),
            api_key_masked: display_api_key(&row),
            interface_type: row.get("interface_type"),
            status: row.get("status"),
            model_count: row.get("model_count"),
            last_checked_at: row.get("last_checked_at"),
            created_at: row.get("created_at"),
        }))
    }

    pub async fn update_provider_connection_status(
        &self,
        provider_id: &str,
        status: &str,
        checked_at: &str,
    ) -> anyhow::Result<()> {
        let result = sqlx::query(
            "UPDATE providers
             SET status = ?, last_checked_at = ?
             WHERE id = ?;",
        )
        .bind(status)
        .bind(checked_at)
        .bind(provider_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("provider").into());
        }

        Ok(())
    }

    pub async fn replace_provider_models(
        &self,
        provider_id: &str,
        models: Vec<DiscoveredModel>,
        scanned_at: &str,
    ) -> anyhow::Result<Vec<ModelSummary>> {
        let provider_exists: Option<i64> =
            sqlx::query_scalar("SELECT 1 FROM providers WHERE id = ?;")
                .bind(provider_id)
                .fetch_optional(&self.pool)
                .await?;
        if provider_exists.is_none() {
            return Err(AppError::not_found("provider").into());
        }

        let mut tx = self.pool.begin().await?;

        clear_provider_model_cache(&mut tx, provider_id).await?;

        for model in models {
            sqlx::query(
                "INSERT INTO models
                 (id, provider_id, name, model_type, capabilities, supports_streaming, recommended_concurrency, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?);",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(provider_id)
            .bind(model.name)
            .bind(model.model_type)
            .bind(serde_json::to_string(&model.capabilities)?)
            .bind(if model.supports_streaming { 1 } else { 0 })
            .bind(model.recommended_concurrency)
            .bind(scanned_at)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        self.list_provider_models(provider_id).await
    }

    pub async fn save_provider_diagnostics(
        &self,
        result: &ProviderDiagnosticsResult,
    ) -> anyhow::Result<()> {
        let result_json = serde_json::to_string(result)?;
        sqlx::query(
            "INSERT INTO provider_diagnostics
             (provider_id, status, checked_at, engine_mode, result_json, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(provider_id) DO UPDATE SET
                status = excluded.status,
                checked_at = excluded.checked_at,
                engine_mode = excluded.engine_mode,
                result_json = excluded.result_json,
                updated_at = excluded.updated_at;",
        )
        .bind(&result.provider_id)
        .bind(&result.status)
        .bind(&result.checked_at)
        .bind(&result.engine_mode)
        .bind(result_json)
        .bind(now())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_provider_diagnostics(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<Option<ProviderDiagnosticsResult>> {
        let row =
            sqlx::query("SELECT result_json FROM provider_diagnostics WHERE provider_id = ?;")
                .bind(provider_id)
                .fetch_optional(&self.pool)
                .await?;
        row.map(|row| {
            let result_json: String = row.get("result_json");
            serde_json::from_str::<ProviderDiagnosticsResult>(&result_json)
                .map_err(anyhow::Error::from)
        })
        .transpose()
    }
}

async fn clear_provider_model_cache(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    provider_id: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE benchmark_tasks
         SET model_id = NULL
         WHERE provider_id = ? AND model_id IN (
             SELECT id FROM models WHERE provider_id = ?
         );",
    )
    .bind(provider_id)
    .bind(provider_id)
    .execute(&mut **tx)
    .await?;

    sqlx::query("DELETE FROM models WHERE provider_id = ?;")
        .bind(provider_id)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

fn display_api_key(row: &sqlx::sqlite::SqliteRow) -> String {
    let plaintext: String = row.get("api_key_plaintext");
    if plaintext.trim().is_empty() {
        let stored: String = row.get("api_key_masked");
        mask_secret(Some(&stored))
    } else {
        mask_secret(Some(&plaintext))
    }
}
