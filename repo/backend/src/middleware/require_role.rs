//! Role-gating extractors.
//!
//! Each handler that mutates state declares which roles are permitted by adding
//! one of these extractors to its signature. The extractor pulls `CurrentUser`
//! out of the request extensions (set by `session_middleware`) and returns
//! `AppError::Auth` when no session is present or `AppError::Forbidden` when the
//! session belongs to a user whose role is not in the allow-list.

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use shared::UserRole;

use crate::error::AppError;
use crate::middleware::session::CurrentUser;
use crate::models::user::User;

/// Helper used by handlers that already extracted `CurrentUser`. Returns Ok if
/// the user's role is one of `allowed`, else `AppError::Forbidden`.
pub fn require_any_role(user: &User, allowed: &[UserRole]) -> Result<(), AppError> {
    let role = UserRole::from_str(&user.role).ok_or(AppError::Forbidden)?;
    if allowed.contains(&role) {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

/// Generic role-set extractor — instantiate with the allowed roles per route.
/// Handlers use the type-aliased newtypes below for readability.
pub struct AuthenticatedUser(pub User);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for AuthenticatedUser {
    type Rejection = AppError;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let CurrentUser(user) = parts
            .extensions
            .get::<CurrentUser>()
            .cloned()
            .ok_or(AppError::Auth)?;
        Ok(AuthenticatedUser(user))
    }
}

macro_rules! role_extractor {
    ($name:ident, $($variant:ident),+ $(,)?) => {
        #[doc = concat!("Role-gated extractor allowing: ", stringify!($($variant),+))]
        pub struct $name(pub User);

        #[async_trait]
        impl<S: Send + Sync> FromRequestParts<S> for $name {
            type Rejection = AppError;
            async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
                let CurrentUser(user) = parts
                    .extensions
                    .get::<CurrentUser>()
                    .cloned()
                    .ok_or(AppError::Auth)?;
                let role = UserRole::from_str(&user.role).ok_or(AppError::Forbidden)?;
                let allowed = &[$(UserRole::$variant),+];
                if allowed.contains(&role) {
                    Ok($name(user))
                } else {
                    Err(AppError::Forbidden)
                }
            }
        }
    };
}

role_extractor!(RequireAdmin, Administrator);
role_extractor!(RequireCurator, Administrator, ContentCurator);
role_extractor!(RequireReviewer, Administrator, Reviewer);
role_extractor!(RequireFinance, Administrator, FinanceManager);
role_extractor!(RequireStore, Administrator, StoreManager);
