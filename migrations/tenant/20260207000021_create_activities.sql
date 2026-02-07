CREATE TABLE activities (
    id            BIGSERIAL PRIMARY KEY,
    title         TEXT,
    type          TEXT NOT NULL CHECK (type IN ('call', 'meeting', 'note', 'task', 'lunch')),
    comment       TEXT,
    additional    JSONB,
    location      TEXT,
    schedule_from TIMESTAMPTZ,
    schedule_to   TIMESTAMPTZ,
    is_done       BOOLEAN NOT NULL DEFAULT false,
    user_id       BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
