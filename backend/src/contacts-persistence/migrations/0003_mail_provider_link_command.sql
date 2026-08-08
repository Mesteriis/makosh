CREATE TABLE makosh_data.contacts_mail_provider_link_inbox (
    logical_owner_id TEXT NOT NULL,
    command_message_id BYTEA NOT NULL,
    command_envelope_sha256 BYTEA NOT NULL,
    command_id BYTEA NOT NULL,
    contact_id BYTEA NOT NULL,
    expected_contact_revision BIGINT NOT NULL,
    source_account_id TEXT NOT NULL,
    provider_kind SMALLINT NOT NULL,
    provider_entry_id TEXT NOT NULL,
    provider_etag TEXT,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    reject_code SMALLINT,
    result_message_id BYTEA,
    received_at_unix_millis BIGINT NOT NULL,
    completed_at_unix_millis BIGINT,
    PRIMARY KEY (logical_owner_id, command_message_id),
    UNIQUE (logical_owner_id, command_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(command_message_id) = 16),
    CHECK (length(command_envelope_sha256) = 32),
    CHECK (length(command_id) = 16),
    CHECK (length(contact_id) = 16),
    CHECK (expected_contact_revision > 0),
    CHECK (length(source_account_id) BETWEEN 1 AND 256),
    CHECK (provider_kind IN (1, 2)),
    CHECK (length(provider_entry_id) BETWEEN 1 AND 512),
    CHECK (provider_etag IS NULL OR length(provider_etag) BETWEEN 1 AND 512),
    CHECK (received_at_unix_millis > 0),
    CHECK (
        (NOT completed AND reject_code IS NULL AND result_message_id IS NULL
            AND completed_at_unix_millis IS NULL)
        OR (completed AND (reject_code IS NULL OR reject_code IN (1, 2, 3, 4, 5))
            AND length(result_message_id) = 16
            AND completed_at_unix_millis >= received_at_unix_millis)
    )
);
