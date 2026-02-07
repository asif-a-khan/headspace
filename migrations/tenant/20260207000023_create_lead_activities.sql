CREATE TABLE lead_activities (
    lead_id     BIGINT NOT NULL REFERENCES leads(id) ON DELETE CASCADE,
    activity_id BIGINT NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    PRIMARY KEY (lead_id, activity_id)
);
