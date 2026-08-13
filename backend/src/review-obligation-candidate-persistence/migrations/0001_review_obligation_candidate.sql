CREATE TABLE makosh_data.review_obligation_candidate_submissions (
    logical_owner_id TEXT NOT NULL,
    submission_message_id BYTEA NOT NULL,
    submission_envelope_sha256 BYTEA NOT NULL,
    submission_id BYTEA NOT NULL,
    candidate_id BYTEA NOT NULL,
    candidate_digest BYTEA NOT NULL,
    source_evidence_id BYTEA NOT NULL,
    source_evidence_revision BIGINT NOT NULL,
    candidate_blob_reference_id BYTEA NOT NULL,
    candidate_blob_declared_bytes BIGINT NOT NULL,
    candidate_blob_sha256 BYTEA NOT NULL,
    candidate_blob_custody_proof BYTEA NOT NULL,
    materialized_blob_reference_id BYTEA,
    cleanup_completed_at_unix_millis BIGINT,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    rejected BOOLEAN NOT NULL DEFAULT FALSE,
    review_id BYTEA,
    received_at_unix_millis BIGINT NOT NULL,
    completed_at_unix_millis BIGINT,
    PRIMARY KEY (logical_owner_id, submission_message_id),
    UNIQUE (logical_owner_id, submission_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(submission_message_id) = 16),
    CHECK (length(submission_envelope_sha256) = 32),
    CHECK (length(submission_id) = 16),
    CHECK (length(candidate_id) = 16),
    CHECK (length(candidate_digest) = 32),
    CHECK (length(source_evidence_id) = 16),
    CHECK (source_evidence_revision > 0),
    CHECK (length(candidate_blob_reference_id) = 16),
    CHECK (candidate_blob_declared_bytes BETWEEN 1 AND 16384),
    CHECK (length(candidate_blob_sha256) = 32),
    CHECK (length(candidate_blob_custody_proof) BETWEEN 1 AND 2048),
    CHECK (materialized_blob_reference_id IS NULL OR length(materialized_blob_reference_id) = 16),
    CHECK (cleanup_completed_at_unix_millis IS NULL OR (
        materialized_blob_reference_id IS NOT NULL
        AND cleanup_completed_at_unix_millis >= received_at_unix_millis
    )),
    CHECK (received_at_unix_millis > 0),
    CHECK (
        (NOT completed AND NOT rejected AND review_id IS NULL AND completed_at_unix_millis IS NULL)
        OR (completed AND NOT rejected AND length(review_id) = 16
            AND completed_at_unix_millis >= received_at_unix_millis)
        OR (completed AND rejected AND review_id IS NULL
            AND completed_at_unix_millis >= received_at_unix_millis)
    )
);

CREATE INDEX review_obligation_candidate_submission_recovery_idx
ON makosh_data.review_obligation_candidate_submissions (
    logical_owner_id,
    completed,
    received_at_unix_millis
);

CREATE TABLE makosh_data.review_obligation_candidate_state (
    logical_owner_id TEXT NOT NULL,
    review_id BYTEA NOT NULL,
    candidate_id BYTEA NOT NULL,
    candidate_digest BYTEA NOT NULL,
    source_evidence_id BYTEA NOT NULL,
    source_evidence_revision BIGINT NOT NULL,
    statement TEXT NOT NULL,
    due_text_hint TEXT,
    condition TEXT,
    state SMALLINT NOT NULL,
    promotion_status SMALLINT NOT NULL,
    review_revision BIGINT NOT NULL,
    decided_by_owner_device_id BYTEA,
    decided_at_unix_seconds BIGINT,
    decided_at_nanos INTEGER,
    promoted_obligation_id BYTEA,
    updated_at_unix_seconds BIGINT NOT NULL,
    updated_at_nanos INTEGER NOT NULL,
    PRIMARY KEY (logical_owner_id, review_id),
    UNIQUE (logical_owner_id, candidate_id, candidate_digest),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(review_id) = 16),
    CHECK (length(candidate_id) = 16),
    CHECK (length(candidate_digest) = 32),
    CHECK (length(source_evidence_id) = 16),
    CHECK (source_evidence_revision > 0),
    CHECK (char_length(statement) BETWEEN 1 AND 240),
    CHECK (due_text_hint IS NULL OR char_length(due_text_hint) BETWEEN 1 AND 120),
    CHECK (condition IS NULL OR char_length(condition) BETWEEN 1 AND 120),
    CHECK (state BETWEEN 1 AND 3),
    CHECK (promotion_status BETWEEN 1 AND 4),
    CHECK (review_revision > 0),
    CHECK (updated_at_unix_seconds > 0),
    CHECK (updated_at_nanos BETWEEN 0 AND 999999999),
    CHECK (
        (state = 1 AND promotion_status = 1 AND decided_by_owner_device_id IS NULL
            AND decided_at_unix_seconds IS NULL AND decided_at_nanos IS NULL
            AND promoted_obligation_id IS NULL)
        OR (state = 2 AND promotion_status BETWEEN 2 AND 4
            AND length(decided_by_owner_device_id) = 16
            AND decided_at_unix_seconds > 0 AND decided_at_nanos BETWEEN 0 AND 999999999
            AND ((promotion_status = 3 AND length(promoted_obligation_id) = 16)
                OR (promotion_status IN (2, 4) AND promoted_obligation_id IS NULL)))
        OR (state = 3 AND promotion_status = 1
            AND length(decided_by_owner_device_id) = 16
            AND decided_at_unix_seconds > 0 AND decided_at_nanos BETWEEN 0 AND 999999999
            AND promoted_obligation_id IS NULL)
    )
);

CREATE INDEX review_obligation_candidate_promotion_recovery_idx
ON makosh_data.review_obligation_candidate_state (
    logical_owner_id,
    state,
    promotion_status,
    review_revision
);

CREATE TABLE makosh_data.review_obligation_candidate_operations (
    logical_owner_id TEXT NOT NULL,
    operation_id BYTEA NOT NULL,
    request_sha256 BYTEA NOT NULL,
    decision_fingerprint BYTEA NOT NULL,
    review_id BYTEA NOT NULL,
    expected_review_revision BIGINT NOT NULL,
    result_review_revision BIGINT NOT NULL,
    completed_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, operation_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(operation_id) = 16),
    CHECK (length(request_sha256) = 32),
    CHECK (length(decision_fingerprint) = 32),
    CHECK (length(review_id) = 16),
    CHECK (expected_review_revision > 0),
    CHECK (result_review_revision > expected_review_revision),
    CHECK (completed_at_unix_millis > 0)
);

CREATE TABLE makosh_data.review_obligation_candidate_promotion_inbox (
    logical_owner_id TEXT NOT NULL,
    result_message_id BYTEA NOT NULL,
    result_envelope_sha256 BYTEA NOT NULL,
    review_id BYTEA NOT NULL,
    result_review_revision BIGINT NOT NULL,
    processed_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, result_message_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(result_message_id) = 16),
    CHECK (length(result_envelope_sha256) = 32),
    CHECK (length(review_id) = 16),
    CHECK (result_review_revision > 0),
    CHECK (processed_at_unix_millis > 0)
);

CREATE TABLE makosh_data.review_obligation_candidate_outbox (
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

CREATE INDEX review_obligation_candidate_outbox_pending_idx
ON makosh_data.review_obligation_candidate_outbox (
    logical_owner_id,
    created_at_unix_millis,
    message_id
)
WHERE published_at_unix_millis IS NULL;

CREATE TABLE makosh_data.review_obligation_candidate_realtime (
    realtime_sequence BIGSERIAL PRIMARY KEY,
    logical_owner_id TEXT NOT NULL,
    review_id BYTEA NOT NULL,
    candidate_id BYTEA NOT NULL,
    state SMALLINT NOT NULL,
    promotion_status SMALLINT NOT NULL,
    review_revision BIGINT NOT NULL,
    occurred_at_unix_millis BIGINT NOT NULL,
    UNIQUE (logical_owner_id, review_id, review_revision),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(review_id) = 16),
    CHECK (length(candidate_id) = 16),
    CHECK (state BETWEEN 1 AND 3),
    CHECK (promotion_status BETWEEN 1 AND 4),
    CHECK (review_revision > 0),
    CHECK (occurred_at_unix_millis > 0)
);

CREATE INDEX review_obligation_candidate_realtime_owner_idx
ON makosh_data.review_obligation_candidate_realtime (
    logical_owner_id,
    realtime_sequence
);
