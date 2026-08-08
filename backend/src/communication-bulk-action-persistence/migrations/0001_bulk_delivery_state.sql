CREATE TABLE makosh_data.communication_bulk_action_batches (
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  batch_id BYTEA NOT NULL CHECK (octet_length(batch_id) = 16),
  request_fingerprint BYTEA NOT NULL CHECK (
    octet_length(request_fingerprint) = 32
  ),
  target_count SMALLINT NOT NULL CHECK (target_count BETWEEN 1 AND 100),
  state_revision BIGINT NOT NULL DEFAULT 1 CHECK (state_revision > 0),
  created_at_unix_seconds BIGINT NOT NULL CHECK (
    created_at_unix_seconds > 0
  ),
  updated_at_unix_seconds BIGINT NOT NULL CHECK (
    updated_at_unix_seconds > 0
  ),
  PRIMARY KEY (logical_owner_id, batch_id)
);

CREATE TABLE makosh_data.communication_bulk_action_targets (
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  batch_id BYTEA NOT NULL CHECK (octet_length(batch_id) = 16),
  target_operation_id BYTEA NOT NULL CHECK (
    octet_length(target_operation_id) = 16
  ),
  ordinal SMALLINT NOT NULL CHECK (ordinal BETWEEN 0 AND 99),
  canonical_conversation_id BYTEA NOT NULL CHECK (
    octet_length(canonical_conversation_id) = 16
  ),
  canonical_reply_message_id BYTEA CHECK (
    canonical_reply_message_id IS NULL
    OR octet_length(canonical_reply_message_id) = 16
  ),
  body_utf8 BYTEA NOT NULL CHECK (
    octet_length(body_utf8) BETWEEN 1 AND 65536
  ),
  state SMALLINT NOT NULL CHECK (state BETWEEN 1 AND 5),
  attempt_count SMALLINT NOT NULL DEFAULT 0 CHECK (
    attempt_count BETWEEN 0 AND 3
  ),
  claimed_by TEXT CHECK (
    claimed_by IS NULL OR char_length(claimed_by) BETWEEN 1 AND 128
  ),
  claim_epoch BIGINT NOT NULL DEFAULT 0 CHECK (claim_epoch >= 0),
  lease_expires_at_unix_seconds BIGINT,
  next_attempt_at_unix_seconds BIGINT,
  delivery_intent_id BYTEA CHECK (
    delivery_intent_id IS NULL OR octet_length(delivery_intent_id) = 16
  ),
  error_code SMALLINT CHECK (
    error_code IS NULL OR error_code BETWEEN 1 AND 5
  ),
  created_at_unix_seconds BIGINT NOT NULL CHECK (
    created_at_unix_seconds > 0
  ),
  updated_at_unix_seconds BIGINT NOT NULL CHECK (
    updated_at_unix_seconds > 0
  ),
  PRIMARY KEY (logical_owner_id, batch_id, target_operation_id),
  UNIQUE (logical_owner_id, batch_id, ordinal),
  FOREIGN KEY (logical_owner_id, batch_id) REFERENCES
    makosh_data.communication_bulk_action_batches (
      logical_owner_id,
      batch_id
    ),
  CHECK (
    (state = 2 AND claimed_by IS NOT NULL
      AND lease_expires_at_unix_seconds IS NOT NULL)
    OR (state <> 2 AND claimed_by IS NULL
      AND lease_expires_at_unix_seconds IS NULL)
  ),
  CHECK (
    (state = 3 AND delivery_intent_id IS NOT NULL AND error_code IS NULL)
    OR (state IN (4, 5) AND delivery_intent_id IS NULL
      AND error_code IS NOT NULL)
    OR (state IN (1, 2) AND delivery_intent_id IS NULL
      AND error_code IS NULL)
  )
);

CREATE INDEX communication_bulk_action_claim_idx
  ON makosh_data.communication_bulk_action_targets (
    logical_owner_id,
    next_attempt_at_unix_seconds,
    updated_at_unix_seconds,
    batch_id,
    ordinal
  )
  WHERE state IN (1, 2, 4);
