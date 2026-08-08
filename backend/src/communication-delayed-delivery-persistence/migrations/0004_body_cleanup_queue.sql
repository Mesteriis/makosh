CREATE TABLE makosh_data.communication_delayed_delivery_body_cleanup (
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  delayed_operation_id BYTEA NOT NULL CHECK (
    octet_length(delayed_operation_id) = 16
  ),
  reason SMALLINT NOT NULL CHECK (reason BETWEEN 1 AND 3),
  attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (
    attempt_count BETWEEN 0 AND 32
  ),
  next_attempt_at_unix_millis BIGINT NOT NULL CHECK (
    next_attempt_at_unix_millis > 0
  ),
  completed_at_unix_millis BIGINT CHECK (
    completed_at_unix_millis IS NULL OR completed_at_unix_millis > 0
  ),
  created_at_unix_millis BIGINT NOT NULL CHECK (
    created_at_unix_millis > 0
  ),
  updated_at_unix_millis BIGINT NOT NULL CHECK (
    updated_at_unix_millis > 0
  ),
  PRIMARY KEY (logical_owner_id, delayed_operation_id),
  FOREIGN KEY (logical_owner_id, delayed_operation_id) REFERENCES
    makosh_data.communication_delayed_delivery_operations (
      logical_owner_id,
      delayed_operation_id
    )
);

CREATE INDEX communication_delayed_delivery_cleanup_pending_idx
  ON makosh_data.communication_delayed_delivery_body_cleanup (
    logical_owner_id,
    next_attempt_at_unix_millis,
    delayed_operation_id
  )
  WHERE completed_at_unix_millis IS NULL;
