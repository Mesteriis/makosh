ALTER TABLE makosh_data.persons_outbox
    ADD COLUMN command_message_id BYTEA NOT NULL DEFAULT decode(repeat('00', 16), 'hex'),
    ADD COLUMN resulting_owner_revision BIGINT NOT NULL DEFAULT -1,
    ADD COLUMN outbox_ordinal INTEGER NOT NULL DEFAULT -1,
    ADD COLUMN semantic_order_key BYTEA NOT NULL DEFAULT ''::BYTEA;

-- V2 rows predate command ordering metadata. Preserve them as independent
-- one-record legacy commands; all V3 writes use the exact originating command.
UPDATE makosh_data.persons_outbox
SET command_message_id = message_id,
    resulting_owner_revision = 0,
    outbox_ordinal = 0,
    semantic_order_key = decode('00', 'hex');

ALTER TABLE makosh_data.persons_outbox
    ADD CONSTRAINT persons_outbox_command_message_id_size
        CHECK (octet_length(command_message_id) = 16
               AND command_message_id <> decode(repeat('00', 16), 'hex')),
    ADD CONSTRAINT persons_outbox_resulting_owner_revision_bounds
        CHECK (resulting_owner_revision >= 0),
    ADD CONSTRAINT persons_outbox_ordinal_bounds
        CHECK (outbox_ordinal BETWEEN 0 AND 256),
    ADD CONSTRAINT persons_outbox_semantic_order_key_bounds
        CHECK (octet_length(semantic_order_key) BETWEEN 1 AND 128),
    ADD CONSTRAINT persons_outbox_terminal_order_key
        CHECK (outbox_ordinal <> 0 OR semantic_order_key = decode('00', 'hex'));

CREATE UNIQUE INDEX persons_outbox_command_ordinal_unique
    ON makosh_data.persons_outbox (logical_owner_id, command_message_id, outbox_ordinal);

CREATE INDEX persons_outbox_pending_semantic_order
    ON makosh_data.persons_outbox
    (logical_owner_id, resulting_owner_revision, created_at_unix_millis, command_message_id,
     semantic_order_key,
     outbox_ordinal, message_id)
    WHERE published_at_unix_millis IS NULL;
