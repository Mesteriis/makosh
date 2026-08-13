CREATE TABLE makosh_data.obligations_reviewed_candidate_inbox (
    logical_owner_id TEXT NOT NULL,
    command_message_id BYTEA NOT NULL,
    command_envelope_sha256 BYTEA NOT NULL,
    command_id BYTEA NOT NULL,
    command_fingerprint BYTEA NOT NULL,
    approved_candidate_id BYTEA NOT NULL,
    candidate_digest BYTEA NOT NULL,
    source_evidence_id BYTEA NOT NULL,
    source_evidence_revision BIGINT NOT NULL,
    review_id BYTEA NOT NULL,
    decision_revision BIGINT NOT NULL,
    decided_by_owner_device_id BYTEA NOT NULL,
    candidate_blob_reference_id BYTEA NOT NULL,
    candidate_blob_declared_bytes BIGINT NOT NULL,
    candidate_blob_sha256 BYTEA NOT NULL,
    candidate_blob_custody_proof BYTEA NOT NULL,
    materialized_blob_reference_id BYTEA,
    cleanup_completed_at_unix_millis BIGINT,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    rejected BOOLEAN NOT NULL DEFAULT FALSE,
    obligation_id BYTEA,
    received_at_unix_millis BIGINT NOT NULL,
    completed_at_unix_millis BIGINT,
    PRIMARY KEY (logical_owner_id, command_message_id),
    UNIQUE (logical_owner_id, command_id),
    UNIQUE (logical_owner_id, approved_candidate_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(command_message_id) = 16),
    CHECK (length(command_envelope_sha256) = 32),
    CHECK (length(command_id) = 16),
    CHECK (length(command_fingerprint) = 32),
    CHECK (length(approved_candidate_id) = 16),
    CHECK (length(candidate_digest) = 32),
    CHECK (length(source_evidence_id) = 16),
    CHECK (source_evidence_revision > 0),
    CHECK (length(review_id) = 16),
    CHECK (decision_revision > 0),
    CHECK (length(decided_by_owner_device_id) = 16),
    CHECK (length(candidate_blob_reference_id) = 16),
    CHECK (candidate_blob_declared_bytes BETWEEN 1 AND 16384),
    CHECK (length(candidate_blob_sha256) = 32),
    CHECK (length(candidate_blob_custody_proof) BETWEEN 1 AND 2048),
    CHECK (materialized_blob_reference_id IS NULL OR length(materialized_blob_reference_id) = 16),
    CHECK (received_at_unix_millis > 0),
    CHECK (cleanup_completed_at_unix_millis IS NULL OR (
        materialized_blob_reference_id IS NOT NULL
        AND cleanup_completed_at_unix_millis >= received_at_unix_millis
    )),
    CHECK (
        (NOT completed AND NOT rejected AND obligation_id IS NULL AND completed_at_unix_millis IS NULL)
        OR (completed AND NOT rejected AND length(obligation_id) = 16
            AND completed_at_unix_millis >= received_at_unix_millis)
        OR (completed AND rejected AND obligation_id IS NULL
            AND completed_at_unix_millis >= received_at_unix_millis)
    )
);

CREATE INDEX obligations_reviewed_candidate_recovery_idx
ON makosh_data.obligations_reviewed_candidate_inbox (
    logical_owner_id,
    completed,
    received_at_unix_millis
);

CREATE TABLE makosh_data.obligations_state (
    logical_owner_id TEXT NOT NULL,
    obligation_id BYTEA NOT NULL,
    statement TEXT NOT NULL,
    due_text_hint TEXT,
    condition TEXT,
    status SMALLINT NOT NULL,
    obligation_revision BIGINT NOT NULL,
    approved_candidate_id BYTEA NOT NULL,
    candidate_digest BYTEA NOT NULL,
    source_evidence_id BYTEA NOT NULL,
    source_evidence_revision BIGINT NOT NULL,
    review_id BYTEA NOT NULL,
    decision_revision BIGINT NOT NULL,
    decided_by_owner_device_id BYTEA NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL,
    created_at_nanos INTEGER NOT NULL,
    updated_at_unix_seconds BIGINT NOT NULL,
    updated_at_nanos INTEGER NOT NULL,
    PRIMARY KEY (logical_owner_id, obligation_id),
    UNIQUE (logical_owner_id, approved_candidate_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(obligation_id) = 16),
    CHECK (char_length(statement) BETWEEN 1 AND 240),
    CHECK (due_text_hint IS NULL OR char_length(due_text_hint) BETWEEN 1 AND 120),
    CHECK (condition IS NULL OR char_length(condition) BETWEEN 1 AND 120),
    CHECK (status = 1),
    CHECK (obligation_revision > 0),
    CHECK (length(approved_candidate_id) = 16),
    CHECK (length(candidate_digest) = 32),
    CHECK (length(source_evidence_id) = 16),
    CHECK (source_evidence_revision > 0),
    CHECK (length(review_id) = 16),
    CHECK (decision_revision > 0),
    CHECK (length(decided_by_owner_device_id) = 16),
    CHECK (created_at_unix_seconds > 0),
    CHECK (created_at_nanos BETWEEN 0 AND 999999999),
    CHECK (updated_at_unix_seconds >= created_at_unix_seconds),
    CHECK (updated_at_nanos BETWEEN 0 AND 999999999)
);

CREATE TABLE makosh_data.obligations_outbox (
    logical_owner_id TEXT NOT NULL,
    message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    envelope_bytes BYTEA NOT NULL,
    created_at_unix_millis BIGINT NOT NULL,
    published_at_unix_millis BIGINT,
    PRIMARY KEY (logical_owner_id, message_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (length(envelope_bytes) BETWEEN 1 AND 65536),
    CHECK (created_at_unix_millis > 0),
    CHECK (published_at_unix_millis IS NULL OR published_at_unix_millis >= created_at_unix_millis)
);

CREATE INDEX obligations_outbox_pending_idx
ON makosh_data.obligations_outbox (logical_owner_id, created_at_unix_millis, message_id)
WHERE published_at_unix_millis IS NULL;
