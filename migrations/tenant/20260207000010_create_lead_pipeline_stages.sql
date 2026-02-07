CREATE TABLE lead_pipeline_stages (
    id               BIGSERIAL PRIMARY KEY,
    probability      INTEGER NOT NULL DEFAULT 100,
    sort_order       INTEGER NOT NULL DEFAULT 0,
    lead_stage_id    BIGINT NOT NULL REFERENCES lead_stages(id) ON DELETE CASCADE,
    lead_pipeline_id BIGINT NOT NULL REFERENCES lead_pipelines(id) ON DELETE CASCADE
);
