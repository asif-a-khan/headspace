//! Authentication guard middleware.
//!
//! `require_super_admin` reads `super_admin_id` from the session,
//! loads the admin with role permissions, and injects `SuperAdmin`
//! into request extensions. Redirects to /super/login if unauthenticated.

use axum::{
    extract::{Extension, Request},
    http::header,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tower_sessions::Session;

use crate::db::Database;
use crate::models::super_admin::SuperAdmin;

const SUPER_ADMIN_SESSION_KEY: &str = "super_admin_id";

/// Middleware: require an authenticated super admin.
pub async fn require_super_admin(
    session: Session,
    Extension(db): Extension<Database>,
    mut req: Request,
    next: Next,
) -> Response {
    let admin_id: Option<i64> = session
        .get(SUPER_ADMIN_SESSION_KEY)
        .await
        .unwrap_or(None);

    let Some(admin_id) = admin_id else {
        return redirect_to_login();
    };

    let admin = sqlx::query_as::<_, SuperAdmin>(
        "SELECT sa.*, sr.permission_type, sr.permissions AS role_permissions
         FROM main.super_admins sa
         JOIN main.super_roles sr ON sr.id = sa.role_id
         WHERE sa.id = $1 AND sa.status = true",
    )
    .bind(admin_id)
    .fetch_optional(db.reader())
    .await;

    match admin {
        Ok(Some(admin)) => {
            req.extensions_mut().insert(admin);
            next.run(req).await
        }
        Ok(None) => {
            // Admin no longer exists or is inactive — clear session
            let _ = session.delete().await;
            redirect_to_login()
        }
        Err(e) => {
            tracing::error!("Failed to load super admin: {e}");
            redirect_to_login()
        }
    }
}

fn redirect_to_login() -> Response {
    (
        axum::http::StatusCode::SEE_OTHER,
        [(header::LOCATION, "/super/login")],
    )
        .into_response()
}

/// Set the super admin ID in the session (called on login).
pub async fn set_super_admin_session(session: &Session, admin_id: i64) -> anyhow::Result<()> {
    session
        .insert(SUPER_ADMIN_SESSION_KEY, admin_id)
        .await
        .map_err(|e| anyhow::anyhow!("Session error: {e}"))?;
    Ok(())
}

/// Clear the super admin session (called on logout).
pub async fn clear_super_admin_session(session: &Session) -> anyhow::Result<()> {
    session
        .delete()
        .await
        .map_err(|e| anyhow::anyhow!("Session error: {e}"))?;
    Ok(())
}
