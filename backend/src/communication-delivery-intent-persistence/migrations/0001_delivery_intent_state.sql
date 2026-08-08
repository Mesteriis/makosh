CREATE TABLE makosh_data.communication_delivery_intent_jobs (
  intent_id BYTEA NOT NULL CHECK (octet_length(intent_id) = 16),
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  request_fingerprint BYTEA NOT NULL CHECK (
    octet_length(request_fingerprint) = 32
  ),
  canonical_conversation_id BYTEA NOT NULL CHECK (
    octet_length(canonical_conversation_id) = 16
  ),
  canonical_reply_message_id BYTEA CHECK (
    canonical_reply_message_id IS NULL
    OR octet_length(canonical_reply_message_id) = 16
  ),
  provider_kind SMALLINT NOT NULL CHECK (provider_kind BETWEEN 1 AND 6),
  account_cursor BYTEA NOT NULL CHECK (octet_length(account_cursor) = 32),
  conversation_cursor BYTEA NOT NULL CHECK (
    octet_length(conversation_cursor) = 32
  ),
  reply_source_cursor BYTEA CHECK (
    reply_source_cursor IS NULL OR octet_length(reply_source_cursor) = 32
  ),
  body_reference_id BYTEA NOT NULL CHECK (
    octet_length(body_reference_id) = 16
  ),
  body_declared_bytes BIGINT NOT NULL CHECK (
    body_declared_bytes BETWEEN 1 AND 65536
  ),
  body_sha256 BYTEA NOT NULL CHECK (
    octet_length(body_sha256) = 32
  ),
  body_custody_source_proof BYTEA NOT NULL CHECK (
    octet_length(body_custody_source_proof) BETWEEN 1 AND 2048
  ),
  state SMALLINT NOT NULL CHECK (state BETWEEN 1 AND 5),
  state_revision BIGINT NOT NULL DEFAULT 1 CHECK (state_revision > 0),
  created_at_unix_seconds BIGINT NOT NULL CHECK (
    created_at_unix_seconds > 0
  ),
  updated_at_unix_seconds BIGINT NOT NULL CHECK (
    updated_at_unix_seconds > 0
  ),
  claimed_by TEXT CHECK (
    claimed_by IS NULL OR char_length(claimed_by) BETWEEN 1 AND 128
  ),
  claim_epoch BIGINT NOT NULL DEFAULT 0 CHECK (claim_epoch >= 0),
  lease_expires_at_unix_seconds BIGINT,
  provider_operation_id BYTEA CHECK (
    provider_operation_id IS NULL
    OR octet_length(provider_operation_id) BETWEEN 1 AND 256
  ),
  rejection_code SMALLINT CHECK (
    rejection_code IS NULL OR rejection_code BETWEEN 1 AND 32
  ),
  PRIMARY KEY (logical_owner_id, intent_id),
  CHECK (
    (state = 2 AND claimed_by IS NOT NULL
      AND lease_expires_at_unix_seconds IS NOT NULL)
    OR (state <> 2 AND claimed_by IS NULL
      AND lease_expires_at_unix_seconds IS NULL)
  ),
  CHECK (
    (state IN (1, 2) AND provider_operation_id IS NULL
      AND rejection_code IS NULL)
    OR (state = 3
      AND provider_operation_id IS NOT NULL AND rejection_code IS NULL)
    OR (state = 4
      AND provider_operation_id IS NOT NULL AND rejection_code IS NULL)
    OR (state = 5 AND rejection_code IS NOT NULL)
  )
);

CREATE TABLE makosh_data.communication_delivery_intent_transitions (
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  intent_id BYTEA NOT NULL CHECK (octet_length(intent_id) = 16),
  state_revision BIGINT NOT NULL CHECK (state_revision > 0),
  state SMALLINT NOT NULL CHECK (state BETWEEN 1 AND 5),
  occurred_at_unix_seconds BIGINT NOT NULL CHECK (
    occurred_at_unix_seconds > 0
  ),
  PRIMARY KEY (logical_owner_id, intent_id, state_revision),
  FOREIGN KEY (logical_owner_id, intent_id) REFERENCES
    makosh_data.communication_delivery_intent_jobs (
      logical_owner_id,
      intent_id
    )
);

CREATE INDEX communication_delivery_intent_claim_idx
  ON makosh_data.communication_delivery_intent_jobs (
    logical_owner_id,
    updated_at_unix_seconds,
    intent_id
  )
  WHERE state IN (1, 2);
