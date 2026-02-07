CREATE TABLE IF NOT EXISTS warehouses (
    id              BIGSERIAL PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    contact_name    TEXT,
    contact_emails  JSONB DEFAULT '[]',
    contact_numbers JSONB DEFAULT '[]',
    contact_address JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
