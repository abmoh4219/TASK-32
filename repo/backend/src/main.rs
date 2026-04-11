//! ScholarVault backend entrypoint.
//!
//! Boots the SQLite pool (WAL mode), runs migrations, builds the Axum router and
//! listens on `HOST:PORT`. All real wiring lives in the `backend` library crate so
//! integration tests can construct the same router without spawning a process.

use std::net::SocketAddr;
use std::sync::Arc;

use backend::middleware::rate_limit::RateLimitState;
use backend::services::backup_scheduler::start_backup_scheduler;
use backend::services::backup_service::BackupService;
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
    // Fail fast on missing or known-insecure cryptographic secrets. We refuse to
    // boot with the legacy shipped defaults so a misconfigured deployment cannot
    // silently run with predictable key material. See audit issue #4.
    let encryption_key_material = require_secret("ENCRYPTION_KEY")?;
    let signing_key = require_secret("SIGNING_KEY")?;
    // Transport-security gate. In production-like deployments we refuse to boot
    // unless TLS is either terminated upstream by a trusted proxy
    // (`TRUSTED_TLS_PROXY=true`) or the operator has explicitly opted in to a
    // plain-HTTP path with `COOKIE_SECURE=false`. Dev/local keeps the loose
    // default so `docker compose up` still works out of the box.
    enforce_transport_security()?;
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
        invalid_search_tracker: backend::services::abuse::InvalidSearchTracker::new(),
    };

    // Start the backup scheduler so the SPEC default 02:00 daily job is wired.
    let backup_service = Arc::new(BackupService::new(
        state.db.clone(),
        match database_url.strip_prefix("sqlite://") {
            Some(rest) => std::path::PathBuf::from(rest),
            None => std::path::PathBuf::from(&database_url),
        },
        (*state.evidence_dir).clone(),
        (*state.backup_dir).clone(),
        *state.encryption_key,
    ));
    let cron_expr = std::env::var("BACKUP_SCHEDULE").unwrap_or_else(|_| "0 0 2 * * *".to_string());
    let _scheduler_handle = match start_backup_scheduler(backup_service, &cron_expr).await {
        Ok(s) => Some(s),
        Err(e) => {
            tracing::warn!(error = ?e, "backup scheduler failed to start (continuing without)");
            None
        }
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

/// Require a cryptographic secret from the environment. Returns an error if the
/// variable is missing, shorter than 32 bytes, or matches the legacy shipped
/// defaults — any of which indicate a dangerously weak configuration.
///
/// This is the explicit enforcement point referenced by the static audit: the
/// backend refuses to start with insecure default key material, eliminating the
/// "forgot to change the key" failure mode.
fn require_secret(var_name: &str) -> AppResult<String> {
    const INSECURE_DEFAULTS: &[&str] = &[
        "scholarvault-aes256-key-32bytes!!",
        "scholarvault-jwt-signing-key-secret!",
        "changeme",
        "secret",
    ];
    let value = std::env::var(var_name).map_err(|_| {
        AppError::Internal(format!(
            "{} is required — set a random value of at least 32 bytes",
            var_name
        ))
    })?;
    if value.len() < 32 {
        return Err(AppError::Internal(format!(
            "{} must be at least 32 bytes — got {} bytes",
            var_name,
            value.len()
        )));
    }
    if INSECURE_DEFAULTS.iter().any(|d| *d == value) {
        return Err(AppError::Internal(format!(
            "{} matches a known insecure default — rotate it before starting",
            var_name
        )));
    }
    Ok(value)
}

/// Refuse to boot in production-like deployments unless transport security is
/// set up correctly. The rules are:
///   • APP_ENV=dev|development|local|test   → always allowed (HTTP dev flow).
///   • TRUSTED_TLS_PROXY=true                → allowed (TLS terminated upstream).
///   • COOKIE_SECURE explicitly set          → allowed (operator made a choice).
///   • otherwise                             → hard error.
fn enforce_transport_security() -> AppResult<()> {
    let app_env = std::env::var("APP_ENV").unwrap_or_default().to_ascii_lowercase();
    let is_dev = matches!(
        app_env.as_str(),
        "dev" | "development" | "local" | "test"
    );
    if is_dev {
        return Ok(());
    }
    let trusted_proxy = std::env::var("TRUSTED_TLS_PROXY")
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false);
    let cookie_secure_explicit = std::env::var("COOKIE_SECURE").is_ok();
    if trusted_proxy || cookie_secure_explicit {
        return Ok(());
    }
    Err(AppError::Internal(
        "transport security not configured — set APP_ENV=dev for local HTTP, \
         TRUSTED_TLS_PROXY=true when a reverse proxy terminates TLS, \
         or COOKIE_SECURE=true/false to make an explicit choice"
            .into(),
    ))
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
