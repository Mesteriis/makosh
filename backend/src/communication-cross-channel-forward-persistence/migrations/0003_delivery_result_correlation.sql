ALTER TABLE makosh_data.communication_cross_channel_forward_operations
  ADD COLUMN delivery_intent_command_id BYTEA CHECK (
    delivery_intent_command_id IS NULL
    OR octet_length(delivery_intent_command_id) = 16
  );

CREATE UNIQUE INDEX communication_cross_channel_forward_delivery_intent_idx
  ON makosh_data.communication_cross_channel_forward_operations (
    logical_owner_id,
    delivery_intent_command_id
  )
  WHERE delivery_intent_command_id IS NOT NULL;
