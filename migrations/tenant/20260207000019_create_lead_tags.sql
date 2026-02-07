CREATE TABLE lead_tags (
    lead_id BIGINT NOT NULL REFERENCES leads(id) ON DELETE CASCADE,
    tag_id  BIGINT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (lead_id, tag_id)
);
