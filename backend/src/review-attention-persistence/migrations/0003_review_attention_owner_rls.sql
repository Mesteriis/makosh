ALTER TABLE makosh_data.review_attention_state ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.review_attention_state FORCE ROW LEVEL SECURITY;
CREATE POLICY review_attention_state_owner_v1
ON makosh_data.review_attention_state
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.review_attention_operations ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.review_attention_operations FORCE ROW LEVEL SECURITY;
CREATE POLICY review_attention_operations_owner_v1
ON makosh_data.review_attention_operations
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.review_attention_realtime ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.review_attention_realtime FORCE ROW LEVEL SECURITY;
CREATE POLICY review_attention_realtime_owner_v1
ON makosh_data.review_attention_realtime
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
