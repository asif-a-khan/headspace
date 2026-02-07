CREATE TABLE lead_quotes (
    lead_id  BIGINT NOT NULL REFERENCES leads(id) ON DELETE CASCADE,
    quote_id BIGINT NOT NULL REFERENCES quotes(id) ON DELETE CASCADE,
    PRIMARY KEY (lead_id, quote_id)
);
