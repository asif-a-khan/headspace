-- Tenant-level key-value configuration table.
CREATE TABLE IF NOT EXISTS tenant_config (
    id          BIGSERIAL PRIMARY KEY,
    code        TEXT NOT NULL UNIQUE,
    value       TEXT NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed default configuration values.
INSERT INTO tenant_config (code, value) VALUES
    ('general.currency_symbol', '$'),
    ('general.date_format', 'YYYY-MM-DD'),
    ('general.timezone', 'UTC'),
    ('general.locale', 'en'),
    ('appearance.brand_color', '#6366F1')
ON CONFLICT (code) DO NOTHING;
