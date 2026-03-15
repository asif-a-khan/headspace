use axum::Json;
use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use validator::Validate;

use crate::auth::bouncer::{bouncer, validate_payload};
use crate::auth::password::hash_password;
use crate::db::Database;
use crate::db::guard::TenantGuard;
use crate::models::company::Company;
use crate::models::tenant_admin::TenantUser;

#[derive(Deserialize, Validate)]
pub struct UserCreatePayload {
    #[validate(length(min = 1, message = "First name is required."))]
    pub first_name: String,
    #[validate(length(min = 1, message = "Last name is required."))]
    pub last_name: String,
    #[validate(email(message = "Invalid email address."))]
    pub email: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters."))]
    pub password: String,
    pub role_id: i64,
    pub view_permission: Option<String>,
    pub status: Option<bool>,
    pub group_ids: Option<Vec<i64>>,
}

#[derive(Deserialize, Validate)]
pub struct UserUpdatePayload {
    #[validate(length(min = 1, message = "First name is required."))]
    pub first_name: String,
    #[validate(length(min = 1, message = "Last name is required."))]
    pub last_name: String,
    #[validate(email(message = "Invalid email address."))]
    pub email: String,
    pub password: Option<String>,
    pub role_id: i64,
    pub view_permission: Option<String>,
    pub status: Option<bool>,
    pub group_ids: Option<Vec<i64>>,
}

pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.users") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let users = guard
        .fetch_all(sqlx::query_as::<_, TenantUser>(
            "SELECT u.*, r.permission_type, r.permissions AS role_permissions
             FROM users u
             JOIN roles r ON r.id = u.role_id
             ORDER BY u.id DESC",
        ))
        .await;

    let _ = guard.release().await;

    match users {
        Ok(users) => Json(serde_json::json!({ "data": users })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list users: {e}");
            internal_error()
        }
    }
}

pub async fn store(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<UserCreatePayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.users.create") {
        return resp;
    }
    if let Err(resp) = validate_payload(&payload) {
        return resp;
    }

    let password_hash = match hash_password(&payload.password) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("Failed to hash password: {e}");
            return internal_error();
        }
    };

    let view_permission = payload.view_permission.as_deref().unwrap_or("global");

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .fetch_one(sqlx::query_as::<_, TenantUser>(
            "INSERT INTO users (first_name, last_name, email, password_hash, role_id, view_permission, status)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING *, NULL::text AS permission_type, NULL::jsonb AS role_permissions",
        )
        .bind(&payload.first_name)
        .bind(&payload.last_name)
        .bind(&payload.email)
        .bind(&password_hash)
        .bind(payload.role_id)
        .bind(view_permission)
        .bind(payload.status.unwrap_or(true)))
        .await;

    let new_user = match result {
        Ok(u) => u,
        Err(e) => {
            let _ = guard.release().await;
            tracing::error!("Failed to create user: {e}");
            let msg = if e.to_string().contains("duplicate key") {
                "A user with this email already exists."
            } else {
                "Failed to create user."
            };
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": msg })),
            )
                .into_response();
        }
    };

    // Sync group memberships
    if let Some(ref group_ids) = payload.group_ids {
        sync_user_groups(&mut guard, new_user.id, group_ids).await;
    }

    let _ = guard.release().await;

    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "data": new_user, "message": "User created successfully." })),
    )
        .into_response()
}

pub async fn show(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.users.edit") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let target = guard
        .fetch_optional(
            sqlx::query_as::<_, TenantUser>(
                "SELECT u.*, r.permission_type, r.permissions AS role_permissions
             FROM users u
             JOIN roles r ON r.id = u.role_id
             WHERE u.id = $1",
            )
            .bind(id),
        )
        .await;

    // Also fetch group IDs for this user
    let group_ids = guard
        .fetch_all(
            sqlx::query_as::<_, (i64,)>("SELECT group_id FROM user_groups WHERE user_id = $1")
                .bind(id),
        )
        .await;

    let _ = guard.release().await;

    match target {
        Ok(Some(u)) => {
            let gids: Vec<i64> = group_ids
                .unwrap_or_default()
                .into_iter()
                .map(|r| r.0)
                .collect();
            Json(serde_json::json!({ "data": u, "group_ids": gids })).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "User not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch user: {e}");
            internal_error()
        }
    }
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<UserUpdatePayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "settings.users.edit") {
        return resp;
    }
    if let Err(resp) = validate_payload(&payload) {
        return resp;
    }

    let view_permission = payload.view_permission.as_deref().unwrap_or("global");

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    // If password provided, hash it
    let result = if let Some(ref password) = payload.password {
        if !password.is_empty() {
            let password_hash = match hash_password(password) {
                Ok(h) => h,
                Err(e) => {
                    let _ = guard.release().await;
                    tracing::error!("Failed to hash password: {e}");
                    return internal_error();
                }
            };

            guard
                .fetch_optional(
                    sqlx::query_as::<_, TenantUser>(
                        "UPDATE users
                     SET first_name = $1, last_name = $2, email = $3, password_hash = $4,
                         role_id = $5, view_permission = $6, status = $7, updated_at = NOW()
                     WHERE id = $8
                     RETURNING *, NULL::text AS permission_type, NULL::jsonb AS role_permissions",
                    )
                    .bind(&payload.first_name)
                    .bind(&payload.last_name)
                    .bind(&payload.email)
                    .bind(&password_hash)
                    .bind(payload.role_id)
                    .bind(view_permission)
                    .bind(payload.status.unwrap_or(true))
                    .bind(id),
                )
                .await
        } else {
            update_without_password(&mut guard, &payload, view_permission, id).await
        }
    } else {
        update_without_password(&mut guard, &payload, view_permission, id).await
    };

    match result {
        Ok(Some(updated)) => {
            if let Some(ref group_ids) = payload.group_ids {
                sync_user_groups(&mut guard, updated.id, group_ids).await;
            }
            let _ = guard.release().await;
            Json(serde_json::json!({ "data": updated, "message": "User updated successfully." }))
                .into_response()
        }
        Ok(None) => {
            let _ = guard.release().await;
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "User not found." })),
            )
                .into_response()
        }
        Err(e) => {
            let _ = guard.release().await;
            tracing::error!("Failed to update user: {e}");
            let msg = if e.to_string().contains("duplicate key") {
                "A user with this email already exists."
            } else {
                "Failed to update user."
            };
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": msg })),
            )
                .into_response()
        }
    }
}

async fn update_without_password(
    guard: &mut TenantGuard,
    payload: &UserUpdatePayload,
    view_permission: &str,
    id: i64,
) -> Result<Option<TenantUser>, sqlx::Error> {
    guard
        .fetch_optional(
            sqlx::query_as::<_, TenantUser>(
                "UPDATE users
             SET first_name = $1, last_name = $2, email = $3,
                 role_id = $4, view_permission = $5, status = $6, updated_at = NOW()
             WHERE id = $7
             RETURNING *, NULL::text AS permission_type, NULL::jsonb AS role_permissions",
            )
            .bind(&payload.first_name)
            .bind(&payload.last_name)
            .bind(&payload.email)
            .bind(payload.role_id)
            .bind(view_permission)
            .bind(payload.status.unwrap_or(true))
            .bind(id),
        )
        .await
}

pub async fn destroy(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(current_user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&current_user, "settings.users.delete") {
        return resp;
    }

    if current_user.id == id {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "You cannot delete your own account." })),
        )
            .into_response();
    }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let count = guard
        .fetch_one(sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM users"))
        .await;

    if let Ok((c,)) = count
        && c <= 1
    {
        let _ = guard.release().await;
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "Cannot delete the last user." })),
        )
            .into_response();
    }

    let result = guard
        .execute(sqlx::query("DELETE FROM users WHERE id = $1").bind(id))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(serde_json::json!({ "message": "User deleted successfully." })).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "User not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to delete user: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete user." })),
            )
                .into_response()
        }
    }
}

async fn sync_user_groups(guard: &mut TenantGuard, user_id: i64, group_ids: &[i64]) {
    // Delete existing
    let _ = guard
        .execute(sqlx::query("DELETE FROM user_groups WHERE user_id = $1").bind(user_id))
        .await;

    // Insert new
    for &gid in group_ids {
        let _ = guard
            .execute(
                sqlx::query("INSERT INTO user_groups (user_id, group_id) VALUES ($1, $2)")
                    .bind(user_id)
                    .bind(gid),
            )
            .await;
    }
}

fn internal_error() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "An internal error occurred." })),
    )
        .into_response()
}
