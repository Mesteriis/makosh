CREATE TABLE makosh_data.communication_cross_channel_forward_event_inbox (
  message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
  envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
  event_kind SMALLINT NOT NULL CHECK (event_kind BETWEEN 1 AND 4),
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  forward_id BYTEA NOT NULL CHECK (octet_length(forward_id) = 16),
  consumed_at_unix_millis BIGINT NOT NULL CHECK (
    consumed_at_unix_millis > 0
  ),
  FOREIGN KEY (logical_owner_id, forward_id) REFERENCES
    makosh_data.communication_cross_channel_forward_operations (
      logical_owner_id,
      forward_id
    )
);

CREATE TABLE makosh_data.communication_cross_channel_forward_event_outbox (
  message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
  envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
  exact_envelope_bytes BYTEA NOT NULL CHECK (
    octet_length(exact_envelope_bytes) BETWEEN 1 AND 1048576
  ),
  event_kind SMALLINT NOT NULL CHECK (event_kind IN (1, 2)),
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  forward_id BYTEA NOT NULL CHECK (octet_length(forward_id) = 16),
  created_at_unix_millis BIGINT NOT NULL CHECK (
    created_at_unix_millis > 0
  ),
  published_at_unix_millis BIGINT CHECK (
    published_at_unix_millis IS NULL
    OR published_at_unix_millis >= created_at_unix_millis
  ),
  UNIQUE (logical_owner_id, forward_id, event_kind),
  FOREIGN KEY (logical_owner_id, forward_id) REFERENCES
    makosh_data.communication_cross_channel_forward_operations (
      logical_owner_id,
      forward_id
    )
);

CREATE INDEX communication_cross_channel_forward_event_outbox_pending_idx
  ON makosh_data.communication_cross_channel_forward_event_outbox (
    created_at_unix_millis,
    message_id
  )
  WHERE published_at_unix_millis IS NULL;

ALTER TABLE makosh_data.communication_cross_channel_forward_operations
  ADD COLUMN source_evidence_id BYTEA CHECK (
    source_evidence_id IS NULL OR octet_length(source_evidence_id) = 16
  ),
  ADD COLUMN source_result_message_id BYTEA UNIQUE REFERENCES
    makosh_data.communication_cross_channel_forward_event_inbox (message_id),
  ADD COLUMN delivery_body_reference_id BYTEA CHECK (
    delivery_body_reference_id IS NULL
    OR octet_length(delivery_body_reference_id) = 16
  ),
  ADD COLUMN delivery_body_declared_bytes BIGINT CHECK (
    delivery_body_declared_bytes IS NULL
    OR delivery_body_declared_bytes BETWEEN 1 AND 16777216
  ),
  ADD COLUMN delivery_body_sha256 BYTEA CHECK (
    delivery_body_sha256 IS NULL OR octet_length(delivery_body_sha256) = 32
  ),
  ADD COLUMN delivery_body_custody_proof BYTEA CHECK (
    delivery_body_custody_proof IS NULL
    OR octet_length(delivery_body_custody_proof) BETWEEN 1 AND 2048
  ),
  ADD COLUMN delivery_submit_message_id BYTEA UNIQUE REFERENCES
    makosh_data.communication_cross_channel_forward_event_outbox (message_id),
  ADD CONSTRAINT communication_cross_channel_forward_event_receipts_complete
  CHECK (
    (
      source_evidence_id IS NULL
      AND source_result_message_id IS NULL
      AND delivery_body_reference_id IS NULL
      AND delivery_body_declared_bytes IS NULL
      AND delivery_body_sha256 IS NULL
      AND delivery_body_custody_proof IS NULL
      AND delivery_submit_message_id IS NULL
    )
    OR (
      source_evidence_id IS NULL
      AND source_result_message_id IS NOT NULL
      AND delivery_body_reference_id IS NULL
      AND delivery_body_declared_bytes IS NULL
      AND delivery_body_sha256 IS NULL
      AND delivery_body_custody_proof IS NULL
      AND delivery_submit_message_id IS NULL
    )
    OR (
      source_evidence_id IS NOT NULL
      AND source_result_message_id IS NOT NULL
      AND delivery_body_reference_id IS NOT NULL
      AND delivery_body_declared_bytes IS NOT NULL
      AND delivery_body_sha256 IS NOT NULL
      AND delivery_body_custody_proof IS NOT NULL
      AND delivery_submit_message_id IS NOT NULL
    )
  );
