CREATE TABLE makosh_data.contacts_mail_entry_inbox (
    logical_owner_id TEXT NOT NULL,
    command_message_id BYTEA NOT NULL,
    command_envelope_sha256 BYTEA NOT NULL,
    command_id BYTEA NOT NULL,
    command_fingerprint BYTEA NOT NULL,
    entry_digest BYTEA NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    contact_id BYTEA,
    contact_revision BIGINT,
    outcome SMALLINT,
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
    CHECK (length(command_fingerprint) = 32),
    CHECK (length(entry_digest) = 32),
    CHECK (received_at_unix_millis > 0),
    CHECK (
        (NOT completed AND contact_id IS NULL AND contact_revision IS NULL AND outcome IS NULL
            AND reject_code IS NULL AND result_message_id IS NULL
            AND completed_at_unix_millis IS NULL)
        OR (completed AND length(contact_id) = 16 AND contact_revision > 0
            AND outcome IN (1, 2, 3) AND reject_code IS NULL
            AND length(result_message_id) = 16
            AND completed_at_unix_millis >= received_at_unix_millis)
        OR (completed AND contact_id IS NULL AND contact_revision IS NULL AND outcome IS NULL
            AND reject_code IN (1, 2, 3, 4, 5) AND length(result_message_id) = 16
            AND completed_at_unix_millis >= received_at_unix_millis)
    )
);

CREATE TABLE makosh_data.contacts_state (
    logical_owner_id TEXT NOT NULL,
    contact_id BYTEA NOT NULL,
    display_name TEXT NOT NULL,
    contact_revision BIGINT NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL,
    created_at_nanos INTEGER NOT NULL,
    updated_at_unix_seconds BIGINT NOT NULL,
    updated_at_nanos INTEGER NOT NULL,
    PRIMARY KEY (logical_owner_id, contact_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(contact_id) = 16),
    CHECK (char_length(display_name) BETWEEN 0 AND 240),
    CHECK (contact_revision > 0),
    CHECK (created_at_unix_seconds > 0),
    CHECK (created_at_nanos BETWEEN 0 AND 999999999),
    CHECK (updated_at_unix_seconds >= created_at_unix_seconds),
    CHECK (updated_at_nanos BETWEEN 0 AND 999999999)
);

CREATE TABLE makosh_data.contacts_email_identities (
    logical_owner_id TEXT NOT NULL,
    normalized_email TEXT NOT NULL,
    contact_id BYTEA NOT NULL,
    PRIMARY KEY (logical_owner_id, normalized_email),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (char_length(normalized_email) BETWEEN 3 AND 320),
    CHECK (length(contact_id) = 16)
);

CREATE INDEX contacts_email_contact_idx
ON makosh_data.contacts_email_identities (logical_owner_id, contact_id);

CREATE TABLE makosh_data.contacts_phone_identities (
    logical_owner_id TEXT NOT NULL,
    normalized_phone TEXT NOT NULL,
    contact_id BYTEA NOT NULL,
    PRIMARY KEY (logical_owner_id, normalized_phone),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (char_length(normalized_phone) BETWEEN 8 AND 16),
    CHECK (length(contact_id) = 16)
);

CREATE INDEX contacts_phone_contact_idx
ON makosh_data.contacts_phone_identities (logical_owner_id, contact_id);

CREATE TABLE makosh_data.contacts_provider_links (
    logical_owner_id TEXT NOT NULL,
    provider_kind SMALLINT NOT NULL,
    source_account_id TEXT NOT NULL,
    provider_entry_id TEXT NOT NULL,
    contact_id BYTEA NOT NULL,
    provider_etag TEXT,
    source_revision BIGINT NOT NULL,
    entry_digest BYTEA NOT NULL,
    observed_at_unix_seconds BIGINT NOT NULL,
    observed_at_nanos INTEGER NOT NULL,
    PRIMARY KEY (logical_owner_id, provider_kind, source_account_id, provider_entry_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (provider_kind IN (1, 2)),
    CHECK (length(source_account_id) BETWEEN 1 AND 256),
    CHECK (length(provider_entry_id) BETWEEN 1 AND 512),
    CHECK (length(contact_id) = 16),
    CHECK (provider_etag IS NULL OR length(provider_etag) BETWEEN 1 AND 512),
    CHECK (source_revision > 0),
    CHECK (length(entry_digest) = 32),
    CHECK (observed_at_unix_seconds > 0),
    CHECK (observed_at_nanos BETWEEN 0 AND 999999999)
);

CREATE INDEX contacts_provider_contact_idx
ON makosh_data.contacts_provider_links (logical_owner_id, contact_id);

CREATE TABLE makosh_data.contacts_outbox (
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
    CHECK (published_at_unix_millis IS NULL OR published_at_unix_millis >= created_at_unix_millis)
);

CREATE INDEX contacts_outbox_pending_idx
ON makosh_data.contacts_outbox (logical_owner_id, created_at_unix_millis, message_id)
WHERE published_at_unix_millis IS NULL;
