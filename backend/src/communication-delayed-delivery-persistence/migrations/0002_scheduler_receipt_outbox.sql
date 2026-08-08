CREATE TABLE makosh_data.communication_delayed_delivery_scheduler_receipt_outbox (
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  message_id BYTEA NOT NULL CHECK (octet_length(message_id) = 16),
  delayed_operation_id BYTEA NOT NULL CHECK (
    octet_length(delayed_operation_id) = 16
  ),
  receipt_kind TEXT NOT NULL CHECK (
    receipt_kind IN (
      'scheduler.job_run.acceptance.v1',
      'scheduler.job_run.result.v1'
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

CREATE INDEX communication_delayed_delivery_scheduler_receipt_pending_idx
  ON makosh_data.communication_delayed_delivery_scheduler_receipt_outbox (
    logical_owner_id,
    created_at_unix_millis,
    message_id
  )
  WHERE published_at_unix_millis IS NULL;
