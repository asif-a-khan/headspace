CREATE TABLE web_forms (
    id                       BIGSERIAL PRIMARY KEY,
    form_id                  TEXT NOT NULL UNIQUE,
    title                    TEXT NOT NULL,
    description              TEXT,
    submit_button_label      TEXT NOT NULL DEFAULT 'Submit',
    submit_success_action    TEXT NOT NULL DEFAULT 'message' CHECK (submit_success_action IN ('message', 'redirect')),
    submit_success_content   TEXT NOT NULL DEFAULT 'Thank you for your submission.',
    create_lead              BOOLEAN NOT NULL DEFAULT true,
    background_color         TEXT DEFAULT '#F7F8F9',
    form_background_color    TEXT DEFAULT '#FFFFFF',
    form_title_color         TEXT DEFAULT '#263238',
    form_submit_button_color TEXT DEFAULT '#0E90D9',
    attribute_label_color    TEXT DEFAULT '#546E7A',
    created_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE web_form_attributes (
    id           BIGSERIAL PRIMARY KEY,
    name         TEXT,
    placeholder  TEXT,
    is_required  BOOLEAN NOT NULL DEFAULT false,
    sort_order   INT DEFAULT 0,
    attribute_id BIGINT NOT NULL REFERENCES attributes(id) ON DELETE CASCADE,
    web_form_id  BIGINT NOT NULL REFERENCES web_forms(id) ON DELETE CASCADE
);
