ALTER TABLE makosh_data.communication_delivery_intent_transitions
  ADD COLUMN realtime_sequence BIGINT GENERATED ALWAYS AS IDENTITY;

ALTER TABLE makosh_data.communication_delivery_intent_transitions
  ADD COLUMN rejection_code SMALLINT CHECK (
    rejection_code IS NULL OR rejection_code BETWEEN 1 AND 32
  );

CREATE UNIQUE INDEX communication_delivery_intent_realtime_sequence_idx
  ON makosh_data.communication_delivery_intent_transitions (
    logical_owner_id,
    realtime_sequence
  );
