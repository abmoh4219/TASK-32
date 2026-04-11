//! Thin gloo-net wrapper used by every other API client module.
//!
//! - Reads the CSRF token from the `csrf_token` cookie and attaches it as the
//!   `X-CSRF-Token` header on every state-changing request.
//! - Decodes JSON error envelopes (`ErrorResponse`) and surfaces them as a
//!   typed `ApiError` with the message ready for UI display.

use gloo_net::http::{Method, Request};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use shared::ErrorResponse;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiError {
    pub status: u16,
    pub code: String,
    pub message: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.message, self.code)
    }
}

impl std::error::Error for ApiError {}

/// Read the CSRF token cookie if present (only available in a real browser).
pub fn read_csrf_cookie() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let document = web_sys::window()?.document()?;
        let html_doc = document.dyn_into::<web_sys::HtmlDocument>().ok()?;
        let cookie_str = html_doc.cookie().ok()?;
        for part in cookie_str.split(';') {
            let trimmed = part.trim();
            if let Some(rest) = trimmed.strip_prefix("csrf_token=") {
                return Some(rest.to_string());
            }
        }
        None
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

/// Issue a GET request and decode the JSON body. Used for read endpoints.
pub async fn get_json<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
    let res = Request::get(path)
        .send()
        .await
        .map_err(|e| ApiError {
            status: 0,
            code: "NETWORK".into(),
            message: e.to_string(),
        })?;
    handle_response(res).await
}

/// Issue a POST request with a JSON body and decode the JSON response.
/// Automatically attaches the `X-CSRF-Token` header from the cookie.
pub async fn post_json<B: Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<T, ApiError> {
    send_with_body(Method::POST, path, body).await
}

/// PUT helper.
pub async fn put_json<B: Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<T, ApiError> {
    send_with_body(Method::PUT, path, body).await
}

/// DELETE helper that returns a JSON response.
pub async fn delete_json<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
    let mut req = Request::delete(path);
    if let Some(token) = read_csrf_cookie() {
        req = req.header("X-CSRF-Token", &token);
    }
    let res = req.send().await.map_err(|e| ApiError {
        status: 0,
        code: "NETWORK".into(),
        message: e.to_string(),
    })?;
    handle_response(res).await
}

async fn send_with_body<B: Serialize, T: DeserializeOwned>(
    method: Method,
    path: &str,
    body: &B,
) -> Result<T, ApiError> {
    let mut req = match method {
        Method::POST => Request::post(path),
        Method::PUT => Request::put(path),
        Method::PATCH => Request::patch(path),
        _ => Request::post(path),
    };
    if let Some(token) = read_csrf_cookie() {
        req = req.header("X-CSRF-Token", &token);
    }
    let res = req
        .json(body)
        .map_err(|e| ApiError {
            status: 0,
            code: "JSON_ENCODE".into(),
            message: e.to_string(),
        })?
        .send()
        .await
        .map_err(|e| ApiError {
            status: 0,
            code: "NETWORK".into(),
            message: e.to_string(),
        })?;
    handle_response(res).await
}

async fn handle_response<T: DeserializeOwned>(res: gloo_net::http::Response) -> Result<T, ApiError> {
    let status = res.status();
    if (200..300).contains(&status) {
        res.json::<T>().await.map_err(|e| ApiError {
            status,
            code: "JSON_DECODE".into(),
            message: e.to_string(),
        })
    } else {
        match res.json::<ErrorResponse>().await {
            Ok(err) => Err(ApiError {
                status,
                code: err.code,
                message: err.message,
            }),
            Err(e) => Err(ApiError {
                status,
                code: "HTTP_ERROR".into(),
                message: format!("HTTP {} ({})", status, e),
            }),
        }
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
