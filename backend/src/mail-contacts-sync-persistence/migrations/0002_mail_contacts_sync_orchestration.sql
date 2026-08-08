CREATE TABLE makosh_data.mail_contacts_sync_pages (
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    page_sequence BIGINT NOT NULL,
    expected_entries BIGINT NOT NULL,
    next_continuation_cursor BYTEA,
    completed_message_id BYTEA NOT NULL,
    completed_envelope_sha256 BYTEA NOT NULL,
    PRIMARY KEY (logical_owner_id, run_id, page_sequence),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(run_id) = 16),
    CHECK (page_sequence > 0),
    CHECK (expected_entries BETWEEN 0 AND 500),
    CHECK (next_continuation_cursor IS NULL OR length(next_continuation_cursor) BETWEEN 1 AND 4096),
    CHECK (length(completed_message_id) = 16),
    CHECK (length(completed_envelope_sha256) = 32)
);

CREATE TABLE makosh_data.mail_contacts_sync_entries (
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    page_sequence BIGINT NOT NULL,
    contact_command_id BYTEA NOT NULL,
    entry_digest BYTEA NOT NULL,
    observation_message_id BYTEA NOT NULL,
    observation_envelope_sha256 BYTEA NOT NULL,
    outcome SMALLINT NOT NULL,
    outcome_message_id BYTEA,
    outcome_envelope_sha256 BYTEA,
    outcome_accounted BOOLEAN NOT NULL,
    PRIMARY KEY (logical_owner_id, contact_command_id),
    UNIQUE (logical_owner_id, observation_message_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(run_id) = 16),
    CHECK (page_sequence > 0),
    CHECK (length(contact_command_id) = 16),
    CHECK (length(entry_digest) = 32),
    CHECK (length(observation_message_id) = 16),
    CHECK (length(observation_envelope_sha256) = 32),
    CHECK (outcome BETWEEN 0 AND 4),
    CHECK ((outcome = 0 AND outcome_message_id IS NULL AND outcome_envelope_sha256 IS NULL AND NOT outcome_accounted)
        OR (outcome != 0 AND length(outcome_message_id) = 16 AND length(outcome_envelope_sha256) = 32)),
    FOREIGN KEY (logical_owner_id, run_id)
        REFERENCES makosh_data.mail_contacts_sync_runs (logical_owner_id, run_id)
);

CREATE INDEX mail_contacts_sync_entries_run_idx
ON makosh_data.mail_contacts_sync_entries (
    logical_owner_id, run_id, page_sequence, outcome_accounted, contact_command_id
);
