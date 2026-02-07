CREATE TABLE activity_participants (
    id          BIGSERIAL PRIMARY KEY,
    activity_id BIGINT NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    user_id     BIGINT REFERENCES users(id) ON DELETE CASCADE,
    person_id   BIGINT REFERENCES persons(id) ON DELETE CASCADE
);
