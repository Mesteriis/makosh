ALTER TABLE makosh_data.mail_gmail_oauth_credential_bindings
    ADD COLUMN IF NOT EXISTS contacts_write_authorized BOOLEAN NOT NULL DEFAULT FALSE;

CREATE TABLE makosh_data.mail_address_book_upsert_inbox (
    command_message_id BYTEA PRIMARY KEY CHECK (octet_length(command_message_id) = 16),
    command_envelope_sha256 BYTEA NOT NULL CHECK (octet_length(command_envelope_sha256) = 32),
    command_id BYTEA NOT NULL UNIQUE CHECK (octet_length(command_id) = 16),
    run_id BYTEA NOT NULL CHECK (octet_length(run_id) = 16),
    logical_owner_id TEXT NOT NULL CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    account_id TEXT NOT NULL CHECK (length(account_id) BETWEEN 1 AND 256),
    contact_snapshot_reference_id BYTEA NOT NULL CHECK (
        octet_length(contact_snapshot_reference_id) = 16
    ),
    contact_snapshot_sha256 BYTEA NOT NULL CHECK (octet_length(contact_snapshot_sha256) = 32),
    expected_contact_revision BIGINT NOT NULL CHECK (expected_contact_revision > 0),
    contact_snapshot_declared_bytes BIGINT NOT NULL CHECK (
        contact_snapshot_declared_bytes BETWEEN 1 AND 32768
    ),
    contact_snapshot_custody_source_proof BYTEA NOT NULL CHECK (
        octet_length(contact_snapshot_custody_source_proof) BETWEEN 1 AND 4096
    ),
    state SMALLINT NOT NULL CHECK (state BETWEEN 0 AND 2),
    execution_attempt INTEGER NOT NULL DEFAULT 1 CHECK (execution_attempt > 0),
    accepted_at_unix_seconds BIGINT NOT NULL CHECK (accepted_at_unix_seconds > 0),
    dispatch_started_at_unix_seconds BIGINT CHECK (
        dispatch_started_at_unix_seconds IS NULL OR dispatch_started_at_unix_seconds > 0
    ),
    UNIQUE (command_message_id, command_id)
);

CREATE INDEX mail_address_book_upsert_pending_idx
ON makosh_data.mail_address_book_upsert_inbox (
    state,
    accepted_at_unix_seconds,
    command_message_id
);

CREATE TABLE makosh_data.mail_address_book_upsert_result_outbox (
    message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
    envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
    exact_envelope_bytes BYTEA NOT NULL CHECK (octet_length(exact_envelope_bytes) > 0),
    command_id BYTEA NOT NULL UNIQUE CHECK (octet_length(command_id) = 16),
    command_message_id BYTEA NOT NULL UNIQUE CHECK (octet_length(command_message_id) = 16),
    created_at_unix_seconds BIGINT NOT NULL CHECK (created_at_unix_seconds > 0),
    published_at_unix_seconds BIGINT CHECK (
        published_at_unix_seconds IS NULL OR published_at_unix_seconds > 0
    ),
    FOREIGN KEY (command_message_id, command_id) REFERENCES
        makosh_data.mail_address_book_upsert_inbox (
            command_message_id,
            command_id
        )
);

CREATE INDEX mail_address_book_upsert_result_pending_idx
ON makosh_data.mail_address_book_upsert_result_outbox (
    created_at_unix_seconds,
    message_id
)
WHERE published_at_unix_seconds IS NULL;
