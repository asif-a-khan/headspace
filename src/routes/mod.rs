//! Route definitions.
//!
//! All application routes are defined here, grouped by middleware requirements.

use axum::routing::{get, post};
use axum::{middleware, Extension, Router};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::PostgresStore;

use crate::api;
use crate::config::Config;
use crate::db::Database;
use crate::handlers;
use crate::middleware::auth::{require_super_admin, require_tenant_admin};
use crate::middleware::csrf::require_csrf;
use crate::middleware::tenant::require_tenant;

/// Build the application router with all route groups.
pub fn app_router(
    db: Database,
    config: Config,
    session_layer: SessionManagerLayer<PostgresStore>,
) -> Router {
    Router::new()
        .route("/health", get(handlers::health::health_check))
        // Super admin: public routes (no auth)
        .route("/super/login", get(handlers::super_admin::auth::login_page))
        .route("/super/api/login", post(api::super_admin::auth::login))
        // Super admin: protected routes (require auth)
        .merge(super_admin_routes())
        // Tenant admin: all routes (require tenant resolution)
        .merge(tenant_admin_routes())
        // Static files
        .nest_service("/static", ServeDir::new("static"))
        // Global layers (order: outermost applied last — CSRF runs after session is available)
        .layer(middleware::from_fn(require_csrf))
        .layer(session_layer)
        .layer(Extension(db))
        .layer(Extension(config))
        .layer(TraceLayer::new_for_http())
}

/// Super admin routes that require authentication.
fn super_admin_routes() -> Router {
    Router::new()
        // HTML page routes
        .route("/super/logout", post(handlers::super_admin::auth::logout))
        .route("/super/tenants", get(handlers::super_admin::tenants::index))
        .route("/super/tenants/create", get(handlers::super_admin::tenants::create))
        .route("/super/tenants/{id}/edit", get(handlers::super_admin::tenants::edit))
        .route("/super/settings", get(handlers::super_admin::settings::index))
        .route("/super/settings/agents", get(handlers::super_admin::agents::index))
        .route("/super/settings/agents/create", get(handlers::super_admin::agents::create))
        .route("/super/settings/agents/{id}/edit", get(handlers::super_admin::agents::edit))
        .route("/super/settings/roles", get(handlers::super_admin::roles::index))
        .route("/super/settings/roles/create", get(handlers::super_admin::roles::create))
        .route("/super/settings/roles/{id}/edit", get(handlers::super_admin::roles::edit))
        .route("/super/account", get(handlers::super_admin::account::edit))
        // JSON API routes
        .route("/super/api/tenants", get(api::super_admin::tenants::list).post(api::super_admin::tenants::store))
        .route("/super/api/tenants/{id}", get(api::super_admin::tenants::show).put(api::super_admin::tenants::update).delete(api::super_admin::tenants::destroy))
        .route("/super/api/agents", get(api::super_admin::agents::list).post(api::super_admin::agents::store))
        .route("/super/api/agents/{id}", get(api::super_admin::agents::show).put(api::super_admin::agents::update).delete(api::super_admin::agents::destroy))
        .route("/super/api/roles", get(api::super_admin::roles::list).post(api::super_admin::roles::store))
        .route("/super/api/roles/{id}", get(api::super_admin::roles::show).put(api::super_admin::roles::update).delete(api::super_admin::roles::destroy))
        .route("/super/api/account", get(api::super_admin::account::show).put(api::super_admin::account::update))
        // Auth middleware for all routes in this group
        .layer(middleware::from_fn(require_super_admin))
}

/// Tenant admin routes — all wrapped in tenant resolution middleware.
///
/// Middleware order: require_tenant (outer, runs first) → require_tenant_admin (inner).
/// This ensures Company is available in extensions before auth queries the tenant schema.
fn tenant_admin_routes() -> Router {
    Router::new()
        // Public routes (tenant resolution only, no auth)
        .route("/admin/login", get(handlers::tenant_admin::auth::login_page))
        .route("/admin/api/login", post(api::tenant_admin::auth::login))
        // Protected routes (tenant resolution + auth)
        .merge(tenant_admin_protected_routes())
        // Tenant resolution middleware for ALL /admin/* routes
        .layer(middleware::from_fn(require_tenant))
}

/// Tenant admin routes that require authentication.
fn tenant_admin_protected_routes() -> Router {
    Router::new()
        .route("/admin/logout", post(handlers::tenant_admin::auth::logout))
        .route("/admin/dashboard", get(handlers::tenant_admin::dashboard::index))
        .layer(middleware::from_fn(require_tenant_admin))
}
