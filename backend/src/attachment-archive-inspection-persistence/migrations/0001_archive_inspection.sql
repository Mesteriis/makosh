CREATE TABLE makosh_data.attachment_archive_inspection_runs (
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    operation_id BYTEA NOT NULL,
    request_fingerprint BYTEA NOT NULL,
    attachment_anchor_id BYTEA NOT NULL,
    state SMALLINT NOT NULL,
    state_revision BIGINT NOT NULL,
    error_code SMALLINT,
    rejection_evidence_id BYTEA,
    created_at_unix_millis BIGINT NOT NULL,
    updated_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, run_id),
    UNIQUE (logical_owner_id, operation_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(run_id) = 16),
    CHECK (length(operation_id) = 16),
    CHECK (length(request_fingerprint) = 32),
    CHECK (length(attachment_anchor_id) = 16),
    CHECK (state BETWEEN 1 AND 5),
    CHECK (state_revision > 0),
    CHECK (error_code IS NULL OR error_code BETWEEN 1 AND 5),
    CHECK (rejection_evidence_id IS NULL OR length(rejection_evidence_id) = 16),
    CHECK (
        (state = 5 AND error_code IS NOT NULL AND length(rejection_evidence_id) = 16)
        OR (state != 5 AND error_code IS NULL AND rejection_evidence_id IS NULL)
    ),
    CHECK (created_at_unix_millis > 0),
    CHECK (updated_at_unix_millis >= created_at_unix_millis)
);

CREATE INDEX attachment_archive_inspection_runs_anchor_idx
ON makosh_data.attachment_archive_inspection_runs (
    logical_owner_id,
    attachment_anchor_id,
    state,
    run_id
);

CREATE TABLE makosh_data.attachment_archive_inspection_event_inbox (
    logical_owner_id TEXT NOT NULL,
    message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    event_kind SMALLINT NOT NULL,
    attachment_anchor_id BYTEA NOT NULL,
    processed_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, message_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (event_kind IN (1, 2)),
    CHECK (length(attachment_anchor_id) = 16),
    CHECK (processed_at_unix_millis > 0)
);

CREATE TABLE makosh_data.attachment_archive_inspection_scan_candidates (
    logical_owner_id TEXT NOT NULL,
    attachment_anchor_id BYTEA NOT NULL,
    message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    blob_reference_id BYTEA NOT NULL,
    declared_size BIGINT NOT NULL,
    blob_receipt_sha256 BYTEA NOT NULL,
    custody_transfer_source_proof BYTEA NOT NULL,
    observed_at_unix_seconds BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, attachment_anchor_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(attachment_anchor_id) = 16),
    CHECK (length(message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (length(blob_reference_id) = 16),
    CHECK (declared_size BETWEEN 1 AND 104857600),
    CHECK (length(blob_receipt_sha256) = 32),
    CHECK (length(custody_transfer_source_proof) BETWEEN 1 AND 2048)
);

CREATE TABLE makosh_data.attachment_archive_inspection_safety_facts (
    logical_owner_id TEXT NOT NULL,
    attachment_anchor_id BYTEA NOT NULL,
    message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    expected_state SMALLINT NOT NULL,
    next_state SMALLINT NOT NULL,
    evidence_id BYTEA NOT NULL,
    observed_at_unix_seconds BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, attachment_anchor_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(attachment_anchor_id) = 16),
    CHECK (length(message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (expected_state BETWEEN 1 AND 6),
    CHECK (next_state IN (4, 5, 6)),
    CHECK (expected_state != next_state),
    CHECK (length(evidence_id) = 16)
);

CREATE TABLE makosh_data.attachment_archive_inspection_custody_delegation_requests (
    logical_owner_id TEXT NOT NULL,
    request_id BYTEA NOT NULL,
    run_id BYTEA NOT NULL,
    attachment_anchor_id BYTEA NOT NULL,
    candidate_message_id BYTEA NOT NULL,
    candidate_envelope_sha256 BYTEA NOT NULL,
    safety_message_id BYTEA NOT NULL,
    safety_evidence_id BYTEA NOT NULL,
    state SMALLINT NOT NULL,
    envelope_sha256 BYTEA,
    exact_envelope_bytes BYTEA,
    published_at_unix_millis BIGINT,
    result_message_id BYTEA,
    created_at_unix_millis BIGINT NOT NULL,
    updated_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, request_id),
    UNIQUE (logical_owner_id, run_id),
    FOREIGN KEY (logical_owner_id, run_id)
        REFERENCES makosh_data.attachment_archive_inspection_runs (
            logical_owner_id,
            run_id
        ),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(request_id) = 16),
    CHECK (length(run_id) = 16),
    CHECK (length(attachment_anchor_id) = 16),
    CHECK (length(candidate_message_id) = 16),
    CHECK (length(candidate_envelope_sha256) = 32),
    CHECK (length(safety_message_id) = 16),
    CHECK (length(safety_evidence_id) = 16),
    CHECK (state BETWEEN 1 AND 4),
    CHECK (
        (
            state = 1
            AND envelope_sha256 IS NULL
            AND exact_envelope_bytes IS NULL
            AND published_at_unix_millis IS NULL
            AND result_message_id IS NULL
        )
        OR (
            state = 2
            AND length(envelope_sha256) = 32
            AND length(exact_envelope_bytes) BETWEEN 1 AND 8192
            AND result_message_id IS NULL
        )
        OR (
            state IN (3, 4)
            AND length(envelope_sha256) = 32
            AND length(exact_envelope_bytes) BETWEEN 1 AND 8192
            AND length(result_message_id) = 16
        )
    ),
    CHECK (
        published_at_unix_millis IS NULL
        OR published_at_unix_millis >= created_at_unix_millis
    ),
    CHECK (created_at_unix_millis > 0),
    CHECK (updated_at_unix_millis >= created_at_unix_millis)
);

CREATE INDEX attachment_archive_inspection_custody_delegation_outbox_idx
ON makosh_data.attachment_archive_inspection_custody_delegation_requests (
    logical_owner_id,
    state,
    published_at_unix_millis,
    created_at_unix_millis,
    request_id
);

CREATE TABLE makosh_data.attachment_archive_inspection_custody_result_inbox (
    logical_owner_id TEXT NOT NULL,
    message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    request_id BYTEA NOT NULL,
    result_kind SMALLINT NOT NULL,
    processed_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, message_id),
    FOREIGN KEY (logical_owner_id, request_id)
        REFERENCES makosh_data.attachment_archive_inspection_custody_delegation_requests (
            logical_owner_id,
            request_id
        ),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (length(request_id) = 16),
    CHECK (result_kind IN (1, 2)),
    CHECK (processed_at_unix_millis > 0)
);

CREATE TABLE makosh_data.attachment_archive_inspection_jobs (
    logical_owner_id TEXT NOT NULL,
    job_id BYTEA NOT NULL,
    run_id BYTEA NOT NULL,
    candidate_message_id BYTEA NOT NULL,
    safety_message_id BYTEA NOT NULL,
    delegation_request_id BYTEA NOT NULL,
    delegation_result_message_id BYTEA NOT NULL,
    attachment_anchor_id BYTEA NOT NULL,
    source_reference_id BYTEA NOT NULL,
    target_reference_id BYTEA,
    target_receipt_sha256 BYTEA,
    declared_size BIGINT NOT NULL,
    blob_receipt_sha256 BYTEA NOT NULL,
    custody_transfer_source_proof BYTEA NOT NULL,
    safety_evidence_id BYTEA NOT NULL,
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
    FOREIGN KEY (logical_owner_id, delegation_request_id)
        REFERENCES makosh_data.attachment_archive_inspection_custody_delegation_requests (
            logical_owner_id,
            request_id
        ),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(job_id) = 16),
    CHECK (length(run_id) = 16),
    CHECK (length(candidate_message_id) = 16),
    CHECK (length(safety_message_id) = 16),
    CHECK (length(delegation_request_id) = 16),
    CHECK (length(delegation_result_message_id) = 16),
    CHECK (length(attachment_anchor_id) = 16),
    CHECK (length(source_reference_id) = 16),
    CHECK (
        (
            target_reference_id IS NULL
            AND target_receipt_sha256 IS NULL
        )
        OR (
            length(target_reference_id) = 16
            AND length(target_receipt_sha256) = 32
        )
    ),
    CHECK (declared_size BETWEEN 1 AND 104857600),
    CHECK (length(blob_receipt_sha256) = 32),
    CHECK (length(custody_transfer_source_proof) BETWEEN 1 AND 2048),
    CHECK (length(safety_evidence_id) = 16),
    CHECK (state BETWEEN 1 AND 3),
    CHECK (attempt_count BETWEEN 0 AND max_attempts),
    CHECK (max_attempts BETWEEN 1 AND 32),
    CHECK (lease_fence >= 0),
    CHECK (
        (
            state = 2
            AND length(worker_id) BETWEEN 1 AND 128
            AND runtime_generation > 0
            AND grant_epoch > 0
            AND lease_expires_at_unix_millis > updated_at_unix_millis
        )
        OR
        (
            state != 2
            AND worker_id IS NULL
            AND runtime_generation IS NULL
            AND grant_epoch IS NULL
            AND lease_expires_at_unix_millis IS NULL
        )
    ),
    CHECK (created_at_unix_millis > 0),
    CHECK (updated_at_unix_millis >= created_at_unix_millis)
);

CREATE INDEX attachment_archive_inspection_jobs_claim_idx
ON makosh_data.attachment_archive_inspection_jobs (
    logical_owner_id,
    state,
    created_at_unix_millis,
    job_id
);

CREATE TABLE makosh_data.attachment_archive_inspection_reports (
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    entry_count INTEGER NOT NULL,
    total_uncompressed_bytes BIGINT NOT NULL,
    completed_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, run_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(run_id) = 16),
    CHECK (entry_count BETWEEN 0 AND 1000),
    CHECK (total_uncompressed_bytes BETWEEN 0 AND 1073741824),
    CHECK (completed_at_unix_millis > 0)
);

CREATE TABLE makosh_data.attachment_archive_inspection_report_entries (
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    entry_ordinal INTEGER NOT NULL,
    normalized_path_utf8 BYTEA NOT NULL,
    compressed_size BIGINT NOT NULL,
    uncompressed_size BIGINT NOT NULL,
    entry_kind SMALLINT NOT NULL,
    PRIMARY KEY (logical_owner_id, run_id, entry_ordinal),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(run_id) = 16),
    CHECK (entry_ordinal BETWEEN 0 AND 999),
    CHECK (length(normalized_path_utf8) BETWEEN 1 AND 1024),
    CHECK (compressed_size >= 0),
    CHECK (uncompressed_size BETWEEN 0 AND 268435456),
    CHECK (entry_kind IN (1, 2))
);

CREATE TABLE makosh_data.attachment_archive_inspection_realtime (
    realtime_sequence BIGSERIAL PRIMARY KEY,
    logical_owner_id TEXT NOT NULL,
    run_id BYTEA NOT NULL,
    state SMALLINT NOT NULL,
    state_revision BIGINT NOT NULL,
    error_code SMALLINT,
    occurred_at_unix_millis BIGINT NOT NULL,
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(run_id) = 16),
    CHECK (state BETWEEN 1 AND 5),
    CHECK (state_revision > 0),
    CHECK (error_code IS NULL OR error_code BETWEEN 1 AND 5),
    CHECK (occurred_at_unix_millis > 0)
);

CREATE INDEX attachment_archive_inspection_realtime_owner_idx
ON makosh_data.attachment_archive_inspection_realtime (
    logical_owner_id,
    realtime_sequence
);
