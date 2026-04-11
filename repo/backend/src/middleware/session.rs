//! Session-loader middleware.
//!
//! Reads the `sv_session` cookie from the request, looks the session row up in
//! SQLite, and — if it is still alive — attaches a `CurrentUser` extension to
//! the request so handlers and the `require_role` extractor can read it.
//!
//! Expired sessions are deleted from the table on first sight; missing or
//! malformed cookies are silently ignored (the request continues unauthenticated).

use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;
use chrono::{DateTime, Utc};

use crate::models::user::User;
use crate::AppState;

/// Wrapper newtype attached to request extensions when a session is active.
/// Handlers extract it via the `FromRequestParts` impl in `require_role.rs`.
#[derive(Clone, Debug)]
pub struct CurrentUser(pub User);

#[derive(Clone, Debug)]
pub struct CurrentSession {
    pub session_id: String,
    pub csrf_token: String,
}

/// Axum middleware: pulls the session cookie, loads the user, and attaches
/// `CurrentUser` + `CurrentSession` extensions if everything checks out.
pub async fn session_middleware(
    State(state): State<AppState>,
    cookies: CookieJar,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    if let Some(session_cookie) = cookies.get("sv_session") {
        let session_id = session_cookie.value().to_string();
        if let Some((user, csrf_token)) = load_session_user(&state, &session_id).await {
            req.extensions_mut().insert(CurrentUser(user));
            req.extensions_mut().insert(CurrentSession {
                session_id,
                csrf_token,
            });
        }
    }
    next.run(req).await
}

async fn load_session_user(state: &AppState, session_id: &str) -> Option<(User, String)> {
    #[derive(sqlx::FromRow)]
    struct SessionRow {
        user_id: String,
        csrf_token: String,
        expires_at: String,
    }

    let row: Option<SessionRow> = sqlx::query_as::<_, SessionRow>(
        "SELECT user_id, csrf_token, expires_at FROM sessions WHERE id = ?",
    )
    .bind(session_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let row = row?;

    // Reject expired sessions and clean them up.
    if let Ok(exp) = DateTime::parse_from_rfc3339(&row.expires_at) {
        if exp.with_timezone(&Utc) < Utc::now() {
            let _ = sqlx::query("DELETE FROM sessions WHERE id = ?")
                .bind(session_id)
                .execute(&state.db)
                .await;
            return None;
        }
    }

    let user: Option<User> =
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ? AND is_active = 1")
            .bind(&row.user_id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();

    user.map(|u| (u, row.csrf_token))
}
