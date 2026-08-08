CREATE TABLE makosh_data.communication_delivery_intent_ingress_inbox (
  command_message_id BYTEA PRIMARY KEY CHECK (
    octet_length(command_message_id) = 16
  ),
  envelope_sha256 BYTEA NOT NULL CHECK (
    octet_length(envelope_sha256) = 32
  ),
  correlation_id BYTEA NOT NULL CHECK (
    octet_length(correlation_id) = 16
  ),
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  intent_id BYTEA NOT NULL CHECK (octet_length(intent_id) = 16),
  consumed_at_unix_seconds BIGINT NOT NULL CHECK (
    consumed_at_unix_seconds > 0
  )
);

CREATE UNIQUE INDEX communication_delivery_intent_ingress_identity_idx
  ON makosh_data.communication_delivery_intent_ingress_inbox (
    logical_owner_id,
    intent_id
  );

CREATE TABLE makosh_data.communication_delivery_intent_ingress_result_outbox (
  message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
  envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
  exact_envelope_bytes BYTEA NOT NULL CHECK (
    octet_length(exact_envelope_bytes) BETWEEN 1 AND 1048576
  ),
  result_kind SMALLINT NOT NULL CHECK (result_kind BETWEEN 1 AND 2),
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  intent_id BYTEA NOT NULL CHECK (octet_length(intent_id) = 16),
  command_message_id BYTEA NOT NULL UNIQUE CHECK (
    octet_length(command_message_id) = 16
  ),
  created_at_unix_seconds BIGINT NOT NULL CHECK (
    created_at_unix_seconds > 0
  ),
  published_at_unix_seconds BIGINT CHECK (
    published_at_unix_seconds IS NULL
    OR published_at_unix_seconds >= created_at_unix_seconds
  ),
  FOREIGN KEY (command_message_id) REFERENCES
    makosh_data.communication_delivery_intent_ingress_inbox (
      command_message_id
    )
);

CREATE INDEX communication_delivery_intent_ingress_result_pending_idx
  ON makosh_data.communication_delivery_intent_ingress_result_outbox (
    created_at_unix_seconds,
    message_id
  )
  WHERE published_at_unix_seconds IS NULL;
