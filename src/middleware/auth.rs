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
use crate::db::tenant::{reset_tenant, set_tenant};
use crate::models::company::Company;
use crate::models::super_admin::SuperAdmin;
use crate::models::tenant_admin::TenantUser;

const SUPER_ADMIN_SESSION_KEY: &str = "super_admin_id";
const TENANT_ADMIN_SESSION_KEY: &str = "tenant_admin_id";

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

// -- Tenant admin auth --

/// Middleware: require an authenticated tenant admin.
///
/// Expects `Company` to already be in request extensions (set by `require_tenant`).
/// Acquires a connection, sets tenant search_path, queries, then resets before
/// returning the connection to the pool.
pub async fn require_tenant_admin(
    session: Session,
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    mut req: Request,
    next: Next,
) -> Response {
    let admin_id: Option<i64> = session
        .get(TENANT_ADMIN_SESSION_KEY)
        .await
        .unwrap_or(None);

    let Some(admin_id) = admin_id else {
        return redirect_to_admin_login();
    };

    let mut conn = match db.reader().acquire().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to acquire connection: {e}");
            return redirect_to_admin_login();
        }
    };

    if let Err(e) = set_tenant(&mut conn, &company.schema_name).await {
        tracing::error!("Failed to set tenant: {e}");
        return redirect_to_admin_login();
    }

    let user = sqlx::query_as::<_, TenantUser>(
        "SELECT u.*, r.permission_type, r.permissions AS role_permissions
         FROM users u
         JOIN roles r ON r.id = u.role_id
         WHERE u.id = $1 AND u.status = true",
    )
    .bind(admin_id)
    .fetch_optional(&mut *conn)
    .await;

    if let Err(e) = reset_tenant(&mut conn).await {
        tracing::error!("Failed to reset tenant: {e}");
    }

    match user {
        Ok(Some(user)) => {
            req.extensions_mut().insert(user);
            next.run(req).await
        }
        Ok(None) => {
            let _ = session.delete().await;
            redirect_to_admin_login()
        }
        Err(e) => {
            tracing::error!("Failed to load tenant admin: {e}");
            redirect_to_admin_login()
        }
    }
}

fn redirect_to_admin_login() -> Response {
    (
        axum::http::StatusCode::SEE_OTHER,
        [(header::LOCATION, "/admin/login")],
    )
        .into_response()
}

/// Set the tenant admin ID in the session (called on login).
pub async fn set_tenant_admin_session(session: &Session, admin_id: i64) -> anyhow::Result<()> {
    session
        .insert(TENANT_ADMIN_SESSION_KEY, admin_id)
        .await
        .map_err(|e| anyhow::anyhow!("Session error: {e}"))?;
    Ok(())
}

/// Clear the tenant admin session (called on logout).
pub async fn clear_tenant_admin_session(session: &Session) -> anyhow::Result<()> {
    session
        .delete()
        .await
        .map_err(|e| anyhow::anyhow!("Session error: {e}"))?;
    Ok(())
}
