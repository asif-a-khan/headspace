pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod views;

use config::Config;
use db::Database;

use time::Duration;
use tower_sessions::cookie::SameSite;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;
use tracing_subscriber::EnvFilter;

pub async fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("headspace=debug,tower_http=debug")),
        )
        .init();

    let config = Config::from_env()?;
    tracing::info!(
        host = %config.app_host,
        port = config.app_port,
        domain = %config.primary_domain,
        "Configuration loaded"
    );

    let db = Database::connect(&config.database_writer_url, &config.database_reader_url).await?;
    tracing::info!("Database connected");

    db::migrate::run_main_migrations(db.writer()).await?;
    tracing::info!("Main schema migrations applied");

    db::seed::seed_default_super_admin(db.writer()).await?;

    db::migrate::run_all_tenant_migrations(db.writer()).await?;
    tracing::info!("Tenant schema migrations applied");

    db::seed::seed_all_tenant_admins(db.writer()).await?;

    // Session store (PostgreSQL-backed)
    let session_store = PostgresStore::new(db.writer().clone());
    session_store.migrate().await?;
    tracing::info!("Session store ready");

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // TODO: set true in production with HTTPS
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::hours(8)));

    let addr = format!("{}:{}", config.app_host, config.app_port);

    let app = routes::app_router(db, config, session_layer);
    tracing::info!("Starting server on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
