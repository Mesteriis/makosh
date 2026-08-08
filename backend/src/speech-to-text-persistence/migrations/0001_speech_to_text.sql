CREATE TABLE makosh_data.speech_to_text_runs (
    logical_owner_id TEXT NOT NULL,
    request_id BYTEA NOT NULL,
    request_digest BYTEA NOT NULL,
    source_reference_id BYTEA NOT NULL,
    source_declared_bytes BIGINT NOT NULL,
    source_sha256 BYTEA NOT NULL,
    audio_format SMALLINT NOT NULL,
    duration_millis BIGINT NOT NULL,
    requested_language SMALLINT NOT NULL,
    consent_receipt_id BYTEA NOT NULL,
    consent_policy_revision INTEGER NOT NULL,
    maximum_transcript_bytes INTEGER NOT NULL,
    maximum_segments INTEGER NOT NULL,
    state_revision BIGINT NOT NULL,
    run_state SMALLINT NOT NULL,
    transcript_reference_id BYTEA,
    transcript_declared_bytes BIGINT,
    transcript_sha256 BYTEA,
    detected_language SMALLINT,
    segment_count INTEGER,
    completeness SMALLINT,
    confidence_basis_points INTEGER,
    provider_contract_schema_sha256 BYTEA,
    model_revision_sha256 BYTEA,
    provider_settings_revision BIGINT,
    provider_policy_revision INTEGER,
    rejection_code SMALLINT,
    PRIMARY KEY (logical_owner_id, request_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(request_id) = 16),
    CHECK (length(request_digest) = 32),
    CHECK (length(source_reference_id) = 16),
    CHECK (source_declared_bytes BETWEEN 1 AND 536870912),
    CHECK (length(source_sha256) = 32),
    CHECK (audio_format = 1),
    CHECK (duration_millis BETWEEN 1 AND 14400000),
    CHECK (requested_language BETWEEN 1 AND 4),
    CHECK (length(consent_receipt_id) = 16),
    CHECK (consent_policy_revision > 0),
    CHECK (maximum_transcript_bytes BETWEEN 1 AND 4194304),
    CHECK (maximum_segments BETWEEN 1 AND 100000),
    CHECK (state_revision > 0),
    CHECK (run_state BETWEEN 1 AND 4),
    CHECK (
        (run_state IN (1, 2)
         AND transcript_reference_id IS NULL
         AND transcript_declared_bytes IS NULL
         AND transcript_sha256 IS NULL
         AND detected_language IS NULL
         AND segment_count IS NULL
         AND completeness IS NULL
         AND confidence_basis_points IS NULL
         AND provider_contract_schema_sha256 IS NULL
         AND model_revision_sha256 IS NULL
         AND provider_settings_revision IS NULL
         AND provider_policy_revision IS NULL
         AND rejection_code IS NULL)
        OR
        (run_state = 3
         AND length(transcript_reference_id) = 16
         AND transcript_declared_bytes BETWEEN 1 AND maximum_transcript_bytes
         AND length(transcript_sha256) = 32
         AND detected_language BETWEEN 1 AND 4
         AND segment_count BETWEEN 0 AND maximum_segments
         AND completeness BETWEEN 1 AND 2
         AND confidence_basis_points BETWEEN 0 AND 10000
         AND length(provider_contract_schema_sha256) = 32
         AND length(model_revision_sha256) = 32
         AND provider_settings_revision > 0
         AND provider_policy_revision > 0
         AND rejection_code IS NULL)
        OR
        (run_state = 4
         AND transcript_reference_id IS NULL
         AND transcript_declared_bytes IS NULL
         AND transcript_sha256 IS NULL
         AND detected_language IS NULL
         AND segment_count IS NULL
         AND completeness IS NULL
         AND confidence_basis_points IS NULL
         AND provider_contract_schema_sha256 IS NULL
         AND model_revision_sha256 IS NULL
         AND provider_settings_revision IS NULL
         AND provider_policy_revision IS NULL
         AND rejection_code BETWEEN 1 AND 6)
    )
);

CREATE INDEX speech_to_text_runs_recovery_idx
ON makosh_data.speech_to_text_runs (
    logical_owner_id,
    run_state,
    state_revision
);
