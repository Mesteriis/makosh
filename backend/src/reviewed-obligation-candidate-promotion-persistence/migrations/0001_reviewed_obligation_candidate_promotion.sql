CREATE TABLE makosh_data.reviewed_obligation_candidate_promotion_requests (
    logical_owner_id TEXT NOT NULL,
    approval_message_id BYTEA NOT NULL,
    approval_envelope_sha256 BYTEA NOT NULL,
    review_id BYTEA NOT NULL,
    candidate_id BYTEA NOT NULL,
    decision_revision BIGINT NOT NULL,
    obligations_id BYTEA NOT NULL,
    obligations_message_id BYTEA NOT NULL,
    obligations_result_message_id BYTEA,
    promotion_outcome SMALLINT,
    obligation_id BYTEA,
    failure_code INTEGER,
    created_at_unix_millis BIGINT NOT NULL,
    updated_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, approval_message_id),
    UNIQUE (logical_owner_id, obligations_id),
    UNIQUE (logical_owner_id, review_id, decision_revision),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(approval_message_id) = 16),
    CHECK (length(approval_envelope_sha256) = 32),
    CHECK (length(review_id) = 16),
    CHECK (length(candidate_id) = 16),
    CHECK (decision_revision > 0),
    CHECK (length(obligations_id) = 16),
    CHECK (length(obligations_message_id) = 16),
    CHECK (obligations_id = obligations_message_id),
    CHECK (created_at_unix_millis > 0),
    CHECK (updated_at_unix_millis >= created_at_unix_millis),
    CHECK (
        (
            obligations_result_message_id IS NULL
            AND promotion_outcome IS NULL
            AND obligation_id IS NULL
            AND failure_code IS NULL
        )
        OR (
            length(obligations_result_message_id) = 16
            AND promotion_outcome = 1
            AND length(obligation_id) = 16
            AND failure_code IS NULL
        )
        OR (
            length(obligations_result_message_id) = 16
            AND promotion_outcome = 2
            AND obligation_id IS NULL
            AND failure_code BETWEEN 1 AND 65535
        )
    )
);

CREATE INDEX reviewed_obligation_candidate_promotion_pending_idx
ON makosh_data.reviewed_obligation_candidate_promotion_requests (
    logical_owner_id,
    created_at_unix_millis,
    obligations_id
)
WHERE obligations_result_message_id IS NULL;

CREATE TABLE makosh_data.reviewed_obligation_candidate_promotion_result_inbox (
    logical_owner_id TEXT NOT NULL,
    result_message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    obligations_id BYTEA NOT NULL,
    review_id BYTEA NOT NULL,
    processed_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, result_message_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(result_message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (length(obligations_id) = 16),
    CHECK (length(review_id) = 16),
    CHECK (processed_at_unix_millis > 0)
);

CREATE TABLE makosh_data.reviewed_obligation_candidate_promotion_outbox (
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

CREATE INDEX reviewed_obligation_candidate_promotion_outbox_pending_idx
ON makosh_data.reviewed_obligation_candidate_promotion_outbox (
    logical_owner_id,
    created_at_unix_millis,
    message_id
)
WHERE published_at_unix_millis IS NULL;
