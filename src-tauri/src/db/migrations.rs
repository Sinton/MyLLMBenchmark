use super::Database;
use sqlx::Row;

const INITIAL_SCHEMA_VERSION: i64 = 1;
const EVIDENCE_SCHEMA_VERSION: i64 = 2;
const RELEASE_PREP_SCHEMA_VERSION: i64 = 3;

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
}
