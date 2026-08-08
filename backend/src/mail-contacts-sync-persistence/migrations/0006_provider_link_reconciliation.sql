CREATE TABLE makosh_data.mail_contacts_sync_provider_link_reconciliation (
    logical_owner_id TEXT NOT NULL,
    operation_id BYTEA NOT NULL,
    mail_result_message_id BYTEA NOT NULL,
    mail_result_envelope_sha256 BYTEA NOT NULL,
    contacts_command_message_id BYTEA NOT NULL,
    state SMALLINT NOT NULL DEFAULT 1,
    terminal_message_id BYTEA,
    reject_code SMALLINT,
    created_at_unix_millis BIGINT NOT NULL,
    updated_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, operation_id),
    UNIQUE (logical_owner_id, mail_result_message_id),
    UNIQUE (logical_owner_id, contacts_command_message_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(operation_id) = 16),
    CHECK (length(mail_result_message_id) = 16),
    CHECK (length(mail_result_envelope_sha256) = 32),
    CHECK (length(contacts_command_message_id) = 16),
    CHECK (state IN (1, 2, 3)),
    CHECK (terminal_message_id IS NULL OR length(terminal_message_id) = 16),
    CHECK (reject_code IS NULL OR reject_code IN (1, 2, 3, 4, 5)),
    CHECK (created_at_unix_millis > 0),
    CHECK (updated_at_unix_millis >= created_at_unix_millis),
    CHECK (
        (state = 1 AND terminal_message_id IS NULL AND reject_code IS NULL)
        OR (state = 2 AND length(terminal_message_id) = 16 AND reject_code IS NULL)
        OR (state = 3 AND length(terminal_message_id) = 16 AND reject_code IS NOT NULL)
    )
);
