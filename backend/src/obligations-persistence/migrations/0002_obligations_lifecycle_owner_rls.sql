ALTER TABLE makosh_data.obligations_state
    ADD COLUMN due_at_unix_seconds BIGINT,
    ADD COLUMN due_at_nanos INTEGER,
    ADD COLUMN priority SMALLINT NOT NULL DEFAULT 2;

ALTER TABLE makosh_data.obligations_state
    DROP CONSTRAINT obligations_state_status_check,
    DROP CONSTRAINT obligations_state_condition_check,
    ALTER COLUMN approved_candidate_id DROP NOT NULL,
    ALTER COLUMN candidate_digest DROP NOT NULL,
    ALTER COLUMN source_evidence_id DROP NOT NULL,
    ALTER COLUMN source_evidence_revision DROP NOT NULL,
    ALTER COLUMN review_id DROP NOT NULL,
    ALTER COLUMN decision_revision DROP NOT NULL,
    ALTER COLUMN decided_by_owner_device_id DROP NOT NULL;

ALTER TABLE makosh_data.obligations_state
    ADD CONSTRAINT obligations_state_lifecycle_status_check CHECK (status BETWEEN 1 AND 5),
    ADD CONSTRAINT obligations_state_priority_check CHECK (priority BETWEEN 1 AND 4),
    ADD CONSTRAINT obligations_state_condition_check CHECK (
        condition IS NULL OR char_length(condition) BETWEEN 1 AND 4000
    ),
    ADD CONSTRAINT obligations_state_due_at_check CHECK (
        (due_at_unix_seconds IS NULL AND due_at_nanos IS NULL)
        OR (due_at_unix_seconds > 0 AND due_at_nanos BETWEEN 0 AND 999999999)
    ),
    ADD CONSTRAINT obligations_state_provenance_shape_check CHECK (
        (approved_candidate_id IS NULL
            AND candidate_digest IS NULL
            AND source_evidence_id IS NULL
            AND source_evidence_revision IS NULL
            AND review_id IS NULL
            AND decision_revision IS NULL
            AND decided_by_owner_device_id IS NULL)
        OR (length(approved_candidate_id) = 16
            AND length(candidate_digest) = 32
            AND length(source_evidence_id) = 16
            AND source_evidence_revision > 0
            AND length(review_id) = 16
            AND decision_revision > 0
            AND length(decided_by_owner_device_id) = 16)
    );

CREATE TABLE makosh_data.obligations_dependencies (
    logical_owner_id TEXT NOT NULL,
    obligation_id BYTEA NOT NULL,
    dependency_id BYTEA NOT NULL,
    depends_on_obligation_id BYTEA NOT NULL,
    created_at_obligation_revision BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, obligation_id, dependency_id),
    UNIQUE (logical_owner_id, obligation_id, depends_on_obligation_id),
    FOREIGN KEY (logical_owner_id, obligation_id)
        REFERENCES makosh_data.obligations_state (logical_owner_id, obligation_id) ON DELETE CASCADE,
    FOREIGN KEY (logical_owner_id, depends_on_obligation_id)
        REFERENCES makosh_data.obligations_state (logical_owner_id, obligation_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(obligation_id) = 16),
    CHECK (length(dependency_id) = 16),
    CHECK (length(depends_on_obligation_id) = 16),
    CHECK (obligation_id <> depends_on_obligation_id),
    CHECK (created_at_obligation_revision > 0)
);

CREATE INDEX obligations_dependencies_reverse_idx
ON makosh_data.obligations_dependencies (logical_owner_id, depends_on_obligation_id, obligation_id);

CREATE TABLE makosh_data.obligations_checklist (
    logical_owner_id TEXT NOT NULL,
    obligation_id BYTEA NOT NULL,
    checklist_item_id BYTEA NOT NULL,
    label TEXT NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    position INTEGER NOT NULL,
    updated_at_obligation_revision BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, obligation_id, checklist_item_id),
    FOREIGN KEY (logical_owner_id, obligation_id)
        REFERENCES makosh_data.obligations_state (logical_owner_id, obligation_id) ON DELETE CASCADE,
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(obligation_id) = 16),
    CHECK (length(checklist_item_id) = 16),
    CHECK (char_length(label) BETWEEN 1 AND 240),
    CHECK (position >= 0),
    CHECK (updated_at_obligation_revision > 0)
);

CREATE INDEX obligations_checklist_order_idx
ON makosh_data.obligations_checklist (logical_owner_id, obligation_id, position, checklist_item_id);

CREATE TABLE makosh_data.obligations_client_operations (
    logical_owner_id TEXT NOT NULL,
    operation_id BYTEA NOT NULL,
    operation_kind SMALLINT NOT NULL,
    request_sha256 BYTEA NOT NULL,
    request_bytes BYTEA NOT NULL,
    obligation_id BYTEA NOT NULL,
    obligation_revision BIGINT NOT NULL,
    response_sha256 BYTEA NOT NULL,
    response_bytes BYTEA NOT NULL,
    received_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, operation_id),
    FOREIGN KEY (logical_owner_id, obligation_id)
        REFERENCES makosh_data.obligations_state (logical_owner_id, obligation_id) ON DELETE CASCADE,
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(operation_id) = 16),
    CHECK (operation_kind BETWEEN 1 AND 9),
    CHECK (length(request_sha256) = 32),
    CHECK (length(request_bytes) BETWEEN 1 AND 65536),
    CHECK (length(obligation_id) = 16),
    CHECK (obligation_revision > 0),
    CHECK (length(response_sha256) = 32),
    CHECK (length(response_bytes) BETWEEN 1 AND 65536),
    CHECK (received_at_unix_millis > 0)
);

ALTER TABLE makosh_data.obligations_reviewed_candidate_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.obligations_reviewed_candidate_inbox FORCE ROW LEVEL SECURITY;
CREATE POLICY obligations_reviewed_candidate_inbox_owner_policy
ON makosh_data.obligations_reviewed_candidate_inbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.obligations_state ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.obligations_state FORCE ROW LEVEL SECURITY;
CREATE POLICY obligations_state_owner_policy
ON makosh_data.obligations_state
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.obligations_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.obligations_outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY obligations_outbox_owner_policy
ON makosh_data.obligations_outbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.obligations_dependencies ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.obligations_dependencies FORCE ROW LEVEL SECURITY;
CREATE POLICY obligations_dependencies_owner_policy
ON makosh_data.obligations_dependencies
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.obligations_checklist ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.obligations_checklist FORCE ROW LEVEL SECURITY;
CREATE POLICY obligations_checklist_owner_policy
ON makosh_data.obligations_checklist
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.obligations_client_operations ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.obligations_client_operations FORCE ROW LEVEL SECURITY;
CREATE POLICY obligations_client_operations_owner_policy
ON makosh_data.obligations_client_operations
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
