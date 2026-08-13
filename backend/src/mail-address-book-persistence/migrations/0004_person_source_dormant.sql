CREATE TABLE makosh_data.mail_address_book_person_source_accounts (
    logical_owner_id TEXT NOT NULL CHECK (octet_length(logical_owner_id) BETWEEN 1 AND 128)
        CHECK (logical_owner_id ~ '^[a-z0-9._-]+$'),
    private_account_key TEXT NOT NULL CHECK (octet_length(private_account_key) BETWEEN 1 AND 256),
    integration_public_id BYTEA NOT NULL CHECK (octet_length(integration_public_id) = 16),
    account_public_id BYTEA NOT NULL CHECK (octet_length(account_public_id) = 16),
    mapping_revision BIGINT NOT NULL CHECK (mapping_revision > 0),
    created_at_unix_millis BIGINT NOT NULL CHECK (created_at_unix_millis > 0),
    PRIMARY KEY (logical_owner_id, private_account_key),
    UNIQUE (logical_owner_id, account_public_id),
    UNIQUE (logical_owner_id, integration_public_id, account_public_id)
);

CREATE TABLE makosh_data.mail_address_book_person_sources (
    logical_owner_id TEXT NOT NULL,
    account_public_id BYTEA NOT NULL,
    provider_record_key BYTEA NOT NULL CHECK (octet_length(provider_record_key) BETWEEN 1 AND 512),
    provider_record_etag BYTEA CHECK (provider_record_etag IS NULL OR octet_length(provider_record_etag) BETWEEN 1 AND 512),
    provider_source_contact_public_id BYTEA NOT NULL CHECK (octet_length(provider_source_contact_public_id) = 16),
    claims_digest BYTEA NOT NULL CHECK (octet_length(claims_digest) = 32),
    source_revision BIGINT NOT NULL CHECK (source_revision > 0),
    active BOOLEAN NOT NULL,
    last_terminal_run_id BYTEA CHECK (last_terminal_run_id IS NULL OR octet_length(last_terminal_run_id) = 16),
    updated_at_unix_millis BIGINT NOT NULL CHECK (updated_at_unix_millis > 0),
    PRIMARY KEY (logical_owner_id, account_public_id, provider_record_key),
    UNIQUE (logical_owner_id, account_public_id, provider_source_contact_public_id),
    FOREIGN KEY (logical_owner_id, account_public_id)
        REFERENCES makosh_data.mail_address_book_person_source_accounts(logical_owner_id, account_public_id)
        ON DELETE RESTRICT
);

CREATE TABLE makosh_data.mail_address_book_person_source_runs (
    logical_owner_id TEXT NOT NULL,
    account_public_id BYTEA NOT NULL,
    run_id BYTEA NOT NULL CHECK (octet_length(run_id) = 16),
    provider_snapshot_generation BYTEA NOT NULL CHECK (octet_length(provider_snapshot_generation) BETWEEN 1 AND 512),
    provider_cursor BYTEA CHECK (provider_cursor IS NULL OR octet_length(provider_cursor) BETWEEN 1 AND 4096),
    page_sequence BIGINT NOT NULL CHECK (page_sequence BETWEEN 1 AND 4096),
    state SMALLINT NOT NULL CHECK (state BETWEEN 1 AND 4),
    terminal_snapshot_succeeded BOOLEAN NOT NULL,
    terminal_command_id BYTEA CHECK (terminal_command_id IS NULL OR octet_length(terminal_command_id) = 16),
    terminal_envelope_sha256 BYTEA CHECK (terminal_envelope_sha256 IS NULL OR octet_length(terminal_envelope_sha256) = 32),
    terminal_envelope_bytes BYTEA CHECK (terminal_envelope_bytes IS NULL OR octet_length(terminal_envelope_bytes) BETWEEN 1 AND 262144),
    terminal_fingerprint BYTEA CHECK (terminal_fingerprint IS NULL OR octet_length(terminal_fingerprint) = 32),
    terminal_plan_sha256 BYTEA CHECK (terminal_plan_sha256 IS NULL OR octet_length(terminal_plan_sha256) = 32),
    created_at_unix_millis BIGINT NOT NULL CHECK (created_at_unix_millis > 0),
    updated_at_unix_millis BIGINT NOT NULL CHECK (updated_at_unix_millis >= created_at_unix_millis),
    PRIMARY KEY (logical_owner_id, account_public_id, run_id),
    FOREIGN KEY (logical_owner_id, account_public_id)
        REFERENCES makosh_data.mail_address_book_person_source_accounts(logical_owner_id, account_public_id)
        ON DELETE RESTRICT,
    CHECK (NOT terminal_snapshot_succeeded OR state = 3),
    CHECK ((terminal_command_id IS NULL) = (terminal_envelope_sha256 IS NULL)),
    CHECK ((terminal_command_id IS NULL) = (terminal_envelope_bytes IS NULL)),
    CHECK ((terminal_command_id IS NULL) = (terminal_fingerprint IS NULL)),
    CHECK ((terminal_command_id IS NULL) = (terminal_plan_sha256 IS NULL)),
    CHECK (terminal_snapshot_succeeded = (terminal_command_id IS NOT NULL))
);

CREATE TABLE makosh_data.mail_address_book_person_source_seen (
    logical_owner_id TEXT NOT NULL,
    account_public_id BYTEA NOT NULL,
    run_id BYTEA NOT NULL,
    provider_source_contact_public_id BYTEA NOT NULL CHECK (octet_length(provider_source_contact_public_id) = 16),
    PRIMARY KEY (logical_owner_id, account_public_id, run_id, provider_source_contact_public_id),
    FOREIGN KEY (logical_owner_id, account_public_id, run_id)
        REFERENCES makosh_data.mail_address_book_person_source_runs(logical_owner_id, account_public_id, run_id)
        ON DELETE RESTRICT
);

CREATE TABLE makosh_data.mail_address_book_person_source_fetch_inbox (
    logical_owner_id TEXT NOT NULL,
    command_id BYTEA NOT NULL CHECK (octet_length(command_id) = 16),
    envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
    envelope_bytes BYTEA NOT NULL CHECK (octet_length(envelope_bytes) BETWEEN 1 AND 262144),
    request_sha256 BYTEA NOT NULL CHECK (octet_length(request_sha256) = 32),
    plan_sha256 BYTEA NOT NULL CHECK (octet_length(plan_sha256) = 32),
    account_public_id BYTEA NOT NULL CHECK (octet_length(account_public_id) = 16),
    run_id BYTEA NOT NULL CHECK (octet_length(run_id) = 16),
    page_sequence BIGINT NOT NULL CHECK (page_sequence BETWEEN 1 AND 4096),
    processed_at_unix_millis BIGINT NOT NULL CHECK (processed_at_unix_millis > 0),
    PRIMARY KEY (logical_owner_id, command_id),
    UNIQUE (logical_owner_id, account_public_id, run_id, page_sequence),
    FOREIGN KEY (logical_owner_id, account_public_id, run_id)
        REFERENCES makosh_data.mail_address_book_person_source_runs(logical_owner_id, account_public_id, run_id)
        ON DELETE RESTRICT
);

CREATE TABLE makosh_data.mail_address_book_person_source_fetch_outbox (
    logical_owner_id TEXT NOT NULL,
    message_id BYTEA NOT NULL CHECK (octet_length(message_id) = 16),
    envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
    envelope_bytes BYTEA NOT NULL CHECK (octet_length(envelope_bytes) BETWEEN 1 AND 262144),
    account_public_id BYTEA NOT NULL CHECK (octet_length(account_public_id) = 16),
    run_id BYTEA NOT NULL CHECK (octet_length(run_id) = 16),
    page_sequence BIGINT NOT NULL CHECK (page_sequence BETWEEN 1 AND 4096),
    semantic_order_key BYTEA NOT NULL CHECK (octet_length(semantic_order_key) BETWEEN 1 AND 64),
    created_at_unix_millis BIGINT NOT NULL CHECK (created_at_unix_millis > 0),
    published_at_unix_millis BIGINT CHECK (published_at_unix_millis IS NULL OR published_at_unix_millis >= created_at_unix_millis),
    PRIMARY KEY (logical_owner_id, message_id),
    UNIQUE (logical_owner_id, account_public_id, run_id, semantic_order_key),
    FOREIGN KEY (logical_owner_id, account_public_id, run_id)
        REFERENCES makosh_data.mail_address_book_person_source_runs(logical_owner_id, account_public_id, run_id)
        ON DELETE RESTRICT
);

CREATE TABLE makosh_data.mail_address_book_person_source_lifecycle_outbox (
    logical_owner_id TEXT NOT NULL,
    outbox_sequence BIGINT GENERATED ALWAYS AS IDENTITY,
    message_id BYTEA NOT NULL CHECK (octet_length(message_id) = 16),
    envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
    envelope_bytes BYTEA NOT NULL CHECK (octet_length(envelope_bytes) BETWEEN 1 AND 262144),
    account_public_id BYTEA NOT NULL CHECK (octet_length(account_public_id) = 16),
    mapping_revision BIGINT NOT NULL CHECK (mapping_revision > 0),
    semantic_kind SMALLINT NOT NULL CHECK (semantic_kind IN (1, 2)),
    created_at_unix_millis BIGINT NOT NULL CHECK (created_at_unix_millis > 0),
    published_at_unix_millis BIGINT CHECK (
        published_at_unix_millis IS NULL OR published_at_unix_millis >= created_at_unix_millis
    ),
    PRIMARY KEY (logical_owner_id, outbox_sequence),
    UNIQUE (logical_owner_id, message_id),
    UNIQUE (logical_owner_id, account_public_id, mapping_revision, semantic_kind),
    FOREIGN KEY (logical_owner_id, account_public_id)
        REFERENCES makosh_data.mail_address_book_person_source_accounts(logical_owner_id, account_public_id)
        ON DELETE RESTRICT
);

CREATE INDEX mail_address_book_person_source_lifecycle_outbox_pending
ON makosh_data.mail_address_book_person_source_lifecycle_outbox
    (logical_owner_id, outbox_sequence)
WHERE published_at_unix_millis IS NULL;

CREATE INDEX mail_address_book_person_source_fetch_outbox_pending
ON makosh_data.mail_address_book_person_source_fetch_outbox
    (logical_owner_id, account_public_id, run_id, semantic_order_key, message_id)
WHERE published_at_unix_millis IS NULL;

ALTER TABLE makosh_data.mail_address_book_person_source_accounts ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.mail_address_book_person_source_accounts FORCE ROW LEVEL SECURITY;
CREATE POLICY mail_address_book_person_source_accounts_owner_rls ON makosh_data.mail_address_book_person_source_accounts
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.mail_address_book_person_sources ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.mail_address_book_person_sources FORCE ROW LEVEL SECURITY;
CREATE POLICY mail_address_book_person_sources_owner_rls ON makosh_data.mail_address_book_person_sources
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.mail_address_book_person_source_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.mail_address_book_person_source_runs FORCE ROW LEVEL SECURITY;
CREATE POLICY mail_address_book_person_source_runs_owner_rls ON makosh_data.mail_address_book_person_source_runs
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.mail_address_book_person_source_seen ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.mail_address_book_person_source_seen FORCE ROW LEVEL SECURITY;
CREATE POLICY mail_address_book_person_source_seen_owner_rls ON makosh_data.mail_address_book_person_source_seen
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.mail_address_book_person_source_fetch_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.mail_address_book_person_source_fetch_inbox FORCE ROW LEVEL SECURITY;
CREATE POLICY mail_address_book_person_source_fetch_inbox_owner_rls ON makosh_data.mail_address_book_person_source_fetch_inbox
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.mail_address_book_person_source_fetch_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.mail_address_book_person_source_fetch_outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY mail_address_book_person_source_fetch_outbox_owner_rls ON makosh_data.mail_address_book_person_source_fetch_outbox
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.mail_address_book_person_source_lifecycle_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.mail_address_book_person_source_lifecycle_outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY mail_address_book_person_source_lifecycle_outbox_owner_rls
ON makosh_data.mail_address_book_person_source_lifecycle_outbox
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
