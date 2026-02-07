CREATE TABLE organization_tags (
    organization_id BIGINT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    tag_id          BIGINT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (organization_id, tag_id)
);
