use super::Database;
use sqlx::Row;

const INITIAL_SCHEMA_VERSION: i64 = 1;
const EVIDENCE_SCHEMA_VERSION: i64 = 2;
const RELEASE_PREP_SCHEMA_VERSION: i64 = 3;
const REQUEST_LOG_SCHEMA_VERSION: i64 = 4;
const ENDPOINT_PROBE_SCHEMA_VERSION: i64 = 6;

impl Database {
    pub(super) async fn configure(&self) -> anyhow::Result<()> {
        sqlx::query("PRAGMA journal_mode = WAL;")
            .execute(&self.pool)
            .await?;
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&self.pool)
            .await?;
        sqlx::query("PRAGMA busy_timeout = 5000;")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub(super) async fn migrate(&self) -> anyhow::Result<()> {
        self.ensure_schema_migrations_table().await?;
        self.migrate_initial_schema().await?;
        self.record_migration(INITIAL_SCHEMA_VERSION, "initial_schema")
            .await?;
        self.migrate_benchmark_evidence_schema().await?;
        self.record_migration(EVIDENCE_SCHEMA_VERSION, "benchmark_evidence_schema")
            .await?;
        self.migrate_release_prep_schema().await?;
        self.record_migration(RELEASE_PREP_SCHEMA_VERSION, "release_prep_schema")
            .await?;
        self.migrate_request_log_schema().await?;
        self.record_migration(REQUEST_LOG_SCHEMA_VERSION, "request_log_schema")
            .await?;
        self.migrate_endpoint_probe_schema().await?;
        self.record_migration(ENDPOINT_PROBE_SCHEMA_VERSION, "endpoint_probe_batch_schema")
            .await?;
        Ok(())
    }

    async fn migrate_initial_schema(&self) -> anyhow::Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS providers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                base_url TEXT NOT NULL,
                api_key_masked TEXT NOT NULL,
                api_key_plaintext TEXT NOT NULL DEFAULT '',
                interface_type TEXT NOT NULL,
                status TEXT NOT NULL,
                last_checked_at TEXT,
                created_at TEXT NOT NULL
            );",
        )
        .execute(&self.pool)
        .await?;
        self.ensure_provider_columns().await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS models (
                id TEXT PRIMARY KEY,
                provider_id TEXT NOT NULL,
                name TEXT NOT NULL,
                model_type TEXT NOT NULL,
                capabilities TEXT NOT NULL DEFAULT '[]',
                supports_streaming INTEGER NOT NULL,
                recommended_concurrency INTEGER,
                created_at TEXT NOT NULL,
                FOREIGN KEY(provider_id) REFERENCES providers(id) ON DELETE CASCADE
            );",
        )
        .execute(&self.pool)
        .await?;
        self.ensure_model_columns().await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS datasets (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                dataset_type TEXT NOT NULL,
                sample_count INTEGER NOT NULL,
                average_tokens INTEGER NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT
            );",
        )
        .execute(&self.pool)
        .await?;
        self.ensure_dataset_columns().await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS benchmark_tasks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                provider_id TEXT NOT NULL,
                model_id TEXT,
                dataset_id TEXT NOT NULL,
                mode TEXT NOT NULL,
                concurrency INTEGER NOT NULL,
                duration_seconds INTEGER NOT NULL,
                workload_config TEXT NOT NULL DEFAULT '{}',
                engine_mode TEXT NOT NULL DEFAULT 'mock',
                stage_sample_rounds INTEGER NOT NULL DEFAULT 0,
                warmup_rounds INTEGER NOT NULL DEFAULT 0,
                request_timeout_seconds INTEGER NOT NULL DEFAULT 120,
                sla_stop_policy TEXT NOT NULL DEFAULT 'continue_full_staircase',
                planned_stages TEXT NOT NULL DEFAULT '[]',
                preflight_result TEXT,
                diagnostics_snapshot TEXT,
                sla_p95_ms INTEGER NOT NULL DEFAULT 5000,
                min_success_rate REAL NOT NULL DEFAULT 99,
                status TEXT NOT NULL,
                success_rate REAL NOT NULL DEFAULT 0,
                p95_latency_ms INTEGER NOT NULL DEFAULT 0,
                goodput_qps REAL NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                completed_at TEXT,
                FOREIGN KEY(provider_id) REFERENCES providers(id),
                FOREIGN KEY(model_id) REFERENCES models(id),
                FOREIGN KEY(dataset_id) REFERENCES datasets(id)
            );",
        )
        .execute(&self.pool)
        .await?;
        self.ensure_dataset_sample_tables().await?;
        self.ensure_task_columns().await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS benchmark_stages (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                stage_index INTEGER NOT NULL,
                concurrency INTEGER NOT NULL,
                goodput_qps REAL NOT NULL,
                p95_latency_ms INTEGER NOT NULL,
                ttft_ms INTEGER NOT NULL DEFAULT 0,
                tps REAL NOT NULL DEFAULT 0,
                success_rate REAL NOT NULL,
                error_rate REAL NOT NULL,
                input_tokens INTEGER NOT NULL DEFAULT 0,
                output_tokens INTEGER NOT NULL DEFAULT 0,
                total_tokens INTEGER NOT NULL DEFAULT 0,
                batch_size INTEGER NOT NULL DEFAULT 0,
                text_count INTEGER NOT NULL DEFAULT 0,
                documents_per_query INTEGER NOT NULL DEFAULT 0,
                pair_count INTEGER NOT NULL DEFAULT 0,
                image_count INTEGER NOT NULL DEFAULT 0,
                sample_rounds INTEGER NOT NULL DEFAULT 0,
                warmup_rounds INTEGER NOT NULL DEFAULT 0,
                request_count INTEGER NOT NULL DEFAULT 0,
                success_count INTEGER NOT NULL DEFAULT 0,
                failure_count INTEGER NOT NULL DEFAULT 0,
                sla_passed INTEGER NOT NULL DEFAULT 1,
                stop_reason TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY(task_id) REFERENCES benchmark_tasks(id) ON DELETE CASCADE
            );",
        )
        .execute(&self.pool)
        .await?;
        self.ensure_stage_columns().await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS benchmark_ticks (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                elapsed_seconds INTEGER NOT NULL,
                qps REAL NOT NULL,
                latency_ms INTEGER NOT NULL,
                ttft_ms INTEGER NOT NULL,
                tps REAL NOT NULL,
                success_rate REAL NOT NULL,
                errors INTEGER NOT NULL,
                in_flight INTEGER NOT NULL,
                input_tokens INTEGER NOT NULL DEFAULT 0,
                output_tokens INTEGER NOT NULL DEFAULT 0,
                total_tokens INTEGER NOT NULL DEFAULT 0,
                batch_size INTEGER NOT NULL DEFAULT 0,
                text_count INTEGER NOT NULL DEFAULT 0,
                documents_per_query INTEGER NOT NULL DEFAULT 0,
                pair_count INTEGER NOT NULL DEFAULT 0,
                image_count INTEGER NOT NULL DEFAULT 0,
                request_count INTEGER NOT NULL DEFAULT 0,
                success_count INTEGER NOT NULL DEFAULT 0,
                failure_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                FOREIGN KEY(task_id) REFERENCES benchmark_tasks(id) ON DELETE CASCADE
            );",
        )
        .execute(&self.pool)
        .await?;
        self.ensure_tick_columns().await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS benchmark_errors (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                error_kind TEXT NOT NULL,
                message TEXT NOT NULL,
                count INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(task_id) REFERENCES benchmark_tasks(id) ON DELETE CASCADE
            );",
        )
        .execute(&self.pool)
        .await?;
        self.ensure_request_log_table().await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS provider_diagnostics (
                provider_id TEXT PRIMARY KEY,
                status TEXT NOT NULL,
                checked_at TEXT NOT NULL,
                engine_mode TEXT NOT NULL,
                result_json TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY(provider_id) REFERENCES providers(id) ON DELETE CASCADE
            );",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS reports (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                model_name TEXT NOT NULL,
                provider_name TEXT NOT NULL,
                recommendation TEXT NOT NULL,
                recommended_concurrency INTEGER NOT NULL,
                max_stable_concurrency INTEGER NOT NULL,
                p95_latency_ms INTEGER NOT NULL,
                success_rate REAL NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(task_id) REFERENCES benchmark_tasks(id) ON DELETE CASCADE
            );",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn ensure_provider_columns(&self) -> anyhow::Result<()> {
        if !self.column_exists("providers", "api_key_plaintext").await? {
            sqlx::query(
                "ALTER TABLE providers ADD COLUMN api_key_plaintext TEXT NOT NULL DEFAULT '';",
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    async fn ensure_schema_migrations_table(&self) -> anyhow::Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );",
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn record_migration(&self, version: i64, name: &str) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO schema_migrations (version, name, applied_at)
             VALUES (?, ?, CURRENT_TIMESTAMP);",
        )
        .bind(version)
        .bind(name)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn ensure_model_columns(&self) -> anyhow::Result<()> {
        if !self.column_exists("models", "capabilities").await? {
            sqlx::query("ALTER TABLE models ADD COLUMN capabilities TEXT NOT NULL DEFAULT '[]';")
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    async fn ensure_dataset_columns(&self) -> anyhow::Result<()> {
        if !self.column_exists("datasets", "deleted_at").await? {
            sqlx::query("ALTER TABLE datasets ADD COLUMN deleted_at TEXT;")
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    async fn ensure_task_columns(&self) -> anyhow::Result<()> {
        if !self
            .column_exists("benchmark_tasks", "workload_config")
            .await?
        {
            sqlx::query(
                "ALTER TABLE benchmark_tasks ADD COLUMN workload_config TEXT NOT NULL DEFAULT '{}';",
            )
            .execute(&self.pool)
            .await?;
        }
        if !self.column_exists("benchmark_tasks", "sla_p95_ms").await? {
            sqlx::query(
                "ALTER TABLE benchmark_tasks ADD COLUMN sla_p95_ms INTEGER NOT NULL DEFAULT 5000;",
            )
            .execute(&self.pool)
            .await?;
        }
        if !self
            .column_exists("benchmark_tasks", "min_success_rate")
            .await?
        {
            sqlx::query(
                "ALTER TABLE benchmark_tasks ADD COLUMN min_success_rate REAL NOT NULL DEFAULT 99;",
            )
            .execute(&self.pool)
            .await?;
        }
        if !self.column_exists("benchmark_tasks", "engine_mode").await? {
            sqlx::query(
                "ALTER TABLE benchmark_tasks ADD COLUMN engine_mode TEXT NOT NULL DEFAULT 'mock';",
            )
            .execute(&self.pool)
            .await?;
        }
        self.ensure_task_evidence_columns().await?;
        Ok(())
    }

    async fn migrate_benchmark_evidence_schema(&self) -> anyhow::Result<()> {
        self.ensure_task_evidence_columns().await?;
        self.ensure_stage_evidence_columns().await?;
        self.ensure_tick_evidence_columns().await?;
        Ok(())
    }

    async fn migrate_release_prep_schema(&self) -> anyhow::Result<()> {
        self.ensure_provider_diagnostics_table().await?;
        self.ensure_task_release_prep_columns().await?;
        Ok(())
    }

    async fn migrate_request_log_schema(&self) -> anyhow::Result<()> {
        self.ensure_request_log_table().await
    }

    async fn migrate_endpoint_probe_schema(&self) -> anyhow::Result<()> {
        self.ensure_endpoint_probe_tables().await?;
        if !self.table_exists("site_probe_runs").await? {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT OR IGNORE INTO endpoint_probe_batches
             (id, name, status, streaming, max_output_tokens, timeout_seconds, save_body,
              concurrency, prompt_preview, created_at, finished_at)
             SELECT id, name, 'completed', 0, 512, 60,
                    CASE WHEN body_ref IS NULL THEN 0 ELSE 1 END,
                    1, prompt_preview, created_at, created_at
             FROM site_probe_runs;",
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "INSERT OR IGNORE INTO endpoint_probe_runs
             (id, batch_id, source_type, provider_id, name, base_url, interface_type, model,
              status, latency_ms, ttft_ms, input_tokens, output_tokens, total_tokens,
              error_kind, error_message, prompt_preview, response_preview, body_ref,
              created_at, finished_at)
             SELECT id, id, 'temporary', NULL, name, base_url, interface_type, model,
                    status, latency_ms, ttft_ms, input_tokens, output_tokens, total_tokens,
                    error_kind, error_message, prompt_preview, response_preview, body_ref,
                    created_at, created_at
             FROM site_probe_runs;",
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query("DROP INDEX IF EXISTS idx_site_probe_runs_created_at;")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DROP TABLE site_probe_runs;")
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn ensure_task_evidence_columns(&self) -> anyhow::Result<()> {
        for (column, definition) in [
            ("stage_sample_rounds", "INTEGER NOT NULL DEFAULT 0"),
            ("warmup_rounds", "INTEGER NOT NULL DEFAULT 0"),
            ("request_timeout_seconds", "INTEGER NOT NULL DEFAULT 120"),
            (
                "sla_stop_policy",
                "TEXT NOT NULL DEFAULT 'continue_full_staircase'",
            ),
            ("planned_stages", "TEXT NOT NULL DEFAULT '[]'"),
        ] {
            if !self.column_exists("benchmark_tasks", column).await? {
                sqlx::query(&format!(
                    "ALTER TABLE benchmark_tasks ADD COLUMN {column} {definition};"
                ))
                .execute(&self.pool)
                .await?;
            }
        }
        Ok(())
    }

    async fn ensure_task_release_prep_columns(&self) -> anyhow::Result<()> {
        for (column, definition) in [
            ("preflight_result", "TEXT"),
            ("diagnostics_snapshot", "TEXT"),
        ] {
            if !self.column_exists("benchmark_tasks", column).await? {
                sqlx::query(&format!(
                    "ALTER TABLE benchmark_tasks ADD COLUMN {column} {definition};"
                ))
                .execute(&self.pool)
                .await?;
            }
        }
        Ok(())
    }

    async fn ensure_provider_diagnostics_table(&self) -> anyhow::Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS provider_diagnostics (
                provider_id TEXT PRIMARY KEY,
                status TEXT NOT NULL,
                checked_at TEXT NOT NULL,
                engine_mode TEXT NOT NULL,
                result_json TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY(provider_id) REFERENCES providers(id) ON DELETE CASCADE
            );",
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn ensure_dataset_sample_tables(&self) -> anyhow::Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS dataset_samples (
                id TEXT PRIMARY KEY,
                dataset_id TEXT NOT NULL,
                sample_index INTEGER NOT NULL,
                prompt TEXT NOT NULL,
                estimated_tokens INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(dataset_id) REFERENCES datasets(id) ON DELETE CASCADE
            );",
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn ensure_request_log_table(&self) -> anyhow::Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS benchmark_request_logs (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                stage_index INTEGER NOT NULL,
                request_index INTEGER NOT NULL,
                sample_index INTEGER NOT NULL,
                status TEXT NOT NULL,
                latency_ms INTEGER NOT NULL,
                ttft_ms INTEGER NOT NULL,
                input_tokens INTEGER NOT NULL,
                output_tokens INTEGER NOT NULL,
                total_tokens INTEGER NOT NULL,
                error_kind TEXT,
                error_message TEXT,
                prompt_preview TEXT,
                response_preview TEXT,
                body_ref TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY(task_id) REFERENCES benchmark_tasks(id) ON DELETE CASCADE
            );",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_benchmark_request_logs_task
             ON benchmark_request_logs(task_id, stage_index, request_index);",
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn ensure_endpoint_probe_tables(&self) -> anyhow::Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS endpoint_probe_batches (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                status TEXT NOT NULL,
                streaming INTEGER NOT NULL DEFAULT 0,
                max_output_tokens INTEGER NOT NULL DEFAULT 512,
                timeout_seconds INTEGER NOT NULL DEFAULT 60,
                save_body INTEGER NOT NULL DEFAULT 0,
                concurrency INTEGER NOT NULL DEFAULT 1,
                prompt_preview TEXT,
                created_at TEXT NOT NULL,
                finished_at TEXT
            );",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS endpoint_probe_runs (
                id TEXT PRIMARY KEY,
                batch_id TEXT NOT NULL,
                source_type TEXT NOT NULL,
                provider_id TEXT,
                name TEXT NOT NULL,
                base_url TEXT NOT NULL,
                interface_type TEXT NOT NULL,
                model TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                latency_ms INTEGER NOT NULL DEFAULT 0,
                ttft_ms INTEGER NOT NULL DEFAULT 0,
                input_tokens INTEGER NOT NULL DEFAULT 0,
                output_tokens INTEGER NOT NULL DEFAULT 0,
                total_tokens INTEGER NOT NULL DEFAULT 0,
                error_kind TEXT,
                error_message TEXT,
                prompt_preview TEXT,
                response_preview TEXT,
                body_ref TEXT,
                created_at TEXT NOT NULL,
                finished_at TEXT,
                FOREIGN KEY(batch_id) REFERENCES endpoint_probe_batches(id) ON DELETE CASCADE,
                FOREIGN KEY(provider_id) REFERENCES providers(id) ON DELETE SET NULL
            );",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_endpoint_probe_batches_created_at
             ON endpoint_probe_batches(created_at DESC);",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_endpoint_probe_runs_batch
             ON endpoint_probe_runs(batch_id, created_at ASC);",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_endpoint_probe_runs_provider
             ON endpoint_probe_runs(provider_id, status);",
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn ensure_stage_columns(&self) -> anyhow::Result<()> {
        for (column, definition) in [
            ("ttft_ms", "INTEGER NOT NULL DEFAULT 0"),
            ("tps", "REAL NOT NULL DEFAULT 0"),
            ("input_tokens", "INTEGER NOT NULL DEFAULT 0"),
            ("output_tokens", "INTEGER NOT NULL DEFAULT 0"),
            ("total_tokens", "INTEGER NOT NULL DEFAULT 0"),
            ("batch_size", "INTEGER NOT NULL DEFAULT 0"),
            ("text_count", "INTEGER NOT NULL DEFAULT 0"),
            ("documents_per_query", "INTEGER NOT NULL DEFAULT 0"),
            ("pair_count", "INTEGER NOT NULL DEFAULT 0"),
            ("image_count", "INTEGER NOT NULL DEFAULT 0"),
        ] {
            if !self.column_exists("benchmark_stages", column).await? {
                sqlx::query(&format!(
                    "ALTER TABLE benchmark_stages ADD COLUMN {column} {definition};"
                ))
                .execute(&self.pool)
                .await?;
            }
        }
        self.ensure_stage_evidence_columns().await?;
        Ok(())
    }

    async fn ensure_stage_evidence_columns(&self) -> anyhow::Result<()> {
        for (column, definition) in [
            ("sample_rounds", "INTEGER NOT NULL DEFAULT 0"),
            ("warmup_rounds", "INTEGER NOT NULL DEFAULT 0"),
            ("request_count", "INTEGER NOT NULL DEFAULT 0"),
            ("success_count", "INTEGER NOT NULL DEFAULT 0"),
            ("failure_count", "INTEGER NOT NULL DEFAULT 0"),
            ("sla_passed", "INTEGER NOT NULL DEFAULT 1"),
            ("stop_reason", "TEXT"),
        ] {
            if !self.column_exists("benchmark_stages", column).await? {
                sqlx::query(&format!(
                    "ALTER TABLE benchmark_stages ADD COLUMN {column} {definition};"
                ))
                .execute(&self.pool)
                .await?;
            }
        }
        Ok(())
    }

    async fn ensure_tick_columns(&self) -> anyhow::Result<()> {
        for (column, definition) in [
            ("input_tokens", "INTEGER NOT NULL DEFAULT 0"),
            ("output_tokens", "INTEGER NOT NULL DEFAULT 0"),
            ("total_tokens", "INTEGER NOT NULL DEFAULT 0"),
            ("batch_size", "INTEGER NOT NULL DEFAULT 0"),
            ("text_count", "INTEGER NOT NULL DEFAULT 0"),
            ("documents_per_query", "INTEGER NOT NULL DEFAULT 0"),
            ("pair_count", "INTEGER NOT NULL DEFAULT 0"),
            ("image_count", "INTEGER NOT NULL DEFAULT 0"),
        ] {
            if !self.column_exists("benchmark_ticks", column).await? {
                sqlx::query(&format!(
                    "ALTER TABLE benchmark_ticks ADD COLUMN {column} {definition};"
                ))
                .execute(&self.pool)
                .await?;
            }
        }
        self.ensure_tick_evidence_columns().await?;
        Ok(())
    }

    async fn ensure_tick_evidence_columns(&self) -> anyhow::Result<()> {
        for (column, definition) in [
            ("request_count", "INTEGER NOT NULL DEFAULT 0"),
            ("success_count", "INTEGER NOT NULL DEFAULT 0"),
            ("failure_count", "INTEGER NOT NULL DEFAULT 0"),
        ] {
            if !self.column_exists("benchmark_ticks", column).await? {
                sqlx::query(&format!(
                    "ALTER TABLE benchmark_ticks ADD COLUMN {column} {definition};"
                ))
                .execute(&self.pool)
                .await?;
            }
        }
        Ok(())
    }

    async fn column_exists(&self, table: &str, column: &str) -> anyhow::Result<bool> {
        let rows = sqlx::query(&format!("PRAGMA table_info({table});"))
            .fetch_all(&self.pool)
            .await?;
        Ok(rows
            .iter()
            .any(|row| row.get::<String, _>("name") == column))
    }

    async fn table_exists(&self, table: &str) -> anyhow::Result<bool> {
        let found: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ? LIMIT 1;",
        )
        .bind(table)
        .fetch_optional(&self.pool)
        .await?;
        Ok(found.is_some())
    }
}
