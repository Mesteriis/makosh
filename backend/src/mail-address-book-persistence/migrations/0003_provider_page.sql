CREATE TABLE makosh_data.mail_address_book_fetch_inbox (
    command_message_id BYTEA PRIMARY KEY CHECK (octet_length(command_message_id) = 16),
    command_envelope_sha256 BYTEA NOT NULL CHECK (octet_length(command_envelope_sha256) = 32),
    command_id BYTEA NOT NULL UNIQUE CHECK (octet_length(command_id) = 16),
    run_id BYTEA NOT NULL CHECK (octet_length(run_id) = 16),
    logical_owner_id TEXT NOT NULL CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    account_id TEXT NOT NULL CHECK (length(account_id) BETWEEN 1 AND 256),
    page_sequence BIGINT NOT NULL CHECK (page_sequence > 0),
    continuation_cursor BYTEA CHECK (
        continuation_cursor IS NULL OR octet_length(continuation_cursor) BETWEEN 1 AND 4096
    ),
    page_size INTEGER NOT NULL CHECK (page_size BETWEEN 1 AND 500),
    state SMALLINT NOT NULL CHECK (state IN (0, 1)),
    execution_attempt INTEGER NOT NULL CHECK (execution_attempt > 0),
    accepted_at_unix_seconds BIGINT NOT NULL CHECK (accepted_at_unix_seconds > 0),
    completed_at_unix_seconds BIGINT CHECK (completed_at_unix_seconds > 0),
    UNIQUE (command_message_id, command_id)
);

CREATE INDEX mail_address_book_fetch_pending_idx
ON makosh_data.mail_address_book_fetch_inbox (
    state,
    accepted_at_unix_seconds,
    command_message_id
);

CREATE TABLE makosh_data.mail_address_book_fetch_outbox (
    message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
    envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
    exact_envelope_bytes BYTEA NOT NULL CHECK (octet_length(exact_envelope_bytes) BETWEEN 1 AND 4194304),
    command_id BYTEA NOT NULL CHECK (octet_length(command_id) = 16),
    command_message_id BYTEA NOT NULL CHECK (octet_length(command_message_id) = 16),
    ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
    created_at_unix_seconds BIGINT NOT NULL CHECK (created_at_unix_seconds > 0),
    published_at_unix_seconds BIGINT CHECK (published_at_unix_seconds > 0),
    UNIQUE (command_id, ordinal),
    FOREIGN KEY (command_message_id, command_id) REFERENCES
        makosh_data.mail_address_book_fetch_inbox (
            command_message_id,
            command_id
        )
);

CREATE INDEX mail_address_book_fetch_outbox_pending_idx
ON makosh_data.mail_address_book_fetch_outbox (
    published_at_unix_seconds,
    created_at_unix_seconds,
    command_id,
    ordinal
);
