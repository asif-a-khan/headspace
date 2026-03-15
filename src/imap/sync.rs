//! IMAP sync logic: fetch messages from IMAP folders, dedup, thread, and store.

use std::collections::HashMap;

use async_imap::types::Fetch;
use futures_util::StreamExt;
use sqlx::PgConnection;

use super::parser::ParsedAttachment;
use super::{ImapConfig, ImapSyncError};

/// Stats returned after syncing a tenant.
#[derive(Debug, Default)]
pub struct SyncStats {
    pub folders_synced: u32,
    pub messages_fetched: u32,
    pub messages_inserted: u32,
    pub messages_skipped: u32,
    pub attachments_saved: u32,
    pub errors: u32,
}

/// Map an IMAP folder name to our internal folder name.
/// Returns None if the folder should be skipped.
fn map_folder(imap_name: &str) -> Option<&'static str> {
    let lower = imap_name.to_lowercase();
    if lower == "inbox" {
        Some("inbox")
    } else if lower.contains("sent") {
        Some("sent")
    } else if lower.contains("draft") {
        Some("draft")
    } else if lower.contains("trash") || lower.contains("deleted") {
        Some("trash")
    } else {
        None
    }
}

/// Sync a single tenant's IMAP mailbox.
///
/// Connects to IMAP, lists folders, fetches recent messages, and stores them.
/// `conn` must already have the tenant's search_path set.
pub async fn sync_tenant(
    conn: &mut PgConnection,
    config: &ImapConfig,
    schema_name: &str,
) -> Result<SyncStats, ImapSyncError> {
    let mut stats = SyncStats::default();

    // Connect based on encryption type
    match config.encryption.as_str() {
        "ssl" => {
            let mut session = super::connect_ssl(config).await?;
            sync_with_session(&mut session, conn, schema_name, &mut stats).await?;
            let _ = session.logout().await;
        }
        "tls" => {
            let mut session = super::connect_starttls(config).await?;
            sync_with_session(&mut session, conn, schema_name, &mut stats).await?;
            let _ = session.logout().await;
        }
        _ => {
            let mut session = super::connect_plain(config).await?;
            sync_with_session(&mut session, conn, schema_name, &mut stats).await?;
            let _ = session.logout().await;
        }
    }

    // Update last_sync_at timestamp
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO tenant_config (code, value) VALUES ('email.imap.last_sync_at', $1)
         ON CONFLICT (code) DO UPDATE SET value = $1",
    )
    .bind(&now)
    .execute(&mut *conn)
    .await
    .map_err(ImapSyncError::Db)?;

    Ok(stats)
}

/// The actual sync logic, generic over the session stream type.
async fn sync_with_session<T>(
    session: &mut async_imap::Session<T>,
    conn: &mut PgConnection,
    schema_name: &str,
    stats: &mut SyncStats,
) -> Result<(), ImapSyncError>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + std::fmt::Debug + Send,
{
    // List all IMAP folders — collect the stream into a Vec
    let folder_stream = session
        .list(Some(""), Some("*"))
        .await
        .map_err(|e| ImapSyncError::Protocol(format!("LIST: {e}")))?;

    let folders: Vec<_> = folder_stream
        .filter_map(|r| async { r.ok() })
        .collect()
        .await;

    for folder in &folders {
        let imap_name = folder.name();
        let our_folder = match map_folder(imap_name) {
            Some(f) => f,
            None => continue,
        };

        if let Err(e) = sync_folder(session, conn, imap_name, our_folder, schema_name, stats).await
        {
            tracing::warn!(
                folder = imap_name,
                schema = schema_name,
                error = %e,
                "Failed to sync IMAP folder"
            );
            stats.errors += 1;
        }
    }

    Ok(())
}

/// Sync a single IMAP folder.
async fn sync_folder<T>(
    session: &mut async_imap::Session<T>,
    conn: &mut PgConnection,
    imap_folder: &str,
    our_folder: &str,
    schema_name: &str,
    stats: &mut SyncStats,
) -> Result<(), ImapSyncError>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + std::fmt::Debug + Send,
{
    // SELECT the folder as read-only (EXAMINE) to avoid marking messages as read
    session
        .examine(imap_folder)
        .await
        .map_err(|e| ImapSyncError::Protocol(format!("EXAMINE {imap_folder}: {e}")))?;

    // Search for messages from the last 10 days
    let since_date = (chrono::Utc::now() - chrono::Duration::days(10))
        .format("%d-%b-%Y")
        .to_string();
    let search_query = format!("SINCE {since_date}");

    let uids = session
        .uid_search(&search_query)
        .await
        .map_err(|e| ImapSyncError::Protocol(format!("UID SEARCH: {e}")))?;

    if uids.is_empty() {
        stats.folders_synced += 1;
        return Ok(());
    }

    // Build UID set string (e.g. "1,2,3,4")
    let uid_set: String = uids
        .iter()
        .map(|u| u.to_string())
        .collect::<Vec<_>>()
        .join(",");

    // FETCH with BODY.PEEK[] to avoid setting \Seen flag
    let fetch_query = "(UID FLAGS BODY.PEEK[])";
    let message_stream = session
        .uid_fetch(&uid_set, fetch_query)
        .await
        .map_err(|e| ImapSyncError::Protocol(format!("UID FETCH: {e}")))?;

    let messages: Vec<_> = message_stream
        .filter_map(|r| async { r.ok() })
        .collect()
        .await;

    for message in &messages {
        stats.messages_fetched += 1;

        if let Err(e) = process_message(message, conn, our_folder, schema_name, stats).await {
            tracing::debug!(
                uid = ?message.uid,
                folder = imap_folder,
                error = %e,
                "Failed to process IMAP message"
            );
            stats.errors += 1;
        }
    }

    stats.folders_synced += 1;
    Ok(())
}

/// Process a single fetched IMAP message: parse, dedup, thread, store.
async fn process_message(
    fetch: &Fetch,
    conn: &mut PgConnection,
    our_folder: &str,
    schema_name: &str,
    stats: &mut SyncStats,
) -> Result<(), ImapSyncError> {
    let uid = fetch
        .uid
        .ok_or_else(|| ImapSyncError::Protocol("Message missing UID".to_string()))?;

    let body = fetch
        .body()
        .ok_or_else(|| ImapSyncError::Parse(format!("UID {uid}: no body")))?;

    // Check \Seen flag
    let is_seen = fetch
        .flags()
        .any(|f| matches!(f, async_imap::types::Flag::Seen));

    let parsed = super::parser::parse_rfc822(body, is_seen)
        .ok_or_else(|| ImapSyncError::Parse(format!("UID {uid}: parse failed")))?;

    // Dedup by message_id — if we already have this message, skip
    if let Some(ref msg_id) = parsed.message_id {
        let existing: Option<(i64,)> =
            sqlx::query_as("SELECT id FROM emails WHERE message_id = $1 LIMIT 1")
                .bind(msg_id)
                .fetch_optional(&mut *conn)
                .await
                .map_err(ImapSyncError::Db)?;

        if existing.is_some() {
            stats.messages_skipped += 1;
            return Ok(());
        }
    }

    // Also dedup by imap_uid for messages without message_id
    if parsed.message_id.is_none() {
        let existing: Option<(i64,)> =
            sqlx::query_as("SELECT id FROM emails WHERE imap_uid = $1 AND folder = $2 LIMIT 1")
                .bind(uid as i64)
                .bind(our_folder)
                .fetch_optional(&mut *conn)
                .await
                .map_err(ImapSyncError::Db)?;

        if existing.is_some() {
            stats.messages_skipped += 1;
            return Ok(());
        }
    }

    // Threading: find parent by in_reply_to
    let parent_id: Option<i64> = if let Some(ref irt) = parsed.in_reply_to {
        sqlx::query_as::<_, (i64,)>("SELECT id FROM emails WHERE message_id = $1 LIMIT 1")
            .bind(irt)
            .fetch_optional(&mut *conn)
            .await
            .map_err(ImapSyncError::Db)?
            .map(|row| row.0)
    } else {
        None
    };

    // Build body — prefer HTML, fall back to text wrapped in <pre>
    let body_content = parsed
        .body_html
        .clone()
        .or_else(|| {
            parsed.body_text.as_ref().map(|t| {
                format!(
                    "<pre style=\"white-space: pre-wrap;\">{}</pre>",
                    html_escape(t)
                )
            })
        })
        .unwrap_or_default();

    let to_json = serde_json::to_value(&parsed.to).unwrap_or_default();
    let cc_json = serde_json::to_value(&parsed.cc).unwrap_or_default();
    let bcc_json = serde_json::to_value(&parsed.bcc).unwrap_or_default();
    let created_at = parsed.date.unwrap_or_else(chrono::Utc::now);

    // Insert email
    let email_id: (i64,) = sqlx::query_as(
        "INSERT INTO emails (subject, body, from_address, from_name,
            reply_to, cc, bcc, folder, is_read, source, message_id,
            in_reply_to, parent_id, imap_uid, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'imap', $10, $11, $12, $13, $14, NOW())
         RETURNING id",
    )
    .bind(&parsed.subject)
    .bind(&body_content)
    .bind(&parsed.from_address)
    .bind(&parsed.from_name)
    .bind(&to_json)
    .bind(&cc_json)
    .bind(&bcc_json)
    .bind(our_folder)
    .bind(parsed.is_read)
    .bind(&parsed.message_id)
    .bind(&parsed.in_reply_to)
    .bind(parent_id)
    .bind(uid as i64)
    .bind(created_at)
    .fetch_one(&mut *conn)
    .await
    .map_err(ImapSyncError::Db)?;

    stats.messages_inserted += 1;

    // Save attachments
    for attachment in &parsed.attachments {
        if let Err(e) = save_attachment(email_id.0, attachment, schema_name, conn).await {
            tracing::warn!(
                email_id = email_id.0,
                file = %attachment.file_name,
                error = %e,
                "Failed to save email attachment"
            );
            // Continue — email is already saved
        } else {
            stats.attachments_saved += 1;
        }
    }

    Ok(())
}

/// Save an email attachment to disk and record in email_attachments table.
async fn save_attachment(
    email_id: i64,
    attachment: &ParsedAttachment,
    schema_name: &str,
    conn: &mut PgConnection,
) -> Result<(), ImapSyncError> {
    let dir = format!("uploads/{schema_name}/emails");
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(ImapSyncError::Io)?;

    let safe_name = attachment.file_name.replace(['/', '\\', '\0'], "_");
    let file_name = format!("{}_{safe_name}", uuid::Uuid::new_v4());
    let file_path = format!("{dir}/{file_name}");

    tokio::fs::write(&file_path, &attachment.data)
        .await
        .map_err(ImapSyncError::Io)?;

    let file_size = attachment.data.len() as i64;
    sqlx::query(
        "INSERT INTO email_attachments (email_id, file_name, file_path, file_type, file_size)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(email_id)
    .bind(&attachment.file_name)
    .bind(&file_path)
    .bind(&attachment.content_type)
    .bind(file_size)
    .fetch_optional(&mut *conn)
    .await
    .map_err(ImapSyncError::Db)?;

    Ok(())
}

/// Sync all tenants that have IMAP enabled.
pub async fn sync_all_tenants(pool: &sqlx::PgPool) {
    // Get all active tenants
    let tenants = match sqlx::query_as::<_, crate::models::company::Company>(
        "SELECT * FROM main.companies WHERE is_active = true",
    )
    .fetch_all(pool)
    .await
    {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("IMAP sync: failed to list tenants: {e}");
            return;
        }
    };

    for tenant in &tenants {
        // Acquire a detached connection for this tenant (dropped after use)
        let pool_conn = match pool.acquire().await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(
                    schema = %tenant.schema_name,
                    error = %e,
                    "IMAP sync: failed to acquire connection"
                );
                continue;
            }
        };
        let mut conn = pool_conn.detach();

        // Set tenant search_path
        let set_sql = format!("SET search_path TO {}, public", tenant.schema_name);
        if let Err(e) = sqlx::query(&set_sql).execute(&mut conn).await {
            tracing::error!(
                schema = %tenant.schema_name,
                error = %e,
                "IMAP sync: failed to set search_path"
            );
            continue;
        }

        // Load tenant config
        let config_rows: Vec<(String, String)> = match sqlx::query_as(
            "SELECT code, value FROM tenant_config WHERE code LIKE 'email.imap.%'",
        )
        .fetch_all(&mut conn)
        .await
        {
            Ok(rows) => rows,
            Err(e) => {
                tracing::debug!(
                    schema = %tenant.schema_name,
                    error = %e,
                    "IMAP sync: failed to load config"
                );
                continue;
            }
        };

        let config_map: HashMap<String, String> = config_rows.into_iter().collect();

        let imap_config = match ImapConfig::from_config_map(&config_map) {
            Some(c) if c.enabled && c.is_configured() => c,
            _ => continue, // IMAP not configured or not enabled
        };

        tracing::debug!(
            schema = %tenant.schema_name,
            host = %imap_config.host,
            "IMAP sync: starting for tenant"
        );

        match sync_tenant(&mut conn, &imap_config, &tenant.schema_name).await {
            Ok(stats) => {
                tracing::info!(
                    schema = %tenant.schema_name,
                    folders = stats.folders_synced,
                    fetched = stats.messages_fetched,
                    inserted = stats.messages_inserted,
                    skipped = stats.messages_skipped,
                    attachments = stats.attachments_saved,
                    errors = stats.errors,
                    "IMAP sync completed"
                );
            }
            Err(e) => {
                tracing::warn!(
                    schema = %tenant.schema_name,
                    error = %e,
                    "IMAP sync failed for tenant"
                );
            }
        }

        // Connection is dropped here (detached, so it's closed)
    }
}

/// Basic HTML escaping for plain-text email bodies.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
