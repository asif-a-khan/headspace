use axum::Json;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::auth::password::{hash_password, verify_password};
use crate::db::Database;
use crate::models::super_admin::SuperAdmin;

#[derive(Deserialize)]
pub struct AccountUpdatePayload {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub current_password: String,
    pub password: Option<String>,
    pub password_confirmation: Option<String>,
}

pub async fn show(Extension(admin): Extension<SuperAdmin>) -> Response {
    Json(serde_json::json!({
        "data": {
            "id": admin.id,
            "first_name": admin.first_name,
            "last_name": admin.last_name,
            "email": admin.email,
            "image": admin.image,
        }
    }))
    .into_response()
}

pub async fn update(
    Extension(db): Extension<Database>,
    Extension(admin): Extension<SuperAdmin>,
    Json(payload): Json<AccountUpdatePayload>,
) -> Response {
    // Verify current password
    match verify_password(&payload.current_password, &admin.password_hash) {
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
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "An internal error occurred." })),
            )
                .into_response();
        }
    }

    // If new password provided, validate confirmation and hash it
    if let Some(ref new_password) = payload.password
        && !new_password.is_empty()
    {
        let confirmation = payload.password_confirmation.as_deref().unwrap_or("");
        if new_password != confirmation {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Password confirmation does not match." })),
            )
                .into_response();
        }

        let password_hash = match hash_password(new_password) {
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

        let result = sqlx::query(
                "UPDATE main.super_admins
                 SET first_name = $1, last_name = $2, email = $3, password_hash = $4, updated_at = NOW()
                 WHERE id = $5",
            )
            .bind(&payload.first_name)
            .bind(&payload.last_name)
            .bind(&payload.email)
            .bind(&password_hash)
            .bind(admin.id)
            .execute(db.writer())
            .await;

        return match result {
            Ok(_) => Json(serde_json::json!({ "message": "Account updated successfully." }))
                .into_response(),
            Err(e) => {
                tracing::error!("Failed to update account: {e}");
                update_error(e)
            }
        };
    }

    // Update without changing password
    let result = sqlx::query(
        "UPDATE main.super_admins
         SET first_name = $1, last_name = $2, email = $3, updated_at = NOW()
         WHERE id = $4",
    )
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.email)
    .bind(admin.id)
    .execute(db.writer())
    .await;

    match result {
        Ok(_) => {
            Json(serde_json::json!({ "message": "Account updated successfully." })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to update account: {e}");
            update_error(e)
        }
    }
}

fn update_error(e: sqlx::Error) -> Response {
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
