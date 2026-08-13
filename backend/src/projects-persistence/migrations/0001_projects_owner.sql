CREATE TABLE makosh_data.projects_records (
    logical_owner_id TEXT NOT NULL,
    project_id BYTEA NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    project_state SMALLINT NOT NULL,
    start_at_unix_seconds BIGINT,
    start_at_nanos INTEGER,
    target_at_unix_seconds BIGINT,
    target_at_nanos INTEGER,
    project_revision BIGINT NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL,
    created_at_nanos INTEGER NOT NULL,
    updated_at_unix_seconds BIGINT NOT NULL,
    updated_at_nanos INTEGER NOT NULL,
    PRIMARY KEY (logical_owner_id, project_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(project_id) = 16),
    CHECK (char_length(name) BETWEEN 1 AND 240),
    CHECK (char_length(description) <= 8000),
    CHECK (project_state BETWEEN 1 AND 5),
    CHECK ((start_at_unix_seconds IS NULL) = (start_at_nanos IS NULL)),
    CHECK ((target_at_unix_seconds IS NULL) = (target_at_nanos IS NULL)),
    CHECK (start_at_unix_seconds IS NULL OR start_at_unix_seconds > 0),
    CHECK (target_at_unix_seconds IS NULL OR target_at_unix_seconds > 0),
    CHECK (start_at_nanos IS NULL OR start_at_nanos BETWEEN 0 AND 999999999),
    CHECK (target_at_nanos IS NULL OR target_at_nanos BETWEEN 0 AND 999999999),
    CHECK (start_at_unix_seconds IS NULL OR target_at_unix_seconds IS NULL OR target_at_unix_seconds > start_at_unix_seconds),
    CHECK (project_revision > 0),
    CHECK (created_at_unix_seconds > 0),
    CHECK (created_at_nanos BETWEEN 0 AND 999999999),
    CHECK (updated_at_unix_seconds >= created_at_unix_seconds),
    CHECK (updated_at_nanos BETWEEN 0 AND 999999999)
);

CREATE TABLE makosh_data.projects_outcomes (
    logical_owner_id TEXT NOT NULL,
    project_id BYTEA NOT NULL,
    outcome_id BYTEA NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    outcome_state SMALLINT NOT NULL,
    target_at_unix_seconds BIGINT,
    target_at_nanos INTEGER,
    outcome_revision BIGINT NOT NULL,
    updated_at_project_revision BIGINT NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL,
    created_at_nanos INTEGER NOT NULL,
    updated_at_unix_seconds BIGINT NOT NULL,
    updated_at_nanos INTEGER NOT NULL,
    PRIMARY KEY (logical_owner_id, project_id, outcome_id),
    FOREIGN KEY (logical_owner_id, project_id)
        REFERENCES makosh_data.projects_records (logical_owner_id, project_id) ON DELETE CASCADE,
    CHECK (length(outcome_id) = 16),
    CHECK (char_length(title) BETWEEN 1 AND 320),
    CHECK (char_length(description) <= 8000),
    CHECK (outcome_state BETWEEN 1 AND 4),
    CHECK ((target_at_unix_seconds IS NULL) = (target_at_nanos IS NULL)),
    CHECK (target_at_unix_seconds IS NULL OR target_at_unix_seconds > 0),
    CHECK (target_at_nanos IS NULL OR target_at_nanos BETWEEN 0 AND 999999999),
    CHECK (outcome_revision > 0),
    CHECK (updated_at_project_revision > 0),
    CHECK (created_at_unix_seconds > 0),
    CHECK (created_at_nanos BETWEEN 0 AND 999999999),
    CHECK (updated_at_unix_seconds >= created_at_unix_seconds),
    CHECK (updated_at_nanos BETWEEN 0 AND 999999999)
);

CREATE TABLE makosh_data.projects_references (
    logical_owner_id TEXT NOT NULL,
    project_id BYTEA NOT NULL,
    reference_id BYTEA NOT NULL,
    reference_kind SMALLINT NOT NULL,
    public_id BYTEA NOT NULL,
    label TEXT NOT NULL,
    reference_state SMALLINT NOT NULL,
    updated_at_project_revision BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, project_id, reference_id),
    UNIQUE (logical_owner_id, project_id, reference_kind, public_id),
    FOREIGN KEY (logical_owner_id, project_id)
        REFERENCES makosh_data.projects_records (logical_owner_id, project_id) ON DELETE CASCADE,
    CHECK (length(reference_id) = 16),
    CHECK (reference_kind BETWEEN 1 AND 6),
    CHECK (length(public_id) = 16),
    CHECK (char_length(label) <= 320),
    CHECK (reference_state BETWEEN 1 AND 2),
    CHECK (updated_at_project_revision > 0)
);

CREATE TABLE makosh_data.projects_client_operations (
    logical_owner_id TEXT NOT NULL,
    operation_id BYTEA NOT NULL,
    operation_kind SMALLINT NOT NULL,
    request_sha256 BYTEA NOT NULL,
    request_bytes BYTEA NOT NULL,
    project_id BYTEA NOT NULL,
    project_revision BIGINT NOT NULL,
    response_sha256 BYTEA NOT NULL,
    response_bytes BYTEA NOT NULL,
    received_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, operation_id),
    FOREIGN KEY (logical_owner_id, project_id)
        REFERENCES makosh_data.projects_records (logical_owner_id, project_id) ON DELETE CASCADE,
    CHECK (operation_kind BETWEEN 1 AND 9),
    CHECK (length(operation_id) = 16),
    CHECK (length(request_sha256) = 32),
    CHECK (length(request_bytes) BETWEEN 1 AND 65536),
    CHECK (project_revision > 0),
    CHECK (length(response_sha256) = 32),
    CHECK (length(response_bytes) BETWEEN 1 AND 65536),
    CHECK (received_at_unix_millis > 0)
);

CREATE TABLE makosh_data.projects_outbox (
    logical_owner_id TEXT NOT NULL,
    outbox_sequence BIGSERIAL NOT NULL,
    message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    envelope_bytes BYTEA NOT NULL,
    created_at_unix_millis BIGINT NOT NULL,
    published_at_unix_millis BIGINT,
    PRIMARY KEY (logical_owner_id, message_id),
    UNIQUE (logical_owner_id, outbox_sequence),
    CHECK (length(message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (length(envelope_bytes) BETWEEN 1 AND 65536),
    CHECK (created_at_unix_millis > 0),
    CHECK (published_at_unix_millis IS NULL OR published_at_unix_millis >= created_at_unix_millis)
);

CREATE INDEX projects_order_idx ON makosh_data.projects_records (logical_owner_id, project_id);
CREATE INDEX projects_outcomes_order_idx ON makosh_data.projects_outcomes (logical_owner_id, project_id, outcome_id);
CREATE INDEX projects_references_order_idx ON makosh_data.projects_references (logical_owner_id, project_id, reference_id);
CREATE INDEX projects_outbox_pending_idx ON makosh_data.projects_outbox (logical_owner_id, outbox_sequence)
WHERE published_at_unix_millis IS NULL;

ALTER TABLE makosh_data.projects_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.projects_records FORCE ROW LEVEL SECURITY;
CREATE POLICY projects_records_owner_policy ON makosh_data.projects_records
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.projects_outcomes ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.projects_outcomes FORCE ROW LEVEL SECURITY;
CREATE POLICY projects_outcomes_owner_policy ON makosh_data.projects_outcomes
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.projects_references ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.projects_references FORCE ROW LEVEL SECURITY;
CREATE POLICY projects_references_owner_policy ON makosh_data.projects_references
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.projects_client_operations ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.projects_client_operations FORCE ROW LEVEL SECURITY;
CREATE POLICY projects_client_operations_owner_policy ON makosh_data.projects_client_operations
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.projects_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.projects_outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY projects_outbox_owner_policy ON makosh_data.projects_outbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
