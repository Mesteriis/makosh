CREATE TABLE makosh_data.mail_contacts_sync_scheduler_runs (
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    command_message_id BYTEA NOT NULL,
    lease_epoch BIGINT NOT NULL,
    lease_expires_at_unix_millis BIGINT NOT NULL,
    terminal_receipt_queued BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (logical_owner_id, run_id),
    UNIQUE (logical_owner_id, command_message_id),
    FOREIGN KEY (logical_owner_id, run_id)
        REFERENCES makosh_data.mail_contacts_sync_runs (logical_owner_id, run_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(run_id) = 16),
    CHECK (length(command_message_id) = 16),
    CHECK (lease_epoch > 0),
    CHECK (lease_expires_at_unix_millis > 0)
);

CREATE INDEX mail_contacts_sync_scheduler_terminal_pending_idx
ON makosh_data.mail_contacts_sync_scheduler_runs (
    logical_owner_id, terminal_receipt_queued, lease_expires_at_unix_millis, run_id
)
WHERE NOT terminal_receipt_queued;
