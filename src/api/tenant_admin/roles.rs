use axum::Json;
use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use validator::Validate;

use crate::auth::bouncer::{bouncer, validate_payload};
use crate::db::Database;
use crate::db::guard::TenantGuard;
use crate::models::company::Company;
use crate::models::tenant_admin::{TenantRole, TenantUser};

#[derive(Deserialize, Validate)]
pub struct RolePayload {
    #[validate(length(min = 1, message = "Name is required."))]
    pub name: String,
    pub description: Option<String>,
    #[validate(length(min = 1, message = "Permission type is required."))]
    pub permission_type: String,
    pub permissions: serde_json::Value,
}

pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.roles") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let roles = guard
        .fetch_all(sqlx::query_as::<_, TenantRole>(
            "SELECT * FROM roles ORDER BY id DESC",
        ))
        .await;

    let _ = guard.release().await;

    match roles {
        Ok(roles) => Json(serde_json::json!({ "data": roles })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list roles: {e}");
            internal_error()
        }
    }
}

pub async fn store(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<RolePayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.roles.create") {
        return resp;
    }
    if let Err(resp) = validate_payload(&payload) {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .fetch_one(
            sqlx::query_as::<_, TenantRole>(
                "INSERT INTO roles (name, description, permission_type, permissions)
             VALUES ($1, $2, $3, $4)
             RETURNING *",
            )
            .bind(&payload.name)
            .bind(&payload.description)
            .bind(&payload.permission_type)
            .bind(&payload.permissions),
        )
        .await;

    let _ = guard.release().await;

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
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.roles.edit") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let role = guard
        .fetch_optional(
            sqlx::query_as::<_, TenantRole>("SELECT * FROM roles WHERE id = $1").bind(id),
        )
        .await;

    let _ = guard.release().await;

    match role {
        Ok(Some(r)) => Json(serde_json::json!({ "data": r })).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Role not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch role: {e}");
            internal_error()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<RolePayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.roles.edit") {
        return resp;
    }
    if let Err(resp) = validate_payload(&payload) {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .fetch_optional(sqlx::query_as::<_, TenantRole>(
            "UPDATE roles
             SET name = $1, description = $2, permission_type = $3, permissions = $4, updated_at = NOW()
             WHERE id = $5
             RETURNING *",
        )
        .bind(&payload.name)
        .bind(&payload.description)
        .bind(&payload.permission_type)
        .bind(&payload.permissions)
        .bind(id))
        .await;

    let _ = guard.release().await;

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
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.roles.delete") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    // Cannot delete your own role
    if user.role_id == id {
        let _ = guard.release().await;
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "Cannot delete your own role." })),
        )
            .into_response();
    }

    // Cannot delete the last role
    let role_count = guard
        .fetch_one(sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM roles"))
        .await;

    if let Ok((c,)) = role_count
        && c <= 1
    {
        let _ = guard.release().await;
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "Cannot delete the last role." })),
        )
            .into_response();
    }

    // Check if any users are assigned to this role
    let user_count = guard
        .fetch_one(
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM users WHERE role_id = $1").bind(id),
        )
        .await;

    if let Ok((c,)) = user_count
        && c > 0
    {
        let _ = guard.release().await;
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "Cannot delete a role that has users assigned." })),
        )
            .into_response();
    }

    let result = guard
        .execute(sqlx::query("DELETE FROM roles WHERE id = $1").bind(id))
        .await;

    let _ = guard.release().await;

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

fn internal_error() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "An internal error occurred." })),
    )
        .into_response()
}
