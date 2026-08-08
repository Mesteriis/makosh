CREATE TABLE makosh_data.communications_retained_evidence_replay_command_inbox (
    message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
    envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
    operation_id BYTEA NOT NULL UNIQUE CHECK (octet_length(operation_id) = 16),
    logical_owner_id TEXT NOT NULL CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    state SMALLINT NOT NULL CHECK (state BETWEEN 0 AND 1),
    accepted_at_unix_seconds BIGINT NOT NULL CHECK (accepted_at_unix_seconds > 0),
    UNIQUE (message_id, operation_id)
);

CREATE TABLE makosh_data.communications_retained_evidence_replay_result_outbox (
    message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
    envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
    exact_envelope_bytes BYTEA NOT NULL CHECK (octet_length(exact_envelope_bytes) > 0),
    operation_id BYTEA NOT NULL UNIQUE CHECK (octet_length(operation_id) = 16),
    command_message_id BYTEA NOT NULL UNIQUE CHECK (octet_length(command_message_id) = 16),
    created_at_unix_seconds BIGINT NOT NULL CHECK (created_at_unix_seconds > 0),
    published_at_unix_seconds BIGINT CHECK (
        published_at_unix_seconds IS NULL OR published_at_unix_seconds > 0
    ),
    FOREIGN KEY (command_message_id, operation_id) REFERENCES
        makosh_data.communications_retained_evidence_replay_command_inbox (
            message_id,
            operation_id
        )
);

CREATE INDEX communications_retained_evidence_replay_result_pending_idx
ON makosh_data.communications_retained_evidence_replay_result_outbox (
    created_at_unix_seconds,
    message_id
)
WHERE published_at_unix_seconds IS NULL;
