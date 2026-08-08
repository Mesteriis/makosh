CREATE TABLE makosh_data.mail_contacts_sync_runs (
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    operation_id BYTEA NOT NULL,
    request_fingerprint BYTEA NOT NULL,
    account_id TEXT NOT NULL,
    direction SMALLINT NOT NULL,
    trigger_kind SMALLINT NOT NULL,
    state SMALLINT NOT NULL,
    state_revision BIGINT NOT NULL,
    page_sequence BIGINT NOT NULL,
    continuation_cursor BYTEA,
    provider_entries_seen BIGINT NOT NULL,
    contacts_created BIGINT NOT NULL,
    contacts_updated BIGINT NOT NULL,
    contacts_unchanged BIGINT NOT NULL,
    provider_entries_written BIGINT NOT NULL,
    rejected_entries BIGINT NOT NULL,
    rejection_code SMALLINT,
    created_at_unix_millis BIGINT NOT NULL,
    updated_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, run_id),
    UNIQUE (logical_owner_id, operation_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(run_id) = 16),
    CHECK (length(operation_id) = 16),
    CHECK (length(request_fingerprint) = 32),
    CHECK (length(account_id) BETWEEN 1 AND 256),
    CHECK (direction IN (1, 2)),
    CHECK (trigger_kind IN (1, 2)),
    CHECK (state BETWEEN 1 AND 7),
    CHECK (state_revision > 0),
    CHECK (page_sequence >= 0),
    CHECK (continuation_cursor IS NULL OR length(continuation_cursor) BETWEEN 1 AND 4096),
    CHECK (provider_entries_seen >= 0),
    CHECK (contacts_created >= 0),
    CHECK (contacts_updated >= 0),
    CHECK (contacts_unchanged >= 0),
    CHECK (provider_entries_written >= 0),
    CHECK (rejected_entries >= 0),
    CHECK ((state = 7 AND rejection_code BETWEEN 1 AND 8) OR (state != 7 AND rejection_code IS NULL)),
    CHECK (created_at_unix_millis > 0),
    CHECK (updated_at_unix_millis >= created_at_unix_millis)
);

CREATE INDEX mail_contacts_sync_recoverable_idx
ON makosh_data.mail_contacts_sync_runs (logical_owner_id, state, state_revision)
WHERE state NOT IN (6, 7);

CREATE TABLE makosh_data.mail_contacts_sync_inbox (
    logical_owner_id TEXT NOT NULL,
    message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    run_id BYTEA NOT NULL,
    processed_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, message_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (length(run_id) = 16),
    CHECK (processed_at_unix_millis > 0)
);

CREATE TABLE makosh_data.mail_contacts_sync_outbox (
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

CREATE INDEX mail_contacts_sync_outbox_pending_idx
ON makosh_data.mail_contacts_sync_outbox (logical_owner_id, created_at_unix_millis, message_id)
WHERE published_at_unix_millis IS NULL;

CREATE TABLE makosh_data.mail_contacts_sync_realtime (
    realtime_sequence BIGSERIAL PRIMARY KEY,
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    state SMALLINT NOT NULL,
    state_revision BIGINT NOT NULL,
    rejection_code SMALLINT,
    occurred_at_unix_millis BIGINT NOT NULL,
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(run_id) = 16),
    CHECK (state BETWEEN 1 AND 7),
    CHECK (state_revision > 0),
    CHECK (rejection_code IS NULL OR rejection_code BETWEEN 1 AND 8),
    CHECK (occurred_at_unix_millis > 0)
);

CREATE INDEX mail_contacts_sync_realtime_owner_idx
ON makosh_data.mail_contacts_sync_realtime (logical_owner_id, realtime_sequence);
