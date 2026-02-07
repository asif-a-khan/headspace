use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::models::company::Company;
use crate::models::pipeline::LeadType;

#[derive(Deserialize)]
pub struct TypePayload {
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

    let types = guard
        .fetch_all(sqlx::query_as::<_, LeadType>("SELECT * FROM lead_types ORDER BY id"))
        .await;

    let _ = guard.release().await;

    match types {
        Ok(t) => Json(serde_json::json!({ "data": t })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list types: {e}");
            internal_error()
        }
    }
}

pub async fn store(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Json(payload): Json<TypePayload>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .fetch_one(sqlx::query_as::<_, LeadType>(
            "INSERT INTO lead_types (name) VALUES ($1) RETURNING *",
        ).bind(&payload.name))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(t) => (StatusCode::CREATED, Json(serde_json::json!({ "data": t, "message": "Type created successfully." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to create type: {e}");
            (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Failed to create type." }))).into_response()
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

    let lead_type = guard
        .fetch_optional(sqlx::query_as::<_, LeadType>("SELECT * FROM lead_types WHERE id = $1").bind(id))
        .await;

    let _ = guard.release().await;

    match lead_type {
        Ok(Some(t)) => Json(serde_json::json!({ "data": t })).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Type not found." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch type: {e}");
            internal_error()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Path(id): Path<i64>,
    Json(payload): Json<TypePayload>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .fetch_optional(sqlx::query_as::<_, LeadType>(
            "UPDATE lead_types SET name = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
        ).bind(&payload.name).bind(id))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(Some(t)) => Json(serde_json::json!({ "data": t, "message": "Type updated successfully." })).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Type not found." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to update type: {e}");
            (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Failed to update type." }))).into_response()
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

    let result = guard.execute(sqlx::query("DELETE FROM lead_types WHERE id = $1").bind(id)).await;
    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => Json(serde_json::json!({ "message": "Type deleted successfully." })).into_response(),
        Ok(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Type not found." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to delete type: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Failed to delete type." }))).into_response()
        }
    }
}

fn internal_error() -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "An internal error occurred." }))).into_response()
}
