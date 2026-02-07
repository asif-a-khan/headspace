-- IMAP settings (stored in tenant_config).
INSERT INTO tenant_config (code, value) VALUES
    ('email.imap.host', ''),
    ('email.imap.port', '993'),
    ('email.imap.username', ''),
    ('email.imap.password', ''),
    ('email.imap.encryption', 'ssl')
ON CONFLICT (code) DO NOTHING;
