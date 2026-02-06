# Implementation Phase 3: Communication & Background Tasks

## Goal

Add email integration (IMAP inbound + SMTP outbound), background task processing, file uploads with storage, and data import/export. These features require long-running operations that don't fit the request-response cycle.

**Prerequisite:** Phase 2 complete (API layer, interactive islands).

**After this phase:** Users can connect email accounts, send/receive emails within the CRM linked to contacts, upload files to records, import/export data in bulk, and all long-running operations happen in the background.

---

## Background Task Architecture

CRM operations that can't complete within a single HTTP request:

| Task | Duration | Trigger |
|------|----------|---------|
| IMAP email sync | 5-60s per mailbox | Periodic (every 2-5 min) |
| Data import (CSV) | 1-30s depending on size | User action |
| Data export (CSV/XLSX) | 1-30s | User action |
| Bulk email send | Seconds per batch | User action |
| Password reset email | <1s | User action |
| Notification emails | <1s | System events |

### Task Queue Design

For Phase 1-2 (single server), PostgreSQL-backed job queue. No Redis needed.

```sql
-- migrations/main/010_create_jobs.sql
CREATE TABLE main.jobs (
    id              BIGSERIAL PRIMARY KEY,
    queue           TEXT NOT NULL DEFAULT 'default',
    job_type        TEXT NOT NULL,          -- "email_sync", "data_import", "send_email"
    payload         JSONB NOT NULL,         -- Job-specific data
    tenant_schema   TEXT NOT NULL,          -- Which tenant this job belongs to
    status          TEXT NOT NULL DEFAULT 'pending',  -- pending, running, completed, failed
    attempts        INT NOT NULL DEFAULT 0,
    max_attempts    INT NOT NULL DEFAULT 3,
    error_message   TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    scheduled_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),  -- For delayed jobs
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ
);

CREATE INDEX idx_jobs_queue_status ON main.jobs (queue, status, scheduled_at);
```

### Job Processing

```rust
// src/jobs/mod.rs
pub mod email_sync;
pub mod data_import;
pub mod data_export;
pub mod send_email;

use sqlx::PgPool;

/// Background job worker. Polls the jobs table and processes tasks.
pub async fn run_worker(db: &Database) {
    tracing::info!("Job worker started");

    loop {
        match claim_next_job(db.writer()).await {
            Ok(Some(job)) => {
                let db = db.clone();
                // Spawn each job as a separate task so the worker continues
                tokio::spawn(async move {
                    if let Err(e) = process_job(&db, &job).await {
                        tracing::error!(job_id = job.id, error = %e, "Job failed");
                        mark_failed(db.writer(), job.id, &e.to_string()).await.ok();
                    }
                });
            }
            Ok(None) => {
                // No jobs available, sleep before polling again
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            Err(e) => {
                tracing::error!(error = %e, "Error claiming job");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

/// Claim the next pending job using SELECT FOR UPDATE SKIP LOCKED.
/// This is safe with multiple workers (Phase 3 with multiple app servers).
async fn claim_next_job(pool: &PgPool) -> Result<Option<Job>, sqlx::Error> {
    sqlx::query_as::<_, Job>(
        "UPDATE main.jobs
         SET status = 'running', started_at = NOW(), attempts = attempts + 1
         WHERE id = (
             SELECT id FROM main.jobs
             WHERE status = 'pending'
               AND scheduled_at <= NOW()
               AND attempts < max_attempts
             ORDER BY created_at ASC
             FOR UPDATE SKIP LOCKED
             LIMIT 1
         )
         RETURNING *"
    )
    .fetch_optional(pool)
    .await
}

async fn process_job(db: &Database, job: &Job) -> Result<(), anyhow::Error> {
    // Set tenant context for this job
    let mut conn = db.writer().acquire().await?;
    set_tenant(&mut conn, &job.tenant_schema).await?;

    match job.job_type.as_str() {
        "email_sync" => email_sync::process(&mut conn, &job.payload).await?,
        "data_import" => data_import::process(&mut conn, &job.payload).await?,
        "data_export" => data_export::process(&mut conn, &job.payload).await?,
        "send_email" => send_email::process(&job.payload).await?,
        other => anyhow::bail!("Unknown job type: {other}"),
    }

    mark_completed(db.writer(), job.id).await?;
    Ok(())
}
```

### Enqueuing Jobs

```rust
/// Enqueue a job from a request handler.
pub async fn enqueue(
    pool: &PgPool,
    tenant_schema: &str,
    job_type: &str,
    payload: serde_json::Value,
) -> Result<i64, sqlx::Error> {
    let job_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO main.jobs (queue, job_type, payload, tenant_schema)
         VALUES ('default', $1, $2, $3)
         RETURNING id"
    )
    .bind(job_type)
    .bind(payload)
    .bind(tenant_schema)
    .fetch_one(pool)
    .await?;

    Ok(job_id)
}
```

### Starting the Worker

```rust
// src/main.rs
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // ... config, db, router setup ...

    // Spawn background worker
    let worker_db = db.clone();
    tokio::spawn(async move {
        jobs::run_worker(&worker_db).await;
    });

    // Spawn IMAP sync scheduler
    let sync_db = db.clone();
    tokio::spawn(async move {
        email::imap_scheduler::run(&sync_db).await;
    });

    // Start HTTP server
    axum::serve(listener, app).await?;
    Ok(())
}
```

---

## Email Architecture

### Overview

```
┌──────────────────────────────────────────────────┐
│                    Headspace                       │
│                                                    │
│  ┌──────────┐   ┌────────────┐   ┌────────────┐ │
│  │ IMAP     │   │ Email DB   │   │ SMTP       │ │
│  │ Sync     │──▶│ Storage    │──▶│ Send       │ │
│  │ Worker   │   │ (tenant    │   │ (lettre)   │ │
│  │ (async-  │   │  schema)   │   │            │ │
│  │  imap)   │   └────────────┘   └────────────┘ │
│  └──────────┘         ▲                           │
│       │               │                           │
│       ▼               │                           │
│  ┌──────────┐   ┌────────────┐                   │
│  │ mail-    │   │ Contact    │                   │
│  │ parser   │   │ Linker     │                   │
│  │ (MIME    │   │ (match by  │                   │
│  │  decode) │   │  email     │                   │
│  └──────────┘   │  address)  │                   │
│                  └────────────┘                   │
└──────────────────────────────────────────────────┘
         │                              │
         ▼                              ▼
┌──────────────┐              ┌──────────────┐
│  User's      │              │  Recipient's │
│  IMAP Server │              │  Mail Server │
│  (Gmail,     │              │  (via SMTP)  │
│   Outlook,   │              │              │
│   etc.)      │              │              │
└──────────────┘              └──────────────┘
```

### Email Account Storage

```sql
-- migrations/tenant/020_create_email_accounts.sql
CREATE TABLE email_accounts (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES users(id),
    email_address   TEXT NOT NULL,
    display_name    TEXT,

    -- IMAP settings
    imap_host       TEXT NOT NULL,           -- "imap.gmail.com"
    imap_port       INT NOT NULL DEFAULT 993,
    imap_encryption TEXT NOT NULL DEFAULT 'tls',  -- "tls", "starttls", "none"
    imap_username   TEXT NOT NULL,
    imap_password   TEXT NOT NULL,           -- Encrypted at rest (see below)

    -- SMTP settings
    smtp_host       TEXT NOT NULL,           -- "smtp.gmail.com"
    smtp_port       INT NOT NULL DEFAULT 587,
    smtp_encryption TEXT NOT NULL DEFAULT 'starttls',
    smtp_username   TEXT NOT NULL,
    smtp_password   TEXT NOT NULL,           -- Encrypted at rest

    -- Sync state
    last_synced_at  TIMESTAMPTZ,
    last_uid        BIGINT DEFAULT 0,        -- IMAP UID of last fetched message
    sync_enabled    BOOLEAN NOT NULL DEFAULT true,

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

```sql
-- migrations/tenant/021_create_emails.sql
CREATE TABLE emails (
    id              BIGSERIAL PRIMARY KEY,
    account_id      BIGINT NOT NULL REFERENCES email_accounts(id),
    message_id      TEXT UNIQUE,             -- RFC 2822 Message-ID
    in_reply_to     TEXT,                    -- For threading
    thread_id       TEXT,                    -- Computed thread grouping

    folder          TEXT NOT NULL DEFAULT 'INBOX',
    subject         TEXT,
    from_address    TEXT NOT NULL,
    from_name       TEXT,
    to_addresses    JSONB NOT NULL DEFAULT '[]',
    cc_addresses    JSONB NOT NULL DEFAULT '[]',
    bcc_addresses   JSONB NOT NULL DEFAULT '[]',

    text_body       TEXT,
    html_body       TEXT,
    has_attachments BOOLEAN NOT NULL DEFAULT false,

    is_read         BOOLEAN NOT NULL DEFAULT false,
    is_draft        BOOLEAN NOT NULL DEFAULT false,
    direction       TEXT NOT NULL DEFAULT 'inbound',  -- "inbound" or "outbound"

    -- Contact linking
    person_id       BIGINT REFERENCES persons(id),
    lead_id         BIGINT REFERENCES leads(id),

    imap_uid        BIGINT,                  -- IMAP UID for sync tracking
    sent_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_emails_account ON emails (account_id, folder);
CREATE INDEX idx_emails_thread ON emails (thread_id);
CREATE INDEX idx_emails_person ON emails (person_id);
CREATE INDEX idx_emails_lead ON emails (lead_id);
```

```sql
-- migrations/tenant/022_create_email_attachments.sql
CREATE TABLE email_attachments (
    id              BIGSERIAL PRIMARY KEY,
    email_id        BIGINT NOT NULL REFERENCES emails(id) ON DELETE CASCADE,
    filename        TEXT NOT NULL,
    content_type    TEXT NOT NULL,
    size_bytes      BIGINT NOT NULL,
    storage_path    TEXT NOT NULL,            -- Path in object storage or local fs
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

---

## IMAP Sync

### Sync Scheduler

```rust
// src/email/imap_scheduler.rs

/// Periodically enqueues email sync jobs for all active email accounts.
pub async fn run(db: &Database) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(120)); // Every 2 min

    loop {
        interval.tick().await;

        if let Err(e) = schedule_syncs(db).await {
            tracing::error!(error = %e, "Failed to schedule email syncs");
        }
    }
}

async fn schedule_syncs(db: &Database) -> Result<(), anyhow::Error> {
    // Get all tenants with active email accounts
    let accounts: Vec<SyncTarget> = sqlx::query_as(
        "SELECT c.schema_name, ea.id as account_id
         FROM main.companies c
         JOIN LATERAL (
             SELECT id FROM (
                 -- Query each tenant's email_accounts
                 SELECT id, sync_enabled, last_synced_at
                 FROM email_accounts  -- will use search_path
                 WHERE sync_enabled = true
                   AND (last_synced_at IS NULL OR last_synced_at < NOW() - INTERVAL '2 minutes')
             ) sub
         ) ea ON true"
    )
    .fetch_all(db.reader())
    .await?;

    // Alternate approach: iterate tenants and query each
    let tenants: Vec<String> = sqlx::query_scalar(
        "SELECT schema_name FROM main.companies WHERE schema_name IS NOT NULL"
    )
    .fetch_all(db.reader())
    .await?;

    for schema in &tenants {
        let mut conn = db.reader().acquire().await?;
        set_tenant(&mut conn, schema).await?;

        let accounts: Vec<i64> = sqlx::query_scalar(
            "SELECT id FROM email_accounts
             WHERE sync_enabled = true
               AND (last_synced_at IS NULL OR last_synced_at < NOW() - INTERVAL '2 minutes')"
        )
        .fetch_all(&mut *conn)
        .await?;

        for account_id in accounts {
            enqueue(
                db.writer(),
                schema,
                "email_sync",
                serde_json::json!({ "account_id": account_id }),
            ).await?;
        }
    }

    Ok(())
}
```

### IMAP Sync Worker

```rust
// src/jobs/email_sync.rs
use async_imap::Session;
use mail_parser::MessageParser;

pub async fn process(conn: &mut PgConnection, payload: &serde_json::Value) -> Result<(), anyhow::Error> {
    let account_id: i64 = payload["account_id"].as_i64()
        .ok_or_else(|| anyhow::anyhow!("Missing account_id"))?;

    // Load account credentials
    let account = sqlx::query_as::<_, EmailAccount>(
        "SELECT * FROM email_accounts WHERE id = $1"
    )
    .bind(account_id)
    .fetch_one(&mut *conn)
    .await?;

    // Connect to IMAP
    let tls = async_native_tls::TlsConnector::new();
    let client = async_imap::connect(
        (&*account.imap_host, account.imap_port as u16),
        &account.imap_host,
        tls,
    ).await?;

    let mut session = client
        .login(&account.imap_username, &decrypt_password(&account.imap_password)?)
        .await
        .map_err(|e| anyhow::anyhow!("IMAP login failed: {:?}", e.0))?;

    // Select INBOX
    let mailbox = session.select("INBOX").await?;

    // Fetch messages newer than our last synced UID
    let fetch_range = if account.last_uid > 0 {
        format!("{}:*", account.last_uid + 1)
    } else {
        // First sync: fetch last 100 messages
        let total = mailbox.exists;
        let start = total.saturating_sub(100) + 1;
        format!("{start}:*")
    };

    let messages = session
        .fetch(fetch_range, "(UID RFC822 FLAGS)")
        .await?;

    let parser = MessageParser::default();
    let mut max_uid: i64 = account.last_uid;

    for msg in messages.iter() {
        let uid = msg.uid.unwrap_or(0) as i64;
        if uid <= account.last_uid { continue; }

        let body = msg.body().unwrap_or_default();
        let parsed = parser.parse(body)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse email"))?;

        // Extract fields
        let message_id = parsed.message_id().map(|s| s.to_string());
        let in_reply_to = parsed.in_reply_to().as_text_ref().map(|s| s.to_string());
        let subject = parsed.subject().map(|s| s.to_string());
        let from_address = parsed.from().and_then(|f| f.first())
            .and_then(|a| a.address()).map(|s| s.to_string())
            .unwrap_or_default();
        let from_name = parsed.from().and_then(|f| f.first())
            .and_then(|a| a.name()).map(|s| s.to_string());
        let text_body = parsed.body_text(0).map(|s| s.to_string());
        let html_body = parsed.body_html(0).map(|s| s.to_string());
        let sent_at = parsed.date().map(|d| d.to_rfc3339());
        let has_attachments = parsed.attachment_count() > 0;

        let to_addresses: Vec<String> = parsed.to().map(|list|
            list.iter().filter_map(|a| a.address().map(|s| s.to_string())).collect()
        ).unwrap_or_default();

        // Try to link to a contact by email address
        let person_id: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM persons
             WHERE emails::text ILIKE '%' || $1 || '%'
             LIMIT 1"
        )
        .bind(&from_address)
        .fetch_optional(&mut *conn)
        .await?;

        // Insert email
        let email_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO emails
             (account_id, message_id, in_reply_to, subject, from_address, from_name,
              to_addresses, text_body, html_body, has_attachments, person_id,
              imap_uid, sent_at, direction)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13::timestamptz, 'inbound')
             ON CONFLICT (message_id) DO NOTHING
             RETURNING id"
        )
        .bind(account_id)
        .bind(&message_id)
        .bind(&in_reply_to)
        .bind(&subject)
        .bind(&from_address)
        .bind(&from_name)
        .bind(serde_json::json!(to_addresses))
        .bind(&text_body)
        .bind(&html_body)
        .bind(has_attachments)
        .bind(person_id)
        .bind(uid)
        .bind(&sent_at)
        .fetch_optional(&mut *conn)
        .await?;

        // Handle attachments
        if has_attachments {
            if let Some(email_id) = email_id {
                for (i, attachment) in parsed.attachments().enumerate() {
                    let filename = attachment.attachment_name()
                        .unwrap_or(&format!("attachment_{i}"))
                        .to_string();
                    let content_type = attachment.content_type()
                        .map(|ct| ct.c_type.to_string())
                        .unwrap_or_else(|| "application/octet-stream".into());
                    let contents = attachment.contents();

                    // Store attachment (local filesystem in Phase 1, S3 in Phase 2+)
                    let storage_path = store_attachment(
                        &account.imap_username, email_id, &filename, contents
                    ).await?;

                    sqlx::query(
                        "INSERT INTO email_attachments
                         (email_id, filename, content_type, size_bytes, storage_path)
                         VALUES ($1, $2, $3, $4, $5)"
                    )
                    .bind(email_id)
                    .bind(&filename)
                    .bind(&content_type)
                    .bind(contents.len() as i64)
                    .bind(&storage_path)
                    .execute(&mut *conn)
                    .await?;
                }
            }
        }

        if uid > max_uid { max_uid = uid; }
    }

    // Update sync state
    sqlx::query(
        "UPDATE email_accounts SET last_uid = $1, last_synced_at = NOW() WHERE id = $2"
    )
    .bind(max_uid)
    .bind(account_id)
    .execute(&mut *conn)
    .await?;

    session.logout().await?;

    tracing::info!(
        account_id = account_id,
        messages_synced = messages.len(),
        "Email sync complete"
    );

    Ok(())
}
```

---

## SMTP Sending

### Composing and Sending Emails

```rust
// src/email/smtp.rs
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{header::ContentType, Attachment, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
};

pub async fn send_email(account: &EmailAccount, email: &OutboundEmail) -> Result<(), anyhow::Error> {
    let creds = Credentials::new(
        account.smtp_username.clone(),
        decrypt_password(&account.smtp_password)?,
    );

    let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&account.smtp_host)?
        .port(account.smtp_port as u16)
        .credentials(creds)
        .build();

    let mut builder = Message::builder()
        .from(format!("{} <{}>", account.display_name.as_deref().unwrap_or(""), account.email_address).parse()?)
        .subject(&email.subject);

    // Add recipients
    for to in &email.to {
        builder = builder.to(to.parse()?);
    }
    for cc in &email.cc {
        builder = builder.cc(cc.parse()?);
    }
    for bcc in &email.bcc {
        builder = builder.bcc(bcc.parse()?);
    }

    // Reply threading
    if let Some(ref reply_to_id) = email.in_reply_to {
        builder = builder.in_reply_to(reply_to_id.clone());
    }

    // Build message body
    let message = if email.attachments.is_empty() {
        builder
            .header(ContentType::TEXT_HTML)
            .body(email.html_body.clone())?
    } else {
        let mut multipart = MultiPart::mixed()
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_HTML)
                    .body(email.html_body.clone())
            );

        for attachment in &email.attachments {
            let content = tokio::fs::read(&attachment.path).await?;
            multipart = multipart.singlepart(
                Attachment::new(attachment.filename.clone())
                    .body(content, attachment.content_type.parse()?)
            );
        }

        builder.multipart(multipart)?
    };

    transport.send(message).await?;
    Ok(())
}
```

### Send Email Handler

```rust
// src/handlers/emails.rs
pub async fn send(
    Extension(user): Extension<User>,
    Extension(tenant): Extension<Tenant>,
    Extension(db): Extension<Database>,
    session: Session,
    Form(input): Form<ComposeEmailInput>,
) -> Result<impl IntoResponse, AppError> {
    validate_csrf(&session, &input.csrf).await?;

    let mut conn = db.writer().acquire().await?;
    set_tenant(&mut conn, &tenant.schema_name).await?;

    let account = sqlx::query_as::<_, EmailAccount>(
        "SELECT * FROM email_accounts WHERE id = $1 AND user_id = $2"
    )
    .bind(input.account_id)
    .bind(user.id)
    .fetch_one(&mut *conn)
    .await?;

    // Store outbound email in DB
    let email_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO emails
         (account_id, subject, from_address, from_name, to_addresses, cc_addresses,
          html_body, text_body, direction, person_id, lead_id, in_reply_to, sent_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'outbound', $9, $10, $11, NOW())
         RETURNING id"
    )
    .bind(account.id)
    .bind(&input.subject)
    .bind(&account.email_address)
    .bind(&account.display_name)
    .bind(serde_json::json!(&input.to))
    .bind(serde_json::json!(&input.cc))
    .bind(&input.body)
    .bind(&strip_html(&input.body))
    .bind(input.person_id)
    .bind(input.lead_id)
    .bind(&input.in_reply_to)
    .fetch_one(&mut *conn)
    .await?;

    // Enqueue send job (async, so the user isn't waiting for SMTP)
    enqueue(
        db.writer(),
        &tenant.schema_name,
        "send_email",
        serde_json::json!({
            "email_id": email_id,
            "account_id": account.id,
        }),
    ).await?;

    session.insert("flash", FlashMessage::success("Email queued for sending")).await?;
    Ok(Redirect::to(&format!("/admin/emails/{email_id}")))
}
```

---

## Email Threading

Emails are grouped into threads using the `Message-ID`, `In-Reply-To`, and `References` headers:

```rust
/// Compute thread_id when inserting an email.
/// If it's a reply, use the same thread_id as the parent.
/// If it's a new conversation, use its own message_id as the thread_id.
pub async fn compute_thread_id(
    conn: &mut PgConnection,
    message_id: &Option<String>,
    in_reply_to: &Option<String>,
) -> Result<String, sqlx::Error> {
    // If replying, find the parent's thread
    if let Some(ref parent_id) = in_reply_to {
        let thread: Option<String> = sqlx::query_scalar(
            "SELECT thread_id FROM emails WHERE message_id = $1"
        )
        .bind(parent_id)
        .fetch_optional(&mut *conn)
        .await?;

        if let Some(thread_id) = thread {
            return Ok(thread_id);
        }
    }

    // New thread: use this email's message_id
    Ok(message_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string()))
}
```

---

## Contact Linking

When an email arrives, automatically link it to existing contacts:

```rust
/// Match an email address to a person in the CRM.
/// Searches the persons table for matching email addresses.
pub async fn link_to_contact(
    conn: &mut PgConnection,
    email_address: &str,
) -> Result<Option<i64>, sqlx::Error> {
    // persons.emails is a JSONB array of { "value": "foo@bar.com", "label": "work" }
    sqlx::query_scalar(
        "SELECT id FROM persons
         WHERE EXISTS (
             SELECT 1 FROM jsonb_array_elements(emails) elem
             WHERE elem->>'value' ILIKE $1
         )
         LIMIT 1"
    )
    .bind(email_address)
    .fetch_optional(conn)
    .await
}
```

---

## File Uploads

### Upload Handler

```rust
// src/handlers/files.rs
use axum::extract::Multipart;

pub async fn upload(
    Extension(tenant): Extension<Tenant>,
    Extension(db): Extension<Database>,
    Extension(user): Extension<User>,
    mut multipart: Multipart,
) -> Result<Json<Vec<UploadedFile>>, AppError> {
    let mut uploaded = Vec::new();

    while let Some(field) = multipart.next_field().await? {
        let filename = field.file_name()
            .ok_or(AppError::Validation(vec![("file".into(), "Filename required".into())]))?
            .to_string();
        let content_type = field.content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let data = field.bytes().await?;

        // Validate file size (10MB limit)
        if data.len() > 10 * 1024 * 1024 {
            return Err(AppError::Validation(vec![
                ("file".into(), "File too large (max 10MB)".into())
            ]));
        }

        // Store file
        let storage_path = store_file(&tenant.schema_name, &filename, &data).await?;

        uploaded.push(UploadedFile {
            filename,
            content_type,
            size_bytes: data.len() as i64,
            storage_path,
        });
    }

    Ok(Json(uploaded))
}
```

### File Storage

```rust
// src/storage.rs

/// Phase 1: Local filesystem storage
/// Phase 2+: Replace with S3-compatible object storage
pub async fn store_file(
    tenant_schema: &str,
    filename: &str,
    data: &[u8],
) -> Result<String, anyhow::Error> {
    let safe_filename = sanitize_filename(filename);
    let unique = uuid::Uuid::new_v4();
    let path = format!("uploads/{tenant_schema}/{unique}/{safe_filename}");

    let full_path = std::path::Path::new("data").join(&path);
    tokio::fs::create_dir_all(full_path.parent().unwrap()).await?;
    tokio::fs::write(&full_path, data).await?;

    Ok(path)
}

/// Retrieve a stored file
pub async fn get_file(storage_path: &str) -> Result<Vec<u8>, anyhow::Error> {
    let full_path = std::path::Path::new("data").join(storage_path);
    Ok(tokio::fs::read(full_path).await?)
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .collect()
}
```

---

## Data Import

### Import Handler

```rust
// src/handlers/imports.rs
pub async fn start_import(
    Extension(tenant): Extension<Tenant>,
    Extension(db): Extension<Database>,
    session: Session,
    Form(input): Form<ImportInput>,
) -> Result<impl IntoResponse, AppError> {
    validate_csrf(&session, &input.csrf).await?;

    // Store the uploaded CSV temporarily
    let storage_path = store_file(&tenant.schema_name, &input.filename, &input.file_data).await?;

    // Enqueue import job
    let job_id = enqueue(
        db.writer(),
        &tenant.schema_name,
        "data_import",
        serde_json::json!({
            "entity_type": input.entity_type,  // "leads", "persons", "organizations"
            "file_path": storage_path,
            "column_mapping": input.column_mapping,  // User-defined CSV column → field mapping
        }),
    ).await?;

    session.insert("flash", FlashMessage::info("Import started. You'll be notified when complete.")).await?;
    Ok(Redirect::to("/admin/settings/data-transfer/imports"))
}
```

### Import Worker

```rust
// src/jobs/data_import.rs
pub async fn process(conn: &mut PgConnection, payload: &serde_json::Value) -> Result<(), anyhow::Error> {
    let entity_type = payload["entity_type"].as_str().unwrap();
    let file_path = payload["file_path"].as_str().unwrap();
    let mapping: HashMap<String, String> = serde_json::from_value(
        payload["column_mapping"].clone()
    )?;

    let file_data = get_file(file_path).await?;
    let mut reader = csv::Reader::from_reader(&file_data[..]);

    let mut imported = 0;
    let mut errors: Vec<String> = Vec::new();

    for (i, result) in reader.records().enumerate() {
        let record = result?;
        let row = i + 2; // 1-indexed, skip header

        match entity_type {
            "leads" => {
                if let Err(e) = import_lead(conn, &record, &mapping).await {
                    errors.push(format!("Row {row}: {e}"));
                } else {
                    imported += 1;
                }
            }
            "persons" => {
                if let Err(e) = import_person(conn, &record, &mapping).await {
                    errors.push(format!("Row {row}: {e}"));
                } else {
                    imported += 1;
                }
            }
            other => anyhow::bail!("Unsupported entity type: {other}"),
        }
    }

    tracing::info!(
        entity_type = entity_type,
        imported = imported,
        errors = errors.len(),
        "Import complete"
    );

    Ok(())
}
```

---

## Data Export

```rust
// src/jobs/data_export.rs
pub async fn process(conn: &mut PgConnection, payload: &serde_json::Value) -> Result<(), anyhow::Error> {
    let entity_type = payload["entity_type"].as_str().unwrap();
    let tenant_schema = payload["tenant_schema"].as_str().unwrap();

    let mut csv_writer = csv::Writer::from_writer(Vec::new());

    match entity_type {
        "leads" => {
            csv_writer.write_record(["ID", "Title", "Value", "Stage", "Person", "Organization", "Created"])?;

            let leads = sqlx::query_as::<_, LeadExportRow>(
                "SELECT l.id, l.title, l.lead_value, ps.name as stage,
                        p.name as person, o.name as organization, l.created_at
                 FROM leads l
                 LEFT JOIN pipeline_stages ps ON l.stage_id = ps.id
                 LEFT JOIN persons p ON l.person_id = p.id
                 LEFT JOIN organizations o ON l.organization_id = o.id
                 ORDER BY l.created_at DESC"
            )
            .fetch_all(&mut *conn)
            .await?;

            for lead in leads {
                csv_writer.write_record([
                    lead.id.to_string(),
                    lead.title,
                    lead.lead_value.map(|v| v.to_string()).unwrap_or_default(),
                    lead.stage.unwrap_or_default(),
                    lead.person.unwrap_or_default(),
                    lead.organization.unwrap_or_default(),
                    lead.created_at.to_string(),
                ])?;
            }
        }
        other => anyhow::bail!("Unsupported entity type: {other}"),
    }

    let csv_data = csv_writer.into_inner()?;
    let filename = format!("{entity_type}_export_{}.csv", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    let path = store_file(tenant_schema, &filename, &csv_data).await?;

    // Store export reference for user to download
    sqlx::query(
        "INSERT INTO exports (entity_type, filename, storage_path, created_at)
         VALUES ($1, $2, $3, NOW())"
    )
    .bind(entity_type)
    .bind(&filename)
    .bind(&path)
    .execute(&mut *conn)
    .await?;

    Ok(())
}
```

---

## Password Encryption for Email Credentials

Email account passwords are encrypted at rest using a server-side key:

```rust
// src/crypto.rs
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

/// Encrypt a plaintext password for storage.
pub fn encrypt_password(plaintext: &str, key: &[u8; 32]) -> Result<String, anyhow::Error> {
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())?;

    // Encode as base64: nonce + ciphertext
    let mut combined = nonce_bytes.to_vec();
    combined.extend(ciphertext);
    Ok(BASE64.encode(combined))
}

/// Decrypt a stored password.
pub fn decrypt_password(encrypted: &str, key: &[u8; 32]) -> Result<String, anyhow::Error> {
    let combined = BASE64.decode(encrypted)?;
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(key)?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    Ok(String::from_utf8(plaintext)?)
}
```

This requires `aes-gcm` and `base64` as additional dependencies:

```toml
aes-gcm = "0.10"
base64 = "0.22"
```

---

## What Phase 3 Delivers

| Feature | Status |
|---------|--------|
| Background job queue (PostgreSQL-backed) | Complete |
| Job worker with retry and error handling | Complete |
| Email account setup (IMAP + SMTP credentials) | Complete |
| IMAP email sync (periodic background) | Complete |
| Email parsing (MIME, attachments, HTML/text) | Complete |
| Email sending via SMTP | Complete |
| Email threading (In-Reply-To / References) | Complete |
| Contact linking by email address | Complete |
| Email attachment storage | Complete |
| File upload (multipart, size validation) | Complete |
| File storage (local filesystem, S3-ready abstraction) | Complete |
| Data import from CSV | Complete |
| Data export to CSV | Complete |
| Password encryption for stored credentials | Complete |
| Notification emails (password reset, etc.) | Complete |

---

## Additional Dependencies (Phase 3)

```toml
# Add to Cargo.toml alongside STACK.md dependencies
csv = "1"                    # CSV parsing for import/export
aes-gcm = "0.10"            # Encryption for email credentials
base64 = "0.22"             # Encoding encrypted data
async-native-tls = "0.5"    # TLS for IMAP connections
```
