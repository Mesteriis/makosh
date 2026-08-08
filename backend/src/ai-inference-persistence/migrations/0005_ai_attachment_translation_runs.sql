CREATE TABLE makosh_data.ai_attachment_translation_runs (
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    request_digest BYTEA NOT NULL,
    context_id BYTEA NOT NULL,
    source_evidence_id BYTEA NOT NULL,
    source_evidence_revision BIGINT NOT NULL,
    contract_major INTEGER NOT NULL,
    contract_revision INTEGER NOT NULL,
    contract_schema_sha256 BYTEA NOT NULL,
    source_reference_id BYTEA NOT NULL,
    source_declared_bytes BIGINT NOT NULL,
    source_sha256 BYTEA NOT NULL,
    source_custody_proof BYTEA NOT NULL,
    requested_target_language SMALLINT NOT NULL,
    maximum_output_bytes INTEGER NOT NULL,
    maximum_output_tokens INTEGER NOT NULL,
    egress_policy SMALLINT NOT NULL,
    egress_policy_revision INTEGER NOT NULL,
    state_revision BIGINT NOT NULL,
    run_state SMALLINT NOT NULL,
    selected_provider_settings_revision BIGINT,
    result_translated_text_utf8 BYTEA,
    result_detected_source_language SMALLINT,
    result_target_language SMALLINT,
    result_model_revision_sha256 BYTEA,
    result_prompt_policy_sha256 BYTEA,
    result_provider_policy_revision INTEGER,
    result_completeness SMALLINT,
    result_confidence_basis_points INTEGER,
    result_terminal_status SMALLINT,
    PRIMARY KEY (logical_owner_id, run_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(run_id) = 16),
    CHECK (length(request_digest) = 32),
    CHECK (length(context_id) = 16),
    CHECK (length(source_evidence_id) = 16),
    CHECK (source_evidence_revision > 0),
    CHECK (contract_major > 0),
    CHECK (contract_revision > 0),
    CHECK (length(contract_schema_sha256) = 32),
    CHECK (length(source_reference_id) = 16),
    CHECK (source_declared_bytes BETWEEN 1 AND 1048576),
    CHECK (length(source_sha256) = 32),
    CHECK (length(source_custody_proof) BETWEEN 1 AND 2048),
    CHECK (requested_target_language BETWEEN 1 AND 3),
    CHECK (maximum_output_bytes BETWEEN 1 AND 65536),
    CHECK (maximum_output_tokens BETWEEN 1 AND 4096),
    CHECK (egress_policy = 1),
    CHECK (egress_policy_revision > 0),
    CHECK (state_revision > 0),
    CHECK (run_state BETWEEN 1 AND 4),
    CHECK (selected_provider_settings_revision IS NULL OR selected_provider_settings_revision > 0),
    CHECK (
        (run_state IN (1, 2, 4) AND selected_provider_settings_revision IS NULL)
        OR (run_state = 3 AND selected_provider_settings_revision IS NOT NULL)
    ),
    CHECK (
        (run_state IN (1, 2) AND result_terminal_status IS NULL
            AND result_translated_text_utf8 IS NULL
            AND result_detected_source_language IS NULL AND result_target_language IS NULL
            AND result_model_revision_sha256 IS NULL
            AND result_prompt_policy_sha256 IS NULL AND result_provider_policy_revision IS NULL
            AND result_completeness IS NULL AND result_confidence_basis_points IS NULL)
        OR (run_state = 3 AND result_terminal_status = 1
            AND length(result_translated_text_utf8) BETWEEN 1 AND 65536
            AND result_detected_source_language BETWEEN 1 AND 4
            AND result_target_language BETWEEN 1 AND 3
            AND result_target_language = requested_target_language
            AND length(result_model_revision_sha256) = 32
            AND length(result_prompt_policy_sha256) = 32
            AND result_provider_policy_revision > 0
            AND result_completeness IN (1, 2)
            AND result_confidence_basis_points BETWEEN 0 AND 10000)
        OR (run_state = 4 AND result_terminal_status BETWEEN 2 AND 5
            AND result_translated_text_utf8 = ''::BYTEA
            AND result_detected_source_language = 0 AND result_target_language = 0
            AND result_model_revision_sha256 IS NULL
            AND result_prompt_policy_sha256 IS NULL AND result_provider_policy_revision IS NULL
            AND result_completeness = 0 AND result_confidence_basis_points = 0)
    )
);

CREATE INDEX ai_attachment_translation_runs_pending_idx
ON makosh_data.ai_attachment_translation_runs (logical_owner_id, run_state, state_revision);
