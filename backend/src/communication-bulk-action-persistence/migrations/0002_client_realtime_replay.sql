CREATE TABLE makosh_data.communication_bulk_action_realtime (
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  batch_id BYTEA NOT NULL CHECK (octet_length(batch_id) = 16),
  state_revision BIGINT NOT NULL CHECK (state_revision > 0),
  state SMALLINT NOT NULL CHECK (state BETWEEN 1 AND 4),
  occurred_at_unix_seconds BIGINT NOT NULL CHECK (
    occurred_at_unix_seconds > 0
  ),
  realtime_sequence BIGINT GENERATED ALWAYS AS IDENTITY,
  PRIMARY KEY (logical_owner_id, batch_id, state_revision),
  FOREIGN KEY (logical_owner_id, batch_id) REFERENCES
    makosh_data.communication_bulk_action_batches (
      logical_owner_id,
      batch_id
    )
);

CREATE UNIQUE INDEX communication_bulk_action_realtime_sequence_idx
  ON makosh_data.communication_bulk_action_realtime (
    logical_owner_id,
    realtime_sequence
  );
