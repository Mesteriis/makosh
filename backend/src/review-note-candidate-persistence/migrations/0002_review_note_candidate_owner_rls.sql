ALTER TABLE makosh_data.review_note_candidate_submissions ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.review_note_candidate_submissions FORCE ROW LEVEL SECURITY;
CREATE POLICY review_note_candidate_submissions_owner_v1 ON makosh_data.review_note_candidate_submissions
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.review_note_candidate_state ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.review_note_candidate_state FORCE ROW LEVEL SECURITY;
CREATE POLICY review_note_candidate_state_owner_v1 ON makosh_data.review_note_candidate_state
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.review_note_candidate_operations ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.review_note_candidate_operations FORCE ROW LEVEL SECURITY;
CREATE POLICY review_note_candidate_operations_owner_v1 ON makosh_data.review_note_candidate_operations
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.review_note_candidate_promotion_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.review_note_candidate_promotion_inbox FORCE ROW LEVEL SECURITY;
CREATE POLICY review_note_candidate_promotion_inbox_owner_v1 ON makosh_data.review_note_candidate_promotion_inbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.review_note_candidate_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.review_note_candidate_outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY review_note_candidate_outbox_owner_v1 ON makosh_data.review_note_candidate_outbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.review_note_candidate_realtime ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.review_note_candidate_realtime FORCE ROW LEVEL SECURITY;
CREATE POLICY review_note_candidate_realtime_owner_v1 ON makosh_data.review_note_candidate_realtime
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
