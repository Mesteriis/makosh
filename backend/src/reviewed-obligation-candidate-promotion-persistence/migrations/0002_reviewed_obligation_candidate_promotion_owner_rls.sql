ALTER TABLE makosh_data.reviewed_obligation_candidate_promotion_requests
    ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.reviewed_obligation_candidate_promotion_requests
    FORCE ROW LEVEL SECURITY;
CREATE POLICY reviewed_obligation_candidate_promotion_requests_owner_policy
ON makosh_data.reviewed_obligation_candidate_promotion_requests
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.reviewed_obligation_candidate_promotion_result_inbox
    ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.reviewed_obligation_candidate_promotion_result_inbox
    FORCE ROW LEVEL SECURITY;
CREATE POLICY reviewed_obligation_candidate_promotion_result_inbox_owner_policy
ON makosh_data.reviewed_obligation_candidate_promotion_result_inbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.reviewed_obligation_candidate_promotion_outbox
    ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.reviewed_obligation_candidate_promotion_outbox
    FORCE ROW LEVEL SECURITY;
CREATE POLICY reviewed_obligation_candidate_promotion_outbox_owner_policy
ON makosh_data.reviewed_obligation_candidate_promotion_outbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
