CREATE TABLE persons (
    id              BIGSERIAL PRIMARY KEY,
    name            TEXT NOT NULL,
    emails          JSONB NOT NULL DEFAULT '[]',
    contact_numbers JSONB NOT NULL DEFAULT '[]',
    job_title       TEXT,
    organization_id BIGINT REFERENCES organizations(id) ON DELETE SET NULL,
    user_id         BIGINT REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
