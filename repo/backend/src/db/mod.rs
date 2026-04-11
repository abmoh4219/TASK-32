//! Database setup. Initializes a SQLite pool with WAL journal mode and a busy
//! timeout, and exposes a `run_migrations` helper that loads SQL files from the
//! configured migrations directory at runtime.

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use crate::error::{AppError, AppResult};

/// Open a SQLite connection pool against `database_url`. Enables WAL mode for
/// better concurrent read/write throughput and sets a 5-second busy timeout so
/// transactions retry instead of failing immediately under contention.
pub async fn init_pool(database_url: &str) -> AppResult<SqlitePool> {
    let options = SqliteConnectOptions::from_str(database_url)
        .map_err(|e| AppError::Internal(format!("invalid DATABASE_URL: {e}")))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(options)
        .await?;

    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;

    Ok(pool)
}

/// Run all SQL migration files found under `migrations_dir`. Uses the runtime
/// `Migrator::new` so the directory is read at startup rather than embedded at
/// compile time — this lets the same binary serve any environment.
pub async fn run_migrations(pool: &SqlitePool, migrations_dir: &str) -> AppResult<()> {
    let path = Path::new(migrations_dir);
    if !path.exists() {
        tracing::warn!(dir = %migrations_dir, "migrations directory does not exist; skipping");
        return Ok(());
    }
    let migrator = sqlx::migrate::Migrator::new(path)
        .await
        .map_err(|e| AppError::Internal(format!("migrator load failed: {e}")))?;
    migrator
        .run(pool)
        .await
        .map_err(|e| AppError::Internal(format!("migration run failed: {e}")))?;
    tracing::info!(dir = %migrations_dir, "migrations applied");
    Ok(())
}
