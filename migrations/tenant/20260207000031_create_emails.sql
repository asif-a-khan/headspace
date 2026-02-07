-- Email messages table.
CREATE TABLE IF NOT EXISTS emails (
    id              BIGSERIAL PRIMARY KEY,
    subject         TEXT NOT NULL DEFAULT '',
    body            TEXT NOT NULL DEFAULT '',
    from_address    TEXT NOT NULL DEFAULT '',
    from_name       TEXT NOT NULL DEFAULT '',
    reply_to        JSONB NOT NULL DEFAULT '[]',
    cc              JSONB NOT NULL DEFAULT '[]',
    bcc             JSONB NOT NULL DEFAULT '[]',
    folder          TEXT NOT NULL DEFAULT 'inbox'
        CHECK (folder IN ('inbox','sent','draft','trash')),
    is_read         BOOLEAN NOT NULL DEFAULT false,
    source          TEXT NOT NULL DEFAULT 'web'
        CHECK (source IN ('web','imap')),
    message_id      TEXT,
    in_reply_to     TEXT,
    person_id       BIGINT REFERENCES persons(id) ON DELETE SET NULL,
    lead_id         BIGINT REFERENCES leads(id) ON DELETE SET NULL,
    parent_id       BIGINT REFERENCES emails(id) ON DELETE CASCADE,
    user_id         BIGINT REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_emails_folder ON emails(folder);
CREATE INDEX idx_emails_person_id ON emails(person_id);
CREATE INDEX idx_emails_lead_id ON emails(lead_id);
CREATE INDEX idx_emails_user_id ON emails(user_id);
CREATE INDEX idx_emails_parent_id ON emails(parent_id);

-- Email attachments table.
CREATE TABLE IF NOT EXISTS email_attachments (
    id              BIGSERIAL PRIMARY KEY,
    email_id        BIGINT NOT NULL REFERENCES emails(id) ON DELETE CASCADE,
    file_name       TEXT NOT NULL,
    file_path       TEXT NOT NULL,
    file_type       TEXT,
    file_size       BIGINT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_email_attachments_email_id ON email_attachments(email_id);

-- SMTP settings (stored in tenant_config).
INSERT INTO tenant_config (code, value) VALUES
    ('email.smtp.host', ''),
    ('email.smtp.port', '587'),
    ('email.smtp.username', ''),
    ('email.smtp.password', ''),
    ('email.smtp.encryption', 'tls'),
    ('email.smtp.from_address', ''),
    ('email.smtp.from_name', '')
ON CONFLICT (code) DO NOTHING;
