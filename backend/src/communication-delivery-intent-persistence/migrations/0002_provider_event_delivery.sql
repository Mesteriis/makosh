CREATE TABLE makosh_data.communication_delivery_intent_provider_outbox (
  message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
  envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
  exact_envelope_bytes BYTEA NOT NULL CHECK (
    octet_length(exact_envelope_bytes) BETWEEN 1 AND 1048576
  ),
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  intent_id BYTEA NOT NULL CHECK (octet_length(intent_id) = 16),
  provider_kind SMALLINT NOT NULL CHECK (provider_kind BETWEEN 1 AND 6),
  claim_epoch BIGINT NOT NULL CHECK (claim_epoch > 0),
  created_at_unix_seconds BIGINT NOT NULL CHECK (
    created_at_unix_seconds > 0
  ),
  published_at_unix_seconds BIGINT CHECK (
    published_at_unix_seconds IS NULL
    OR published_at_unix_seconds >= created_at_unix_seconds
  ),
  FOREIGN KEY (logical_owner_id, intent_id) REFERENCES
    makosh_data.communication_delivery_intent_jobs (
      logical_owner_id,
      intent_id
    )
);

CREATE UNIQUE INDEX communication_delivery_intent_provider_outbox_intent_idx
  ON makosh_data.communication_delivery_intent_provider_outbox (
    logical_owner_id,
    intent_id
  );

CREATE INDEX communication_delivery_intent_provider_outbox_pending_idx
  ON makosh_data.communication_delivery_intent_provider_outbox (
    provider_kind,
    created_at_unix_seconds,
    message_id
  )
  WHERE published_at_unix_seconds IS NULL;

CREATE TABLE makosh_data.communication_delivery_intent_result_inbox (
  message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
  envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  intent_id BYTEA NOT NULL CHECK (octet_length(intent_id) = 16),
  command_message_id BYTEA NOT NULL CHECK (
    octet_length(command_message_id) = 16
  ),
  consumed_at_unix_seconds BIGINT NOT NULL CHECK (
    consumed_at_unix_seconds > 0
  ),
  FOREIGN KEY (logical_owner_id, intent_id) REFERENCES
    makosh_data.communication_delivery_intent_jobs (
      logical_owner_id,
      intent_id
    ),
  FOREIGN KEY (command_message_id) REFERENCES
    makosh_data.communication_delivery_intent_provider_outbox (message_id)
);
