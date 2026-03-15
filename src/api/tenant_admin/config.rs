use axum::Json;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use std::collections::HashMap;

use crate::auth::bouncer::bouncer;
use crate::db::Database;
use crate::db::guard::TenantGuard;
use crate::models::company::Company;
use crate::models::tenant_admin::TenantUser;

#[derive(sqlx::FromRow, serde::Serialize)]
struct ConfigRow {
    code: String,
    value: String,
}

/// GET /admin/api/settings/config — list all config key-value pairs.
pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let rows = guard
        .fetch_all(sqlx::query_as::<_, ConfigRow>(
            "SELECT code, value FROM tenant_config ORDER BY code",
        ))
        .await;
    let _ = guard.release().await;

    match rows {
        Ok(rows) => {
            let config: HashMap<String, String> =
                rows.into_iter().map(|r| (r.code, r.value)).collect();
            Json(serde_json::json!({ "data": config })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to list config: {e}");
            internal_error()
        }
    }
}

#[derive(Deserialize)]
pub struct UpdatePayload {
    pub config: HashMap<String, String>,
}

/// PUT /admin/api/settings/config — update config values (upsert).
pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<UpdatePayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    for (code, value) in &payload.config {
        let result = guard
            .execute(
                sqlx::query(
                    "INSERT INTO tenant_config (code, value)
                     VALUES ($1, $2)
                     ON CONFLICT (code)
                     DO UPDATE SET value = $2, updated_at = NOW()",
                )
                .bind(code)
                .bind(value),
            )
            .await;

        if let Err(e) = result {
            tracing::error!("Failed to update config {code}: {e}");
            let _ = guard.release().await;
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(
                    serde_json::json!({ "error": format!("Failed to save config key '{code}'.") }),
                ),
            )
                .into_response();
        }
    }

    let _ = guard.release().await;

    Json(serde_json::json!({ "message": "Configuration saved successfully." })).into_response()
}

/// Helper: Load tenant config as a HashMap (for use in handlers).
pub async fn load_tenant_config(guard: &mut TenantGuard) -> HashMap<String, String> {
    guard
        .fetch_all(sqlx::query_as::<_, ConfigRow>(
            "SELECT code, value FROM tenant_config",
        ))
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| (r.code, r.value))
        .collect()
}

fn internal_error() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "An internal error occurred." })),
    )
        .into_response()
}
