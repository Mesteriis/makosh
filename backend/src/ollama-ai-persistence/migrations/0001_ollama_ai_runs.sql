CREATE TABLE makosh_data.ollama_ai_runs (
    logical_owner_id TEXT NOT NULL,
    request_id BYTEA NOT NULL,
    request_digest BYTEA NOT NULL,
    settings_revision BIGINT NOT NULL,
    state_revision BIGINT NOT NULL,
    run_state SMALLINT NOT NULL,
    selected_model_revision_sha256 BYTEA,
    result_subject_utf8 BYTEA,
    result_body_utf8 BYTEA,
    result_resolved_tone SMALLINT,
    result_resolved_language SMALLINT,
    result_model_revision_sha256 BYTEA,
    result_input_tokens INTEGER,
    result_output_tokens INTEGER,
    result_terminal_status SMALLINT,
    result_completeness SMALLINT,
    result_confidence_basis_points INTEGER,
    result_provider_settings_revision BIGINT,
    PRIMARY KEY (logical_owner_id, request_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(request_id) = 16),
    CHECK (length(request_digest) = 32),
    CHECK (settings_revision > 0),
    CHECK (state_revision > 0),
    CHECK (run_state BETWEEN 1 AND 5),
    CHECK (
        (run_state = 1 AND selected_model_revision_sha256 IS NULL)
        OR (run_state IN (2, 3, 5) AND length(selected_model_revision_sha256) = 32)
        OR (
            run_state = 4
            AND (
                selected_model_revision_sha256 IS NULL
                OR length(selected_model_revision_sha256) = 32
            )
        )
    ),
    CHECK (
        (
            run_state IN (1, 2, 5)
            AND result_terminal_status IS NULL
            AND result_subject_utf8 IS NULL
            AND result_body_utf8 IS NULL
            AND result_resolved_tone IS NULL
            AND result_resolved_language IS NULL
            AND result_model_revision_sha256 IS NULL
            AND result_input_tokens IS NULL
            AND result_output_tokens IS NULL
            AND result_completeness IS NULL
            AND result_confidence_basis_points IS NULL
            AND result_provider_settings_revision IS NULL
        )
        OR (
            run_state = 3
            AND result_terminal_status = 1
            AND result_subject_utf8 IS NOT NULL
            AND length(result_subject_utf8) <= 998
            AND result_body_utf8 IS NOT NULL
            AND length(result_body_utf8) BETWEEN 1 AND 65536
            AND result_resolved_tone BETWEEN 1 AND 4
            AND result_resolved_language BETWEEN 2 AND 4
            AND length(result_model_revision_sha256) = 32
            AND result_input_tokens >= 0
            AND result_output_tokens >= 0
            AND result_completeness IN (1, 2)
            AND result_confidence_basis_points BETWEEN 0 AND 10000
            AND result_provider_settings_revision = settings_revision
        )
        OR (
            run_state = 4
            AND result_terminal_status BETWEEN 2 AND 5
            AND result_subject_utf8 = ''::BYTEA
            AND result_body_utf8 = ''::BYTEA
            AND result_resolved_tone = 0
            AND result_resolved_language = 0
            AND result_model_revision_sha256 IS NULL
            AND result_input_tokens = 0
            AND result_output_tokens = 0
            AND result_completeness = 0
            AND result_confidence_basis_points = 0
            AND result_provider_settings_revision = 0
        )
    )
);
