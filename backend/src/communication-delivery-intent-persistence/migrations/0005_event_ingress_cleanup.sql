CREATE TABLE makosh_data.communication_delivery_intent_ingress_cleanup (
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  intent_id BYTEA NOT NULL CHECK (octet_length(intent_id) = 16),
  reference_id BYTEA NOT NULL CHECK (octet_length(reference_id) = 16),
  declared_bytes BIGINT NOT NULL CHECK (
    declared_bytes BETWEEN 1 AND 16777216
  ),
  sha256 BYTEA NOT NULL CHECK (octet_length(sha256) = 32),
  custody_source_proof BYTEA NOT NULL CHECK (
    octet_length(custody_source_proof) BETWEEN 1 AND 2048
  ),
  reason SMALLINT NOT NULL CHECK (reason BETWEEN 1 AND 2),
  attempt_count SMALLINT NOT NULL DEFAULT 0 CHECK (
    attempt_count BETWEEN 0 AND 32
  ),
  next_attempt_at_unix_seconds BIGINT NOT NULL CHECK (
    next_attempt_at_unix_seconds > 0
  ),
  completed_at_unix_seconds BIGINT,
  created_at_unix_seconds BIGINT NOT NULL CHECK (
    created_at_unix_seconds > 0
  ),
  updated_at_unix_seconds BIGINT NOT NULL CHECK (
    updated_at_unix_seconds > 0
  ),
  PRIMARY KEY (logical_owner_id, intent_id),
  FOREIGN KEY (logical_owner_id, intent_id) REFERENCES
    makosh_data.communication_delivery_intent_ingress_inbox (
      logical_owner_id,
      intent_id
    )
);

CREATE INDEX communication_delivery_intent_ingress_cleanup_pending_idx
  ON makosh_data.communication_delivery_intent_ingress_cleanup (
    logical_owner_id,
    next_attempt_at_unix_seconds,
    intent_id
  )
  WHERE completed_at_unix_seconds IS NULL;
