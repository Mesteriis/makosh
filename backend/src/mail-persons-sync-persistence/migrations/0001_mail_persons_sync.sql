CREATE TABLE makosh_data.mail_persons_sync_runs (
    logical_owner_id TEXT NOT NULL CHECK (octet_length(logical_owner_id) BETWEEN 1 AND 128)
        CHECK (logical_owner_id ~ '^[a-z0-9._-]+$'),
    account_public_id BYTEA NOT NULL CHECK (octet_length(account_public_id) = 16),
    run_id BYTEA NOT NULL CHECK (octet_length(run_id) = 16),
    run_fingerprint BYTEA NOT NULL CHECK (octet_length(run_fingerprint) = 32),
    state SMALLINT NOT NULL CHECK (state BETWEEN 1 AND 4),
    state_revision BIGINT NOT NULL CHECK (state_revision > 0),
    next_page_sequence BIGINT NOT NULL CHECK (next_page_sequence > 0 AND next_page_sequence <= 4096),
    processed_pages BIGINT NOT NULL CHECK (processed_pages BETWEEN 0 AND 4096),
    processed_sources BIGINT NOT NULL CHECK (processed_sources BETWEEN 0 AND 2048000),
    rejection_code SMALLINT CHECK (rejection_code IS NULL OR rejection_code BETWEEN 1 AND 4),
    created_at_unix_millis BIGINT NOT NULL CHECK (created_at_unix_millis > 0),
    updated_at_unix_millis BIGINT NOT NULL CHECK (updated_at_unix_millis >= created_at_unix_millis),
    PRIMARY KEY (logical_owner_id, run_id),
    UNIQUE (logical_owner_id, account_public_id, run_id)
);

CREATE UNIQUE INDEX mail_persons_sync_one_active_account_run
ON makosh_data.mail_persons_sync_runs (logical_owner_id, account_public_id)
WHERE state IN (1, 2);

CREATE TABLE makosh_data.mail_persons_sync_scheduler_runs (
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    scheduler_message_id BYTEA NOT NULL CHECK (octet_length(scheduler_message_id) = 16),
    scheduler_envelope_sha256 BYTEA NOT NULL CHECK (octet_length(scheduler_envelope_sha256) = 32),
    lease_epoch BIGINT NOT NULL CHECK (lease_epoch > 0),
    lease_expires_at_unix_millis BIGINT NOT NULL CHECK (lease_expires_at_unix_millis > 0),
    acceptance_queued BOOLEAN NOT NULL,
    terminal_queued BOOLEAN NOT NULL,
    PRIMARY KEY (logical_owner_id, run_id),
    UNIQUE (logical_owner_id, scheduler_message_id),
    FOREIGN KEY (logical_owner_id, run_id)
        REFERENCES makosh_data.mail_persons_sync_runs(logical_owner_id, run_id) ON DELETE RESTRICT
);

CREATE TABLE makosh_data.mail_persons_sync_inbox (
    logical_owner_id TEXT NOT NULL,
    message_id BYTEA NOT NULL CHECK (octet_length(message_id) = 16),
    envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
    semantic_kind SMALLINT NOT NULL CHECK (semantic_kind BETWEEN 1 AND 10),
    account_public_id BYTEA NOT NULL CHECK (octet_length(account_public_id) = 16),
    run_id BYTEA NOT NULL CHECK (octet_length(run_id) = 16),
    page_sequence BIGINT CHECK (page_sequence IS NULL OR page_sequence BETWEEN 1 AND 4096),
    command_id BYTEA CHECK (command_id IS NULL OR octet_length(command_id) = 16),
    command_fingerprint BYTEA CHECK (command_fingerprint IS NULL OR octet_length(command_fingerprint) = 32),
    processed_at_unix_millis BIGINT NOT NULL CHECK (processed_at_unix_millis > 0),
    PRIMARY KEY (logical_owner_id, message_id),
    FOREIGN KEY (logical_owner_id, run_id)
        REFERENCES makosh_data.mail_persons_sync_runs(logical_owner_id, run_id) ON DELETE RESTRICT
);

CREATE TABLE makosh_data.mail_persons_sync_pages (
    logical_owner_id TEXT NOT NULL,
    account_public_id BYTEA NOT NULL,
    run_id BYTEA NOT NULL,
    page_sequence BIGINT NOT NULL CHECK (page_sequence BETWEEN 1 AND 4096),
    page_digest BYTEA CHECK (page_digest IS NULL OR octet_length(page_digest) = 32),
    observed_sources INTEGER NOT NULL CHECK (observed_sources BETWEEN 0 AND 500),
    updated_sources INTEGER NOT NULL CHECK (updated_sources BETWEEN 0 AND 500),
    removed_sources INTEGER NOT NULL CHECK (removed_sources BETWEEN 0 AND 500),
    staged_sources INTEGER NOT NULL CHECK (staged_sources BETWEEN 0 AND 500),
    has_more BOOLEAN,
    completed_message_id BYTEA CHECK (completed_message_id IS NULL OR octet_length(completed_message_id) = 16),
    completed_envelope_sha256 BYTEA CHECK (completed_envelope_sha256 IS NULL OR octet_length(completed_envelope_sha256) = 32),
    receipt_id BYTEA CHECK (receipt_id IS NULL OR octet_length(receipt_id) = 16),
    receipt_envelope_sha256 BYTEA CHECK (receipt_envelope_sha256 IS NULL OR octet_length(receipt_envelope_sha256) = 32),
    receipt_envelope_bytes BYTEA CHECK (receipt_envelope_bytes IS NULL OR octet_length(receipt_envelope_bytes) BETWEEN 1 AND 262144),
    continuation_kind SMALLINT CHECK (continuation_kind IS NULL OR continuation_kind BETWEEN 1 AND 3),
    next_fetch_id BYTEA CHECK (next_fetch_id IS NULL OR octet_length(next_fetch_id) = 16),
    next_fetch_envelope_sha256 BYTEA CHECK (next_fetch_envelope_sha256 IS NULL OR octet_length(next_fetch_envelope_sha256) = 32),
    next_fetch_envelope_bytes BYTEA CHECK (next_fetch_envelope_bytes IS NULL OR octet_length(next_fetch_envelope_bytes) BETWEEN 1 AND 262144),
    run_result_id BYTEA CHECK (run_result_id IS NULL OR octet_length(run_result_id) = 16),
    run_result_envelope_sha256 BYTEA CHECK (run_result_envelope_sha256 IS NULL OR octet_length(run_result_envelope_sha256) = 32),
    run_result_envelope_bytes BYTEA CHECK (run_result_envelope_bytes IS NULL OR octet_length(run_result_envelope_bytes) BETWEEN 1 AND 262144),
    scheduler_terminal_id BYTEA CHECK (scheduler_terminal_id IS NULL OR octet_length(scheduler_terminal_id) = 16),
    scheduler_terminal_envelope_sha256 BYTEA CHECK (scheduler_terminal_envelope_sha256 IS NULL OR octet_length(scheduler_terminal_envelope_sha256) = 32),
    scheduler_terminal_envelope_bytes BYTEA CHECK (scheduler_terminal_envelope_bytes IS NULL OR octet_length(scheduler_terminal_envelope_bytes) BETWEEN 1 AND 262144),
    rejection_code SMALLINT CHECK (rejection_code IS NULL OR rejection_code BETWEEN 1 AND 4),
    continuation_queued BOOLEAN NOT NULL DEFAULT FALSE,
    completed_at_unix_millis BIGINT CHECK (completed_at_unix_millis IS NULL OR completed_at_unix_millis > 0),
    PRIMARY KEY (logical_owner_id, run_id, page_sequence),
    FOREIGN KEY (logical_owner_id, account_public_id, run_id)
        REFERENCES makosh_data.mail_persons_sync_runs(logical_owner_id, account_public_id, run_id) ON DELETE RESTRICT,
    CHECK (observed_sources + updated_sources + removed_sources <= 500),
    CHECK ((completed_message_id IS NULL) = (completed_envelope_sha256 IS NULL)),
    CHECK ((page_digest IS NULL) = (completed_message_id IS NULL)),
    CHECK ((receipt_id IS NULL) = (receipt_envelope_sha256 IS NULL)),
    CHECK ((receipt_id IS NULL) = (receipt_envelope_bytes IS NULL))
    ,CHECK ((continuation_kind IS NULL) = (completed_message_id IS NULL))
    ,CHECK ((next_fetch_id IS NULL) = (next_fetch_envelope_sha256 IS NULL))
    ,CHECK ((next_fetch_id IS NULL) = (next_fetch_envelope_bytes IS NULL))
    ,CHECK ((run_result_id IS NULL) = (run_result_envelope_sha256 IS NULL))
    ,CHECK ((run_result_id IS NULL) = (run_result_envelope_bytes IS NULL))
    ,CHECK ((scheduler_terminal_id IS NULL) = (scheduler_terminal_envelope_sha256 IS NULL))
    ,CHECK ((scheduler_terminal_id IS NULL) = (scheduler_terminal_envelope_bytes IS NULL))
    ,CHECK (
        continuation_kind IS NULL OR
        (continuation_kind = 1 AND next_fetch_id IS NOT NULL AND run_result_id IS NULL AND scheduler_terminal_id IS NULL) OR
        (continuation_kind = 2 AND next_fetch_id IS NULL AND run_result_id IS NOT NULL AND scheduler_terminal_id IS NOT NULL) OR
        (continuation_kind = 3 AND next_fetch_id IS NULL AND run_result_id IS NULL AND scheduler_terminal_id IS NULL)
    )
);

CREATE TABLE makosh_data.mail_persons_sync_sources (
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    page_sequence BIGINT NOT NULL,
    observation_message_id BYTEA NOT NULL CHECK (octet_length(observation_message_id) = 16),
    observation_envelope_sha256 BYTEA NOT NULL CHECK (octet_length(observation_envelope_sha256) = 32),
    integration_public_id BYTEA NOT NULL CHECK (octet_length(integration_public_id) = 16),
    account_public_id BYTEA NOT NULL CHECK (octet_length(account_public_id) = 16),
    provider_source_contact_public_id BYTEA NOT NULL CHECK (octet_length(provider_source_contact_public_id) = 16),
    change_kind SMALLINT NOT NULL CHECK (change_kind BETWEEN 1 AND 3),
    source_revision BIGINT NOT NULL CHECK (source_revision > 0),
    source_digest BYTEA NOT NULL CHECK (octet_length(source_digest) = 32),
    persons_command_id BYTEA NOT NULL CHECK (octet_length(persons_command_id) = 16),
    persons_command_fingerprint BYTEA NOT NULL CHECK (octet_length(persons_command_fingerprint) = 32),
    persons_command_envelope_sha256 BYTEA NOT NULL CHECK (octet_length(persons_command_envelope_sha256) = 32),
    persons_command_envelope_bytes BYTEA NOT NULL CHECK (octet_length(persons_command_envelope_bytes) BETWEEN 1 AND 262144),
    persons_result_message_id BYTEA CHECK (persons_result_message_id IS NULL OR octet_length(persons_result_message_id) = 16),
    persons_result_envelope_sha256 BYTEA CHECK (persons_result_envelope_sha256 IS NULL OR octet_length(persons_result_envelope_sha256) = 32),
    outcome SMALLINT CHECK (outcome IS NULL OR outcome BETWEEN 1 AND 2),
    PRIMARY KEY (logical_owner_id, run_id, page_sequence, provider_source_contact_public_id),
    UNIQUE (logical_owner_id, observation_message_id),
    UNIQUE (logical_owner_id, persons_command_id),
    FOREIGN KEY (logical_owner_id, run_id, page_sequence)
        REFERENCES makosh_data.mail_persons_sync_pages(logical_owner_id, run_id, page_sequence) ON DELETE RESTRICT,
    CHECK ((persons_result_message_id IS NULL) = (persons_result_envelope_sha256 IS NULL)),
    CHECK ((outcome IS NULL) = (persons_result_message_id IS NULL))
);

CREATE TABLE makosh_data.mail_persons_sync_outbox (
    logical_owner_id TEXT NOT NULL,
    message_id BYTEA NOT NULL CHECK (octet_length(message_id) = 16),
    envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
    envelope_bytes BYTEA NOT NULL CHECK (octet_length(envelope_bytes) BETWEEN 1 AND 262144),
    run_id BYTEA NOT NULL CHECK (octet_length(run_id) = 16),
    page_sequence BIGINT NOT NULL CHECK (page_sequence BETWEEN 0 AND 4096),
    semantic_kind SMALLINT NOT NULL CHECK (semantic_kind BETWEEN 1 AND 8),
    semantic_order_key BYTEA NOT NULL CHECK (octet_length(semantic_order_key) BETWEEN 12 AND 64),
    source_ordinal INTEGER NOT NULL CHECK (source_ordinal BETWEEN 0 AND 502),
    created_at_unix_millis BIGINT NOT NULL CHECK (created_at_unix_millis > 0),
    published_at_unix_millis BIGINT CHECK (published_at_unix_millis IS NULL OR published_at_unix_millis >= created_at_unix_millis),
    superseded_by_run_id BYTEA CHECK (superseded_by_run_id IS NULL OR octet_length(superseded_by_run_id) = 16),
    superseded_at_unix_millis BIGINT CHECK (superseded_at_unix_millis IS NULL OR superseded_at_unix_millis >= created_at_unix_millis),
    PRIMARY KEY (logical_owner_id, message_id),
    UNIQUE (logical_owner_id, run_id, semantic_order_key),
    FOREIGN KEY (logical_owner_id, run_id)
        REFERENCES makosh_data.mail_persons_sync_runs(logical_owner_id, run_id) ON DELETE RESTRICT,
    CHECK ((superseded_by_run_id IS NULL) = (superseded_at_unix_millis IS NULL))
);

CREATE INDEX mail_persons_sync_pending_outbox
ON makosh_data.mail_persons_sync_outbox (logical_owner_id, run_id, semantic_order_key, message_id)
WHERE published_at_unix_millis IS NULL AND superseded_by_run_id IS NULL;

ALTER TABLE makosh_data.mail_persons_sync_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.mail_persons_sync_runs FORCE ROW LEVEL SECURITY;
CREATE POLICY mail_persons_sync_runs_owner_rls ON makosh_data.mail_persons_sync_runs
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.mail_persons_sync_scheduler_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.mail_persons_sync_scheduler_runs FORCE ROW LEVEL SECURITY;
CREATE POLICY mail_persons_sync_scheduler_runs_owner_rls ON makosh_data.mail_persons_sync_scheduler_runs
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.mail_persons_sync_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.mail_persons_sync_inbox FORCE ROW LEVEL SECURITY;
CREATE POLICY mail_persons_sync_inbox_owner_rls ON makosh_data.mail_persons_sync_inbox
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.mail_persons_sync_pages ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.mail_persons_sync_pages FORCE ROW LEVEL SECURITY;
CREATE POLICY mail_persons_sync_pages_owner_rls ON makosh_data.mail_persons_sync_pages
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.mail_persons_sync_sources ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.mail_persons_sync_sources FORCE ROW LEVEL SECURITY;
CREATE POLICY mail_persons_sync_sources_owner_rls ON makosh_data.mail_persons_sync_sources
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.mail_persons_sync_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.mail_persons_sync_outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY mail_persons_sync_outbox_owner_rls ON makosh_data.mail_persons_sync_outbox
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
