CREATE TABLE leads (
    id                     BIGSERIAL PRIMARY KEY,
    title                  TEXT NOT NULL,
    description            TEXT,
    lead_value             NUMERIC(12,4) DEFAULT 0,
    status                 BOOLEAN,
    lost_reason            TEXT,
    closed_at              TIMESTAMPTZ,
    expected_close_date    DATE,
    user_id                BIGINT REFERENCES users(id) ON DELETE SET NULL,
    person_id              BIGINT REFERENCES persons(id) ON DELETE SET NULL,
    lead_source_id         BIGINT REFERENCES lead_sources(id) ON DELETE SET NULL,
    lead_type_id           BIGINT REFERENCES lead_types(id) ON DELETE SET NULL,
    lead_pipeline_id       BIGINT REFERENCES lead_pipelines(id) ON DELETE SET NULL,
    lead_pipeline_stage_id BIGINT REFERENCES lead_pipeline_stages(id) ON DELETE SET NULL,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_leads_pipeline_stage ON leads(lead_pipeline_id, lead_pipeline_stage_id);
CREATE INDEX idx_leads_user ON leads(user_id);
CREATE INDEX idx_leads_status ON leads(status);
