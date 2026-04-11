//! Authentication service: login, logout, lockout, attempt tracking.
//!
//! Owns every read and write against the `users`, `login_attempts`, and
//! `sessions` tables. The lockout policy from CLAUDE.md (5 failed attempts in a
//! 15-minute sliding window — checked against both username AND client IP) lives
//! in `check_lockout` below.

use chrono::{DateTime, Duration, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::user::User;
use crate::security::{csrf, password};

/// How long an issued session cookie remains valid before requiring a fresh login.
pub const SESSION_TTL_HOURS: i64 = 8;

/// Authentication service. Constructed once and shared via the application state.
#[derive(Clone)]
pub struct AuthService {
    pub db: SqlitePool,
}

/// Result of a successful `login()` call: the user record, the new session id
/// (set as the `sv_session` cookie), and the matching CSRF token.
pub struct LoginOutcome {
    pub user: User,
    pub session_id: String,
    pub csrf_token: String,
}

impl AuthService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Check and enforce account lockout: 5 failed attempts within a 15-minute
    /// sliding window. The lockout expires automatically when the window passes.
    /// Checks both username-based AND IP-based attempt counts.
    pub async fn check_lockout(&self, username: &str, ip: &str) -> AppResult<()> {
        let window_start = (Utc::now() - Duration::minutes(15)).to_rfc3339();
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM login_attempts
             WHERE (username = ? OR ip_address = ?)
             AND attempted_at > ?
             AND success = 0",
        )
        .bind(username)
        .bind(ip)
        .bind(&window_start)
        .fetch_one(&self.db)
        .await?;

        if count >= 5 {
            return Err(AppError::AccountLocked {
                message: "Too many failed login attempts. Try again in 15 minutes.".to_string(),
            });
        }
        Ok(())
    }

    /// Persist a single login attempt for the lockout sliding window.
    pub async fn record_attempt(&self, username: &str, ip: &str, success: bool) -> AppResult<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO login_attempts (id, username, ip_address, attempted_at, success)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(username)
        .bind(ip)
        .bind(&now)
        .bind(if success { 1 } else { 0 })
        .execute(&self.db)
        .await?;
        Ok(())
    }

    /// Authenticate a username/password pair, enforcing the lockout policy
    /// before doing any password work, and create a session+CSRF token on success.
    pub async fn login(
        &self,
        username: &str,
        password_plain: &str,
        ip: &str,
    ) -> AppResult<LoginOutcome> {
        // 1. Lockout gate — runs before we touch the password hasher.
        self.check_lockout(username, ip).await?;

        // 2. Look up the user.
        let user: Option<User> =
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ? AND is_active = 1")
                .bind(username)
                .fetch_optional(&self.db)
                .await?;

        let user = match user {
            Some(u) => u,
            None => {
                self.record_attempt(username, ip, false).await?;
                return Err(AppError::Auth);
            }
        };

        // 3. Verify the password.
        let valid = password::verify_password(password_plain, &user.password_hash)?;
        if !valid {
            self.record_attempt(username, ip, false).await?;
            return Err(AppError::Auth);
        }

        // 4. Record success and mint a session.
        self.record_attempt(username, ip, true).await?;

        let session_id = Uuid::new_v4().to_string();
        let csrf_token = csrf::generate_token();
        let expires_at = (Utc::now() + Duration::hours(SESSION_TTL_HOURS)).to_rfc3339();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO sessions (id, user_id, csrf_token, ip_address, expires_at, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&session_id)
        .bind(&user.id)
        .bind(&csrf_token)
        .bind(ip)
        .bind(&expires_at)
        .bind(&now)
        .execute(&self.db)
        .await?;

        Ok(LoginOutcome {
            user,
            session_id,
            csrf_token,
        })
    }

    /// Drop a session row by id. Idempotent — succeeds even if the session was
    /// already gone.
    pub async fn logout(&self, session_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(session_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Returns the current user paired with their CSRF token if the session id
    /// is valid and unexpired.
    pub async fn get_session_user(&self, session_id: &str) -> AppResult<Option<(User, String)>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            user_id: String,
            csrf_token: String,
            expires_at: String,
        }
        let row: Option<Row> = sqlx::query_as::<_, Row>(
            "SELECT user_id, csrf_token, expires_at FROM sessions WHERE id = ?",
        )
        .bind(session_id)
        .fetch_optional(&self.db)
        .await?;

        let Some(row) = row else { return Ok(None) };
        if let Ok(exp) = DateTime::parse_from_rfc3339(&row.expires_at) {
            if exp.with_timezone(&Utc) < Utc::now() {
                self.logout(session_id).await?;
                return Ok(None);
            }
        }

        let user: Option<User> =
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ? AND is_active = 1")
                .bind(&row.user_id)
                .fetch_optional(&self.db)
                .await?;
        Ok(user.map(|u| (u, row.csrf_token)))
    }
}
