CREATE TABLE users (
    id              BIGSERIAL PRIMARY KEY,
    first_name      TEXT NOT NULL,
    last_name       TEXT NOT NULL,
    email           TEXT NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    image           TEXT,
    status          BOOLEAN NOT NULL DEFAULT true,
    role_id         BIGINT NOT NULL REFERENCES roles(id),
    view_permission TEXT NOT NULL DEFAULT 'global'
        CHECK (view_permission IN ('global', 'group', 'individual')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
