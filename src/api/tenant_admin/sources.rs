use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::models::company::Company;
use crate::models::pipeline::LeadSource;

#[derive(Deserialize)]
pub struct SourcePayload {
    pub name: String,
}

pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let sources = guard
        .fetch_all(sqlx::query_as::<_, LeadSource>(
            "SELECT * FROM lead_sources ORDER BY id",
        ))
        .await;

    let _ = guard.release().await;

    match sources {
        Ok(s) => Json(serde_json::json!({ "data": s })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list sources: {e}");
            internal_error()
        }
    }
}

pub async fn store(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Json(payload): Json<SourcePayload>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .fetch_one(sqlx::query_as::<_, LeadSource>(
            "INSERT INTO lead_sources (name) VALUES ($1) RETURNING *",
        ).bind(&payload.name))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(s) => (StatusCode::CREATED, Json(serde_json::json!({ "data": s, "message": "Source created successfully." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to create source: {e}");
            (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Failed to create source." }))).into_response()
        }
    }
}

pub async fn show(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Path(id): Path<i64>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let source = guard
        .fetch_optional(sqlx::query_as::<_, LeadSource>("SELECT * FROM lead_sources WHERE id = $1").bind(id))
        .await;

    let _ = guard.release().await;

    match source {
        Ok(Some(s)) => Json(serde_json::json!({ "data": s })).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Source not found." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch source: {e}");
            internal_error()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Path(id): Path<i64>,
    Json(payload): Json<SourcePayload>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .fetch_optional(sqlx::query_as::<_, LeadSource>(
            "UPDATE lead_sources SET name = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
        ).bind(&payload.name).bind(id))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(Some(s)) => Json(serde_json::json!({ "data": s, "message": "Source updated successfully." })).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Source not found." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to update source: {e}");
            (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Failed to update source." }))).into_response()
        }
    }
}

pub async fn destroy(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Path(id): Path<i64>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard.execute(sqlx::query("DELETE FROM lead_sources WHERE id = $1").bind(id)).await;
    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => Json(serde_json::json!({ "message": "Source deleted successfully." })).into_response(),
        Ok(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Source not found." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to delete source: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Failed to delete source." }))).into_response()
        }
    }
}

fn internal_error() -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "An internal error occurred." }))).into_response()
}
