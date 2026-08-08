CREATE TABLE makosh_data.attachment_translation_runs (
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    operation_id BYTEA NOT NULL,
    request_fingerprint BYTEA NOT NULL,
    source_extraction_run_id BYTEA NOT NULL,
    expected_source_revision BIGINT NOT NULL,
    target_language SMALLINT NOT NULL,
    state SMALLINT NOT NULL,
    state_revision BIGINT NOT NULL,
    source_sha256 BYTEA,
    inference_request_digest BYTEA,
    inference_request_bytes BYTEA,
    source_reference_id BYTEA,
    source_declared_bytes BIGINT,
    source_receipt_sha256 BYTEA,
    source_custody_proof BYTEA,
    cleanup_completed_at_unix_millis BIGINT,
    pending_translated_sha256 BYTEA,
    pending_translated_size_bytes BIGINT,
    pending_detected_source_language SMALLINT,
    pending_target_language SMALLINT,
    pending_completeness SMALLINT,
    pending_confidence_basis_points INTEGER,
    artifact_id BYTEA,
    artifact_translated_sha256 BYTEA,
    artifact_translated_size_bytes BIGINT,
    artifact_detected_source_language SMALLINT,
    artifact_target_language SMALLINT,
    artifact_completeness SMALLINT,
    artifact_confidence_basis_points INTEGER,
    rejection_code SMALLINT,
    created_at_unix_millis BIGINT NOT NULL,
    updated_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, run_id),
    UNIQUE (logical_owner_id, operation_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(run_id) = 16),
    CHECK (length(operation_id) = 16),
    CHECK (length(request_fingerprint) = 32),
    CHECK (length(source_extraction_run_id) = 16),
    CHECK (expected_source_revision > 0),
    CHECK (target_language BETWEEN 1 AND 3),
    CHECK (state BETWEEN 1 AND 6),
    CHECK (state_revision > 0),
    CHECK (created_at_unix_millis > 0),
    CHECK (updated_at_unix_millis >= created_at_unix_millis),
    CHECK (
        (source_sha256 IS NULL AND inference_request_digest IS NULL)
        OR (length(source_sha256) = 32 AND length(inference_request_digest) = 32)
    ),
    CHECK (
        (inference_request_bytes IS NULL AND source_reference_id IS NULL
          AND source_declared_bytes IS NULL AND source_receipt_sha256 IS NULL
          AND source_custody_proof IS NULL)
        OR (length(inference_request_bytes) BETWEEN 1 AND 16384
          AND length(source_reference_id) = 16
          AND source_declared_bytes BETWEEN 1 AND 1048576
          AND length(source_receipt_sha256) = 32
          AND length(source_custody_proof) BETWEEN 1 AND 2048)
    ),
    CHECK (
        cleanup_completed_at_unix_millis IS NULL
        OR cleanup_completed_at_unix_millis >= created_at_unix_millis
    ),
    CHECK (
        (pending_translated_sha256 IS NULL AND pending_translated_size_bytes IS NULL
          AND pending_detected_source_language IS NULL AND pending_target_language IS NULL
          AND pending_completeness IS NULL AND pending_confidence_basis_points IS NULL)
        OR (length(pending_translated_sha256) = 32
          AND pending_translated_size_bytes BETWEEN 1 AND 65536
          AND pending_detected_source_language BETWEEN 1 AND 4
          AND pending_target_language BETWEEN 1 AND 3
          AND pending_completeness IN (1, 2)
          AND pending_confidence_basis_points BETWEEN 0 AND 10000)
    ),
    CHECK (
        (artifact_id IS NULL AND artifact_translated_sha256 IS NULL
          AND artifact_translated_size_bytes IS NULL
          AND artifact_detected_source_language IS NULL
          AND artifact_target_language IS NULL AND artifact_completeness IS NULL
          AND artifact_confidence_basis_points IS NULL)
        OR (length(artifact_id) = 16 AND length(artifact_translated_sha256) = 32
          AND artifact_translated_size_bytes BETWEEN 1 AND 65536
          AND artifact_detected_source_language BETWEEN 1 AND 4
          AND artifact_target_language BETWEEN 1 AND 3
          AND artifact_completeness IN (1, 2)
          AND artifact_confidence_basis_points BETWEEN 0 AND 10000)
    ),
    CHECK (rejection_code IS NULL OR rejection_code BETWEEN 1 AND 5),
    CHECK (
        (state = 1 AND source_sha256 IS NULL AND pending_translated_sha256 IS NULL
          AND artifact_id IS NULL AND rejection_code IS NULL)
        OR (state = 2 AND source_sha256 IS NULL AND pending_translated_sha256 IS NULL
          AND artifact_id IS NULL AND rejection_code IS NULL)
        OR (state = 3 AND source_sha256 IS NOT NULL AND inference_request_bytes IS NOT NULL
          AND pending_translated_sha256 IS NULL AND artifact_id IS NULL
          AND rejection_code IS NULL)
        OR (state = 4 AND source_sha256 IS NOT NULL
          AND pending_translated_sha256 IS NOT NULL AND artifact_id IS NULL
          AND rejection_code IS NULL)
        OR (state = 5 AND pending_translated_sha256 IS NULL
          AND artifact_id IS NOT NULL AND rejection_code IS NULL)
        OR (state = 6 AND pending_translated_sha256 IS NULL
          AND artifact_id IS NULL AND rejection_code IS NOT NULL)
    )
);

CREATE INDEX attachment_translation_recoverable_idx
ON makosh_data.attachment_translation_runs (
    logical_owner_id,
    state,
    state_revision
);

CREATE TABLE makosh_data.attachment_translation_inbox (
    logical_owner_id TEXT NOT NULL,
    message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    run_id BYTEA NOT NULL,
    processed_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, message_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (length(run_id) = 16),
    CHECK (processed_at_unix_millis > 0)
);

CREATE TABLE makosh_data.attachment_translation_outbox (
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
    CHECK (published_at_unix_millis IS NULL
      OR published_at_unix_millis >= created_at_unix_millis)
);

CREATE INDEX attachment_translation_outbox_pending_idx
ON makosh_data.attachment_translation_outbox (
    logical_owner_id,
    created_at_unix_millis,
    message_id
)
WHERE published_at_unix_millis IS NULL;

CREATE TABLE makosh_data.attachment_translation_realtime (
    realtime_sequence BIGSERIAL PRIMARY KEY,
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    state SMALLINT NOT NULL,
    state_revision BIGINT NOT NULL,
    rejection_code SMALLINT,
    occurred_at_unix_millis BIGINT NOT NULL,
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(run_id) = 16),
    CHECK (state BETWEEN 1 AND 6),
    CHECK (state_revision > 0),
    CHECK (rejection_code IS NULL OR rejection_code BETWEEN 1 AND 5),
    CHECK (occurred_at_unix_millis > 0)
);

CREATE INDEX attachment_translation_realtime_owner_idx
ON makosh_data.attachment_translation_realtime (
    logical_owner_id,
    realtime_sequence
);
