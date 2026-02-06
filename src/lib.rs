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

    let _db = Database::connect(&config.database_writer_url, &config.database_reader_url).await?;
    tracing::info!("Database connected");

    let app = routes::app_router();

    let addr = format!("{}:{}", config.app_host, config.app_port);
    tracing::info!("Starting server on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
