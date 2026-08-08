CREATE TABLE makosh_data.communication_cross_channel_forward_operations (
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  forward_id BYTEA NOT NULL CHECK (octet_length(forward_id) = 16),
  request_fingerprint BYTEA NOT NULL CHECK (
    octet_length(request_fingerprint) = 32
  ),
  source_message_id BYTEA NOT NULL CHECK (
    octet_length(source_message_id) = 16
  ),
  target_conversation_id BYTEA NOT NULL CHECK (
    octet_length(target_conversation_id) = 16
  ),
  target_reply_message_id BYTEA CHECK (
    target_reply_message_id IS NULL
    OR octet_length(target_reply_message_id) = 16
  ),
  state SMALLINT NOT NULL CHECK (state BETWEEN 1 AND 5),
  state_revision BIGINT NOT NULL DEFAULT 1 CHECK (state_revision > 0),
  source_revision BIGINT CHECK (
    source_revision IS NULL OR source_revision > 0
  ),
  source_body_sha256 BYTEA CHECK (
    source_body_sha256 IS NULL OR octet_length(source_body_sha256) = 32
  ),
  source_body_length INTEGER CHECK (
    source_body_length IS NULL
    OR source_body_length BETWEEN 1 AND 65536
  ),
  source_blob_reference BYTEA CHECK (
    source_blob_reference IS NULL
    OR octet_length(source_blob_reference) BETWEEN 1 AND 1024
  ),
  source_custody_proof BYTEA CHECK (
    source_custody_proof IS NULL
    OR octet_length(source_custody_proof) BETWEEN 1 AND 4096
  ),
  delivery_intent_id BYTEA CHECK (
    delivery_intent_id IS NULL OR octet_length(delivery_intent_id) = 16
  ),
  error_code SMALLINT CHECK (
    error_code IS NULL OR error_code BETWEEN 1 AND 7
  ),
  attempt_count SMALLINT NOT NULL DEFAULT 0 CHECK (
    attempt_count BETWEEN 0 AND 32
  ),
  next_attempt_at_unix_millis BIGINT NOT NULL CHECK (
    next_attempt_at_unix_millis > 0
  ),
  claimed_by TEXT CHECK (
    claimed_by IS NULL OR char_length(claimed_by) BETWEEN 1 AND 128
  ),
  claim_epoch BIGINT NOT NULL DEFAULT 0 CHECK (claim_epoch >= 0),
  lease_expires_at_unix_millis BIGINT,
  created_at_unix_millis BIGINT NOT NULL CHECK (
    created_at_unix_millis > 0
  ),
  updated_at_unix_millis BIGINT NOT NULL CHECK (
    updated_at_unix_millis > 0
  ),
  PRIMARY KEY (logical_owner_id, forward_id),
  CHECK (
    (claimed_by IS NULL AND lease_expires_at_unix_millis IS NULL)
    OR (state BETWEEN 1 AND 3 AND claimed_by IS NOT NULL
      AND lease_expires_at_unix_millis IS NOT NULL)
  ),
  CHECK (
    (source_revision IS NULL AND source_body_sha256 IS NULL
      AND source_body_length IS NULL AND source_blob_reference IS NULL
      AND source_custody_proof IS NULL)
    OR (source_revision IS NOT NULL AND source_body_sha256 IS NOT NULL
      AND source_body_length IS NOT NULL AND source_blob_reference IS NOT NULL
      AND source_custody_proof IS NOT NULL)
    OR (state BETWEEN 4 AND 5 AND source_revision IS NOT NULL
      AND source_body_sha256 IS NOT NULL AND source_body_length IS NOT NULL
      AND source_blob_reference IS NULL AND source_custody_proof IS NULL)
  ),
  CHECK (
    (state BETWEEN 1 AND 3 AND delivery_intent_id IS NULL
      AND error_code IS NULL)
    OR (state = 4 AND delivery_intent_id IS NOT NULL AND error_code IS NULL)
    OR (state = 5 AND error_code IS NOT NULL)
  )
);

CREATE INDEX communication_cross_channel_forward_claim_idx
  ON makosh_data.communication_cross_channel_forward_operations (
    logical_owner_id,
    next_attempt_at_unix_millis,
    updated_at_unix_millis,
    forward_id
  )
  WHERE state BETWEEN 1 AND 3;

CREATE TABLE makosh_data.communication_cross_channel_forward_cleanup (
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  forward_id BYTEA NOT NULL CHECK (octet_length(forward_id) = 16),
  source_blob_reference BYTEA NOT NULL CHECK (
    octet_length(source_blob_reference) BETWEEN 1 AND 1024
  ),
  source_custody_proof BYTEA NOT NULL CHECK (
    octet_length(source_custody_proof) BETWEEN 1 AND 4096
  ),
  reason SMALLINT NOT NULL CHECK (reason BETWEEN 1 AND 2),
  attempt_count SMALLINT NOT NULL DEFAULT 0 CHECK (
    attempt_count BETWEEN 0 AND 32
  ),
  next_attempt_at_unix_millis BIGINT NOT NULL CHECK (
    next_attempt_at_unix_millis > 0
  ),
  completed_at_unix_millis BIGINT,
  created_at_unix_millis BIGINT NOT NULL CHECK (
    created_at_unix_millis > 0
  ),
  updated_at_unix_millis BIGINT NOT NULL CHECK (
    updated_at_unix_millis > 0
  ),
  PRIMARY KEY (logical_owner_id, forward_id),
  FOREIGN KEY (logical_owner_id, forward_id) REFERENCES
    makosh_data.communication_cross_channel_forward_operations (
      logical_owner_id,
      forward_id
    )
);

CREATE INDEX communication_cross_channel_forward_cleanup_pending_idx
  ON makosh_data.communication_cross_channel_forward_cleanup (
    logical_owner_id,
    next_attempt_at_unix_millis,
    forward_id
  )
  WHERE completed_at_unix_millis IS NULL;

CREATE TABLE makosh_data.communication_cross_channel_forward_realtime (
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  forward_id BYTEA NOT NULL CHECK (octet_length(forward_id) = 16),
  state_revision BIGINT NOT NULL CHECK (state_revision > 0),
  state SMALLINT NOT NULL CHECK (state BETWEEN 1 AND 5),
  error_code SMALLINT CHECK (
    error_code IS NULL OR error_code BETWEEN 1 AND 7
  ),
  occurred_at_unix_millis BIGINT NOT NULL CHECK (
    occurred_at_unix_millis > 0
  ),
  realtime_sequence BIGINT GENERATED ALWAYS AS IDENTITY,
  PRIMARY KEY (logical_owner_id, forward_id, state_revision),
  FOREIGN KEY (logical_owner_id, forward_id) REFERENCES
    makosh_data.communication_cross_channel_forward_operations (
      logical_owner_id,
      forward_id
    ),
  CHECK (
    (state = 5 AND error_code IS NOT NULL)
    OR (state <> 5 AND error_code IS NULL)
  )
);

CREATE UNIQUE INDEX communication_cross_channel_forward_realtime_sequence_idx
  ON makosh_data.communication_cross_channel_forward_realtime (
    logical_owner_id,
    realtime_sequence
  );
