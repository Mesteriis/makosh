ALTER TABLE makosh_data.ollama_ai_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.ollama_ai_runs FORCE ROW LEVEL SECURITY;
CREATE POLICY ollama_ai_runs_owner_rls
ON makosh_data.ollama_ai_runs
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.ollama_ai_summary_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.ollama_ai_summary_runs FORCE ROW LEVEL SECURITY;
CREATE POLICY ollama_ai_summary_runs_owner_rls
ON makosh_data.ollama_ai_summary_runs
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.ollama_ai_translation_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.ollama_ai_translation_runs FORCE ROW LEVEL SECURITY;
CREATE POLICY ollama_ai_translation_runs_owner_rls
ON makosh_data.ollama_ai_translation_runs
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.ollama_ai_explanation_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.ollama_ai_explanation_runs FORCE ROW LEVEL SECURITY;
CREATE POLICY ollama_ai_explanation_runs_owner_rls
ON makosh_data.ollama_ai_explanation_runs
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
