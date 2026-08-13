ALTER TABLE makosh_data.whisper_stt_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.whisper_stt_runs FORCE ROW LEVEL SECURITY;
CREATE POLICY whisper_stt_runs_owner_rls
ON makosh_data.whisper_stt_runs
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
