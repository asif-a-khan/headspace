CREATE TABLE IF NOT EXISTS email_templates (
    id         BIGSERIAL PRIMARY KEY,
    name       TEXT NOT NULL,
    subject    TEXT NOT NULL,
    content    TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
