CREATE TABLE makosh_data.whisper_stt_runs (
    logical_owner_id TEXT NOT NULL,
    request_id BYTEA NOT NULL,
    request_digest BYTEA NOT NULL,
    source_reference_id BYTEA NOT NULL,
    source_declared_bytes BIGINT NOT NULL,
    source_sha256 BYTEA NOT NULL,
    model_revision_sha256 BYTEA NOT NULL,
    provider_settings_revision BIGINT NOT NULL,
    provider_policy_revision INTEGER NOT NULL,
    state_revision BIGINT NOT NULL,
    run_state SMALLINT NOT NULL,
    transcript_reference_id BYTEA,
    transcript_declared_bytes BIGINT,
    transcript_sha256 BYTEA,
    detected_language SMALLINT,
    segment_count INTEGER,
    completeness SMALLINT,
    confidence_basis_points INTEGER,
    reject_code SMALLINT,
    PRIMARY KEY (logical_owner_id, request_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(request_id) = 16),
    CHECK (length(request_digest) = 32),
    CHECK (length(source_reference_id) = 16),
    CHECK (source_declared_bytes BETWEEN 1 AND 536870912),
    CHECK (length(source_sha256) = 32),
    CHECK (length(model_revision_sha256) = 32),
    CHECK (provider_settings_revision > 0),
    CHECK (provider_policy_revision > 0),
    CHECK (state_revision > 0),
    CHECK (run_state BETWEEN 1 AND 5),
    CHECK (
        (run_state IN (1, 2, 5)
         AND transcript_reference_id IS NULL
         AND transcript_declared_bytes IS NULL
         AND transcript_sha256 IS NULL
         AND detected_language IS NULL
         AND segment_count IS NULL
         AND completeness IS NULL
         AND confidence_basis_points IS NULL
         AND reject_code IS NULL)
        OR
        (run_state = 3
         AND length(transcript_reference_id) = 16
         AND transcript_declared_bytes BETWEEN 1 AND 4194304
         AND length(transcript_sha256) = 32
         AND detected_language BETWEEN 1 AND 4
         AND segment_count BETWEEN 0 AND 100000
         AND completeness BETWEEN 1 AND 2
         AND confidence_basis_points BETWEEN 0 AND 10000
         AND reject_code IS NULL)
        OR
        (run_state = 4
         AND transcript_reference_id IS NULL
         AND transcript_declared_bytes IS NULL
         AND transcript_sha256 IS NULL
         AND detected_language IS NULL
         AND segment_count IS NULL
         AND completeness IS NULL
         AND confidence_basis_points IS NULL
         AND reject_code BETWEEN 1 AND 6)
    )
);

CREATE INDEX whisper_stt_runs_recovery_idx
ON makosh_data.whisper_stt_runs (
    logical_owner_id,
    run_state,
    state_revision
);
