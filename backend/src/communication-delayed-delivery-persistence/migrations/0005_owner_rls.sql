ALTER TABLE makosh_data.communication_delayed_delivery_operations ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.communication_delayed_delivery_operations FORCE ROW LEVEL SECURITY;
CREATE POLICY communication_delayed_delivery_operations_owner_rls
ON makosh_data.communication_delayed_delivery_operations
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.communication_delayed_delivery_scheduler_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.communication_delayed_delivery_scheduler_inbox FORCE ROW LEVEL SECURITY;
CREATE POLICY communication_delayed_delivery_scheduler_inbox_owner_rls
ON makosh_data.communication_delayed_delivery_scheduler_inbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.communication_delayed_delivery_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.communication_delayed_delivery_outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY communication_delayed_delivery_outbox_owner_rls
ON makosh_data.communication_delayed_delivery_outbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.communication_delayed_delivery_scheduler_receipt_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.communication_delayed_delivery_scheduler_receipt_outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY communication_delayed_delivery_scheduler_receipt_outbox_owner_rls
ON makosh_data.communication_delayed_delivery_scheduler_receipt_outbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.communication_delayed_delivery_realtime ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.communication_delayed_delivery_realtime FORCE ROW LEVEL SECURITY;
CREATE POLICY communication_delayed_delivery_realtime_owner_rls
ON makosh_data.communication_delayed_delivery_realtime
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.communication_delayed_delivery_body_cleanup ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.communication_delayed_delivery_body_cleanup FORCE ROW LEVEL SECURITY;
CREATE POLICY communication_delayed_delivery_body_cleanup_owner_rls
ON makosh_data.communication_delayed_delivery_body_cleanup
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
