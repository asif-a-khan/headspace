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
use crate::models::group::Group;
use crate::models::tenant_admin::TenantUser;

#[derive(Deserialize, Validate)]
pub struct GroupPayload {
    #[validate(length(min = 1, message = "Name is required."))]
    pub name: String,
    pub description: Option<String>,
}

pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.groups") { return resp; }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let groups = guard
        .fetch_all(sqlx::query_as::<_, Group>(
            "SELECT * FROM groups ORDER BY id DESC",
        ))
        .await;

    let _ = guard.release().await;

    match groups {
        Ok(groups) => Json(serde_json::json!({ "data": groups })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list groups: {e}");
            internal_error()
        }
    }
}

pub async fn store(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<GroupPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.groups.create") { return resp; }
    if let Err(resp) = validate_payload(&payload) { return resp; }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .fetch_one(sqlx::query_as::<_, Group>(
            "INSERT INTO groups (name, description)
             VALUES ($1, $2)
             RETURNING *",
        )
        .bind(&payload.name)
        .bind(&payload.description))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(group) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "data": group, "message": "Group created successfully." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to create group: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to create group." })),
            )
                .into_response()
        }
    }
}

pub async fn show(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.groups.edit") { return resp; }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let group = guard
        .fetch_optional(sqlx::query_as::<_, Group>(
            "SELECT * FROM groups WHERE id = $1",
        )
        .bind(id))
        .await;

    let _ = guard.release().await;

    match group {
        Ok(Some(g)) => Json(serde_json::json!({ "data": g })).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Group not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch group: {e}");
            internal_error()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<GroupPayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.groups.edit") { return resp; }
    if let Err(resp) = validate_payload(&payload) { return resp; }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .fetch_optional(sqlx::query_as::<_, Group>(
            "UPDATE groups
             SET name = $1, description = $2, updated_at = NOW()
             WHERE id = $3
             RETURNING *",
        )
        .bind(&payload.name)
        .bind(&payload.description)
        .bind(id))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(Some(group)) => {
            Json(serde_json::json!({ "data": group, "message": "Group updated successfully." }))
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Group not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to update group: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to update group." })),
            )
                .into_response()
        }
    }
}

pub async fn destroy(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.groups.delete") { return resp; }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .execute(sqlx::query("DELETE FROM groups WHERE id = $1").bind(id))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(serde_json::json!({ "message": "Group deleted successfully." })).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Group not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to delete group: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete group." })),
            )
                .into_response()
        }
    }
}

fn internal_error() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "An internal error occurred." })),
    )
        .into_response()
}
