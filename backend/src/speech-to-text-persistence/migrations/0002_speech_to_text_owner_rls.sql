ALTER TABLE makosh_data.speech_to_text_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.speech_to_text_runs FORCE ROW LEVEL SECURITY;
CREATE POLICY speech_to_text_runs_owner_rls
ON makosh_data.speech_to_text_runs
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
