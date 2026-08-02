use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

const DATABASE_FILE_NAME: &str = "data.db";
const LEGACY_DATABASE_FILE_NAME: &str = "llmbench.db";
const SQLITE_SIDECAR_SUFFIXES: [&str; 2] = ["-wal", "-shm"];
const DATABASE_RENAME_ATTEMPTS: usize = 10;
const DATABASE_RENAME_RETRY_DELAY: Duration = Duration::from_millis(50);

mod benchmarks;
mod dashboard;
mod datasets;
mod endpoint_probe;
mod migrations;
mod providers;
mod reports;
mod rows;
mod seed;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct Database {
    pub(in crate::db) pool: SqlitePool,
}

impl Database {
    pub async fn initialize(data_dir: &Path) -> anyhow::Result<Self> {
        std::fs::create_dir_all(data_dir)?;

        let db_path = resolve_database_path(data_dir)?;
        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        let db = Self { pool };
        db.configure().await?;
        db.migrate().await?;
        db.seed_defaults().await?;
        Ok(db)
    }
}

fn resolve_database_path(data_dir: &Path) -> anyhow::Result<PathBuf> {
    let db_path = data_dir.join(DATABASE_FILE_NAME);
    if db_path.exists() {
        return Ok(db_path);
    }

    let legacy_db_path = data_dir.join(LEGACY_DATABASE_FILE_NAME);
    if legacy_db_path.exists() {
        migrate_legacy_database_files(&legacy_db_path, &db_path)?;
    }

    Ok(db_path)
}

fn migrate_legacy_database_files(legacy_db_path: &Path, db_path: &Path) -> anyhow::Result<()> {
    rename_database_file(legacy_db_path, db_path)?;

    for suffix in SQLITE_SIDECAR_SUFFIXES {
        let legacy_sidecar = sqlite_sidecar_path(legacy_db_path, suffix);
        if !legacy_sidecar.exists() {
            continue;
        }

        let sidecar = sqlite_sidecar_path(db_path, suffix);
        rename_database_file(&legacy_sidecar, &sidecar)?;
    }

    Ok(())
}

fn rename_database_file(from: &Path, to: &Path) -> io::Result<()> {
    let mut last_error = None;

    for attempt in 0..DATABASE_RENAME_ATTEMPTS {
        match fs::rename(from, to) {
            Ok(()) => return Ok(()),
            Err(error) => {
                last_error = Some(error);
                if attempt + 1 < DATABASE_RENAME_ATTEMPTS {
                    thread::sleep(DATABASE_RENAME_RETRY_DELAY);
                }
            }
        }
    }

    Err(last_error.expect("rename attempt should store the last error"))
}

fn sqlite_sidecar_path(db_path: &Path, suffix: &str) -> PathBuf {
    let file_name = db_path
        .file_name()
        .map(|value| value.to_string_lossy())
        .unwrap_or_default();
    db_path.with_file_name(format!("{file_name}{suffix}"))
}

pub(in crate::db) fn now() -> String {
    Utc::now().to_rfc3339()
}
