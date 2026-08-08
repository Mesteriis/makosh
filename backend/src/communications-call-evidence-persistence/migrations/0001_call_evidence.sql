CREATE TABLE makosh_data.communications_call_evidence_inbox (
    logical_owner_id TEXT NOT NULL CHECK (
        length(logical_owner_id) BETWEEN 1 AND 256
    ),
    message_id BYTEA NOT NULL CHECK (octet_length(message_id) = 16),
    envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
    call_evidence_id BYTEA NOT NULL CHECK (octet_length(call_evidence_id) = 16),
    outcome SMALLINT NOT NULL CHECK (outcome BETWEEN 1 AND 4),
    rejection_code SMALLINT,
    canonical_revision BIGINT CHECK (
        canonical_revision IS NULL
        OR canonical_revision > 0
    ),
    realtime_sequence BIGINT CHECK (
        realtime_sequence IS NULL
        OR realtime_sequence > 0
    ),
    consumed_at_unix_seconds BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, message_id),
    CHECK (
        (
            outcome = 1
            AND rejection_code IS NULL
            AND canonical_revision IS NOT NULL
            AND realtime_sequence IS NOT NULL
        )
        OR (
            outcome IN (2, 3)
            AND rejection_code IS NULL
            AND canonical_revision IS NULL
            AND realtime_sequence IS NULL
        )
        OR (
            outcome = 4
            AND rejection_code IS NOT NULL
            AND canonical_revision IS NULL
            AND realtime_sequence IS NULL
        )
    )
);

CREATE TABLE makosh_data.communications_call_evidence_projection (
    logical_owner_id TEXT NOT NULL CHECK (
        length(logical_owner_id) BETWEEN 1 AND 256
    ),
    call_evidence_id BYTEA NOT NULL CHECK (octet_length(call_evidence_id) = 16),
    source_call_cursor_sha256 BYTEA NOT NULL CHECK (
        octet_length(source_call_cursor_sha256) = 32
    ),
    account_cursor_sha256 BYTEA NOT NULL CHECK (
        octet_length(account_cursor_sha256) = 32
    ),
    conversation_cursor_sha256 BYTEA CHECK (
        conversation_cursor_sha256 IS NULL
        OR octet_length(conversation_cursor_sha256) = 32
    ),
    participant_cursor_sha256 BYTEA CHECK (
        participant_cursor_sha256 IS NULL
        OR octet_length(participant_cursor_sha256) = 32
    ),
    provider SMALLINT NOT NULL CHECK (provider BETWEEN 1 AND 4),
    direction SMALLINT NOT NULL CHECK (direction BETWEEN 1 AND 3),
    media_kind SMALLINT NOT NULL CHECK (media_kind BETWEEN 1 AND 2),
    lifecycle_state SMALLINT NOT NULL CHECK (lifecycle_state BETWEEN 1 AND 5),
    terminal_disposition SMALLINT CHECK (
        terminal_disposition IS NULL
        OR terminal_disposition BETWEEN 1 AND 6
    ),
    source_revision BIGINT NOT NULL CHECK (source_revision > 0),
    canonical_revision BIGINT NOT NULL CHECK (canonical_revision > 0),
    started_at_unix_seconds BIGINT,
    connected_at_unix_seconds BIGINT,
    ended_at_unix_seconds BIGINT,
    duration_seconds BIGINT CHECK (
        duration_seconds IS NULL
        OR duration_seconds BETWEEN 0 AND 2678400
    ),
    participant_display_label TEXT CHECK (
        participant_display_label IS NULL
        OR length(participant_display_label) BETWEEN 1 AND 256
    ),
    payload_sha256 BYTEA NOT NULL CHECK (octet_length(payload_sha256) = 32),
    updated_at_unix_seconds BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, call_evidence_id),
    CHECK (
        (lifecycle_state = 5 AND terminal_disposition IS NOT NULL AND ended_at_unix_seconds IS NOT NULL)
        OR (lifecycle_state <> 5 AND terminal_disposition IS NULL AND ended_at_unix_seconds IS NULL)
    )
);

CREATE INDEX communications_call_evidence_projection_owner_updated_idx
ON makosh_data.communications_call_evidence_projection (
    logical_owner_id,
    updated_at_unix_seconds DESC,
    call_evidence_id
);

CREATE TABLE makosh_data.communications_call_evidence_history (
    logical_owner_id TEXT NOT NULL CHECK (
        length(logical_owner_id) BETWEEN 1 AND 256
    ),
    call_evidence_id BYTEA NOT NULL CHECK (octet_length(call_evidence_id) = 16),
    canonical_revision BIGINT NOT NULL CHECK (canonical_revision > 0),
    source_revision BIGINT NOT NULL CHECK (source_revision > 0),
    message_id BYTEA NOT NULL CHECK (octet_length(message_id) = 16),
    envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
    lifecycle_state SMALLINT NOT NULL CHECK (lifecycle_state BETWEEN 1 AND 5),
    terminal_disposition SMALLINT CHECK (
        terminal_disposition IS NULL
        OR terminal_disposition BETWEEN 1 AND 6
    ),
    observed_at_unix_seconds BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, call_evidence_id, canonical_revision),
    UNIQUE (logical_owner_id, message_id)
);

CREATE TABLE makosh_data.communications_call_evidence_realtime_sequence (
    logical_owner_id TEXT PRIMARY KEY CHECK (
        length(logical_owner_id) BETWEEN 1 AND 256
    ),
    next_sequence BIGINT NOT NULL CHECK (next_sequence > 0)
);

CREATE TABLE makosh_data.communications_call_evidence_realtime_frames (
    logical_owner_id TEXT NOT NULL CHECK (
        length(logical_owner_id) BETWEEN 1 AND 256
    ),
    sequence BIGINT NOT NULL CHECK (sequence > 0),
    call_evidence_id BYTEA NOT NULL CHECK (octet_length(call_evidence_id) = 16),
    canonical_revision BIGINT NOT NULL CHECK (canonical_revision > 0),
    lifecycle_state SMALLINT NOT NULL CHECK (lifecycle_state BETWEEN 1 AND 5),
    terminal_disposition SMALLINT CHECK (
        terminal_disposition IS NULL
        OR terminal_disposition BETWEEN 1 AND 6
    ),
    observed_at_unix_seconds BIGINT NOT NULL,
    participant_display_label TEXT CHECK (
        participant_display_label IS NULL
        OR length(participant_display_label) BETWEEN 1 AND 256
    ),
    PRIMARY KEY (logical_owner_id, sequence),
    UNIQUE (logical_owner_id, call_evidence_id, canonical_revision)
);
