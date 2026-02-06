use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use crate::db::Database;
use crate::models::super_admin::SuperAdmin;
use crate::models::super_role::SuperRole;

#[derive(Deserialize)]
pub struct RolePayload {
    pub name: String,
    pub description: Option<String>,
    pub permission_type: String,
    pub permissions: serde_json::Value,
}

pub async fn list(Extension(db): Extension<Database>) -> Response {
    let roles = sqlx::query_as::<_, SuperRole>(
        "SELECT * FROM main.super_roles ORDER BY id DESC",
    )
    .fetch_all(db.reader())
    .await
    .unwrap_or_default();

    Json(serde_json::json!({ "data": roles })).into_response()
}

pub async fn store(
    Extension(db): Extension<Database>,
    Json(payload): Json<RolePayload>,
) -> Response {
    let result = sqlx::query_as::<_, SuperRole>(
        "INSERT INTO main.super_roles (name, description, permission_type, permissions)
         VALUES ($1, $2, $3, $4)
         RETURNING *",
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.permission_type)
    .bind(&payload.permissions)
    .fetch_one(db.writer())
    .await;

    match result {
        Ok(role) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "data": role, "message": "Role created successfully." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to create role: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to create role." })),
            )
                .into_response()
        }
    }
}

pub async fn show(
    Extension(db): Extension<Database>,
    Path(id): Path<i64>,
) -> Response {
    let role = sqlx::query_as::<_, SuperRole>(
        "SELECT * FROM main.super_roles WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(db.reader())
    .await;

    match role {
        Ok(Some(r)) => Json(serde_json::json!({ "data": r })).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Role not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch role: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "An internal error occurred." })),
            )
                .into_response()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Path(id): Path<i64>,
    Json(payload): Json<RolePayload>,
) -> Response {
    let result = sqlx::query_as::<_, SuperRole>(
        "UPDATE main.super_roles
         SET name = $1, description = $2, permission_type = $3, permissions = $4, updated_at = NOW()
         WHERE id = $5
         RETURNING *",
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.permission_type)
    .bind(&payload.permissions)
    .bind(id)
    .fetch_optional(db.writer())
    .await;

    match result {
        Ok(Some(role)) => {
            Json(serde_json::json!({ "data": role, "message": "Role updated successfully." }))
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Role not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to update role: {e}");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to update role." })),
            )
                .into_response()
        }
    }
}

pub async fn destroy(
    Extension(db): Extension<Database>,
    Extension(admin): Extension<SuperAdmin>,
    Path(id): Path<i64>,
) -> Response {
    // Can't delete the role assigned to the current admin
    if admin.role_id == id {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "You cannot delete your own role." })),
        )
            .into_response();
    }

    // Can't delete a role that has agents assigned to it
    let agent_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM main.super_admins WHERE role_id = $1",
    )
    .bind(id)
    .fetch_one(db.reader())
    .await
    .unwrap_or((0,));

    if agent_count.0 > 0 {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "Cannot delete a role that has agents assigned to it." })),
        )
            .into_response();
    }

    let result = sqlx::query("DELETE FROM main.super_roles WHERE id = $1")
        .bind(id)
        .execute(db.writer())
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(serde_json::json!({ "message": "Role deleted successfully." })).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Role not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to delete role: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete role." })),
            )
                .into_response()
        }
    }
}
