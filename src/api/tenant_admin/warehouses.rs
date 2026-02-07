use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use validator::Validate;

use crate::auth::bouncer::{bouncer, validate_payload};
use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::models::company::Company;
use crate::models::warehouse::Warehouse;
use crate::models::tenant_admin::TenantUser;

#[derive(Deserialize, Validate)]
pub struct WarehousePayload {
    #[validate(length(min = 1, message = "Name is required."))]
    pub name: String,
    pub description: Option<String>,
    pub contact_name: Option<String>,
    pub contact_emails: Option<serde_json::Value>,
    pub contact_numbers: Option<serde_json::Value>,
    pub contact_address: Option<serde_json::Value>,
}

pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.warehouses") { return resp; }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let warehouses = guard
        .fetch_all(sqlx::query_as::<_, Warehouse>(
            "SELECT * FROM warehouses ORDER BY id",
        ))
        .await;

    let _ = guard.release().await;

    match warehouses {
        Ok(w) => Json(serde_json::json!({ "data": w })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list warehouses: {e}");
            internal_error()
        }
    }
}

pub async fn store(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<WarehousePayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.warehouses.create") { return resp; }
    if let Err(resp) = validate_payload(&payload) { return resp; }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .fetch_one(sqlx::query_as::<_, Warehouse>(
            "INSERT INTO warehouses (name, description, contact_name, contact_emails, contact_numbers, contact_address)
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
        ).bind(&payload.name).bind(&payload.description).bind(&payload.contact_name)
         .bind(&payload.contact_emails).bind(&payload.contact_numbers).bind(&payload.contact_address))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(w) => (StatusCode::CREATED, Json(serde_json::json!({ "data": w, "message": "Warehouse created successfully." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to create warehouse: {e}");
            (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Failed to create warehouse." }))).into_response()
        }
    }
}

pub async fn show(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.warehouses.edit") { return resp; }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let warehouse = guard
        .fetch_optional(sqlx::query_as::<_, Warehouse>("SELECT * FROM warehouses WHERE id = $1").bind(id))
        .await;

    let _ = guard.release().await;

    match warehouse {
        Ok(Some(w)) => Json(serde_json::json!({ "data": w })).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Warehouse not found." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch warehouse: {e}");
            internal_error()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<WarehousePayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.warehouses.edit") { return resp; }
    if let Err(resp) = validate_payload(&payload) { return resp; }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .fetch_optional(sqlx::query_as::<_, Warehouse>(
            "UPDATE warehouses SET name = $1, description = $2, contact_name = $3,
                    contact_emails = $4, contact_numbers = $5, contact_address = $6, updated_at = NOW()
             WHERE id = $7 RETURNING *",
        ).bind(&payload.name).bind(&payload.description).bind(&payload.contact_name)
         .bind(&payload.contact_emails).bind(&payload.contact_numbers).bind(&payload.contact_address).bind(id))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(Some(w)) => Json(serde_json::json!({ "data": w, "message": "Warehouse updated successfully." })).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Warehouse not found." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to update warehouse: {e}");
            (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Failed to update warehouse." }))).into_response()
        }
    }
}

pub async fn destroy(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.warehouses.delete") { return resp; }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard.execute(sqlx::query("DELETE FROM warehouses WHERE id = $1").bind(id)).await;
    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => Json(serde_json::json!({ "message": "Warehouse deleted successfully." })).into_response(),
        Ok(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Warehouse not found." }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to delete warehouse: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Failed to delete warehouse." }))).into_response()
        }
    }
}

fn internal_error() -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "An internal error occurred." }))).into_response()
}
