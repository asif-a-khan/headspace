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
        // Settings pages
        .route("/admin/settings", get(handlers::tenant_admin::settings::index))
        .route("/admin/settings/users", get(handlers::tenant_admin::settings::users_index))
        .route("/admin/settings/users/create", get(handlers::tenant_admin::settings::users_create))
        .route("/admin/settings/users/{id}/edit", get(handlers::tenant_admin::settings::users_edit))
        .route("/admin/settings/roles", get(handlers::tenant_admin::settings::roles_index))
        .route("/admin/settings/roles/create", get(handlers::tenant_admin::settings::roles_create))
        .route("/admin/settings/roles/{id}/edit", get(handlers::tenant_admin::settings::roles_edit))
        .route("/admin/settings/groups", get(handlers::tenant_admin::settings::groups_index))
        .route("/admin/settings/groups/create", get(handlers::tenant_admin::settings::groups_create))
        .route("/admin/settings/groups/{id}/edit", get(handlers::tenant_admin::settings::groups_edit))
        .route("/admin/settings/attributes", get(handlers::tenant_admin::attributes::index))
        .route("/admin/settings/attributes/create", get(handlers::tenant_admin::attributes::create))
        .route("/admin/settings/attributes/{id}/edit", get(handlers::tenant_admin::attributes::edit))
        .route("/admin/settings/pipelines", get(handlers::tenant_admin::pipelines::pipelines_index))
        .route("/admin/settings/pipelines/create", get(handlers::tenant_admin::pipelines::pipelines_create))
        .route("/admin/settings/pipelines/{id}/edit", get(handlers::tenant_admin::pipelines::pipelines_edit))
        .route("/admin/settings/sources", get(handlers::tenant_admin::pipelines::sources_index))
        .route("/admin/settings/types", get(handlers::tenant_admin::pipelines::types_index))
        // Dashboard API
        .route("/admin/api/dashboard/stats", get(api::tenant_admin::dashboard::stats))
        // Quote pages
        .route("/admin/quotes", get(handlers::tenant_admin::quotes::index))
        .route("/admin/quotes/create", get(handlers::tenant_admin::quotes::create))
        .route("/admin/quotes/{id}/edit", get(handlers::tenant_admin::quotes::edit))
        // Activity pages
        .route("/admin/activities", get(handlers::tenant_admin::activities::index))
        .route("/admin/activities/create", get(handlers::tenant_admin::activities::create))
        .route("/admin/activities/{id}/edit", get(handlers::tenant_admin::activities::edit))
        // Product pages
        .route("/admin/products", get(handlers::tenant_admin::products::index))
        .route("/admin/products/create", get(handlers::tenant_admin::products::create))
        .route("/admin/products/{id}/edit", get(handlers::tenant_admin::products::edit))
        // Lead pages
        .route("/admin/leads", get(handlers::tenant_admin::leads::index))
        .route("/admin/leads/create", get(handlers::tenant_admin::leads::create))
        .route("/admin/leads/{id}/edit", get(handlers::tenant_admin::leads::edit))
        .route("/admin/leads/kanban", get(handlers::tenant_admin::leads::kanban_page))
        // Contact pages
        .route("/admin/contacts/persons", get(handlers::tenant_admin::contacts::persons_index))
        .route("/admin/contacts/persons/create", get(handlers::tenant_admin::contacts::persons_create))
        .route("/admin/contacts/persons/{id}/edit", get(handlers::tenant_admin::contacts::persons_edit))
        .route("/admin/contacts/organizations", get(handlers::tenant_admin::contacts::organizations_index))
        .route("/admin/contacts/organizations/create", get(handlers::tenant_admin::contacts::organizations_create))
        .route("/admin/contacts/organizations/{id}/edit", get(handlers::tenant_admin::contacts::organizations_edit))
        // Settings API routes
        .route("/admin/api/settings/users", get(api::tenant_admin::users::list).post(api::tenant_admin::users::store))
        .route("/admin/api/settings/users/{id}", get(api::tenant_admin::users::show).put(api::tenant_admin::users::update).delete(api::tenant_admin::users::destroy))
        .route("/admin/api/settings/roles", get(api::tenant_admin::roles::list).post(api::tenant_admin::roles::store))
        .route("/admin/api/settings/roles/{id}", get(api::tenant_admin::roles::show).put(api::tenant_admin::roles::update).delete(api::tenant_admin::roles::destroy))
        .route("/admin/api/settings/groups", get(api::tenant_admin::groups::list).post(api::tenant_admin::groups::store))
        .route("/admin/api/settings/groups/{id}", get(api::tenant_admin::groups::show).put(api::tenant_admin::groups::update).delete(api::tenant_admin::groups::destroy))
        .route("/admin/api/settings/attributes", get(api::tenant_admin::attributes::list).post(api::tenant_admin::attributes::store))
        .route("/admin/api/settings/attributes/{id}", get(api::tenant_admin::attributes::show).put(api::tenant_admin::attributes::update).delete(api::tenant_admin::attributes::destroy))
        .route("/admin/api/settings/attributes/{id}/options", get(api::tenant_admin::attributes::list_options))
        .route("/admin/api/settings/pipelines", get(api::tenant_admin::pipelines::list).post(api::tenant_admin::pipelines::store))
        .route("/admin/api/settings/pipelines/{id}", get(api::tenant_admin::pipelines::show).put(api::tenant_admin::pipelines::update).delete(api::tenant_admin::pipelines::destroy))
        .route("/admin/api/settings/sources", get(api::tenant_admin::sources::list).post(api::tenant_admin::sources::store))
        .route("/admin/api/settings/sources/{id}", get(api::tenant_admin::sources::show).put(api::tenant_admin::sources::update).delete(api::tenant_admin::sources::destroy))
        .route("/admin/api/settings/types", get(api::tenant_admin::types::list).post(api::tenant_admin::types::store))
        .route("/admin/api/settings/types/{id}", get(api::tenant_admin::types::show).put(api::tenant_admin::types::update).delete(api::tenant_admin::types::destroy))
        // Contact API routes
        .route("/admin/api/contacts/persons", get(api::tenant_admin::persons::list).post(api::tenant_admin::persons::store))
        .route("/admin/api/contacts/persons/{id}", get(api::tenant_admin::persons::show).put(api::tenant_admin::persons::update).delete(api::tenant_admin::persons::destroy))
        .route("/admin/api/contacts/organizations", get(api::tenant_admin::organizations::list).post(api::tenant_admin::organizations::store))
        .route("/admin/api/contacts/organizations/{id}", get(api::tenant_admin::organizations::show).put(api::tenant_admin::organizations::update).delete(api::tenant_admin::organizations::destroy))
        // Quote API routes
        .route("/admin/api/quotes", get(api::tenant_admin::quotes::list).post(api::tenant_admin::quotes::store))
        .route("/admin/api/quotes/{id}", get(api::tenant_admin::quotes::show).put(api::tenant_admin::quotes::update).delete(api::tenant_admin::quotes::destroy))
        // Activity API routes
        .route("/admin/api/activities", get(api::tenant_admin::activities::list).post(api::tenant_admin::activities::store))
        .route("/admin/api/activities/{id}", get(api::tenant_admin::activities::show).put(api::tenant_admin::activities::update).delete(api::tenant_admin::activities::destroy))
        // Product API routes
        .route("/admin/api/products", get(api::tenant_admin::products::list).post(api::tenant_admin::products::store))
        .route("/admin/api/products/{id}", get(api::tenant_admin::products::show).put(api::tenant_admin::products::update).delete(api::tenant_admin::products::destroy))
        // Lead API routes
        .route("/admin/api/leads", get(api::tenant_admin::leads::list).post(api::tenant_admin::leads::store))
        .route("/admin/api/leads/kanban", get(api::tenant_admin::leads::kanban))
        .route("/admin/api/leads/{id}", get(api::tenant_admin::leads::show).put(api::tenant_admin::leads::update).delete(api::tenant_admin::leads::destroy))
        .route("/admin/api/leads/{id}/stage", axum::routing::put(api::tenant_admin::leads::update_stage))
        // Tag API routes
        .route("/admin/api/tags", get(api::tenant_admin::tags::list).post(api::tenant_admin::tags::store))
        .route("/admin/api/tags/{id}", get(api::tenant_admin::tags::show).put(api::tenant_admin::tags::update).delete(api::tenant_admin::tags::destroy))
        .route("/admin/api/tags/attach", post(api::tenant_admin::tags::attach))
        .route("/admin/api/tags/detach", post(api::tenant_admin::tags::detach))
        .layer(middleware::from_fn(require_tenant_admin))
}
