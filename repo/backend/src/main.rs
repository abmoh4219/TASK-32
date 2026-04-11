//! ScholarVault backend entrypoint.
//!
//! Boots the SQLite pool (WAL mode), runs migrations, builds the Axum router and
//! listens on `HOST:PORT`. All real wiring lives in the `backend` library crate so
//! integration tests can construct the same router without spawning a process.

use std::net::SocketAddr;
use std::sync::Arc;

use backend::middleware::rate_limit::RateLimitState;
use backend::{db, derive_key, router::build_router, AppError, AppResult, AppState};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> AppResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_target(false)
        .compact()
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:///app/data/scholarvault.db".to_string());
    let encryption_key_material = std::env::var("ENCRYPTION_KEY")
        .unwrap_or_else(|_| "scholarvault-aes256-key-32bytes!!".to_string());
    let signing_key = std::env::var("SIGNING_KEY")
        .unwrap_or_else(|_| "scholarvault-jwt-signing-key-secret!".to_string());
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .map_err(|e| AppError::Internal(format!("invalid PORT: {e}")))?;
    let migrations_dir = std::env::var("MIGRATIONS_DIR")
        .unwrap_or_else(|_| "/app/migrations".to_string());

    ensure_data_dir(&database_url)?;

    tracing::info!(%database_url, "opening sqlite pool");
    let pool = db::init_pool(&database_url).await?;

    db::run_migrations(&pool, &migrations_dir).await?;

    let evidence_dir = std::path::PathBuf::from(
        std::env::var("EVIDENCE_DIR").unwrap_or_else(|_| "/app/evidence".to_string()),
    );
    let backup_dir = std::path::PathBuf::from(
        std::env::var("BACKUP_DIR").unwrap_or_else(|_| "/app/backups".to_string()),
    );
    let reports_dir = std::path::PathBuf::from(
        std::env::var("REPORTS_DIR").unwrap_or_else(|_| "/app/reports".to_string()),
    );
    let _ = std::fs::create_dir_all(&evidence_dir);
    let _ = std::fs::create_dir_all(&backup_dir);
    let _ = std::fs::create_dir_all(&reports_dir);

    let state = AppState {
        db: pool,
        encryption_key: Arc::new(derive_key(&encryption_key_material)),
        signing_key: Arc::new(signing_key),
        rate_limit: RateLimitState::new(60),
        evidence_dir: Arc::new(evidence_dir),
        backup_dir: Arc::new(backup_dir),
        reports_dir: Arc::new(reports_dir),
    };

    let app = build_router(state);

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|e| AppError::Internal(format!("bad bind address: {e}")))?;
    tracing::info!(%addr, "ScholarVault listening");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::Internal(format!("bind failed: {e}")))?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .map_err(|e| AppError::Internal(format!("serve failed: {e}")))?;

    Ok(())
}

/// Ensure the parent directory of a `sqlite://...` URL exists so SQLite can
/// create the database file at startup.
fn ensure_data_dir(database_url: &str) -> AppResult<()> {
    if let Some(rest) = database_url.strip_prefix("sqlite://") {
        let path = std::path::Path::new(rest);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| AppError::Internal(format!("create data dir: {e}")))?;
            }
        }
    }
    Ok(())
}
