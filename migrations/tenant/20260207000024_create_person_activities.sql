CREATE TABLE person_activities (
    person_id   BIGINT NOT NULL REFERENCES persons(id) ON DELETE CASCADE,
    activity_id BIGINT NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    PRIMARY KEY (person_id, activity_id)
);
