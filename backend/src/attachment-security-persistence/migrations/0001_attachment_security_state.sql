CREATE TABLE makosh_data.attachment_security_join_locks (
  attachment_anchor_id BYTEA PRIMARY KEY CHECK (
    octet_length(attachment_anchor_id) = 16
  )
);

CREATE TABLE makosh_data.attachment_security_event_inbox (
  message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
  envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
  event_kind SMALLINT NOT NULL CHECK (event_kind IN (1, 2)),
  consumed_at_unix_seconds BIGINT NOT NULL
);

CREATE TABLE makosh_data.attachment_security_scan_candidates (
  attachment_anchor_id BYTEA PRIMARY KEY REFERENCES
    makosh_data.attachment_security_join_locks (attachment_anchor_id),
  message_id BYTEA NOT NULL UNIQUE REFERENCES
    makosh_data.attachment_security_event_inbox (message_id),
  blob_reference_id BYTEA NOT NULL CHECK (octet_length(blob_reference_id) = 16),
  declared_size BIGINT NOT NULL CHECK (
    declared_size BETWEEN 0 AND 67108864
  ),
  blob_receipt_sha256 BYTEA NOT NULL CHECK (
    octet_length(blob_receipt_sha256) = 32
  ),
  causation_message_id BYTEA NOT NULL CHECK (
    octet_length(causation_message_id) = 16
  ),
  correlation_id BYTEA NOT NULL CHECK (octet_length(correlation_id) = 16),
  observed_at_unix_seconds BIGINT NOT NULL
);

CREATE TABLE makosh_data.attachment_security_canonical_states (
  attachment_anchor_id BYTEA PRIMARY KEY REFERENCES
    makosh_data.attachment_security_join_locks (attachment_anchor_id),
  message_id BYTEA NOT NULL UNIQUE REFERENCES
    makosh_data.attachment_security_event_inbox (message_id),
  expected_state SMALLINT NOT NULL CHECK (expected_state = 2),
  next_state SMALLINT NOT NULL CHECK (next_state = 3),
  evidence_id BYTEA NOT NULL CHECK (octet_length(evidence_id) = 16),
  correlation_id BYTEA NOT NULL CHECK (octet_length(correlation_id) = 16),
  observed_at_unix_seconds BIGINT NOT NULL
);

CREATE TABLE makosh_data.attachment_security_join_quarantines (
  evidence_id BYTEA NOT NULL CHECK (octet_length(evidence_id) = 16),
  source_message_id BYTEA NOT NULL CHECK (octet_length(source_message_id) = 16),
  attachment_anchor_id BYTEA NOT NULL CHECK (
    octet_length(attachment_anchor_id) = 16
  ),
  correlation_id BYTEA NOT NULL CHECK (octet_length(correlation_id) = 16),
  reason SMALLINT NOT NULL CHECK (reason BETWEEN 1 AND 6),
  recorded_at_unix_seconds BIGINT NOT NULL,
  PRIMARY KEY (evidence_id, source_message_id)
);

CREATE TABLE makosh_data.attachment_security_verdict_outbox (
  message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
  envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
  exact_envelope_bytes BYTEA NOT NULL CHECK (
    octet_length(exact_envelope_bytes) > 0
  ),
  created_at_unix_seconds BIGINT NOT NULL,
  published_at_unix_seconds BIGINT
);

CREATE TABLE makosh_data.attachment_security_scan_jobs (
  job_id BYTEA PRIMARY KEY CHECK (octet_length(job_id) = 16),
  candidate_message_id BYTEA NOT NULL UNIQUE REFERENCES
    makosh_data.attachment_security_scan_candidates (message_id),
  canonical_state_message_id BYTEA NOT NULL UNIQUE REFERENCES
    makosh_data.attachment_security_canonical_states (message_id),
  attachment_anchor_id BYTEA NOT NULL UNIQUE REFERENCES
    makosh_data.attachment_security_join_locks (attachment_anchor_id),
  blob_reference_id BYTEA NOT NULL CHECK (octet_length(blob_reference_id) = 16),
  declared_size BIGINT NOT NULL CHECK (
    declared_size BETWEEN 0 AND 67108864
  ),
  blob_receipt_sha256 BYTEA NOT NULL CHECK (
    octet_length(blob_receipt_sha256) = 32
  ),
  causation_message_id BYTEA NOT NULL CHECK (
    octet_length(causation_message_id) = 16
  ),
  correlation_id BYTEA NOT NULL CHECK (octet_length(correlation_id) = 16),
  state SMALLINT NOT NULL CHECK (state IN (1, 2, 3)),
  attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
  max_attempts INTEGER NOT NULL CHECK (max_attempts BETWEEN 1 AND 32),
  next_attempt_at_unix_seconds BIGINT NOT NULL,
  claimed_by TEXT CHECK (
    claimed_by IS NULL OR char_length(claimed_by) BETWEEN 1 AND 128
  ),
  lease_expires_at_unix_seconds BIGINT,
  completed_at_unix_seconds BIGINT,
  outbox_message_id BYTEA REFERENCES
    makosh_data.attachment_security_verdict_outbox (message_id),
  CHECK (
    (state = 1 AND completed_at_unix_seconds IS NULL AND outbox_message_id IS NULL)
    OR (state = 2 AND completed_at_unix_seconds IS NOT NULL AND outbox_message_id IS NOT NULL)
    OR (state = 3 AND completed_at_unix_seconds IS NOT NULL AND outbox_message_id IS NULL)
  ),
  CHECK (
    (claimed_by IS NULL AND lease_expires_at_unix_seconds IS NULL)
    OR (claimed_by IS NOT NULL AND lease_expires_at_unix_seconds IS NOT NULL)
  )
);

CREATE INDEX attachment_security_scan_jobs_pending_idx
  ON makosh_data.attachment_security_scan_jobs (
    next_attempt_at_unix_seconds,
    job_id
  )
  WHERE state = 1;

CREATE INDEX attachment_security_verdict_outbox_pending_idx
  ON makosh_data.attachment_security_verdict_outbox (
    created_at_unix_seconds,
    message_id
  )
  WHERE published_at_unix_seconds IS NULL;
