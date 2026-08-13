ALTER TABLE makosh_data.communication_bulk_action_batches ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.communication_bulk_action_batches FORCE ROW LEVEL SECURITY;
CREATE POLICY communication_bulk_action_batches_owner_rls
ON makosh_data.communication_bulk_action_batches
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.communication_bulk_action_targets ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.communication_bulk_action_targets FORCE ROW LEVEL SECURITY;
CREATE POLICY communication_bulk_action_targets_owner_rls
ON makosh_data.communication_bulk_action_targets
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.communication_bulk_action_realtime ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.communication_bulk_action_realtime FORCE ROW LEVEL SECURITY;
CREATE POLICY communication_bulk_action_realtime_owner_rls
ON makosh_data.communication_bulk_action_realtime
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
