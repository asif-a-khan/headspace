CREATE TABLE quote_items (
    id               BIGSERIAL PRIMARY KEY,
    sku              TEXT,
    name             TEXT,
    quantity         INTEGER NOT NULL DEFAULT 1,
    price            NUMERIC(12,4) NOT NULL DEFAULT 0,
    discount_percent NUMERIC(12,4) DEFAULT 0,
    discount_amount  NUMERIC(12,4) DEFAULT 0,
    tax_percent      NUMERIC(12,4) DEFAULT 0,
    tax_amount       NUMERIC(12,4) DEFAULT 0,
    total            NUMERIC(12,4) NOT NULL DEFAULT 0,
    product_id       BIGINT REFERENCES products(id) ON DELETE SET NULL,
    quote_id         BIGINT NOT NULL REFERENCES quotes(id) ON DELETE CASCADE,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
