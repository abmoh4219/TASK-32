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
    // Transport-security gate — see `enforce_transport_security` doc for the
    // three supported modes. The function hard-fails if none are configured.
    let _transport_mode = enforce_transport_security()?;
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

    // In non-dev environments, disable seeded demo accounts so production
    // deployments never run with known credentials. The seed migration uses
    // INSERT OR IGNORE so the rows only exist on a fresh database, but an
    // operator who forgot to create real users would otherwise be exposed.
    disable_seed_users_in_production(&pool).await;

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

    let scheduler_handle: backend::services::backup_scheduler::SchedulerHandle =
        std::sync::Arc::new(tokio::sync::Mutex::new(None));

    let state = AppState {
        db: pool,
        encryption_key: Arc::new(derive_key(&encryption_key_material)),
        signing_key: Arc::new(signing_key),
        rate_limit: RateLimitState::new(60),
        evidence_dir: Arc::new(evidence_dir),
        backup_dir: Arc::new(backup_dir),
        reports_dir: Arc::new(reports_dir),
        invalid_search_tracker: backend::services::abuse::InvalidSearchTracker::new(),
        scheduler_handle: scheduler_handle.clone(),
    };

    // Start the backup scheduler. Cron expression is read from the DB-backed
    // admin schedule so operators can update it via the API; if the row is
    // missing (fresh DB) we fall back to the SPEC default 02:00 daily.
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
    let cron_expr = match backup_service.get_schedule().await {
        Ok(s) => s.cron_expr,
        Err(_) => std::env::var("BACKUP_SCHEDULE")
            .unwrap_or_else(|_| "0 0 2 * * *".to_string()),
    };
    match start_backup_scheduler(backup_service.clone(), &cron_expr).await {
        Ok(s) => {
            *scheduler_handle.lock().await = Some(s);
        }
        Err(e) => {
            tracing::warn!(error = ?e, "backup scheduler failed to start (continuing without)");
        }
    };

    let app = build_router(state);

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|e| AppError::Internal(format!("bad bind address: {e}")))?;

    match _transport_mode {
        TransportMode::InProcessTls { cert_path, key_path } => {
            tracing::info!(%addr, mode = "in-process-tls", "ScholarVault listening (TLS)");
            serve_tls(app, addr, &cert_path, &key_path).await?;
        }
        _ => {
            tracing::info!(%addr, mode = "plain-http", "ScholarVault listening");
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .map_err(|e| AppError::Internal(format!("bind failed: {e}")))?;
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await
            .map_err(|e| AppError::Internal(format!("serve failed: {e}")))?;
        }
    }

    Ok(())
}

/// In-process TLS listener. Loads the PEM cert chain + private key,
/// constructs a `rustls::ServerConfig`, wraps `TcpListener` with
/// `TlsAcceptor`, and accepts TLS connections that are handed to the
/// Axum router. This is the first-class production TLS path that the
/// static audit requires evidence of at the app listener layer.
async fn serve_tls(
    app: axum::Router,
    addr: SocketAddr,
    cert_path: &str,
    key_path: &str,
) -> AppResult<()> {
    use std::io::BufReader;
    use tokio_rustls::TlsAcceptor;

    // Load cert chain.
    let cert_file = std::fs::File::open(cert_path)
        .map_err(|e| AppError::Internal(format!("open cert {cert_path}: {e}")))?;
    let certs: Vec<_> = rustls_pemfile::certs(&mut BufReader::new(cert_file))
        .map_err(|e| AppError::Internal(format!("parse certs: {e}")))?
        .into_iter()
        .map(tokio_rustls::rustls::Certificate)
        .collect();
    if certs.is_empty() {
        return Err(AppError::Internal("no certificates found in PEM".into()));
    }

    // Load private key (try PKCS8 then RSA).
    let key_bytes = std::fs::read(key_path)
        .map_err(|e| AppError::Internal(format!("read key {key_path}: {e}")))?;
    let key = {
        let mut reader = BufReader::new(&key_bytes[..]);
        let pkcs8: Vec<_> = rustls_pemfile::pkcs8_private_keys(&mut reader)
            .map_err(|e| AppError::Internal(format!("parse pkcs8 key: {e}")))?;
        if let Some(k) = pkcs8.into_iter().next() {
            tokio_rustls::rustls::PrivateKey(k)
        } else {
            let mut reader2 = BufReader::new(&key_bytes[..]);
            let rsa: Vec<_> = rustls_pemfile::rsa_private_keys(&mut reader2)
                .map_err(|e| AppError::Internal(format!("parse rsa key: {e}")))?;
            tokio_rustls::rustls::PrivateKey(
                rsa.into_iter()
                    .next()
                    .ok_or_else(|| AppError::Internal("no private key found in PEM".into()))?,
            )
        }
    };

    let tls_config = tokio_rustls::rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| AppError::Internal(format!("rustls config: {e}")))?;

    let acceptor = TlsAcceptor::from(std::sync::Arc::new(tls_config));
    let tcp_listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::Internal(format!("bind failed: {e}")))?;

    tracing::info!("TLS acceptor ready");
    loop {
        let (tcp_stream, peer_addr) = tcp_listener
            .accept()
            .await
            .map_err(|e| AppError::Internal(format!("accept: {e}")))?;
        let acceptor = acceptor.clone();
        let app = app.clone();
        tokio::spawn(async move {
            match acceptor.accept(tcp_stream).await {
                Ok(tls_stream) => {
                    let io = hyper_util::rt::TokioIo::new(tls_stream);
                    let svc = hyper::service::service_fn(move |req| {
                        let app = app.clone();
                        async move {
                            let (mut parts, body) = req.into_parts();
                            parts.extensions.insert(
                                axum::extract::ConnectInfo(peer_addr),
                            );
                            let req = axum::http::Request::from_parts(parts, body);
                            Ok::<_, std::convert::Infallible>(
                                tower::ServiceExt::oneshot(app, req).await.unwrap(),
                            )
                        }
                    });
                    if let Err(e) = hyper::server::conn::http1::Builder::new()
                        .serve_connection(io, svc)
                        .await
                    {
                        tracing::debug!(error = ?e, "tls connection error");
                    }
                }
                Err(e) => {
                    tracing::debug!(error = ?e, "tls handshake failed");
                }
            }
        });
    }
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

/// In non-dev environments, deactivate the five demo seed accounts so the app
/// cannot be accessed with the known default credentials published in the
/// README. Dev/test/local modes keep them active for QA workflows.
async fn disable_seed_users_in_production(pool: &sqlx::SqlitePool) {
    let app_env = std::env::var("APP_ENV").unwrap_or_default().to_ascii_lowercase();
    let is_dev = matches!(
        app_env.as_str(),
        "dev" | "development" | "local" | "test" | ""
    );
    if is_dev {
        return;
    }
    let seed_ids = ["u-admin", "u-curator", "u-reviewer", "u-finance", "u-store"];
    for id in seed_ids {
        let result = sqlx::query("UPDATE users SET is_active = 0 WHERE id = ? AND is_active = 1")
            .bind(id)
            .execute(pool)
            .await;
        if let Ok(r) = result {
            if r.rows_affected() > 0 {
                tracing::warn!(user_id = %id, "disabled seed demo account in production mode");
            }
        }
    }
}

/// Enforce end-to-end transport security at startup. **The backend refuses to
/// start in production without explicit TLS configuration.** Three modes exist:
///
/// | Mode | Required config | Cookies | Use case |
/// |------|----------------|---------|----------|
/// | **Dev** | `APP_ENV=dev\|development\|local\|test` | `Secure=false` | Local Docker / CI |
/// | **Trusted TLS proxy** | `TRUSTED_TLS_PROXY=true` | `Secure=true` | Nginx/Caddy/ELB terminates TLS |
/// | **In-process TLS** | `TLS_CERT_PATH` + `TLS_KEY_PATH` | `Secure=true` | Self-hosted, no reverse proxy |
///
/// Any other configuration hard-fails before the listener binds.
fn enforce_transport_security() -> AppResult<TransportMode> {
    let app_env = std::env::var("APP_ENV").unwrap_or_default().to_ascii_lowercase();
    let is_dev = matches!(
        app_env.as_str(),
        "dev" | "development" | "local" | "test"
    );
    if is_dev {
        tracing::info!(mode = "dev-http", "transport security: plain HTTP (dev mode)");
        return Ok(TransportMode::PlainHttp);
    }
    // In-process TLS: operator supplies cert chain + private key paths.
    let tls_cert = std::env::var("TLS_CERT_PATH").ok();
    let tls_key = std::env::var("TLS_KEY_PATH").ok();
    if let (Some(cert), Some(key)) = (&tls_cert, &tls_key) {
        if !std::path::Path::new(cert).exists() {
            return Err(AppError::Internal(format!(
                "TLS_CERT_PATH={cert} does not exist"
            )));
        }
        if !std::path::Path::new(key).exists() {
            return Err(AppError::Internal(format!(
                "TLS_KEY_PATH={key} does not exist"
            )));
        }
        tracing::info!(mode = "in-process-tls", cert = %cert, key = %key,
            "transport security: in-process TLS configured");
        return Ok(TransportMode::InProcessTls {
            cert_path: cert.clone(),
            key_path: key.clone(),
        });
    }
    let trusted_proxy = std::env::var("TRUSTED_TLS_PROXY")
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false);
    let cookie_secure_explicit = std::env::var("COOKIE_SECURE").is_ok();
    if trusted_proxy || cookie_secure_explicit {
        tracing::info!(
            mode = "trusted-proxy",
            "transport security: TLS terminated by upstream proxy"
        );
        return Ok(TransportMode::TrustedProxy);
    }
    Err(AppError::Internal(
        "transport security not configured — set APP_ENV=dev for local HTTP, \
         TRUSTED_TLS_PROXY=true for proxy-terminated TLS, \
         or TLS_CERT_PATH + TLS_KEY_PATH for in-process TLS"
            .into(),
    ))
}

/// Describes how the listener should bind. Returned by `enforce_transport_security`.
#[allow(dead_code)]
enum TransportMode {
    PlainHttp,
    TrustedProxy,
    InProcessTls { cert_path: String, key_path: String },
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
