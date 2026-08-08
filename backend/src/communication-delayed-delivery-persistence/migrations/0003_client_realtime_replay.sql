CREATE TABLE makosh_data.communication_delayed_delivery_realtime (
  logical_owner_id TEXT NOT NULL CHECK (
    char_length(logical_owner_id) BETWEEN 1 AND 128
  ),
  delayed_operation_id BYTEA NOT NULL CHECK (
    octet_length(delayed_operation_id) = 16
  ),
  state_revision BIGINT NOT NULL CHECK (state_revision > 0),
  state SMALLINT NOT NULL CHECK (state BETWEEN 1 AND 9),
  occurred_at_unix_millis BIGINT NOT NULL CHECK (
    occurred_at_unix_millis > 0
  ),
  realtime_sequence BIGINT GENERATED ALWAYS AS IDENTITY,
  PRIMARY KEY (
    logical_owner_id,
    delayed_operation_id,
    state_revision
  ),
  FOREIGN KEY (logical_owner_id, delayed_operation_id) REFERENCES
    makosh_data.communication_delayed_delivery_operations (
      logical_owner_id,
      delayed_operation_id
    )
);

CREATE UNIQUE INDEX communication_delayed_delivery_realtime_sequence_idx
  ON makosh_data.communication_delayed_delivery_realtime (
    logical_owner_id,
    realtime_sequence
  );
