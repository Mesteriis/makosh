CREATE TABLE makosh_data.ollama_ai_explanation_runs (
    logical_owner_id TEXT NOT NULL,
    request_id BYTEA NOT NULL,
    request_digest BYTEA NOT NULL,
    settings_revision BIGINT NOT NULL,
    state_revision BIGINT NOT NULL,
    run_state SMALLINT NOT NULL,
    selected_model_revision_sha256 BYTEA,
    result_exact_bytes BYTEA,
    result_terminal_status SMALLINT,
    PRIMARY KEY (logical_owner_id, request_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(request_id) = 16),
    CHECK (length(request_digest) = 32),
    CHECK (settings_revision > 0),
    CHECK (state_revision > 0),
    CHECK (run_state BETWEEN 1 AND 5),
    CHECK (selected_model_revision_sha256 IS NULL OR length(selected_model_revision_sha256) = 32),
    CHECK (
        (run_state = 1 AND selected_model_revision_sha256 IS NULL)
        OR (run_state IN (2, 3, 5) AND length(selected_model_revision_sha256) = 32)
        OR run_state = 4
    ),
    CHECK (
        (run_state IN (1, 2, 5) AND result_terminal_status IS NULL
            AND result_exact_bytes IS NULL)
        OR (run_state = 3 AND result_terminal_status = 1
            AND length(result_exact_bytes) BETWEEN 1 AND 16384)
        OR (run_state = 4 AND result_terminal_status BETWEEN 2 AND 5
            AND length(result_exact_bytes) BETWEEN 1 AND 16384)
    )
);
