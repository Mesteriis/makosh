CREATE TABLE makosh_data.knowledge_reviewed_candidate_inbox (
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
    materialized_blob_declared_bytes BIGINT,
    materialized_blob_sha256 BYTEA,
    materialized_blob_custody_proof BYTEA,
    cleanup_completed_at_unix_millis BIGINT,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    rejected BOOLEAN NOT NULL DEFAULT FALSE,
    note_id BYTEA,
    note_creation_fingerprint BYTEA,
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
    CHECK (
        (materialized_blob_reference_id IS NULL
            AND materialized_blob_declared_bytes IS NULL
            AND materialized_blob_sha256 IS NULL
            AND materialized_blob_custody_proof IS NULL)
        OR (length(materialized_blob_reference_id) = 16
            AND materialized_blob_declared_bytes BETWEEN 1 AND 16384
            AND length(materialized_blob_sha256) = 32
            AND length(materialized_blob_custody_proof) BETWEEN 1 AND 2048)
    ),
    CHECK (note_creation_fingerprint IS NULL OR length(note_creation_fingerprint) = 32),
    CHECK (received_at_unix_millis > 0),
    CHECK (cleanup_completed_at_unix_millis IS NULL OR (
        materialized_blob_reference_id IS NOT NULL
        AND cleanup_completed_at_unix_millis >= received_at_unix_millis
    )),
    CHECK (
        (NOT completed AND NOT rejected AND note_id IS NULL
            AND note_creation_fingerprint IS NULL AND completed_at_unix_millis IS NULL)
        OR (completed AND NOT rejected AND length(note_id) = 16
            AND length(note_creation_fingerprint) = 32
            AND completed_at_unix_millis >= received_at_unix_millis)
        OR (completed AND rejected AND note_id IS NULL AND note_creation_fingerprint IS NULL
            AND completed_at_unix_millis >= received_at_unix_millis)
    )
);

CREATE INDEX knowledge_reviewed_candidate_recovery_idx
ON makosh_data.knowledge_reviewed_candidate_inbox (
    logical_owner_id,
    completed,
    received_at_unix_millis
);

CREATE TABLE makosh_data.knowledge_state (
    logical_owner_id TEXT NOT NULL,
    note_id BYTEA NOT NULL,
    title TEXT NOT NULL,
    excerpt TEXT NOT NULL,
    topic_hints SMALLINT[] NOT NULL,
    source_basis SMALLINT NOT NULL,
    confidence_basis_points INTEGER NOT NULL,
    status SMALLINT NOT NULL,
    note_revision BIGINT NOT NULL,
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
    PRIMARY KEY (logical_owner_id, note_id),
    UNIQUE (logical_owner_id, approved_candidate_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(note_id) = 16),
    CHECK (char_length(title) BETWEEN 1 AND 240),
    CHECK (char_length(excerpt) BETWEEN 1 AND 2000),
    CHECK (cardinality(topic_hints) BETWEEN 1 AND 4),
    CHECK (topic_hints <@ ARRAY[1,2,3,4]::SMALLINT[]),
    CHECK (cardinality(topic_hints) < 2 OR topic_hints[1] < topic_hints[2]),
    CHECK (cardinality(topic_hints) < 3 OR topic_hints[2] < topic_hints[3]),
    CHECK (cardinality(topic_hints) < 4 OR topic_hints[3] < topic_hints[4]),
    CHECK (source_basis BETWEEN 1 AND 3),
    CHECK (confidence_basis_points BETWEEN 1 AND 10000),
    CHECK (status = 1),
    CHECK (note_revision = 1),
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

CREATE TABLE makosh_data.knowledge_outbox (
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

CREATE INDEX knowledge_outbox_pending_idx
ON makosh_data.knowledge_outbox (logical_owner_id, created_at_unix_millis, message_id)
WHERE published_at_unix_millis IS NULL;
