use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use crate::auth::password::hash_password;
use crate::db::Database;
use crate::models::super_admin::SuperAdmin;

#[derive(Deserialize)]
pub struct AgentCreatePayload {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub role_id: i64,
    pub status: Option<bool>,
}

#[derive(Deserialize)]
pub struct AgentUpdatePayload {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: Option<String>,
    pub role_id: i64,
    pub status: Option<bool>,
}

pub async fn list(Extension(db): Extension<Database>) -> Response {
    let agents = sqlx::query_as::<_, SuperAdmin>(
        "SELECT sa.*, sr.permission_type, sr.permissions AS role_permissions
         FROM main.super_admins sa
         JOIN main.super_roles sr ON sr.id = sa.role_id
         ORDER BY sa.id DESC",
    )
    .fetch_all(db.reader())
    .await
    .unwrap_or_default();

    Json(serde_json::json!({ "data": agents })).into_response()
}

pub async fn store(
    Extension(db): Extension<Database>,
    Json(payload): Json<AgentCreatePayload>,
) -> Response {
    let password_hash = match hash_password(&payload.password) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("Failed to hash password: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "An internal error occurred." })),
            )
                .into_response();
        }
    };

    let result = sqlx::query_as::<_, SuperAdmin>(
        "INSERT INTO main.super_admins (first_name, last_name, email, password_hash, role_id, status)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING *, NULL::text AS permission_type, NULL::jsonb AS role_permissions",
    )
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(payload.role_id)
    .bind(payload.status.unwrap_or(true))
    .fetch_one(db.writer())
    .await;

    match result {
        Ok(agent) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "data": agent, "message": "Agent created successfully." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to create agent: {e}");
            let msg = if e.to_string().contains("duplicate key") {
                "An agent with this email already exists."
            } else {
                "Failed to create agent."
            };
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": msg })),
            )
                .into_response()
        }
    }
}

pub async fn show(
    Extension(db): Extension<Database>,
    Path(id): Path<i64>,
) -> Response {
    let agent = sqlx::query_as::<_, SuperAdmin>(
        "SELECT sa.*, sr.permission_type, sr.permissions AS role_permissions
         FROM main.super_admins sa
         JOIN main.super_roles sr ON sr.id = sa.role_id
         WHERE sa.id = $1",
    )
    .bind(id)
    .fetch_optional(db.reader())
    .await;

    match agent {
        Ok(Some(a)) => Json(serde_json::json!({ "data": a })).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Agent not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch agent: {e}");
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
    Json(payload): Json<AgentUpdatePayload>,
) -> Response {
    // If password provided, hash it and update all fields including password
    if let Some(ref password) = payload.password {
        if !password.is_empty() {
            let password_hash = match hash_password(password) {
                Ok(h) => h,
                Err(e) => {
                    tracing::error!("Failed to hash password: {e}");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": "An internal error occurred." })),
                    )
                        .into_response();
                }
            };

            let result = sqlx::query_as::<_, SuperAdmin>(
                "UPDATE main.super_admins
                 SET first_name = $1, last_name = $2, email = $3, password_hash = $4,
                     role_id = $5, status = $6, updated_at = NOW()
                 WHERE id = $7
                 RETURNING *, NULL::text AS permission_type, NULL::jsonb AS role_permissions",
            )
            .bind(&payload.first_name)
            .bind(&payload.last_name)
            .bind(&payload.email)
            .bind(&password_hash)
            .bind(payload.role_id)
            .bind(payload.status.unwrap_or(true))
            .bind(id)
            .fetch_optional(db.writer())
            .await;

            return match result {
                Ok(Some(agent)) => {
                    Json(serde_json::json!({ "data": agent, "message": "Agent updated successfully." }))
                        .into_response()
                }
                Ok(None) => (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "Agent not found." })),
                )
                    .into_response(),
                Err(e) => {
                    tracing::error!("Failed to update agent: {e}");
                    update_error(e)
                }
            };
        }
    }

    // Update without changing password
    let result = sqlx::query_as::<_, SuperAdmin>(
        "UPDATE main.super_admins
         SET first_name = $1, last_name = $2, email = $3,
             role_id = $4, status = $5, updated_at = NOW()
         WHERE id = $6
         RETURNING *, NULL::text AS permission_type, NULL::jsonb AS role_permissions",
    )
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.email)
    .bind(payload.role_id)
    .bind(payload.status.unwrap_or(true))
    .bind(id)
    .fetch_optional(db.writer())
    .await;

    match result {
        Ok(Some(agent)) => {
            Json(serde_json::json!({ "data": agent, "message": "Agent updated successfully." }))
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Agent not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to update agent: {e}");
            update_error(e)
        }
    }
}

pub async fn destroy(
    Extension(db): Extension<Database>,
    Extension(admin): Extension<SuperAdmin>,
    Path(id): Path<i64>,
) -> Response {
    // Can't delete yourself
    if admin.id == id {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "You cannot delete your own account." })),
        )
            .into_response();
    }

    // Can't delete the last admin
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM main.super_admins")
        .fetch_one(db.reader())
        .await
        .unwrap_or((0,));

    if count.0 <= 1 {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "Cannot delete the last administrator." })),
        )
            .into_response();
    }

    let result = sqlx::query("DELETE FROM main.super_admins WHERE id = $1")
        .bind(id)
        .execute(db.writer())
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(serde_json::json!({ "message": "Agent deleted successfully." })).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Agent not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to delete agent: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete agent." })),
            )
                .into_response()
        }
    }
}

fn update_error(e: sqlx::Error) -> Response {
    let msg = if e.to_string().contains("duplicate key") {
        "An agent with this email already exists."
    } else {
        "Failed to update agent."
    };
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": msg })),
    )
        .into_response()
}
