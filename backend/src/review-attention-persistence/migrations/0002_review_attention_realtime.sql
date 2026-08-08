CREATE TABLE makosh_data.review_attention_realtime (
    realtime_sequence BIGSERIAL PRIMARY KEY,
    logical_owner_id TEXT NOT NULL,
    attention_id BYTEA NOT NULL,
    state_revision BIGINT NOT NULL,
    disposition SMALLINT NOT NULL,
    pinned BOOLEAN NOT NULL,
    importance SMALLINT NOT NULL,
    snoozed_until_unix_seconds BIGINT,
    snoozed_until_nanos INTEGER,
    occurred_at_unix_seconds BIGINT NOT NULL,
    occurred_at_nanos INTEGER NOT NULL,
    UNIQUE (logical_owner_id, attention_id, state_revision),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(attention_id) = 16),
    CHECK (state_revision > 0),
    CHECK (disposition BETWEEN 1 AND 3),
    CHECK (importance BETWEEN 1 AND 2),
    CHECK (occurred_at_unix_seconds > 0),
    CHECK (occurred_at_nanos BETWEEN 0 AND 999999999),
    CHECK (
        (snoozed_until_unix_seconds IS NULL AND snoozed_until_nanos IS NULL)
        OR (
            snoozed_until_unix_seconds > 0
            AND snoozed_until_nanos BETWEEN 0 AND 999999999
        )
    )
);

CREATE INDEX review_attention_realtime_owner_sequence_idx
ON makosh_data.review_attention_realtime (
    logical_owner_id,
    realtime_sequence
);
