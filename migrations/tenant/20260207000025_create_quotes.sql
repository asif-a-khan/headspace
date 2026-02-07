CREATE TABLE quotes (
    id                BIGSERIAL PRIMARY KEY,
    subject           TEXT NOT NULL,
    description       TEXT,
    billing_address   JSONB,
    shipping_address  JSONB,
    discount_percent  NUMERIC(12,4) DEFAULT 0,
    discount_amount   NUMERIC(12,4) DEFAULT 0,
    tax_amount        NUMERIC(12,4) DEFAULT 0,
    adjustment_amount NUMERIC(12,4) DEFAULT 0,
    sub_total         NUMERIC(12,4) DEFAULT 0,
    grand_total       NUMERIC(12,4) DEFAULT 0,
    expired_at        TIMESTAMPTZ,
    person_id         BIGINT REFERENCES persons(id) ON DELETE SET NULL,
    user_id           BIGINT REFERENCES users(id) ON DELETE SET NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
