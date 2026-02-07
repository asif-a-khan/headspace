CREATE TABLE attribute_values (
    id              BIGSERIAL PRIMARY KEY,
    entity_type     TEXT NOT NULL,
    entity_id       BIGINT NOT NULL,
    attribute_id    BIGINT NOT NULL REFERENCES attributes(id) ON DELETE CASCADE,
    text_value      TEXT,
    boolean_value   BOOLEAN,
    integer_value   BIGINT,
    float_value     DOUBLE PRECISION,
    date_value      DATE,
    datetime_value  TIMESTAMPTZ,
    json_value      JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (entity_type, entity_id, attribute_id)
);

CREATE INDEX idx_attr_values_entity ON attribute_values(entity_type, entity_id);
