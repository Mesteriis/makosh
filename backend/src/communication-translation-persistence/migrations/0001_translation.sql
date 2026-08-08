CREATE TABLE makosh_data.communication_translation_runs (
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    operation_id BYTEA NOT NULL,
    request_fingerprint BYTEA NOT NULL,
    source_message_id BYTEA NOT NULL,
    expected_source_revision BIGINT NOT NULL,
    target_language SMALLINT NOT NULL,
    state SMALLINT NOT NULL,
    state_revision BIGINT NOT NULL,
    source_evidence_id BYTEA,
    source_evidence_revision BIGINT,
    source_sha256 BYTEA,
    inference_request_digest BYTEA,
    inference_request_bytes BYTEA,
    source_cleanup_reference_id BYTEA,
    source_cleanup_declared_bytes BIGINT,
    source_cleanup_sha256 BYTEA,
    source_cleanup_custody_proof BYTEA,
    cleanup_completed_at_unix_millis BIGINT,
    candidate_translated_text_utf8 BYTEA,
    candidate_detected_source_language SMALLINT,
    candidate_target_language SMALLINT,
    candidate_completeness SMALLINT,
    candidate_confidence_basis_points INTEGER,
    rejection_code SMALLINT,
    created_at_unix_millis BIGINT NOT NULL,
    updated_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, run_id),
    UNIQUE (logical_owner_id, operation_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(run_id) = 16),
    CHECK (length(operation_id) = 16),
    CHECK (length(request_fingerprint) = 32),
    CHECK (length(source_message_id) = 16),
    CHECK (expected_source_revision > 0),
    CHECK (target_language BETWEEN 1 AND 3),
    CHECK (state BETWEEN 1 AND 5),
    CHECK (state_revision > 0),
    CHECK (created_at_unix_millis > 0),
    CHECK (updated_at_unix_millis >= created_at_unix_millis),
    CHECK (
        (state IN (1, 2) AND source_evidence_id IS NULL
          AND source_evidence_revision IS NULL AND source_sha256 IS NULL
          AND inference_request_digest IS NULL)
        OR
        (state IN (3, 4) AND length(source_evidence_id) = 16
          AND source_evidence_revision > 0 AND length(source_sha256) = 32
          AND length(inference_request_digest) = 32)
        OR state = 5
    ),
    CHECK (
        (
            inference_request_bytes IS NULL
            AND source_cleanup_reference_id IS NULL
            AND source_cleanup_declared_bytes IS NULL
            AND source_cleanup_sha256 IS NULL
            AND source_cleanup_custody_proof IS NULL
        )
        OR
        (
            length(inference_request_bytes) BETWEEN 1 AND 16384
            AND length(source_cleanup_reference_id) = 16
            AND source_cleanup_declared_bytes BETWEEN 1 AND 262144
            AND length(source_cleanup_sha256) = 32
            AND length(source_cleanup_custody_proof) BETWEEN 1 AND 2048
        )
    ),
    CHECK (
        (state IN (1, 2, 3) AND cleanup_completed_at_unix_millis IS NULL)
        OR state IN (4, 5)
    ),
    CHECK (
        (state IN (1, 2) AND inference_request_bytes IS NULL)
        OR (state = 3 AND inference_request_bytes IS NOT NULL)
        OR state IN (4, 5)
    ),
    CHECK (
        state != 4
        OR inference_request_bytes IS NOT NULL
        OR cleanup_completed_at_unix_millis IS NOT NULL
    ),
    CHECK (
        cleanup_completed_at_unix_millis IS NULL
        OR cleanup_completed_at_unix_millis >= created_at_unix_millis
    ),
    CHECK (
        (state = 4 AND candidate_translated_text_utf8 IS NOT NULL
          AND length(candidate_translated_text_utf8) BETWEEN 1 AND 65536
          AND candidate_detected_source_language BETWEEN 1 AND 4
          AND candidate_target_language BETWEEN 1 AND 3
          AND candidate_completeness IN (1, 2)
          AND candidate_confidence_basis_points BETWEEN 0 AND 10000
          AND rejection_code IS NULL)
        OR
        (state = 5 AND candidate_translated_text_utf8 IS NULL AND rejection_code BETWEEN 1 AND 4)
        OR
        (state IN (1, 2, 3) AND candidate_translated_text_utf8 IS NULL AND rejection_code IS NULL)
    )
);

CREATE INDEX communication_translation_recoverable_idx
ON makosh_data.communication_translation_runs (
    logical_owner_id,
    state,
    state_revision
);

CREATE TABLE makosh_data.communication_translation_inbox (
    logical_owner_id TEXT NOT NULL,
    result_message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    run_id BYTEA NOT NULL,
    processed_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, result_message_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(result_message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (length(run_id) = 16),
    CHECK (processed_at_unix_millis > 0)
);

CREATE TABLE makosh_data.communication_translation_outbox (
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
    CHECK (
        published_at_unix_millis IS NULL
        OR published_at_unix_millis >= created_at_unix_millis
    )
);

CREATE INDEX communication_translation_outbox_pending_idx
ON makosh_data.communication_translation_outbox (
    logical_owner_id,
    created_at_unix_millis,
    message_id
)
WHERE published_at_unix_millis IS NULL;

CREATE TABLE makosh_data.communication_translation_realtime (
    realtime_sequence BIGSERIAL PRIMARY KEY,
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    state SMALLINT NOT NULL,
    state_revision BIGINT NOT NULL,
    rejection_code SMALLINT,
    occurred_at_unix_millis BIGINT NOT NULL,
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(run_id) = 16),
    CHECK (state BETWEEN 1 AND 5),
    CHECK (state_revision > 0),
    CHECK (rejection_code IS NULL OR rejection_code BETWEEN 1 AND 4),
    CHECK (occurred_at_unix_millis > 0)
);

CREATE INDEX communication_translation_realtime_owner_idx
ON makosh_data.communication_translation_realtime (
    logical_owner_id,
    realtime_sequence
);
