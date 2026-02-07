CREATE TABLE attributes (
    id              BIGSERIAL PRIMARY KEY,
    code            TEXT NOT NULL,
    name            TEXT NOT NULL,
    type            TEXT NOT NULL CHECK (type IN (
        'text', 'textarea', 'boolean', 'integer', 'decimal',
        'date', 'datetime', 'select', 'multiselect',
        'email', 'phone', 'address', 'image', 'file',
        'lookup', 'price'
    )),
    entity_type     TEXT NOT NULL CHECK (entity_type IN (
        'leads', 'persons', 'organizations', 'products', 'quotes'
    )),
    lookup_type     TEXT,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    validation      TEXT,
    is_required     BOOLEAN NOT NULL DEFAULT false,
    is_unique       BOOLEAN NOT NULL DEFAULT false,
    quick_add       BOOLEAN NOT NULL DEFAULT false,
    is_user_defined BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (code, entity_type)
);
