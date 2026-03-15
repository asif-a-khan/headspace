use axum::Json;
use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::auth::bouncer::bouncer;
use crate::db::Database;
use crate::db::guard::TenantGuard;
use crate::models::company::Company;
use crate::models::tenant_admin::TenantUser;

use super::config::load_tenant_config;

// ── Models ──

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct Email {
    pub id: i64,
    pub subject: String,
    pub body: String,
    pub from_address: String,
    pub from_name: String,
    pub reply_to: serde_json::Value,
    pub cc: serde_json::Value,
    pub bcc: serde_json::Value,
    pub folder: String,
    pub is_read: bool,
    pub source: String,
    pub message_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub person_id: Option<i64>,
    pub lead_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub user_id: Option<i64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct EmailRow {
    pub id: i64,
    pub subject: String,
    pub from_address: String,
    pub from_name: String,
    pub reply_to: serde_json::Value,
    pub folder: String,
    pub is_read: bool,
    pub person_name: Option<String>,
    pub lead_title: Option<String>,
    pub user_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ── Payloads ──

#[derive(Deserialize)]
pub struct ListQuery {
    pub folder: Option<String>,
}

#[derive(Deserialize)]
pub struct ComposePayload {
    pub subject: String,
    pub body: String,
    pub to: Vec<String>,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
    pub is_draft: Option<bool>,
    pub person_id: Option<i64>,
    pub lead_id: Option<i64>,
    pub parent_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpdatePayload {
    pub is_read: Option<bool>,
    pub folder: Option<String>,
    pub person_id: Option<i64>,
    pub lead_id: Option<i64>,
}

// ── Handlers ──

/// GET /admin/api/emails?folder=inbox — list emails in a folder.
pub async fn list(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Query(query): Query<ListQuery>,
) -> Response {
    if let Err(resp) = bouncer(&user, "mail") {
        return resp;
    }

    let folder = query.folder.unwrap_or_else(|| "inbox".to_string());
    let allowed = ["inbox", "sent", "draft", "trash"];
    if !allowed.contains(&folder.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid folder." })),
        )
            .into_response();
    }

    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let sql = format!(
        "SELECT e.id, e.subject, e.from_address, e.from_name, e.reply_to,
                e.folder, e.is_read, e.created_at,
                p.name AS person_name,
                l.title AS lead_title,
                CONCAT(u.first_name, ' ', u.last_name) AS user_name
         FROM emails e
         LEFT JOIN persons p ON p.id = e.person_id
         LEFT JOIN leads l ON l.id = e.lead_id
         LEFT JOIN users u ON u.id = e.user_id
         WHERE e.folder = '{folder}'
         ORDER BY e.created_at DESC
         LIMIT 200"
    );

    let emails = guard.fetch_all(sqlx::query_as::<_, EmailRow>(&sql)).await;
    let _ = guard.release().await;

    match emails {
        Ok(e) => Json(serde_json::json!({ "data": e })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list emails: {e}");
            internal_error()
        }
    }
}

/// GET /admin/api/emails/:id — show single email with thread.
pub async fn show(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "mail") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    // Mark as read
    let _ = guard
        .execute(
            sqlx::query("UPDATE emails SET is_read = true, updated_at = NOW() WHERE id = $1")
                .bind(id),
        )
        .await;

    let email = guard
        .fetch_optional(sqlx::query_as::<_, Email>("SELECT * FROM emails WHERE id = $1").bind(id))
        .await;

    // Fetch thread (replies)
    let replies = guard
        .fetch_all(
            sqlx::query_as::<_, Email>(
                "SELECT * FROM emails WHERE parent_id = $1 ORDER BY created_at ASC",
            )
            .bind(id),
        )
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    match email {
        Ok(Some(e)) => Json(serde_json::json!({ "data": e, "replies": replies })).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Email not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch email: {e}");
            internal_error()
        }
    }
}

/// POST /admin/api/emails — compose and send (or save as draft).
pub async fn store(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Json(payload): Json<ComposePayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "mail.compose") {
        return resp;
    }

    if payload.to.is_empty() && payload.is_draft != Some(true) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "At least one recipient is required." })),
        )
            .into_response();
    }

    let is_draft = payload.is_draft.unwrap_or(false);

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    // Load SMTP config for from_address
    let config = load_tenant_config(&mut guard).await;
    let from_address = config
        .get("email.smtp.from_address")
        .cloned()
        .unwrap_or_default();
    let from_name = config
        .get("email.smtp.from_name")
        .cloned()
        .unwrap_or_else(|| company.name.clone());

    let folder = if is_draft { "draft" } else { "sent" };
    let to_json = serde_json::to_value(&payload.to).unwrap_or_default();
    let cc_json = serde_json::to_value(payload.cc.unwrap_or_default()).unwrap_or_default();
    let bcc_json = serde_json::to_value(payload.bcc.unwrap_or_default()).unwrap_or_default();

    let msg_id = format!(
        "{}@headspace",
        uuid::Uuid::new_v4().to_string().replace('-', "")
    );

    let email = guard
        .fetch_one(
            sqlx::query_as::<_, Email>(
                "INSERT INTO emails (subject, body, from_address, from_name,
                    reply_to, cc, bcc, folder, is_read, source, message_id,
                    in_reply_to, person_id, lead_id, parent_id, user_id)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, 'web', $9, $10, $11, $12, $13, $14)
                 RETURNING *",
            )
            .bind(&payload.subject)
            .bind(&payload.body)
            .bind(&from_address)
            .bind(&from_name)
            .bind(&to_json)
            .bind(&cc_json)
            .bind(&bcc_json)
            .bind(folder)
            .bind(&msg_id)
            .bind(payload.parent_id.map(|_| msg_id.clone()))
            .bind(payload.person_id)
            .bind(payload.lead_id)
            .bind(payload.parent_id)
            .bind(user.id),
        )
        .await;

    let _ = guard.release().await;

    let email = match email {
        Ok(e) => e,
        Err(e) => {
            tracing::error!("Failed to create email: {e}");
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "Failed to save email." })),
            )
                .into_response();
        }
    };

    // Send via SMTP if not a draft
    if !is_draft && let Err(e) = send_email_smtp(&config, &email).await {
        tracing::error!("SMTP send failed: {e}");
        // Move to draft since send failed
        let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
            Ok(g) => g,
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Email saved but send failed: {e}"),
                        "data": email,
                    })),
                )
                    .into_response();
            }
        };
        let _ = guard
            .execute(
                sqlx::query("UPDATE emails SET folder = 'draft', updated_at = NOW() WHERE id = $1")
                    .bind(email.id),
            )
            .await;
        let _ = guard.release().await;
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({
                "error": format!("Email saved as draft — send failed: {e}"),
                "data": email,
            })),
        )
            .into_response();
    }

    let msg = if is_draft {
        "Email saved as draft."
    } else {
        "Email sent successfully."
    };
    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "data": email, "message": msg })),
    )
        .into_response()
}

/// PUT /admin/api/emails/:id — update email metadata (read status, folder, linking).
pub async fn update(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdatePayload>,
) -> Response {
    if let Err(resp) = bouncer(&user, "mail") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let mut sets: Vec<String> = vec!["updated_at = NOW()".to_string()];

    if let Some(is_read) = payload.is_read {
        sets.push(format!("is_read = {is_read}"));
    }
    if let Some(ref folder) = payload.folder {
        let allowed = ["inbox", "sent", "draft", "trash"];
        if allowed.contains(&folder.as_str()) {
            sets.push(format!("folder = '{folder}'"));
        }
    }
    if let Some(person_id) = payload.person_id {
        sets.push(format!("person_id = {person_id}"));
    }
    if let Some(lead_id) = payload.lead_id {
        sets.push(format!("lead_id = {lead_id}"));
    }

    let sql = format!(
        "UPDATE emails SET {} WHERE id = $1 RETURNING *",
        sets.join(", ")
    );
    let result = guard
        .fetch_optional(sqlx::query_as::<_, Email>(&sql).bind(id))
        .await;

    let _ = guard.release().await;

    match result {
        Ok(Some(e)) => {
            Json(serde_json::json!({ "data": e, "message": "Email updated." })).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Email not found." })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to update email: {e}");
            internal_error()
        }
    }
}

/// DELETE /admin/api/emails/:id — move to trash or hard delete if already trashed.
pub async fn destroy(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
    Path(id): Path<i64>,
) -> Response {
    if let Err(resp) = bouncer(&user, "mail.delete") {
        return resp;
    }

    let mut guard = match TenantGuard::acquire(db.writer(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    // Check current folder — if already in trash, hard delete
    let current = guard
        .fetch_optional(sqlx::query_as::<_, Email>("SELECT * FROM emails WHERE id = $1").bind(id))
        .await;

    match current {
        Ok(Some(email)) => {
            if email.folder == "trash" {
                let _ = guard
                    .execute(sqlx::query("DELETE FROM emails WHERE id = $1").bind(id))
                    .await;
                let _ = guard.release().await;
                Json(serde_json::json!({ "message": "Email permanently deleted." })).into_response()
            } else {
                let _ = guard
                    .execute(
                        sqlx::query(
                            "UPDATE emails SET folder = 'trash', updated_at = NOW() WHERE id = $1",
                        )
                        .bind(id),
                    )
                    .await;
                let _ = guard.release().await;
                Json(serde_json::json!({ "message": "Email moved to trash." })).into_response()
            }
        }
        Ok(None) => {
            let _ = guard.release().await;
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Email not found." })),
            )
                .into_response()
        }
        Err(e) => {
            let _ = guard.release().await;
            tracing::error!("Failed to fetch email: {e}");
            internal_error()
        }
    }
}

// ── SMTP Sending ──

async fn send_email_smtp(
    config: &std::collections::HashMap<String, String>,
    email: &Email,
) -> Result<(), String> {
    use lettre::message::{Mailbox, MultiPart, SinglePart, header::ContentType};
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

    let host = config
        .get("email.smtp.host")
        .filter(|h| !h.is_empty())
        .ok_or("SMTP host not configured.")?;
    let port: u16 = config
        .get("email.smtp.port")
        .and_then(|p| p.parse().ok())
        .unwrap_or(587);
    let username = config
        .get("email.smtp.username")
        .filter(|u| !u.is_empty())
        .ok_or("SMTP username not configured.")?;
    let password = config
        .get("email.smtp.password")
        .filter(|p| !p.is_empty())
        .ok_or("SMTP password not configured.")?;
    let encryption = config
        .get("email.smtp.encryption")
        .map(|s| s.as_str())
        .unwrap_or("tls");

    let from_mailbox: Mailbox = format!("{} <{}>", email.from_name, email.from_address)
        .parse()
        .map_err(|e| format!("Invalid from address: {e}"))?;

    let recipients: Vec<String> =
        serde_json::from_value(email.reply_to.clone()).unwrap_or_default();

    if recipients.is_empty() {
        return Err("No recipients.".to_string());
    }

    let mut builder = Message::builder()
        .from(from_mailbox)
        .subject(&email.subject);

    for to in &recipients {
        let to_mailbox: Mailbox = to
            .parse()
            .map_err(|e| format!("Invalid recipient '{to}': {e}"))?;
        builder = builder.to(to_mailbox);
    }

    // CC
    let cc_list: Vec<String> = serde_json::from_value(email.cc.clone()).unwrap_or_default();
    for cc in &cc_list {
        if let Ok(mb) = cc.parse::<Mailbox>() {
            builder = builder.cc(mb);
        }
    }

    // BCC
    let bcc_list: Vec<String> = serde_json::from_value(email.bcc.clone()).unwrap_or_default();
    for bcc in &bcc_list {
        if let Ok(mb) = bcc.parse::<Mailbox>() {
            builder = builder.bcc(mb);
        }
    }

    let message = builder
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(html_to_text(&email.body)),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(email.body.clone()),
                ),
        )
        .map_err(|e| format!("Failed to build message: {e}"))?;

    let creds = Credentials::new(username.clone(), password.clone());

    let mailer = match encryption {
        "ssl" => AsyncSmtpTransport::<Tokio1Executor>::relay(host)
            .map_err(|e| format!("SMTP relay error: {e}"))?
            .port(port)
            .credentials(creds)
            .build(),
        "none" => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host)
            .port(port)
            .credentials(creds)
            .build(),
        _ => {
            // tls (STARTTLS)
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)
                .map_err(|e| format!("SMTP STARTTLS error: {e}"))?
                .port(port)
                .credentials(creds)
                .build()
        }
    };

    mailer
        .send(message)
        .await
        .map_err(|e| format!("SMTP send error: {e}"))?;

    Ok(())
}

/// Simple HTML to plain text (strip tags).
fn html_to_text(html: &str) -> String {
    let mut text = html.to_string();
    // Replace <br> and </p> with newlines
    text = text
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n");
    text = text.replace("</p>", "\n\n");
    // Strip remaining tags
    let mut result = String::new();
    let mut in_tag = false;
    for ch in text.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }
    result.trim().to_string()
}

/// POST /admin/api/emails/sync — trigger a manual IMAP sync for this tenant.
pub async fn trigger_sync(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Extension(user): Extension<TenantUser>,
) -> Response {
    if let Err(resp) = bouncer(&user, "mail") {
        return resp;
    }

    // Load IMAP config to validate it's enabled
    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to acquire tenant connection: {e}");
            return internal_error();
        }
    };

    let config = load_tenant_config(&mut guard).await;
    let _ = guard.release().await;

    let imap_config = match crate::imap::ImapConfig::from_config_map(&config) {
        Some(c) if c.enabled && c.is_configured() => c,
        _ => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "IMAP is not configured or not enabled." })),
            )
                .into_response();
        }
    };

    // Spawn a detached task to do the sync
    let pool = db.writer().clone();
    let schema = company.schema_name.clone();
    tokio::spawn(async move {
        let pool_conn = match pool.acquire().await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(schema = %schema, error = %e, "IMAP manual sync: acquire failed");
                return;
            }
        };
        let mut conn = pool_conn.detach();

        let set_sql = format!("SET search_path TO {schema}, public");
        if let Err(e) = sqlx::query(&set_sql).execute(&mut conn).await {
            tracing::error!(schema = %schema, error = %e, "IMAP manual sync: set search_path failed");
            return;
        }

        match crate::imap::sync::sync_tenant(&mut conn, &imap_config, &schema).await {
            Ok(stats) => {
                tracing::info!(
                    schema = %schema,
                    inserted = stats.messages_inserted,
                    skipped = stats.messages_skipped,
                    "IMAP manual sync completed"
                );
            }
            Err(e) => {
                tracing::warn!(schema = %schema, error = %e, "IMAP manual sync failed");
            }
        }
    });

    Json(serde_json::json!({ "message": "IMAP sync started." })).into_response()
}

fn internal_error() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "An internal error occurred." })),
    )
        .into_response()
}
