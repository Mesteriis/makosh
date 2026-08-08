CREATE TABLE makosh_data.communication_delayed_delivery_operations (
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  delayed_operation_id BYTEA NOT NULL CHECK (
    octet_length(delayed_operation_id) = 16
  ),
  delivery_operation_id BYTEA NOT NULL CHECK (
    octet_length(delivery_operation_id) = 16
  ),
  canonical_conversation_id BYTEA NOT NULL CHECK (
    octet_length(canonical_conversation_id) = 16
  ),
  canonical_reply_message_id BYTEA CHECK (
    canonical_reply_message_id IS NULL
    OR octet_length(canonical_reply_message_id) = 16
  ),
  request_fingerprint BYTEA NOT NULL CHECK (
    octet_length(request_fingerprint) = 32
  ),
  body_reference_id BYTEA NOT NULL CHECK (
    octet_length(body_reference_id) = 16
  ),
  body_declared_bytes BIGINT NOT NULL CHECK (
    body_declared_bytes BETWEEN 1 AND 65536
  ),
  body_sha256 BYTEA NOT NULL CHECK (octet_length(body_sha256) = 32),
  body_custody_proof BYTEA NOT NULL CHECK (
    octet_length(body_custody_proof) BETWEEN 1 AND 2048
  ),
  deliver_at_unix_millis BIGINT NOT NULL CHECK (
    deliver_at_unix_millis > 0
  ),
  state SMALLINT NOT NULL CHECK (state BETWEEN 1 AND 9),
  state_revision BIGINT NOT NULL DEFAULT 1 CHECK (state_revision > 0),
  scheduler_schedule_revision BIGINT CHECK (
    scheduler_schedule_revision IS NULL OR scheduler_schedule_revision > 0
  ),
  scheduler_run_id BYTEA CHECK (
    scheduler_run_id IS NULL OR octet_length(scheduler_run_id) = 16
  ),
  scheduler_lease_epoch BIGINT CHECK (
    scheduler_lease_epoch IS NULL OR scheduler_lease_epoch > 0
  ),
  scheduler_lease_expires_at_unix_millis BIGINT CHECK (
    scheduler_lease_expires_at_unix_millis IS NULL
    OR scheduler_lease_expires_at_unix_millis > 0
  ),
  error_code SMALLINT CHECK (
    error_code IS NULL OR error_code BETWEEN 1 AND 7
  ),
  created_at_unix_millis BIGINT NOT NULL CHECK (
    created_at_unix_millis > 0
  ),
  updated_at_unix_millis BIGINT NOT NULL CHECK (
    updated_at_unix_millis > 0
  ),
  PRIMARY KEY (logical_owner_id, delayed_operation_id),
  UNIQUE (logical_owner_id, delivery_operation_id),
  CHECK (
    (scheduler_run_id IS NULL
      AND scheduler_lease_epoch IS NULL
      AND scheduler_lease_expires_at_unix_millis IS NULL)
    OR (scheduler_run_id IS NOT NULL
      AND scheduler_lease_epoch IS NOT NULL
      AND scheduler_lease_expires_at_unix_millis IS NOT NULL)
  )
);

CREATE TABLE makosh_data.communication_delayed_delivery_scheduler_inbox (
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  message_id BYTEA NOT NULL CHECK (octet_length(message_id) = 16),
  envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
  delayed_operation_id BYTEA NOT NULL CHECK (
    octet_length(delayed_operation_id) = 16
  ),
  received_at_unix_millis BIGINT NOT NULL CHECK (
    received_at_unix_millis > 0
  ),
  PRIMARY KEY (logical_owner_id, message_id),
  FOREIGN KEY (logical_owner_id, delayed_operation_id) REFERENCES
    makosh_data.communication_delayed_delivery_operations (
      logical_owner_id,
      delayed_operation_id
    )
);

CREATE TABLE makosh_data.communication_delayed_delivery_outbox (
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  message_id BYTEA NOT NULL CHECK (octet_length(message_id) = 16),
  delayed_operation_id BYTEA NOT NULL CHECK (
    octet_length(delayed_operation_id) = 16
  ),
  contract_kind TEXT NOT NULL CHECK (
    contract_kind IN (
      'scheduler.schedule.command.v1',
      'communication.delayed_delivery.status_changed.v1'
    )
  ),
  envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
  envelope_bytes BYTEA NOT NULL CHECK (
    octet_length(envelope_bytes) BETWEEN 1 AND 131072
  ),
  created_at_unix_millis BIGINT NOT NULL CHECK (
    created_at_unix_millis > 0
  ),
  published_at_unix_millis BIGINT CHECK (
    published_at_unix_millis IS NULL OR published_at_unix_millis > 0
  ),
  PRIMARY KEY (logical_owner_id, message_id),
  FOREIGN KEY (logical_owner_id, delayed_operation_id) REFERENCES
    makosh_data.communication_delayed_delivery_operations (
      logical_owner_id,
      delayed_operation_id
    )
);

CREATE INDEX communication_delayed_delivery_outbox_pending_idx
  ON makosh_data.communication_delayed_delivery_outbox (
    logical_owner_id,
    created_at_unix_millis,
    message_id
  )
  WHERE published_at_unix_millis IS NULL;
