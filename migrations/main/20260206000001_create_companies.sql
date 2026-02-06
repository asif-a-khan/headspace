CREATE TABLE companies (
    id          BIGSERIAL PRIMARY KEY,
    name        TEXT NOT NULL,
    email       TEXT,
    domain      TEXT NOT NULL UNIQUE,
    cname       TEXT,
    description TEXT,
    is_active   BOOLEAN NOT NULL DEFAULT true,
    schema_name TEXT NOT NULL UNIQUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
