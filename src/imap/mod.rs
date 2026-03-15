//! IMAP inbound email sync.
//!
//! Connects to tenant IMAP mailboxes, fetches new messages, deduplicates by
//! message_id, threads via in_reply_to, and stores emails + attachments.

pub mod parser;
pub mod scheduler;
pub mod sync;

use std::collections::HashMap;
use std::sync::Arc;

use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::rustls::ClientConfig;

/// IMAP connection configuration parsed from tenant_config.
#[derive(Debug, Clone)]
pub struct ImapConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub encryption: String, // "ssl", "tls", "none"
    pub enabled: bool,
}

impl ImapConfig {
    /// Parse IMAP config from a tenant_config HashMap.
    /// Returns None if host or username is empty.
    pub fn from_config_map(config: &HashMap<String, String>) -> Option<Self> {
        let host = config.get("email.imap.host")?.trim().to_string();
        let username = config.get("email.imap.username")?.trim().to_string();

        if host.is_empty() || username.is_empty() {
            return None;
        }

        let port = config
            .get("email.imap.port")
            .and_then(|p| p.parse().ok())
            .unwrap_or(993);
        let password = config
            .get("email.imap.password")
            .cloned()
            .unwrap_or_default();
        let encryption = config
            .get("email.imap.encryption")
            .cloned()
            .unwrap_or_else(|| "ssl".to_string());
        let enabled = config
            .get("email.imap.enabled")
            .map(|v| v == "true")
            .unwrap_or(false);

        Some(Self {
            host,
            port,
            username,
            password,
            encryption,
            enabled,
        })
    }

    /// Check if this config is complete enough to attempt connection.
    pub fn is_configured(&self) -> bool {
        !self.host.is_empty() && !self.username.is_empty() && !self.password.is_empty()
    }
}

/// Error types for IMAP sync operations.
#[derive(Debug, thiserror::Error)]
pub enum ImapSyncError {
    #[error("IMAP connection failed: {0}")]
    Connection(String),

    #[error("IMAP authentication failed: {0}")]
    Auth(String),

    #[error("IMAP protocol error: {0}")]
    Protocol(String),

    #[error("Email parse error: {0}")]
    Parse(String),

    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Build a TLS connector using Mozilla root CAs.
fn build_tls_connector() -> TlsConnector {
    let mut root_store = tokio_rustls::rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let tls_config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    TlsConnector::from(Arc::new(tls_config))
}

/// Connect to an IMAP server with SSL (direct TLS) and login.
pub async fn connect_ssl(
    config: &ImapConfig,
) -> Result<async_imap::Session<tokio_rustls::client::TlsStream<TcpStream>>, ImapSyncError> {
    let addr = format!("{}:{}", config.host, config.port);
    let tcp = TcpStream::connect(&addr)
        .await
        .map_err(|e| ImapSyncError::Connection(format!("{addr}: {e}")))?;

    let tls_connector = build_tls_connector();
    let server_name = rustls_pki_types::ServerName::try_from(config.host.clone())
        .map_err(|e| ImapSyncError::Connection(format!("Invalid hostname: {e}")))?;
    let tls_stream = tls_connector
        .connect(server_name, tcp)
        .await
        .map_err(|e| ImapSyncError::Connection(format!("TLS handshake: {e}")))?;

    let client = async_imap::Client::new(tls_stream);
    let session = client
        .login(&config.username, &config.password)
        .await
        .map_err(|e| ImapSyncError::Auth(format!("{}", e.0)))?;

    Ok(session)
}

/// Connect to an IMAP server with STARTTLS and login.
pub async fn connect_starttls(
    config: &ImapConfig,
) -> Result<async_imap::Session<tokio_rustls::client::TlsStream<TcpStream>>, ImapSyncError> {
    let addr = format!("{}:{}", config.host, config.port);
    let tcp = TcpStream::connect(&addr)
        .await
        .map_err(|e| ImapSyncError::Connection(format!("{addr}: {e}")))?;

    let mut client = async_imap::Client::new(tcp);
    // Issue STARTTLS command
    client
        .run_command_and_check_ok("STARTTLS", None)
        .await
        .map_err(|e| ImapSyncError::Connection(format!("STARTTLS command: {e}")))?;

    // Upgrade the underlying stream to TLS
    let inner = client.into_inner();
    let tls_connector = build_tls_connector();
    let server_name = rustls_pki_types::ServerName::try_from(config.host.clone())
        .map_err(|e| ImapSyncError::Connection(format!("Invalid hostname: {e}")))?;
    let tls_stream = tls_connector
        .connect(server_name, inner)
        .await
        .map_err(|e| ImapSyncError::Connection(format!("STARTTLS TLS upgrade: {e}")))?;

    // Create a new client over the TLS stream (no greeting expected)
    let client = async_imap::Client::new(tls_stream);
    let session = client
        .login(&config.username, &config.password)
        .await
        .map_err(|e| ImapSyncError::Auth(format!("{}", e.0)))?;

    Ok(session)
}

/// Connect to an IMAP server without encryption and login.
pub async fn connect_plain(
    config: &ImapConfig,
) -> Result<async_imap::Session<TcpStream>, ImapSyncError> {
    let addr = format!("{}:{}", config.host, config.port);
    let tcp = TcpStream::connect(&addr)
        .await
        .map_err(|e| ImapSyncError::Connection(format!("{addr}: {e}")))?;

    let client = async_imap::Client::new(tcp);
    let session = client
        .login(&config.username, &config.password)
        .await
        .map_err(|e| ImapSyncError::Auth(format!("{}", e.0)))?;

    Ok(session)
}
