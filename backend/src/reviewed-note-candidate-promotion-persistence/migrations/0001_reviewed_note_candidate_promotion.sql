CREATE TABLE makosh_data.reviewed_note_candidate_promotion_requests (
    logical_owner_id TEXT NOT NULL,
    approval_message_id BYTEA NOT NULL,
    approval_envelope_sha256 BYTEA NOT NULL,
    review_id BYTEA NOT NULL,
    candidate_id BYTEA NOT NULL,
    decision_revision BIGINT NOT NULL,
    source_blob_reference_id BYTEA NOT NULL,
    source_blob_declared_bytes BIGINT NOT NULL,
    source_blob_sha256 BYTEA NOT NULL,
    source_blob_custody_proof BYTEA NOT NULL,
    materialized_blob_reference_id BYTEA,
    cleanup_completed_at_unix_millis BIGINT,
    knowledge_command_id BYTEA NOT NULL,
    knowledge_command_message_id BYTEA,
    workflow_failure_result_id BYTEA,
    knowledge_result_message_id BYTEA,
    promotion_outcome SMALLINT,
    note_id BYTEA,
    failure_code INTEGER,
    created_at_unix_millis BIGINT NOT NULL,
    updated_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, approval_message_id),
    UNIQUE (logical_owner_id, knowledge_command_id),
    UNIQUE (logical_owner_id, review_id, decision_revision),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(approval_message_id) = 16),
    CHECK (length(approval_envelope_sha256) = 32),
    CHECK (length(review_id) = 16),
    CHECK (length(candidate_id) = 16),
    CHECK (decision_revision > 0),
    CHECK (length(source_blob_reference_id) = 16),
    CHECK (source_blob_declared_bytes BETWEEN 1 AND 16384),
    CHECK (length(source_blob_sha256) = 32),
    CHECK (length(source_blob_custody_proof) BETWEEN 1 AND 2048),
    CHECK (materialized_blob_reference_id IS NULL OR length(materialized_blob_reference_id) = 16),
    CHECK (cleanup_completed_at_unix_millis IS NULL OR (
        materialized_blob_reference_id IS NOT NULL
        AND (
            knowledge_command_message_id IS NOT NULL
            OR workflow_failure_result_id IS NOT NULL
        )
        AND cleanup_completed_at_unix_millis >= created_at_unix_millis
    )),
    CHECK (length(knowledge_command_id) = 16),
    CHECK (knowledge_command_message_id IS NULL OR (
        length(knowledge_command_message_id) = 16
        AND knowledge_command_id = knowledge_command_message_id
    )),
    CHECK (workflow_failure_result_id IS NULL OR (
        length(workflow_failure_result_id) = 16
        AND knowledge_command_message_id IS NULL
        AND knowledge_result_message_id IS NULL
    )),
    CHECK (created_at_unix_millis > 0),
    CHECK (updated_at_unix_millis >= created_at_unix_millis),
    CHECK (
        (
            knowledge_result_message_id IS NULL
            AND workflow_failure_result_id IS NULL
            AND promotion_outcome IS NULL
            AND note_id IS NULL
            AND failure_code IS NULL
        )
        OR (
            length(knowledge_result_message_id) = 16
            AND workflow_failure_result_id IS NULL
            AND promotion_outcome = 1
            AND length(note_id) = 16
            AND failure_code IS NULL
        )
        OR (
            length(knowledge_result_message_id) = 16
            AND workflow_failure_result_id IS NULL
            AND promotion_outcome = 2
            AND note_id IS NULL
            AND failure_code BETWEEN 1 AND 65535
        )
        OR (
            knowledge_result_message_id IS NULL
            AND length(workflow_failure_result_id) = 16
            AND promotion_outcome = 2
            AND note_id IS NULL
            AND failure_code BETWEEN 1 AND 65535
        )
    ),
    CHECK (knowledge_command_message_id IS NOT NULL OR knowledge_result_message_id IS NULL)
);

CREATE INDEX reviewed_note_candidate_promotion_pending_idx
ON makosh_data.reviewed_note_candidate_promotion_requests (
    logical_owner_id,
    created_at_unix_millis,
    knowledge_command_id
)
WHERE knowledge_command_message_id IS NOT NULL
  AND knowledge_result_message_id IS NULL;

CREATE TABLE makosh_data.reviewed_note_candidate_promotion_result_inbox (
    logical_owner_id TEXT NOT NULL,
    result_message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    knowledge_command_id BYTEA NOT NULL,
    review_id BYTEA NOT NULL,
    processed_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, result_message_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(result_message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (length(knowledge_command_id) = 16),
    CHECK (length(review_id) = 16),
    CHECK (processed_at_unix_millis > 0)
);

CREATE TABLE makosh_data.reviewed_note_candidate_promotion_outbox (
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

CREATE INDEX reviewed_note_candidate_promotion_outbox_pending_idx
ON makosh_data.reviewed_note_candidate_promotion_outbox (
    logical_owner_id,
    created_at_unix_millis,
    message_id
)
WHERE published_at_unix_millis IS NULL;
