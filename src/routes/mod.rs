//! Route definitions.
//!
//! All application routes are defined here, grouped by middleware requirements.

use axum::{routing::get, Router};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::handlers;

/// Build the application router with all route groups.
pub fn app_router() -> Router {
    // Tenant auth routes (login/logout) — no auth middleware
    // let auth_routes = Router::new()
    //     .route("/login", get(handlers::auth::login_form).post(handlers::auth::login))
    //     .route("/logout", post(handlers::auth::logout));

    // Tenant admin routes — requires tenant + auth middleware
    // let admin_routes = Router::new()
    //     .route("/dashboard", get(handlers::dashboard::index))
    //     .route("/leads", get(handlers::leads::index))
    //     ...
    //     .layer(middleware::from_fn(middleware::auth::require_auth));

    // Super admin routes — requires super admin auth middleware
    // let super_admin_routes = Router::new()
    //     .route("/login", get(handlers::super_admin::auth::login_form))
    //     ...

    Router::new()
        .route("/health", get(handlers::health::health_check))
        // .merge(auth_routes)
        // .nest("/admin", admin_routes)
        //     .layer(middleware::from_fn(middleware::tenant::resolve_tenant))
        // .nest("/super", super_admin_routes)
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
}
