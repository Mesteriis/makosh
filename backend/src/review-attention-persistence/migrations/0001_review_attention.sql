CREATE TABLE makosh_data.review_attention_state (
    logical_owner_id TEXT NOT NULL,
    attention_id BYTEA NOT NULL,
    source_evidence_id BYTEA NOT NULL,
    state_revision BIGINT NOT NULL,
    disposition SMALLINT NOT NULL,
    pinned BOOLEAN NOT NULL,
    importance SMALLINT NOT NULL,
    snoozed_until_unix_seconds BIGINT,
    snoozed_until_nanos INTEGER,
    updated_at_unix_seconds BIGINT NOT NULL,
    updated_at_nanos INTEGER NOT NULL,
    PRIMARY KEY (logical_owner_id, source_evidence_id),
    UNIQUE (logical_owner_id, attention_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(attention_id) = 16),
    CHECK (length(source_evidence_id) = 16),
    CHECK (state_revision > 0),
    CHECK (disposition BETWEEN 1 AND 3),
    CHECK (importance BETWEEN 1 AND 2),
    CHECK (updated_at_unix_seconds > 0),
    CHECK (updated_at_nanos BETWEEN 0 AND 999999999),
    CHECK (
        (snoozed_until_unix_seconds IS NULL AND snoozed_until_nanos IS NULL)
        OR (
            snoozed_until_unix_seconds > 0
            AND snoozed_until_nanos BETWEEN 0 AND 999999999
        )
    )
);

CREATE TABLE makosh_data.review_attention_operations (
    logical_owner_id TEXT NOT NULL,
    operation_id BYTEA NOT NULL,
    request_sha256 BYTEA NOT NULL,
    expected_revision BIGINT NOT NULL,
    attention_id BYTEA,
    result_revision BIGINT,
    result_disposition SMALLINT,
    result_pinned BOOLEAN,
    result_importance SMALLINT,
    result_snoozed_until_unix_seconds BIGINT,
    result_snoozed_until_nanos INTEGER,
    result_updated_at_unix_seconds BIGINT,
    result_updated_at_nanos INTEGER,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    requested_at_unix_seconds BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, operation_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(operation_id) = 16),
    CHECK (length(request_sha256) = 32),
    CHECK (expected_revision >= 0),
    CHECK (requested_at_unix_seconds > 0),
    CHECK (
        NOT completed
        OR (
            attention_id IS NOT NULL
            AND length(attention_id) = 16
            AND result_revision IS NOT NULL
            AND result_revision > 0
            AND result_disposition IS NOT NULL
            AND result_disposition BETWEEN 1 AND 3
            AND result_pinned IS NOT NULL
            AND result_importance IS NOT NULL
            AND result_importance BETWEEN 1 AND 2
            AND result_updated_at_unix_seconds IS NOT NULL
            AND result_updated_at_unix_seconds > 0
            AND result_updated_at_nanos IS NOT NULL
            AND result_updated_at_nanos BETWEEN 0 AND 999999999
        )
    ),
    CHECK (
        (result_snoozed_until_unix_seconds IS NULL AND result_snoozed_until_nanos IS NULL)
        OR (
            result_snoozed_until_unix_seconds > 0
            AND result_snoozed_until_nanos BETWEEN 0 AND 999999999
        )
    )
);

CREATE INDEX review_attention_state_pending_idx
ON makosh_data.review_attention_state (
    logical_owner_id,
    disposition,
    pinned,
    importance,
    state_revision
);
