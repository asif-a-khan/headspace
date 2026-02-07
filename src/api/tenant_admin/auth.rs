use axum::extract::Extension;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use tower_sessions::Session;

use crate::auth::password::verify_password;
use crate::db::Database;
use crate::db::guard::TenantGuard;
use crate::middleware::auth::set_tenant_admin_session;
use crate::models::company::Company;
use crate::models::tenant_admin::TenantUser;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn login(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Json(payload): Json<LoginRequest>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "An internal error occurred." })),
            )
                .into_response();
        }
    };

    let admin = guard.fetch_optional(
        sqlx::query_as::<_, TenantUser>(
            "SELECT u.*, r.permission_type, r.permissions AS role_permissions
             FROM users u
             JOIN roles r ON r.id = u.role_id
             WHERE u.email = $1 AND u.status = true",
        )
        .bind(&payload.email),
    )
    .await;

    let _ = guard.release().await;

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
            tracing::error!("Tenant login query failed: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "An internal error occurred." })),
            )
                .into_response();
        }
    };

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

    if let Err(e) = set_tenant_admin_session(&session, admin.id).await {
        tracing::error!("Failed to set session: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "An internal error occurred." })),
        )
            .into_response();
    }

    Json(serde_json::json!({
        "success": true,
        "redirect": "/admin/dashboard",
    }))
    .into_response()
}
