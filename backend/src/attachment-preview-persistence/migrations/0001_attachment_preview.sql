CREATE TABLE makosh_data.attachment_preview_runs (
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    operation_id BYTEA NOT NULL,
    request_fingerprint BYTEA NOT NULL,
    attachment_anchor_id BYTEA NOT NULL,
    state SMALLINT NOT NULL,
    state_revision BIGINT NOT NULL,
    preview_kind SMALLINT,
    content_type SMALLINT,
    preview_size_bytes BIGINT NOT NULL,
    truncated BOOLEAN NOT NULL,
    error_code SMALLINT,
    created_at_unix_millis BIGINT NOT NULL,
    updated_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, run_id),
    UNIQUE (logical_owner_id, operation_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(run_id) = 16),
    CHECK (length(operation_id) = 16),
    CHECK (length(request_fingerprint) = 32),
    CHECK (length(attachment_anchor_id) = 16),
    CHECK (state BETWEEN 1 AND 6),
    CHECK (state_revision > 0),
    CHECK (preview_kind IS NULL OR preview_kind BETWEEN 1 AND 5),
    CHECK (content_type IS NULL OR content_type BETWEEN 1 AND 4),
    CHECK (preview_size_bytes BETWEEN 0 AND 33554432),
    CHECK (error_code IS NULL OR error_code BETWEEN 1 AND 12),
    CHECK (
        (state BETWEEN 1 AND 3 AND preview_kind IS NULL AND content_type IS NULL AND preview_size_bytes = 0 AND truncated = FALSE AND error_code IS NULL)
        OR (state = 4 AND preview_kind IS NOT NULL AND content_type IS NOT NULL AND preview_size_bytes > 0 AND error_code IS NULL)
        OR (state = 5 AND preview_kind IS NULL AND content_type IS NULL AND preview_size_bytes = 0 AND truncated = FALSE AND error_code = 4)
        OR (state = 6 AND preview_kind IS NULL AND content_type IS NULL AND preview_size_bytes = 0 AND truncated = FALSE AND error_code IN (1,3,5,6,7,8,9,12))
    ),
    CHECK (created_at_unix_millis > 0),
    CHECK (updated_at_unix_millis >= created_at_unix_millis)
);

CREATE INDEX attachment_preview_runs_anchor_idx
ON makosh_data.attachment_preview_runs (logical_owner_id, attachment_anchor_id, state, run_id);

CREATE TABLE makosh_data.attachment_preview_event_inbox (
    logical_owner_id TEXT NOT NULL,
    message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    event_kind SMALLINT NOT NULL,
    attachment_anchor_id BYTEA NOT NULL,
    exact_payload_sha256 BYTEA NOT NULL,
    processed_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, message_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (event_kind IN (1, 2)),
    CHECK (length(attachment_anchor_id) = 16),
    CHECK (length(exact_payload_sha256) = 32),
    CHECK (processed_at_unix_millis > 0)
);

CREATE TABLE makosh_data.attachment_preview_scan_candidates (
    logical_owner_id TEXT NOT NULL,
    attachment_anchor_id BYTEA NOT NULL,
    message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    exact_payload_sha256 BYTEA NOT NULL,
    source_reference_id BYTEA NOT NULL,
    declared_size BIGINT NOT NULL,
    source_receipt_sha256 BYTEA NOT NULL,
    custody_transfer_source_proof BYTEA NOT NULL,
    observed_at_unix_seconds BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, attachment_anchor_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(attachment_anchor_id) = 16),
    CHECK (length(message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (length(exact_payload_sha256) = 32),
    CHECK (length(source_reference_id) = 16),
    CHECK (declared_size BETWEEN 1 AND 104857600),
    CHECK (length(source_receipt_sha256) = 32),
    CHECK (length(custody_transfer_source_proof) BETWEEN 1 AND 2048),
    CHECK (observed_at_unix_seconds > 0)
);

CREATE TABLE makosh_data.attachment_preview_safety_facts (
    logical_owner_id TEXT NOT NULL,
    attachment_anchor_id BYTEA NOT NULL,
    message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    exact_payload_sha256 BYTEA NOT NULL,
    expected_state SMALLINT NOT NULL,
    next_state SMALLINT NOT NULL,
    evidence_id BYTEA NOT NULL,
    observed_at_unix_seconds BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, attachment_anchor_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(attachment_anchor_id) = 16),
    CHECK (length(message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (length(exact_payload_sha256) = 32),
    CHECK (expected_state BETWEEN 1 AND 6),
    CHECK (next_state IN (4,5,6)),
    CHECK (expected_state != next_state),
    CHECK (length(evidence_id) = 16),
    CHECK (observed_at_unix_seconds > 0)
);

CREATE TABLE makosh_data.attachment_preview_custody_outbox (
    logical_owner_id TEXT NOT NULL,
    request_id BYTEA NOT NULL,
    run_id BYTEA NOT NULL,
    candidate_message_id BYTEA NOT NULL,
    safety_message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    exact_envelope_bytes BYTEA NOT NULL,
    published_at_unix_millis BIGINT,
    created_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, request_id),
    UNIQUE (logical_owner_id, run_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(request_id) = 16),
    CHECK (length(run_id) = 16),
    CHECK (length(candidate_message_id) = 16),
    CHECK (length(safety_message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (length(exact_envelope_bytes) BETWEEN 1 AND 8192),
    CHECK (published_at_unix_millis IS NULL OR published_at_unix_millis >= created_at_unix_millis),
    CHECK (created_at_unix_millis > 0)
);

CREATE INDEX attachment_preview_custody_outbox_pending_idx
ON makosh_data.attachment_preview_custody_outbox (logical_owner_id, published_at_unix_millis, created_at_unix_millis, request_id);

CREATE TABLE makosh_data.attachment_preview_custody_result_inbox (
    logical_owner_id TEXT NOT NULL,
    message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    request_id BYTEA NOT NULL,
    run_id BYTEA NOT NULL,
    attachment_anchor_id BYTEA NOT NULL,
    result_kind SMALLINT NOT NULL,
    processed_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, message_id),
    UNIQUE (logical_owner_id, request_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (length(request_id) = 16),
    CHECK (length(run_id) = 16),
    CHECK (length(attachment_anchor_id) = 16),
    CHECK (result_kind IN (1,2)),
    CHECK (processed_at_unix_millis > 0)
);

CREATE TABLE makosh_data.attachment_preview_jobs (
    logical_owner_id TEXT NOT NULL,
    job_id BYTEA NOT NULL,
    run_id BYTEA NOT NULL,
    request_id BYTEA NOT NULL,
    result_message_id BYTEA NOT NULL,
    result_envelope_sha256 BYTEA NOT NULL,
    attachment_anchor_id BYTEA NOT NULL,
    candidate_message_id BYTEA NOT NULL,
    safety_message_id BYTEA NOT NULL,
    source_reference_id BYTEA NOT NULL,
    source_receipt_sha256 BYTEA NOT NULL,
    source_declared_size BIGINT NOT NULL,
    custody_transfer_source_proof BYTEA NOT NULL,
    custody_proof_sha256 BYTEA NOT NULL,
    target_reference_id BYTEA,
    target_receipt_sha256 BYTEA,
    state SMALLINT NOT NULL,
    attempt_count INTEGER NOT NULL,
    max_attempts INTEGER NOT NULL,
    worker_id TEXT,
    runtime_generation BIGINT,
    grant_epoch BIGINT,
    lease_fence BIGINT NOT NULL,
    lease_expires_at_unix_millis BIGINT,
    created_at_unix_millis BIGINT NOT NULL,
    updated_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, job_id),
    UNIQUE (logical_owner_id, run_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(job_id) = 16),
    CHECK (length(run_id) = 16),
    CHECK (length(request_id) = 16),
    CHECK (length(result_message_id) = 16),
    CHECK (length(result_envelope_sha256) = 32),
    CHECK (length(attachment_anchor_id) = 16),
    CHECK (length(candidate_message_id) = 16),
    CHECK (length(safety_message_id) = 16),
    CHECK (length(source_reference_id) = 16),
    CHECK (length(source_receipt_sha256) = 32),
    CHECK (source_declared_size BETWEEN 1 AND 104857600),
    CHECK (length(custody_transfer_source_proof) BETWEEN 1 AND 2048),
    CHECK (length(custody_proof_sha256) = 32),
    CHECK ((target_reference_id IS NULL AND target_receipt_sha256 IS NULL) OR (length(target_reference_id) = 16 AND length(target_receipt_sha256) = 32)),
    CHECK (state BETWEEN 1 AND 4),
    CHECK (attempt_count BETWEEN 0 AND max_attempts),
    CHECK (max_attempts BETWEEN 1 AND 32),
    CHECK (lease_fence >= 0),
    CHECK (
        (state = 2 AND length(worker_id) BETWEEN 1 AND 128 AND runtime_generation > 0 AND grant_epoch > 0 AND lease_expires_at_unix_millis > updated_at_unix_millis)
        OR (state != 2 AND worker_id IS NULL AND runtime_generation IS NULL AND grant_epoch IS NULL AND lease_expires_at_unix_millis IS NULL)
    ),
    CHECK (created_at_unix_millis > 0),
    CHECK (updated_at_unix_millis >= created_at_unix_millis)
);

CREATE INDEX attachment_preview_jobs_claim_idx
ON makosh_data.attachment_preview_jobs (logical_owner_id, state, created_at_unix_millis, job_id);

CREATE TABLE makosh_data.attachment_preview_artifacts (
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    derived_reference_id BYTEA NOT NULL,
    derived_receipt_sha256 BYTEA NOT NULL,
    source_receipt_sha256 BYTEA NOT NULL,
    renderer_identity_sha256 BYTEA NOT NULL,
    preview_kind SMALLINT NOT NULL,
    content_type SMALLINT NOT NULL,
    preview_size_bytes BIGINT NOT NULL,
    truncated BOOLEAN NOT NULL,
    runtime_generation BIGINT NOT NULL,
    grant_epoch BIGINT NOT NULL,
    committed_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, run_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(run_id) = 16),
    CHECK (length(derived_reference_id) = 16),
    CHECK (length(derived_receipt_sha256) = 32),
    CHECK (length(source_receipt_sha256) = 32),
    CHECK (length(renderer_identity_sha256) = 32),
    CHECK (preview_kind BETWEEN 1 AND 5),
    CHECK (content_type BETWEEN 1 AND 4),
    CHECK (preview_size_bytes BETWEEN 1 AND 33554432),
    CHECK (runtime_generation > 0),
    CHECK (grant_epoch > 0),
    CHECK (committed_at_unix_millis > 0)
);

CREATE TABLE makosh_data.attachment_preview_read_tickets (
    logical_owner_id TEXT NOT NULL,
    ticket_sha256 BYTEA NOT NULL,
    device_actor_sha256 BYTEA NOT NULL,
    run_id BYTEA NOT NULL,
    state_revision BIGINT NOT NULL,
    derived_reference_id BYTEA NOT NULL,
    derived_receipt_sha256 BYTEA NOT NULL,
    renderer_identity_sha256 BYTEA NOT NULL,
    content_type SMALLINT NOT NULL,
    preview_size_bytes BIGINT NOT NULL,
    runtime_generation BIGINT NOT NULL,
    grant_epoch BIGINT NOT NULL,
    expires_at_unix_seconds BIGINT NOT NULL,
    used_at_unix_seconds BIGINT,
    created_at_unix_seconds BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, ticket_sha256),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(ticket_sha256) = 32),
    CHECK (length(device_actor_sha256) = 32),
    CHECK (length(run_id) = 16),
    CHECK (state_revision > 0),
    CHECK (length(derived_reference_id) = 16),
    CHECK (length(derived_receipt_sha256) = 32),
    CHECK (length(renderer_identity_sha256) = 32),
    CHECK (content_type BETWEEN 1 AND 4),
    CHECK (preview_size_bytes BETWEEN 1 AND 33554432),
    CHECK (runtime_generation > 0),
    CHECK (grant_epoch > 0),
    CHECK (expires_at_unix_seconds > created_at_unix_seconds),
    CHECK (used_at_unix_seconds IS NULL OR used_at_unix_seconds BETWEEN created_at_unix_seconds AND expires_at_unix_seconds),
    CHECK (created_at_unix_seconds > 0)
);

CREATE INDEX attachment_preview_read_tickets_expiry_idx
ON makosh_data.attachment_preview_read_tickets (logical_owner_id, expires_at_unix_seconds, ticket_sha256);

CREATE TABLE makosh_data.attachment_preview_realtime (
    realtime_sequence BIGSERIAL PRIMARY KEY,
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    state SMALLINT NOT NULL,
    state_revision BIGINT NOT NULL,
    preview_kind SMALLINT,
    content_type SMALLINT,
    preview_size_bytes BIGINT NOT NULL,
    truncated BOOLEAN NOT NULL,
    error_code SMALLINT,
    occurred_at_unix_millis BIGINT NOT NULL,
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(run_id) = 16),
    CHECK (state BETWEEN 1 AND 6),
    CHECK (state_revision > 0),
    CHECK (preview_kind IS NULL OR preview_kind BETWEEN 1 AND 5),
    CHECK (content_type IS NULL OR content_type BETWEEN 1 AND 4),
    CHECK (preview_size_bytes BETWEEN 0 AND 33554432),
    CHECK (error_code IS NULL OR error_code BETWEEN 1 AND 12),
    CHECK (
        (state BETWEEN 1 AND 3 AND preview_kind IS NULL AND content_type IS NULL AND preview_size_bytes = 0 AND truncated = FALSE AND error_code IS NULL)
        OR (state = 4 AND preview_kind IS NOT NULL AND content_type IS NOT NULL AND preview_size_bytes > 0 AND error_code IS NULL)
        OR (state = 5 AND preview_kind IS NULL AND content_type IS NULL AND preview_size_bytes = 0 AND truncated = FALSE AND error_code = 4)
        OR (state = 6 AND preview_kind IS NULL AND content_type IS NULL AND preview_size_bytes = 0 AND truncated = FALSE AND error_code IN (1,3,5,6,7,8,9,12))
    ),
    CHECK (occurred_at_unix_millis > 0)
);

CREATE INDEX attachment_preview_realtime_owner_sequence_idx
ON makosh_data.attachment_preview_realtime (logical_owner_id, realtime_sequence);
