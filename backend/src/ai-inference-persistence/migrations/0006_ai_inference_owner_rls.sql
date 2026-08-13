ALTER TABLE makosh_data.ai_inference_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.ai_inference_runs FORCE ROW LEVEL SECURITY;
CREATE POLICY ai_inference_runs_owner_rls
ON makosh_data.ai_inference_runs
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.ai_summary_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.ai_summary_runs FORCE ROW LEVEL SECURITY;
CREATE POLICY ai_summary_runs_owner_rls
ON makosh_data.ai_summary_runs
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.ai_translation_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.ai_translation_runs FORCE ROW LEVEL SECURITY;
CREATE POLICY ai_translation_runs_owner_rls
ON makosh_data.ai_translation_runs
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.ai_explanation_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.ai_explanation_runs FORCE ROW LEVEL SECURITY;
CREATE POLICY ai_explanation_runs_owner_rls
ON makosh_data.ai_explanation_runs
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.ai_attachment_translation_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.ai_attachment_translation_runs FORCE ROW LEVEL SECURITY;
CREATE POLICY ai_attachment_translation_runs_owner_rls
ON makosh_data.ai_attachment_translation_runs
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
