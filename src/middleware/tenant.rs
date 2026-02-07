//! Tenant resolution middleware.
//!
//! Extracts the tenant from the request subdomain
//! (e.g., "demo.headspace.local" -> tenant "demo")
//! and injects `Company` into request extensions.

use axum::{
    extract::{Extension, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use crate::config::Config;
use crate::db::Database;
use crate::models::company::Company;

/// Middleware: resolve tenant from subdomain and inject Company into extensions.
///
/// Extracts the Host header, strips the port, then strips the primary domain
/// suffix to get the subdomain. Queries `main.companies` by domain.
/// Returns 404 if not found, 403 if inactive.
pub async fn require_tenant(
    Extension(config): Extension<Config>,
    Extension(db): Extension<Database>,
    mut req: Request,
    next: Next,
) -> Response {
    let host = req
        .headers()
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Strip port (e.g., "demo.headspace.local:8000" → "demo.headspace.local")
    let hostname = host.split(':').next().unwrap_or(host);

    // Extract subdomain by stripping the primary domain suffix
    let primary = &config.primary_domain;
    let subdomain = if hostname.ends_with(primary) && hostname.len() > primary.len() {
        // "demo.headspace.local" → strip ".headspace.local" → "demo"
        let prefix = &hostname[..hostname.len() - primary.len()];
        prefix.strip_suffix('.').unwrap_or(prefix)
    } else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Tenant not found." })),
        )
            .into_response();
    };

    if subdomain.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Tenant not found." })),
        )
            .into_response();
    }

    // Look up company by domain (subdomain matches companies.domain)
    let company = sqlx::query_as::<_, Company>(
        "SELECT * FROM main.companies WHERE domain = $1",
    )
    .bind(subdomain)
    .fetch_optional(db.reader())
    .await;

    match company {
        Ok(Some(c)) if c.is_active => {
            req.extensions_mut().insert(c);
            next.run(req).await
        }
        Ok(Some(_)) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "This tenant account is inactive." })),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Tenant not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Tenant lookup failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "An internal error occurred." })),
            )
                .into_response()
        }
    }
}
