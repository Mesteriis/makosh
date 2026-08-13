CREATE TABLE makosh_data.mail_persons_sync_account_bindings (
    logical_owner_id TEXT NOT NULL CHECK (octet_length(logical_owner_id) BETWEEN 1 AND 128)
        CHECK (logical_owner_id ~ '^[a-z0-9._-]+$'),
    account_public_id BYTEA NOT NULL CHECK (octet_length(account_public_id) = 16),
    integration_public_id BYTEA NOT NULL CHECK (octet_length(integration_public_id) = 16),
    mapping_revision BIGINT NOT NULL CHECK (mapping_revision > 0),
    state SMALLINT NOT NULL CHECK (state IN (1, 2)),
    schedule_revision BIGINT NOT NULL CHECK (schedule_revision > 0),
    updated_at_unix_millis BIGINT NOT NULL CHECK (updated_at_unix_millis > 0),
    PRIMARY KEY (logical_owner_id, account_public_id)
);

CREATE TABLE makosh_data.mail_persons_sync_account_inbox (
    logical_owner_id TEXT NOT NULL,
    message_id BYTEA NOT NULL CHECK (octet_length(message_id) = 16),
    envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
    envelope_bytes BYTEA NOT NULL CHECK (octet_length(envelope_bytes) BETWEEN 1 AND 262144),
    account_public_id BYTEA NOT NULL CHECK (octet_length(account_public_id) = 16),
    mapping_revision BIGINT NOT NULL CHECK (mapping_revision > 0),
    semantic_kind SMALLINT NOT NULL CHECK (semantic_kind IN (1, 2, 3)),
    processed_at_unix_millis BIGINT NOT NULL CHECK (processed_at_unix_millis > 0),
    PRIMARY KEY (logical_owner_id, message_id),
    FOREIGN KEY (logical_owner_id, account_public_id)
        REFERENCES makosh_data.mail_persons_sync_account_bindings(logical_owner_id, account_public_id)
        ON DELETE RESTRICT
);

CREATE TABLE makosh_data.mail_persons_sync_schedule_control_outbox (
    logical_owner_id TEXT NOT NULL,
    outbox_sequence BIGINT GENERATED ALWAYS AS IDENTITY,
    message_id BYTEA NOT NULL CHECK (octet_length(message_id) = 16),
    envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
    envelope_bytes BYTEA NOT NULL CHECK (octet_length(envelope_bytes) BETWEEN 1 AND 262144),
    account_public_id BYTEA NOT NULL CHECK (octet_length(account_public_id) = 16),
    mapping_revision BIGINT NOT NULL CHECK (mapping_revision > 0),
    schedule_revision BIGINT NOT NULL CHECK (schedule_revision > 0),
    semantic_kind SMALLINT NOT NULL CHECK (semantic_kind IN (1, 2)),
    created_at_unix_millis BIGINT NOT NULL CHECK (created_at_unix_millis > 0),
    published_at_unix_millis BIGINT CHECK (
        published_at_unix_millis IS NULL OR published_at_unix_millis >= created_at_unix_millis
    ),
    PRIMARY KEY (logical_owner_id, outbox_sequence),
    UNIQUE (logical_owner_id, message_id),
    UNIQUE (logical_owner_id, account_public_id, schedule_revision),
    FOREIGN KEY (logical_owner_id, account_public_id)
        REFERENCES makosh_data.mail_persons_sync_account_bindings(logical_owner_id, account_public_id)
        ON DELETE RESTRICT
);

CREATE INDEX mail_persons_sync_schedule_control_pending
ON makosh_data.mail_persons_sync_schedule_control_outbox (logical_owner_id, outbox_sequence)
WHERE published_at_unix_millis IS NULL;

ALTER TABLE makosh_data.mail_persons_sync_account_bindings ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.mail_persons_sync_account_bindings FORCE ROW LEVEL SECURITY;
CREATE POLICY mail_persons_sync_account_bindings_owner_rls
ON makosh_data.mail_persons_sync_account_bindings
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.mail_persons_sync_account_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.mail_persons_sync_account_inbox FORCE ROW LEVEL SECURITY;
CREATE POLICY mail_persons_sync_account_inbox_owner_rls
ON makosh_data.mail_persons_sync_account_inbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.mail_persons_sync_schedule_control_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.mail_persons_sync_schedule_control_outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY mail_persons_sync_schedule_control_outbox_owner_rls
ON makosh_data.mail_persons_sync_schedule_control_outbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
