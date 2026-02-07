CREATE TABLE person_tags (
    person_id BIGINT NOT NULL REFERENCES persons(id) ON DELETE CASCADE,
    tag_id    BIGINT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (person_id, tag_id)
);
