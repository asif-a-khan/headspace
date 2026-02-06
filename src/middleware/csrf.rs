//! CSRF token generation and validation.
//!
//! Generates per-session CSRF tokens. Validates tokens on
//! POST/PUT/DELETE requests via the X-CSRF-Token header.

use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use rand::distr::{Alphanumeric, SampleString};
use rand::rng;
use tower_sessions::Session;

const CSRF_SESSION_KEY: &str = "csrf_token";
const CSRF_TOKEN_LENGTH: usize = 64;
const CSRF_HEADER: &str = "x-csrf-token";

/// Get or create a CSRF token for the current session.
pub async fn get_csrf_token(session: &Session) -> anyhow::Result<String> {
    if let Some(token) = session
        .get::<String>(CSRF_SESSION_KEY)
        .await
        .map_err(|e| anyhow::anyhow!("Session error: {e}"))?
    {
        return Ok(token);
    }

    let token = Alphanumeric.sample_string(&mut rng(), CSRF_TOKEN_LENGTH);
    session
        .insert(CSRF_SESSION_KEY, &token)
        .await
        .map_err(|e| anyhow::anyhow!("Session error: {e}"))?;
    Ok(token)
}

/// Middleware: validate CSRF token on mutating requests (POST/PUT/DELETE).
///
/// Reads the token from the `X-CSRF-Token` header and compares it to the
/// session token. GET/HEAD/OPTIONS requests pass through without validation.
pub async fn require_csrf(session: Session, req: Request, next: Next) -> Response {
    // Only validate on mutating methods
    if matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
        return next.run(req).await;
    }

    let header_token = req
        .headers()
        .get(CSRF_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let session_token: Option<String> = session
        .get(CSRF_SESSION_KEY)
        .await
        .unwrap_or(None);

    let valid = match session_token {
        Some(expected) if !expected.is_empty() => expected == header_token,
        _ => false,
    };

    if !valid {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Invalid CSRF token. Please refresh and try again." })),
        )
            .into_response();
    }

    next.run(req).await
}
