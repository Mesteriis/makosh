ALTER TABLE makosh_data.knowledge_state
    ADD COLUMN lifecycle_state SMALLINT NOT NULL DEFAULT 1,
    ADD COLUMN origin_kind SMALLINT NOT NULL DEFAULT 1;

ALTER TABLE makosh_data.knowledge_state
    DROP CONSTRAINT knowledge_state_excerpt_check,
    DROP CONSTRAINT knowledge_state_note_revision_check,
    ALTER COLUMN topic_hints DROP NOT NULL,
    ALTER COLUMN source_basis DROP NOT NULL,
    ALTER COLUMN confidence_basis_points DROP NOT NULL,
    ALTER COLUMN approved_candidate_id DROP NOT NULL,
    ALTER COLUMN candidate_digest DROP NOT NULL,
    ALTER COLUMN source_evidence_id DROP NOT NULL,
    ALTER COLUMN source_evidence_revision DROP NOT NULL,
    ALTER COLUMN review_id DROP NOT NULL,
    ALTER COLUMN decision_revision DROP NOT NULL,
    ALTER COLUMN decided_by_owner_device_id DROP NOT NULL;

ALTER TABLE makosh_data.knowledge_state
    ADD CONSTRAINT knowledge_state_body_check CHECK (char_length(excerpt) BETWEEN 1 AND 16000),
    ADD CONSTRAINT knowledge_state_lifecycle_check CHECK (lifecycle_state BETWEEN 1 AND 2),
    ADD CONSTRAINT knowledge_state_origin_check CHECK (origin_kind BETWEEN 1 AND 2),
    ADD CONSTRAINT knowledge_state_revision_check CHECK (note_revision > 0),
    ADD CONSTRAINT knowledge_state_origin_shape_check CHECK (
        (origin_kind = 2
            AND topic_hints IS NULL
            AND source_basis IS NULL
            AND confidence_basis_points IS NULL
            AND approved_candidate_id IS NULL
            AND candidate_digest IS NULL
            AND source_evidence_id IS NULL
            AND source_evidence_revision IS NULL
            AND review_id IS NULL
            AND decision_revision IS NULL
            AND decided_by_owner_device_id IS NULL)
        OR (origin_kind = 1
            AND cardinality(topic_hints) BETWEEN 1 AND 4
            AND source_basis BETWEEN 1 AND 3
            AND confidence_basis_points BETWEEN 1 AND 10000
            AND length(approved_candidate_id) = 16
            AND length(candidate_digest) = 32
            AND length(source_evidence_id) = 16
            AND source_evidence_revision > 0
            AND length(review_id) = 16
            AND decision_revision > 0
            AND length(decided_by_owner_device_id) = 16)
    );

CREATE TABLE makosh_data.knowledge_sources (
    logical_owner_id TEXT NOT NULL,
    note_id BYTEA NOT NULL,
    source_id BYTEA NOT NULL,
    source_owner_id TEXT NOT NULL,
    source_record_id BYTEA NOT NULL,
    source_revision BIGINT NOT NULL,
    evidence_digest BYTEA NOT NULL,
    source_state SMALLINT NOT NULL,
    updated_at_note_revision BIGINT NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL,
    created_at_nanos INTEGER NOT NULL,
    updated_at_unix_seconds BIGINT NOT NULL,
    updated_at_nanos INTEGER NOT NULL,
    PRIMARY KEY (logical_owner_id, note_id, source_id),
    UNIQUE (logical_owner_id, note_id, source_owner_id, source_record_id),
    FOREIGN KEY (logical_owner_id, note_id)
        REFERENCES makosh_data.knowledge_state (logical_owner_id, note_id) ON DELETE CASCADE,
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(note_id) = 16),
    CHECK (length(source_id) = 16),
    CHECK (length(source_owner_id) BETWEEN 1 AND 128),
    CHECK (length(source_record_id) = 16),
    CHECK (source_revision > 0),
    CHECK (length(evidence_digest) = 32),
    CHECK (source_state BETWEEN 1 AND 2),
    CHECK (updated_at_note_revision > 0),
    CHECK (created_at_unix_seconds > 0),
    CHECK (created_at_nanos BETWEEN 0 AND 999999999),
    CHECK (updated_at_unix_seconds >= created_at_unix_seconds),
    CHECK (updated_at_nanos BETWEEN 0 AND 999999999)
);

CREATE INDEX knowledge_sources_order_idx
ON makosh_data.knowledge_sources (logical_owner_id, note_id, source_id);

CREATE TABLE makosh_data.knowledge_client_operations (
    logical_owner_id TEXT NOT NULL,
    operation_id BYTEA NOT NULL,
    operation_kind SMALLINT NOT NULL,
    request_sha256 BYTEA NOT NULL,
    request_bytes BYTEA NOT NULL,
    note_id BYTEA NOT NULL,
    note_revision BIGINT NOT NULL,
    response_sha256 BYTEA NOT NULL,
    response_bytes BYTEA NOT NULL,
    received_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, operation_id),
    FOREIGN KEY (logical_owner_id, note_id)
        REFERENCES makosh_data.knowledge_state (logical_owner_id, note_id) ON DELETE CASCADE,
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(operation_id) = 16),
    CHECK (operation_kind BETWEEN 1 AND 5),
    CHECK (length(request_sha256) = 32),
    CHECK (length(request_bytes) BETWEEN 1 AND 65536),
    CHECK (length(note_id) = 16),
    CHECK (note_revision > 0),
    CHECK (length(response_sha256) = 32),
    CHECK (length(response_bytes) BETWEEN 1 AND 65536),
    CHECK (received_at_unix_millis > 0)
);

ALTER TABLE makosh_data.knowledge_outbox
    ADD COLUMN outbox_sequence BIGSERIAL;

ALTER TABLE makosh_data.knowledge_outbox
    ADD CONSTRAINT knowledge_outbox_owner_sequence_unique
    UNIQUE (logical_owner_id, outbox_sequence);

DROP INDEX makosh_data.knowledge_outbox_pending_idx;
CREATE INDEX knowledge_outbox_pending_idx
ON makosh_data.knowledge_outbox (logical_owner_id, outbox_sequence)
WHERE published_at_unix_millis IS NULL;

ALTER TABLE makosh_data.knowledge_reviewed_candidate_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.knowledge_reviewed_candidate_inbox FORCE ROW LEVEL SECURITY;
CREATE POLICY knowledge_reviewed_candidate_inbox_owner_policy
ON makosh_data.knowledge_reviewed_candidate_inbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
ALTER TABLE makosh_data.knowledge_state ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.knowledge_state FORCE ROW LEVEL SECURITY;
CREATE POLICY knowledge_state_owner_policy
ON makosh_data.knowledge_state
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.knowledge_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.knowledge_outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY knowledge_outbox_owner_policy
ON makosh_data.knowledge_outbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.knowledge_sources ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.knowledge_sources FORCE ROW LEVEL SECURITY;
CREATE POLICY knowledge_sources_owner_policy
ON makosh_data.knowledge_sources
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.knowledge_client_operations ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.knowledge_client_operations FORCE ROW LEVEL SECURITY;
CREATE POLICY knowledge_client_operations_owner_policy
ON makosh_data.knowledge_client_operations
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
