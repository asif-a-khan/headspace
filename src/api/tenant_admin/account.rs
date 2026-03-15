use axum::Json;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use validator::Validate;

use crate::auth::bouncer::validate_payload;
use crate::auth::password::{hash_password, verify_password};
use crate::db::Database;
use crate::db::guard::TenantGuard;
use crate::models::company::Company;
use crate::models::tenant_admin::TenantUser;

#[derive(Deserialize, Validate)]
pub struct AccountUpdatePayload {
    #[validate(length(min = 1, message = "First name is required."))]
    pub first_name: String,
    #[validate(length(min = 1, message = "Last name is required."))]
    pub last_name: String,
    #[validate(email(message = "Invalid email address."))]
    pub email: String,
}

#[derive(Deserialize, Validate)]
pub struct PasswordUpdatePayload {
    #[validate(length(min = 1, message = "Current password is required."))]
    pub current_password: String,
    #[validate(length(min = 6, message = "New password must be at least 6 characters."))]
    pub password: String,
    #[validate(length(min = 1, message = "Password confirmation is required."))]
    pub password_confirmation: String,
}

pub async fn show(Extension(user): Extension<TenantUser>) -> Response {
    Json(serde_json::json!({
        "data": {
            "id": user.id,
            "first_name": user.first_name,
            "last_name": user.last_name,
            "email": user.email,
            "image": user.image,
        }
    }))
    .into_response()
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<AccountUpdatePayload>,
) -> Response {
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
        .fetch_optional(
            sqlx::query_as::<_, TenantUser>(
                "UPDATE users
             SET first_name = $1, last_name = $2, email = $3, updated_at = NOW()
             WHERE id = $4
             RETURNING *, NULL::text AS permission_type, NULL::jsonb AS role_permissions",
            )
            .bind(&payload.first_name)
            .bind(&payload.last_name)
            .bind(&payload.email)
            .bind(user.id),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(Some(updated)) => Json(serde_json::json!({
            "data": {
                "id": updated.id,
                "first_name": updated.first_name,
                "last_name": updated.last_name,
                "email": updated.email,
                "image": updated.image,
            },
            "message": "Account updated successfully."
        }))
        .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "User not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to update account: {e}");
            let msg = if e.to_string().contains("duplicate key") {
                "An account with this email already exists."
            } else {
                "Failed to update account."
            };
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": msg })),
            )
                .into_response()
        }
    }
}

pub async fn update_password(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<PasswordUpdatePayload>,
) -> Response {
    if let Err(resp) = validate_payload(&payload) {
        return resp;
    }

    // Validate confirmation matches
    if payload.password != payload.password_confirmation {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "Password confirmation does not match." })),
        )
            .into_response();
    }

    // Verify current password
    match verify_password(&payload.current_password, &user.password_hash) {
        Ok(true) => {}
        Ok(false) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Current password is incorrect." })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Password verification error: {e}");
            return internal_error();
        }
    }

    // Hash new password
    let password_hash = match hash_password(&payload.password) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("Failed to hash password: {e}");
            return internal_error();
        }
    };

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let result = guard
        .execute(
            sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
                .bind(&password_hash)
                .bind(user.id),
        )
        .await;

    let _ = guard.release().await;

    match result {
        Ok(_) => {
            Json(serde_json::json!({ "message": "Password updated successfully." })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to update password: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to update password." })),
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
