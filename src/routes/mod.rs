//! Route definitions.
//!
//! All application routes are defined here, grouped by middleware requirements.

use axum::routing::{get, post, put};
use axum::{Extension, Router, middleware};
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
        .route(
            "/super/tenants/create",
            get(handlers::super_admin::tenants::create),
        )
        .route(
            "/super/tenants/{id}/edit",
            get(handlers::super_admin::tenants::edit),
        )
        .route(
            "/super/settings",
            get(handlers::super_admin::settings::index),
        )
        .route(
            "/super/settings/agents",
            get(handlers::super_admin::agents::index),
        )
        .route(
            "/super/settings/agents/create",
            get(handlers::super_admin::agents::create),
        )
        .route(
            "/super/settings/agents/{id}/edit",
            get(handlers::super_admin::agents::edit),
        )
        .route(
            "/super/settings/roles",
            get(handlers::super_admin::roles::index),
        )
        .route(
            "/super/settings/roles/create",
            get(handlers::super_admin::roles::create),
        )
        .route(
            "/super/settings/roles/{id}/edit",
            get(handlers::super_admin::roles::edit),
        )
        .route("/super/account", get(handlers::super_admin::account::edit))
        // JSON API routes
        .route(
            "/super/api/tenants",
            get(api::super_admin::tenants::list).post(api::super_admin::tenants::store),
        )
        .route(
            "/super/api/tenants/{id}",
            get(api::super_admin::tenants::show)
                .put(api::super_admin::tenants::update)
                .delete(api::super_admin::tenants::destroy),
        )
        .route(
            "/super/api/agents",
            get(api::super_admin::agents::list).post(api::super_admin::agents::store),
        )
        .route(
            "/super/api/agents/{id}",
            get(api::super_admin::agents::show)
                .put(api::super_admin::agents::update)
                .delete(api::super_admin::agents::destroy),
        )
        .route(
            "/super/api/roles",
            get(api::super_admin::roles::list).post(api::super_admin::roles::store),
        )
        .route(
            "/super/api/roles/{id}",
            get(api::super_admin::roles::show)
                .put(api::super_admin::roles::update)
                .delete(api::super_admin::roles::destroy),
        )
        .route(
            "/super/api/account",
            get(api::super_admin::account::show).put(api::super_admin::account::update),
        )
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
        .route(
            "/admin/login",
            get(handlers::tenant_admin::auth::login_page),
        )
        .route("/admin/api/login", post(api::tenant_admin::auth::login))
        // Public web forms (tenant resolution only, no auth)
        .route(
            "/web-forms/{form_id}",
            get(handlers::public::web_forms::render_form),
        )
        .route(
            "/web-forms/{form_id}/submit",
            post(api::public::web_forms::submit),
        )
        // Protected routes (tenant resolution + auth)
        .merge(tenant_admin_protected_routes())
        // Tenant resolution middleware for ALL /admin/* routes
        .layer(middleware::from_fn(require_tenant))
}

/// Tenant admin routes that require authentication.
fn tenant_admin_protected_routes() -> Router {
    Router::new()
        .route("/admin/logout", post(handlers::tenant_admin::auth::logout))
        .route(
            "/admin/dashboard",
            get(handlers::tenant_admin::dashboard::index),
        )
        // Account page
        .route("/admin/account", get(handlers::tenant_admin::account::edit))
        // Settings pages
        .route(
            "/admin/settings",
            get(handlers::tenant_admin::settings::index),
        )
        .route(
            "/admin/settings/users",
            get(handlers::tenant_admin::settings::users_index),
        )
        .route(
            "/admin/settings/users/create",
            get(handlers::tenant_admin::settings::users_create),
        )
        .route(
            "/admin/settings/users/{id}/edit",
            get(handlers::tenant_admin::settings::users_edit),
        )
        .route(
            "/admin/settings/roles",
            get(handlers::tenant_admin::settings::roles_index),
        )
        .route(
            "/admin/settings/roles/create",
            get(handlers::tenant_admin::settings::roles_create),
        )
        .route(
            "/admin/settings/roles/{id}/edit",
            get(handlers::tenant_admin::settings::roles_edit),
        )
        .route(
            "/admin/settings/groups",
            get(handlers::tenant_admin::settings::groups_index),
        )
        .route(
            "/admin/settings/groups/create",
            get(handlers::tenant_admin::settings::groups_create),
        )
        .route(
            "/admin/settings/groups/{id}/edit",
            get(handlers::tenant_admin::settings::groups_edit),
        )
        .route(
            "/admin/settings/attributes",
            get(handlers::tenant_admin::attributes::index),
        )
        .route(
            "/admin/settings/attributes/create",
            get(handlers::tenant_admin::attributes::create),
        )
        .route(
            "/admin/settings/attributes/{id}/edit",
            get(handlers::tenant_admin::attributes::edit),
        )
        .route(
            "/admin/settings/pipelines",
            get(handlers::tenant_admin::pipelines::pipelines_index),
        )
        .route(
            "/admin/settings/pipelines/create",
            get(handlers::tenant_admin::pipelines::pipelines_create),
        )
        .route(
            "/admin/settings/pipelines/{id}/edit",
            get(handlers::tenant_admin::pipelines::pipelines_edit),
        )
        .route(
            "/admin/settings/sources",
            get(handlers::tenant_admin::pipelines::sources_index),
        )
        .route(
            "/admin/settings/types",
            get(handlers::tenant_admin::pipelines::types_index),
        )
        .route(
            "/admin/settings/tags",
            get(handlers::tenant_admin::settings::tags_index),
        )
        .route(
            "/admin/settings/configuration",
            get(handlers::tenant_admin::settings::configuration_index),
        )
        // Dashboard API
        .route(
            "/admin/api/dashboard/stats",
            get(api::tenant_admin::dashboard::stats),
        )
        // Global search
        .route(
            "/admin/api/search",
            get(api::tenant_admin::search::global_search),
        )
        // Quote pages
        .route("/admin/quotes", get(handlers::tenant_admin::quotes::index))
        .route(
            "/admin/quotes/create",
            get(handlers::tenant_admin::quotes::create),
        )
        .route(
            "/admin/quotes/{id}/edit",
            get(handlers::tenant_admin::quotes::edit),
        )
        // Activity pages
        .route(
            "/admin/activities",
            get(handlers::tenant_admin::activities::index),
        )
        .route(
            "/admin/activities/create",
            get(handlers::tenant_admin::activities::create),
        )
        .route(
            "/admin/activities/{id}/edit",
            get(handlers::tenant_admin::activities::edit),
        )
        // Product pages
        .route(
            "/admin/products",
            get(handlers::tenant_admin::products::index),
        )
        .route(
            "/admin/products/create",
            get(handlers::tenant_admin::products::create),
        )
        .route(
            "/admin/products/{id}/edit",
            get(handlers::tenant_admin::products::edit),
        )
        // Lead pages
        .route(
            "/admin/leads",
            get(handlers::tenant_admin::leads::kanban_page),
        )
        .route(
            "/admin/leads/list",
            get(handlers::tenant_admin::leads::index),
        )
        .route(
            "/admin/leads/create",
            get(handlers::tenant_admin::leads::create),
        )
        .route(
            "/admin/leads/{id}",
            get(handlers::tenant_admin::leads::show),
        )
        .route(
            "/admin/leads/{id}/edit",
            get(handlers::tenant_admin::leads::edit),
        )
        // Mail page
        .route("/admin/mail", get(handlers::tenant_admin::emails::index))
        // Contact pages
        .route(
            "/admin/contacts/persons",
            get(handlers::tenant_admin::contacts::persons_index),
        )
        .route(
            "/admin/contacts/persons/create",
            get(handlers::tenant_admin::contacts::persons_create),
        )
        .route(
            "/admin/contacts/persons/{id}",
            get(handlers::tenant_admin::contacts::persons_show),
        )
        .route(
            "/admin/contacts/persons/{id}/edit",
            get(handlers::tenant_admin::contacts::persons_edit),
        )
        .route(
            "/admin/contacts/organizations",
            get(handlers::tenant_admin::contacts::organizations_index),
        )
        .route(
            "/admin/contacts/organizations/create",
            get(handlers::tenant_admin::contacts::organizations_create),
        )
        .route(
            "/admin/contacts/organizations/{id}",
            get(handlers::tenant_admin::contacts::organizations_show),
        )
        .route(
            "/admin/contacts/organizations/{id}/edit",
            get(handlers::tenant_admin::contacts::organizations_edit),
        )
        // Account API routes
        .route(
            "/admin/api/account",
            get(api::tenant_admin::account::show).put(api::tenant_admin::account::update),
        )
        .route(
            "/admin/api/account/password",
            put(api::tenant_admin::account::update_password),
        )
        // Settings API routes
        .route(
            "/admin/api/settings/users",
            get(api::tenant_admin::users::list).post(api::tenant_admin::users::store),
        )
        .route(
            "/admin/api/settings/users/{id}",
            get(api::tenant_admin::users::show)
                .put(api::tenant_admin::users::update)
                .delete(api::tenant_admin::users::destroy),
        )
        .route(
            "/admin/api/settings/roles",
            get(api::tenant_admin::roles::list).post(api::tenant_admin::roles::store),
        )
        .route(
            "/admin/api/settings/roles/{id}",
            get(api::tenant_admin::roles::show)
                .put(api::tenant_admin::roles::update)
                .delete(api::tenant_admin::roles::destroy),
        )
        .route(
            "/admin/api/settings/groups",
            get(api::tenant_admin::groups::list).post(api::tenant_admin::groups::store),
        )
        .route(
            "/admin/api/settings/groups/{id}",
            get(api::tenant_admin::groups::show)
                .put(api::tenant_admin::groups::update)
                .delete(api::tenant_admin::groups::destroy),
        )
        .route(
            "/admin/api/settings/attributes",
            get(api::tenant_admin::attributes::list).post(api::tenant_admin::attributes::store),
        )
        .route(
            "/admin/api/settings/attributes/{id}",
            get(api::tenant_admin::attributes::show)
                .put(api::tenant_admin::attributes::update)
                .delete(api::tenant_admin::attributes::destroy),
        )
        .route(
            "/admin/api/settings/attributes/{id}/options",
            get(api::tenant_admin::attributes::list_options),
        )
        .route(
            "/admin/api/settings/pipelines",
            get(api::tenant_admin::pipelines::list).post(api::tenant_admin::pipelines::store),
        )
        .route(
            "/admin/api/settings/pipelines/{id}",
            get(api::tenant_admin::pipelines::show)
                .put(api::tenant_admin::pipelines::update)
                .delete(api::tenant_admin::pipelines::destroy),
        )
        .route(
            "/admin/api/settings/sources",
            get(api::tenant_admin::sources::list).post(api::tenant_admin::sources::store),
        )
        .route(
            "/admin/api/settings/sources/{id}",
            get(api::tenant_admin::sources::show)
                .put(api::tenant_admin::sources::update)
                .delete(api::tenant_admin::sources::destroy),
        )
        .route(
            "/admin/api/settings/types",
            get(api::tenant_admin::types::list).post(api::tenant_admin::types::store),
        )
        .route(
            "/admin/api/settings/types/{id}",
            get(api::tenant_admin::types::show)
                .put(api::tenant_admin::types::update)
                .delete(api::tenant_admin::types::destroy),
        )
        // Email API routes
        .route(
            "/admin/api/emails",
            get(api::tenant_admin::emails::list).post(api::tenant_admin::emails::store),
        )
        .route(
            "/admin/api/emails/sync",
            post(api::tenant_admin::emails::trigger_sync),
        )
        .route(
            "/admin/api/emails/{id}",
            get(api::tenant_admin::emails::show)
                .put(api::tenant_admin::emails::update)
                .delete(api::tenant_admin::emails::destroy),
        )
        // Configuration API routes
        .route(
            "/admin/api/settings/config",
            get(api::tenant_admin::config::list).put(api::tenant_admin::config::update),
        )
        // Contact API routes
        .route(
            "/admin/api/contacts/persons",
            get(api::tenant_admin::persons::list).post(api::tenant_admin::persons::store),
        )
        .route(
            "/admin/api/contacts/persons/search",
            get(api::tenant_admin::persons::search),
        )
        .route(
            "/admin/api/contacts/persons/{id}",
            get(api::tenant_admin::persons::show)
                .put(api::tenant_admin::persons::update)
                .delete(api::tenant_admin::persons::destroy),
        )
        .route(
            "/admin/api/contacts/organizations",
            get(api::tenant_admin::organizations::list)
                .post(api::tenant_admin::organizations::store),
        )
        .route(
            "/admin/api/contacts/organizations/{id}",
            get(api::tenant_admin::organizations::show)
                .put(api::tenant_admin::organizations::update)
                .delete(api::tenant_admin::organizations::destroy),
        )
        // Quote API routes
        .route(
            "/admin/api/quotes",
            get(api::tenant_admin::quotes::list).post(api::tenant_admin::quotes::store),
        )
        .route(
            "/admin/api/quotes/search",
            get(api::tenant_admin::quotes::search),
        )
        .route(
            "/admin/api/quotes/{id}",
            get(api::tenant_admin::quotes::show)
                .put(api::tenant_admin::quotes::update)
                .delete(api::tenant_admin::quotes::destroy),
        )
        .route(
            "/admin/api/quotes/{id}/pdf",
            get(api::tenant_admin::quotes::download_pdf),
        )
        // Activity API routes
        .route(
            "/admin/api/activities",
            get(api::tenant_admin::activities::list).post(api::tenant_admin::activities::store),
        )
        .route(
            "/admin/api/activities/{id}",
            get(api::tenant_admin::activities::show)
                .put(api::tenant_admin::activities::update)
                .delete(api::tenant_admin::activities::destroy),
        )
        .route(
            "/admin/api/activities/{id}/files",
            get(api::tenant_admin::activities::list_files)
                .post(api::tenant_admin::activities::upload_file),
        )
        .route(
            "/admin/api/activities/{id}/files/{file_id}",
            get(api::tenant_admin::activities::download_file)
                .delete(api::tenant_admin::activities::delete_file),
        )
        // Product API routes
        .route(
            "/admin/api/products",
            get(api::tenant_admin::products::list).post(api::tenant_admin::products::store),
        )
        .route(
            "/admin/api/products/search",
            get(api::tenant_admin::products::search),
        )
        .route(
            "/admin/api/products/{id}",
            get(api::tenant_admin::products::show)
                .put(api::tenant_admin::products::update)
                .delete(api::tenant_admin::products::destroy),
        )
        // Lead API routes
        .route(
            "/admin/api/leads",
            get(api::tenant_admin::leads::list).post(api::tenant_admin::leads::store),
        )
        .route(
            "/admin/api/leads/search",
            get(api::tenant_admin::leads::search),
        )
        .route(
            "/admin/api/leads/kanban",
            get(api::tenant_admin::leads::kanban),
        )
        .route(
            "/admin/api/leads/{id}",
            get(api::tenant_admin::leads::show)
                .put(api::tenant_admin::leads::update)
                .delete(api::tenant_admin::leads::destroy),
        )
        .route(
            "/admin/api/leads/{id}/stage",
            axum::routing::put(api::tenant_admin::leads::update_stage),
        )
        .route(
            "/admin/api/leads/{id}/status",
            axum::routing::put(api::tenant_admin::leads::update_status),
        )
        .route(
            "/admin/api/leads/{id}/products",
            get(api::tenant_admin::leads::list_products)
                .post(api::tenant_admin::leads::add_product),
        )
        .route(
            "/admin/api/leads/{id}/products/{product_line_id}",
            axum::routing::delete(api::tenant_admin::leads::remove_product),
        )
        .route(
            "/admin/api/leads/{id}/quotes",
            get(api::tenant_admin::leads::list_quotes).post(api::tenant_admin::leads::link_quote),
        )
        .route(
            "/admin/api/leads/{id}/quotes/{quote_id}",
            axum::routing::delete(api::tenant_admin::leads::unlink_quote),
        )
        // Tag API routes
        .route(
            "/admin/api/tags",
            get(api::tenant_admin::tags::list).post(api::tenant_admin::tags::store),
        )
        .route(
            "/admin/api/tags/{id}",
            get(api::tenant_admin::tags::show)
                .put(api::tenant_admin::tags::update)
                .delete(api::tenant_admin::tags::destroy),
        )
        .route(
            "/admin/api/tags/attach",
            post(api::tenant_admin::tags::attach),
        )
        .route(
            "/admin/api/tags/detach",
            post(api::tenant_admin::tags::detach),
        )
        // Mass delete routes
        .route(
            "/admin/api/leads/mass-delete",
            post(api::tenant_admin::leads::mass_delete),
        )
        .route(
            "/admin/api/contacts/persons/mass-delete",
            post(api::tenant_admin::persons::mass_delete),
        )
        .route(
            "/admin/api/contacts/organizations/mass-delete",
            post(api::tenant_admin::organizations::mass_delete),
        )
        .route(
            "/admin/api/products/mass-delete",
            post(api::tenant_admin::products::mass_delete),
        )
        .route(
            "/admin/api/quotes/mass-delete",
            post(api::tenant_admin::quotes::mass_delete),
        )
        .route(
            "/admin/api/activities/mass-delete",
            post(api::tenant_admin::activities::mass_delete),
        )
        // Data transfer (CSV import/export)
        .route(
            "/admin/api/leads/export",
            get(api::tenant_admin::data_transfer::export_leads),
        )
        .route(
            "/admin/api/leads/import",
            post(api::tenant_admin::data_transfer::import_leads),
        )
        .route(
            "/admin/api/contacts/persons/export",
            get(api::tenant_admin::data_transfer::export_persons),
        )
        .route(
            "/admin/api/contacts/persons/import",
            post(api::tenant_admin::data_transfer::import_persons),
        )
        .route(
            "/admin/api/contacts/organizations/export",
            get(api::tenant_admin::data_transfer::export_organizations),
        )
        .route(
            "/admin/api/contacts/organizations/import",
            post(api::tenant_admin::data_transfer::import_organizations),
        )
        .route(
            "/admin/api/products/export",
            get(api::tenant_admin::data_transfer::export_products),
        )
        .route(
            "/admin/api/products/import",
            post(api::tenant_admin::data_transfer::import_products),
        )
        // Email Templates
        .route(
            "/admin/settings/email-templates",
            get(handlers::tenant_admin::email_templates::index),
        )
        .route(
            "/admin/settings/email-templates/create",
            get(handlers::tenant_admin::email_templates::create),
        )
        .route(
            "/admin/settings/email-templates/{id}/edit",
            get(handlers::tenant_admin::email_templates::edit),
        )
        .route(
            "/admin/api/settings/email-templates",
            get(api::tenant_admin::email_templates::list)
                .post(api::tenant_admin::email_templates::store),
        )
        .route(
            "/admin/api/settings/email-templates/{id}",
            get(api::tenant_admin::email_templates::show)
                .put(api::tenant_admin::email_templates::update)
                .delete(api::tenant_admin::email_templates::destroy),
        )
        // Warehouses
        .route(
            "/admin/settings/warehouses",
            get(handlers::tenant_admin::warehouses::index),
        )
        .route(
            "/admin/settings/warehouses/create",
            get(handlers::tenant_admin::warehouses::create),
        )
        .route(
            "/admin/settings/warehouses/{id}/edit",
            get(handlers::tenant_admin::warehouses::edit),
        )
        .route(
            "/admin/api/settings/warehouses",
            get(api::tenant_admin::warehouses::list).post(api::tenant_admin::warehouses::store),
        )
        .route(
            "/admin/api/settings/warehouses/{id}",
            get(api::tenant_admin::warehouses::show)
                .put(api::tenant_admin::warehouses::update)
                .delete(api::tenant_admin::warehouses::destroy),
        )
        // Web Forms
        .route(
            "/admin/settings/web-forms",
            get(handlers::tenant_admin::web_forms::index),
        )
        .route(
            "/admin/settings/web-forms/create",
            get(handlers::tenant_admin::web_forms::create),
        )
        .route(
            "/admin/settings/web-forms/{id}/edit",
            get(handlers::tenant_admin::web_forms::edit),
        )
        .route(
            "/admin/api/settings/web-forms",
            get(api::tenant_admin::web_forms::list).post(api::tenant_admin::web_forms::store),
        )
        .route(
            "/admin/api/settings/web-forms/{id}",
            get(api::tenant_admin::web_forms::show)
                .put(api::tenant_admin::web_forms::update)
                .delete(api::tenant_admin::web_forms::destroy),
        )
        .layer(middleware::from_fn(require_tenant_admin))
}
