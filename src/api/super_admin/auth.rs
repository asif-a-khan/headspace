use axum::Json;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use tower_sessions::Session;

use crate::auth::password::verify_password;
use crate::db::Database;
use crate::middleware::auth::set_super_admin_session;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn login(
    session: Session,
    Extension(db): Extension<Database>,
    Json(payload): Json<LoginRequest>,
) -> Response {
    // CSRF is validated by the require_csrf middleware

    // Look up admin by email
    let admin = sqlx::query_as::<_, crate::models::super_admin::SuperAdmin>(
        "SELECT sa.*, sr.permission_type, sr.permissions AS role_permissions
         FROM main.super_admins sa
         JOIN main.super_roles sr ON sr.id = sa.role_id
         WHERE sa.email = $1 AND sa.status = true",
    )
    .bind(&payload.email)
    .fetch_optional(db.reader())
    .await;

    let admin = match admin {
        Ok(Some(a)) => a,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Invalid credentials." })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Login query failed: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "An internal error occurred." })),
            )
                .into_response();
        }
    };

    // Verify password
    match verify_password(&payload.password, &admin.password_hash) {
        Ok(true) => {}
        Ok(false) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Invalid credentials." })),
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

    // Set session
    if let Err(e) = set_super_admin_session(&session, admin.id).await {
        tracing::error!("Failed to set session: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "An internal error occurred." })),
        )
            .into_response();
    }

    Json(serde_json::json!({
        "success": true,
        "redirect": "/super/tenants",
    }))
    .into_response()
}
