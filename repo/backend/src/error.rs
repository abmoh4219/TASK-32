//! Application-wide error type. Implements `IntoResponse` so handlers can simply
//! return `Result<Json<T>, AppError>` and get a properly-shaped JSON error envelope
//! plus the appropriate HTTP status code.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

/// Every error that can be produced by the ScholarVault backend.
/// Variants map to specific HTTP statuses in `IntoResponse`.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("authentication required")]
    Auth,

    #[error("CSRF token missing")]
    CsrfMissing,

    #[error("CSRF token invalid")]
    CsrfInvalid,

    #[error("rate limit exceeded")]
    RateLimit,

    #[error("validation error: {0}")]
    Validation(String),

    #[error("not found")]
    NotFound,

    #[error("forbidden")]
    Forbidden,

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("account locked: {message}")]
    AccountLocked { message: String },

    #[error("file too large: {size} bytes (max {max})")]
    FileTooLarge { size: usize, max: usize },

    #[error("invalid file type: {0}")]
    InvalidFileType(String),

    #[error("MIME mismatch: declared {declared}, detected {detected}")]
    MimeMismatch { declared: String, detected: String },

    #[error("unknown file type")]
    UnknownFileType,

    #[error("database error: {0}")]
    Database(String),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Internal(format!("json: {}", err))
    }
}

impl AppError {
    fn status_and_code(&self) -> (StatusCode, &'static str) {
        match self {
            AppError::Auth => (StatusCode::UNAUTHORIZED, "AUTH_REQUIRED"),
            AppError::CsrfMissing => (StatusCode::FORBIDDEN, "CSRF_MISSING"),
            AppError::CsrfInvalid => (StatusCode::FORBIDDEN, "CSRF_INVALID"),
            AppError::RateLimit => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMIT"),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            AppError::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL"),
            AppError::AccountLocked { .. } => (StatusCode::LOCKED, "ACCOUNT_LOCKED"),
            AppError::FileTooLarge { .. } => (StatusCode::PAYLOAD_TOO_LARGE, "FILE_TOO_LARGE"),
            AppError::InvalidFileType(_) => (StatusCode::BAD_REQUEST, "INVALID_FILE_TYPE"),
            AppError::MimeMismatch { .. } => (StatusCode::BAD_REQUEST, "MIME_MISMATCH"),
            AppError::UnknownFileType => (StatusCode::BAD_REQUEST, "UNKNOWN_FILE_TYPE"),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = self.status_and_code();
        // Never leak raw DB/IO/SQLx details to the caller. Log the detailed
        // message on the server (so ops can debug) but return a generic
        // user-safe string for Internal/Database failures. All other variants
        // carry deliberately shaped messages and are safe to echo.
        let message = match &self {
            AppError::Internal(_) | AppError::Database(_) => {
                tracing::error!(code = %code, detail = %self, "internal error");
                "Internal server error".to_string()
            }
            _ => self.to_string(),
        };
        let body = Json(json!({
            "code": code,
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }));
        (status, body).into_response()
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;
