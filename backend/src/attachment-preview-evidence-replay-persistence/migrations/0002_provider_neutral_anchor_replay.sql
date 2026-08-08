CREATE TABLE makosh_data.attachment_preview_evidence_replay_anchor_producers (
    operation_id BYTEA NOT NULL REFERENCES
        makosh_data.attachment_preview_evidence_replay_operations (operation_id),
    producer SMALLINT NOT NULL CHECK (producer BETWEEN 1 AND 2),
    outcome SMALLINT NOT NULL DEFAULT 0 CHECK (outcome BETWEEN 0 AND 4),
    failure SMALLINT NOT NULL DEFAULT 0 CHECK (failure BETWEEN 0 AND 7),
    PRIMARY KEY (operation_id, producer)
);

CREATE TABLE makosh_data.attachment_preview_evidence_replay_anchor_result_messages (
    operation_id BYTEA NOT NULL,
    producer SMALLINT NOT NULL,
    ordinal SMALLINT NOT NULL CHECK (ordinal BETWEEN 0 AND 15),
    original_message_id BYTEA NOT NULL CHECK (octet_length(original_message_id) = 16),
    PRIMARY KEY (operation_id, producer, ordinal),
    UNIQUE (operation_id, producer, original_message_id),
    FOREIGN KEY (operation_id, producer) REFERENCES
        makosh_data.attachment_preview_evidence_replay_anchor_producers (operation_id, producer)
);

CREATE TABLE makosh_data.attachment_preview_evidence_replay_anchor_command_outbox (
    message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
    envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
    exact_envelope_bytes BYTEA NOT NULL CHECK (octet_length(exact_envelope_bytes) > 0),
    operation_id BYTEA NOT NULL,
    producer SMALLINT NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL CHECK (created_at_unix_seconds > 0),
    published_at_unix_seconds BIGINT CHECK (
        published_at_unix_seconds IS NULL OR published_at_unix_seconds > 0
    ),
    UNIQUE (operation_id, producer),
    FOREIGN KEY (operation_id, producer) REFERENCES
        makosh_data.attachment_preview_evidence_replay_anchor_producers (operation_id, producer)
);

CREATE INDEX attachment_preview_evidence_replay_anchor_command_pending_idx
ON makosh_data.attachment_preview_evidence_replay_anchor_command_outbox (
    created_at_unix_seconds,
    message_id
)
WHERE published_at_unix_seconds IS NULL;

CREATE TABLE makosh_data.attachment_preview_evidence_replay_anchor_result_inbox (
    message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
    envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
    operation_id BYTEA NOT NULL,
    producer SMALLINT NOT NULL,
    accepted_at_unix_seconds BIGINT NOT NULL CHECK (accepted_at_unix_seconds > 0),
    UNIQUE (operation_id, producer),
    FOREIGN KEY (operation_id, producer) REFERENCES
        makosh_data.attachment_preview_evidence_replay_anchor_producers (operation_id, producer)
);
