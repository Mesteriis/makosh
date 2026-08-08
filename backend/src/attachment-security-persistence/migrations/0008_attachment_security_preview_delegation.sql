CREATE TABLE makosh_data.attachment_security_preview_delegation_inbox (
  message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
  envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
  exact_envelope_bytes BYTEA NOT NULL CHECK (octet_length(exact_envelope_bytes) > 0),
  request_id BYTEA NOT NULL UNIQUE CHECK (octet_length(request_id) = 16),
  preview_run_id BYTEA NOT NULL CHECK (octet_length(preview_run_id) = 16),
  attachment_anchor_id BYTEA NOT NULL CHECK (octet_length(attachment_anchor_id) = 16),
  candidate_message_id BYTEA NOT NULL CHECK (octet_length(candidate_message_id) = 16),
  candidate_envelope_sha256 BYTEA NOT NULL CHECK (octet_length(candidate_envelope_sha256) = 32),
  safety_message_id BYTEA NOT NULL CHECK (octet_length(safety_message_id) = 16),
  safety_evidence_id BYTEA NOT NULL CHECK (octet_length(safety_evidence_id) = 16),
  logical_owner_id TEXT NOT NULL CHECK (char_length(logical_owner_id) BETWEEN 1 AND 128),
  consumed_at_unix_seconds BIGINT NOT NULL
);

CREATE TABLE makosh_data.attachment_security_preview_delegation_outbox (
  message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
  envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
  exact_envelope_bytes BYTEA NOT NULL CHECK (octet_length(exact_envelope_bytes) > 0),
  created_at_unix_seconds BIGINT NOT NULL,
  published_at_unix_seconds BIGINT
);

CREATE TABLE makosh_data.attachment_security_preview_delegation_jobs (
  request_message_id BYTEA PRIMARY KEY REFERENCES
    makosh_data.attachment_security_preview_delegation_inbox (message_id),
  request_id BYTEA NOT NULL UNIQUE CHECK (octet_length(request_id) = 16),
  current_reference_id BYTEA CHECK (current_reference_id IS NULL OR octet_length(current_reference_id) = 16),
  current_receipt_sha256 BYTEA CHECK (current_receipt_sha256 IS NULL OR octet_length(current_receipt_sha256) = 32),
  declared_size BIGINT CHECK (declared_size IS NULL OR declared_size BETWEEN 1 AND 104857600),
  predecessor_custody_source_proof BYTEA CHECK (
    predecessor_custody_source_proof IS NULL
    OR octet_length(predecessor_custody_source_proof) BETWEEN 1 AND 2048
  ),
  rejection_code SMALLINT CHECK (rejection_code BETWEEN 1 AND 4),
  state SMALLINT NOT NULL CHECK (state IN (1, 2)),
  attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count BETWEEN 0 AND 8),
  next_attempt_at_unix_seconds BIGINT NOT NULL,
  claimed_by TEXT CHECK (claimed_by IS NULL OR char_length(claimed_by) BETWEEN 1 AND 128),
  lease_expires_at_unix_seconds BIGINT,
  result_message_id BYTEA UNIQUE REFERENCES
    makosh_data.attachment_security_preview_delegation_outbox (message_id),
  completed_at_unix_seconds BIGINT,
  CHECK (
    (rejection_code IS NULL AND current_reference_id IS NOT NULL
      AND current_receipt_sha256 IS NOT NULL AND declared_size IS NOT NULL
      AND predecessor_custody_source_proof IS NOT NULL)
    OR
    (rejection_code IS NOT NULL AND current_reference_id IS NULL
      AND current_receipt_sha256 IS NULL AND declared_size IS NULL
      AND predecessor_custody_source_proof IS NULL)
  ),
  CHECK (
    (state = 1 AND result_message_id IS NULL AND completed_at_unix_seconds IS NULL)
    OR (state = 2 AND result_message_id IS NOT NULL AND completed_at_unix_seconds IS NOT NULL)
  ),
  CHECK (
    (claimed_by IS NULL AND lease_expires_at_unix_seconds IS NULL)
    OR (claimed_by IS NOT NULL AND lease_expires_at_unix_seconds IS NOT NULL)
  )
);

CREATE INDEX attachment_security_preview_delegation_jobs_pending_idx
  ON makosh_data.attachment_security_preview_delegation_jobs (
    next_attempt_at_unix_seconds, request_message_id
  ) WHERE state = 1;

CREATE INDEX attachment_security_preview_delegation_outbox_pending_idx
  ON makosh_data.attachment_security_preview_delegation_outbox (
    created_at_unix_seconds, message_id
  ) WHERE published_at_unix_seconds IS NULL;
