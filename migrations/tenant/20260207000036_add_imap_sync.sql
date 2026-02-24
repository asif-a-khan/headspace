-- Add IMAP UID tracking and dedup support for inbound email sync.

ALTER TABLE emails ADD COLUMN IF NOT EXISTS imap_uid BIGINT;
CREATE INDEX IF NOT EXISTS idx_emails_imap_uid ON emails(imap_uid);
CREATE UNIQUE INDEX IF NOT EXISTS idx_emails_message_id ON emails(message_id) WHERE message_id IS NOT NULL;

INSERT INTO tenant_config (code, value) VALUES
    ('email.imap.enabled', 'false'),
    ('email.imap.last_sync_at', '')
ON CONFLICT (code) DO NOTHING;
