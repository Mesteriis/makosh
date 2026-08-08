CREATE TABLE makosh_data.call_transcription_runs (
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    operation_id BYTEA NOT NULL,
    request_fingerprint BYTEA NOT NULL,
    call_evidence_id BYTEA NOT NULL,
    call_evidence_revision BIGINT NOT NULL,
    recording_evidence_id BYTEA NOT NULL,
    recording_revision BIGINT NOT NULL,
    consent_receipt_id BYTEA NOT NULL,
    consent_policy_revision INTEGER NOT NULL,
    requested_language SMALLINT NOT NULL,
    state SMALLINT NOT NULL,
    state_revision BIGINT NOT NULL,
    source_reference_id BYTEA,
    source_sha256 BYTEA,
    source_declared_bytes BIGINT,
    source_duration_millis BIGINT,
    source_receipt_sha256 BYTEA,
    source_cleanup_completed_at_unix_millis BIGINT,
    stt_request_id BYTEA,
    stt_request_digest BYTEA,
    stt_result_receipt_sha256 BYTEA,
    pending_transcript_reference_id BYTEA,
    pending_transcript_sha256 BYTEA,
    pending_transcript_size_bytes BIGINT,
    pending_detected_language SMALLINT,
    pending_duration_millis BIGINT,
    pending_segment_count INTEGER,
    pending_completeness SMALLINT,
    pending_confidence_basis_points INTEGER,
    artifact_id BYTEA,
    artifact_reference_id BYTEA,
    artifact_receipt_sha256 BYTEA,
    artifact_transcript_sha256 BYTEA,
    artifact_transcript_size_bytes BIGINT,
    artifact_detected_language SMALLINT,
    artifact_duration_millis BIGINT,
    artifact_segment_count INTEGER,
    artifact_completeness SMALLINT,
    artifact_confidence_basis_points INTEGER,
    artifact_runtime_generation BIGINT,
    artifact_grant_epoch BIGINT,
    rejection_code SMALLINT,
    created_at_unix_millis BIGINT NOT NULL,
    updated_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, run_id),
    UNIQUE (logical_owner_id, operation_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(run_id) = 16),
    CHECK (length(operation_id) = 16),
    CHECK (length(request_fingerprint) = 32),
    CHECK (length(call_evidence_id) = 16),
    CHECK (call_evidence_revision > 0),
    CHECK (length(recording_evidence_id) = 16),
    CHECK (recording_revision > 0),
    CHECK (length(consent_receipt_id) = 16),
    CHECK (consent_policy_revision > 0),
    CHECK (requested_language BETWEEN 1 AND 4),
    CHECK (state BETWEEN 1 AND 6),
    CHECK (state_revision > 0),
    CHECK (created_at_unix_millis > 0),
    CHECK (updated_at_unix_millis >= created_at_unix_millis),
    CHECK (
        (source_reference_id IS NULL AND source_sha256 IS NULL
          AND source_declared_bytes IS NULL AND source_duration_millis IS NULL
          AND source_receipt_sha256 IS NULL)
        OR (length(source_reference_id) = 16 AND length(source_sha256) = 32
          AND source_declared_bytes BETWEEN 1 AND 67108864
          AND source_duration_millis BETWEEN 1 AND 14400000
          AND length(source_receipt_sha256) = 32)
    ),
    CHECK (source_cleanup_completed_at_unix_millis IS NULL
      OR source_cleanup_completed_at_unix_millis >= created_at_unix_millis),
    CHECK (
        (stt_request_id IS NULL AND stt_request_digest IS NULL)
        OR (length(stt_request_id) = 16 AND length(stt_request_digest) = 32)
    ),
    CHECK (stt_result_receipt_sha256 IS NULL OR length(stt_result_receipt_sha256) = 32),
    CHECK (
        (pending_transcript_reference_id IS NULL AND pending_transcript_sha256 IS NULL
          AND pending_transcript_size_bytes IS NULL AND pending_detected_language IS NULL
          AND pending_duration_millis IS NULL AND pending_segment_count IS NULL
          AND pending_completeness IS NULL AND pending_confidence_basis_points IS NULL)
        OR (length(pending_transcript_reference_id) = 16
          AND length(pending_transcript_sha256) = 32
          AND pending_transcript_size_bytes BETWEEN 1 AND 4194304
          AND pending_detected_language BETWEEN 1 AND 4
          AND pending_duration_millis BETWEEN 1 AND 14400000
          AND pending_segment_count BETWEEN 0 AND 100000
          AND pending_completeness BETWEEN 1 AND 2
          AND pending_confidence_basis_points BETWEEN 0 AND 10000)
    ),
    CHECK (
        (artifact_id IS NULL AND artifact_reference_id IS NULL
          AND artifact_receipt_sha256 IS NULL AND artifact_transcript_sha256 IS NULL
          AND artifact_transcript_size_bytes IS NULL AND artifact_detected_language IS NULL
          AND artifact_duration_millis IS NULL AND artifact_segment_count IS NULL
          AND artifact_completeness IS NULL AND artifact_confidence_basis_points IS NULL
          AND artifact_runtime_generation IS NULL AND artifact_grant_epoch IS NULL)
        OR (length(artifact_id) = 16 AND length(artifact_reference_id) = 16
          AND length(artifact_receipt_sha256) = 32 AND length(artifact_transcript_sha256) = 32
          AND artifact_transcript_size_bytes BETWEEN 1 AND 4194304
          AND artifact_detected_language BETWEEN 1 AND 4
          AND artifact_duration_millis BETWEEN 1 AND 14400000
          AND artifact_segment_count BETWEEN 0 AND 100000
          AND artifact_completeness BETWEEN 1 AND 2
          AND artifact_confidence_basis_points BETWEEN 0 AND 10000
          AND artifact_runtime_generation > 0 AND artifact_grant_epoch > 0)
    ),
    CHECK (rejection_code IS NULL OR rejection_code BETWEEN 1 AND 5),
    CHECK (
        (state IN (1, 2) AND stt_request_id IS NULL
          AND pending_transcript_reference_id IS NULL AND artifact_id IS NULL
          AND rejection_code IS NULL)
        OR (state = 3 AND source_reference_id IS NOT NULL AND stt_request_id IS NOT NULL
          AND pending_transcript_reference_id IS NULL AND artifact_id IS NULL
          AND rejection_code IS NULL)
        OR (state = 4 AND source_sha256 IS NOT NULL AND stt_request_id IS NOT NULL
          AND stt_result_receipt_sha256 IS NOT NULL
          AND pending_transcript_reference_id IS NOT NULL AND artifact_id IS NULL
          AND rejection_code IS NULL)
        OR (state = 5 AND pending_transcript_reference_id IS NULL
          AND artifact_id IS NOT NULL AND rejection_code IS NULL)
        OR (state = 6 AND pending_transcript_reference_id IS NULL
          AND artifact_id IS NULL AND rejection_code IS NOT NULL)
    )
);

CREATE INDEX call_transcription_runs_recovery_idx
ON makosh_data.call_transcription_runs (logical_owner_id, state, state_revision);

CREATE TABLE makosh_data.call_transcription_inbox (
    logical_owner_id TEXT NOT NULL,
    message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    run_id BYTEA NOT NULL,
    processed_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, message_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (length(run_id) = 16),
    CHECK (processed_at_unix_millis > 0)
);

CREATE TABLE makosh_data.call_transcription_jobs (
    logical_owner_id TEXT NOT NULL,
    job_id BYTEA NOT NULL,
    run_id BYTEA NOT NULL,
    stt_request_id BYTEA NOT NULL,
    stt_request_digest BYTEA NOT NULL,
    state SMALLINT NOT NULL,
    attempt_count INTEGER NOT NULL,
    max_attempts INTEGER NOT NULL,
    worker_id TEXT,
    runtime_generation BIGINT,
    grant_epoch BIGINT,
    lease_fence BIGINT NOT NULL,
    lease_expires_at_unix_millis BIGINT,
    result_receipt_sha256 BYTEA,
    created_at_unix_millis BIGINT NOT NULL,
    updated_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, job_id),
    UNIQUE (logical_owner_id, run_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(job_id) = 16),
    CHECK (length(run_id) = 16),
    CHECK (length(stt_request_id) = 16),
    CHECK (length(stt_request_digest) = 32),
    CHECK (state BETWEEN 1 AND 4),
    CHECK (attempt_count BETWEEN 0 AND max_attempts),
    CHECK (max_attempts BETWEEN 1 AND 10),
    CHECK (lease_fence >= 0),
    CHECK (result_receipt_sha256 IS NULL OR length(result_receipt_sha256) = 32),
    CHECK (created_at_unix_millis > 0),
    CHECK (updated_at_unix_millis >= created_at_unix_millis),
    CHECK (
        (state = 1 AND worker_id IS NULL AND runtime_generation IS NULL
          AND grant_epoch IS NULL AND lease_expires_at_unix_millis IS NULL
          AND result_receipt_sha256 IS NULL)
        OR (state = 2 AND length(worker_id) BETWEEN 1 AND 128
          AND runtime_generation > 0 AND grant_epoch > 0
          AND lease_expires_at_unix_millis > updated_at_unix_millis
          AND result_receipt_sha256 IS NULL)
        OR (state IN (3, 4) AND worker_id IS NULL AND runtime_generation IS NOT NULL
          AND grant_epoch IS NOT NULL AND lease_expires_at_unix_millis IS NULL)
    )
);

CREATE INDEX call_transcription_jobs_claim_idx
ON makosh_data.call_transcription_jobs (logical_owner_id, state, created_at_unix_millis, job_id);

CREATE TABLE makosh_data.call_transcription_outbox (
    logical_owner_id TEXT NOT NULL,
    message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    envelope_bytes BYTEA NOT NULL,
    created_at_unix_millis BIGINT NOT NULL,
    published_at_unix_millis BIGINT,
    PRIMARY KEY (logical_owner_id, message_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (length(envelope_bytes) BETWEEN 1 AND 65536),
    CHECK (created_at_unix_millis > 0),
    CHECK (published_at_unix_millis IS NULL
      OR published_at_unix_millis >= created_at_unix_millis)
);

CREATE INDEX call_transcription_outbox_pending_idx
ON makosh_data.call_transcription_outbox (logical_owner_id, created_at_unix_millis, message_id)
WHERE published_at_unix_millis IS NULL;

CREATE TABLE makosh_data.call_transcription_realtime (
    realtime_sequence BIGSERIAL PRIMARY KEY,
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    state SMALLINT NOT NULL,
    state_revision BIGINT NOT NULL,
    rejection_code SMALLINT,
    occurred_at_unix_millis BIGINT NOT NULL,
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(run_id) = 16),
    CHECK (state BETWEEN 1 AND 6),
    CHECK (state_revision > 0),
    CHECK (rejection_code IS NULL OR rejection_code BETWEEN 1 AND 5),
    CHECK (occurred_at_unix_millis > 0)
);

CREATE INDEX call_transcription_realtime_owner_idx
ON makosh_data.call_transcription_realtime (logical_owner_id, realtime_sequence);

CREATE TABLE makosh_data.call_transcription_read_tickets (
    logical_owner_id TEXT NOT NULL,
    ticket_sha256 BYTEA NOT NULL,
    device_actor_sha256 BYTEA NOT NULL,
    client_session_sha256 BYTEA NOT NULL,
    run_id BYTEA NOT NULL,
    state_revision BIGINT NOT NULL,
    artifact_reference_id BYTEA NOT NULL,
    artifact_receipt_sha256 BYTEA NOT NULL,
    transcript_size_bytes BIGINT NOT NULL,
    runtime_generation BIGINT NOT NULL,
    grant_epoch BIGINT NOT NULL,
    expires_at_unix_seconds BIGINT NOT NULL,
    used_at_unix_seconds BIGINT,
    created_at_unix_seconds BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, ticket_sha256),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(ticket_sha256) = 32),
    CHECK (length(device_actor_sha256) = 32),
    CHECK (length(client_session_sha256) = 32),
    CHECK (length(run_id) = 16),
    CHECK (state_revision > 0),
    CHECK (length(artifact_reference_id) = 16),
    CHECK (length(artifact_receipt_sha256) = 32),
    CHECK (transcript_size_bytes BETWEEN 1 AND 4194304),
    CHECK (runtime_generation > 0),
    CHECK (grant_epoch > 0),
    CHECK (created_at_unix_seconds > 0),
    CHECK (expires_at_unix_seconds >= created_at_unix_seconds),
    CHECK (used_at_unix_seconds IS NULL OR used_at_unix_seconds >= created_at_unix_seconds)
);

CREATE INDEX call_transcription_read_tickets_expiry_idx
ON makosh_data.call_transcription_read_tickets (logical_owner_id, expires_at_unix_seconds);
