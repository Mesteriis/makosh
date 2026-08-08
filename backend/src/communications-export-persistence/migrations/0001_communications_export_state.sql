CREATE TABLE makosh_data.communications_export_event_inbox (
  message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
  envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
  event_kind SMALLINT NOT NULL CHECK (event_kind IN (1, 2)),
  consumed_at_unix_seconds BIGINT NOT NULL
);

CREATE TABLE makosh_data.communications_export_outbox (
  message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
  envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
  exact_envelope_bytes BYTEA NOT NULL CHECK (
    octet_length(exact_envelope_bytes) > 0
  ),
  created_at_unix_seconds BIGINT NOT NULL,
  published_at_unix_seconds BIGINT
);

CREATE TABLE makosh_data.communications_export_jobs (
  export_id BYTEA PRIMARY KEY CHECK (octet_length(export_id) = 16),
  state SMALLINT NOT NULL CHECK (state IN (1, 2, 3, 4)),
  requested_items INTEGER NOT NULL CHECK (requested_items BETWEEN 1 AND 64),
  completed_items INTEGER NOT NULL DEFAULT 0 CHECK (
    completed_items BETWEEN 0 AND 64
  ),
  created_at_unix_seconds BIGINT NOT NULL,
  updated_at_unix_seconds BIGINT NOT NULL,
  source_result_message_id BYTEA UNIQUE REFERENCES
    makosh_data.communications_export_event_inbox (message_id),
  claimed_by TEXT CHECK (
    claimed_by IS NULL OR char_length(claimed_by) BETWEEN 1 AND 128
  ),
  lease_expires_at_unix_seconds BIGINT,
  artifact_reference_id BYTEA CHECK (
    artifact_reference_id IS NULL
    OR octet_length(artifact_reference_id) = 16
  ),
  artifact_declared_bytes BIGINT CHECK (
    artifact_declared_bytes IS NULL
    OR artifact_declared_bytes BETWEEN 1 AND 25165824
  ),
  artifact_sha256 BYTEA CHECK (
    artifact_sha256 IS NULL OR octet_length(artifact_sha256) = 32
  ),
  rejection_code SMALLINT CHECK (
    rejection_code IS NULL OR rejection_code BETWEEN 1 AND 16
  ),
  CHECK (
    (claimed_by IS NULL AND lease_expires_at_unix_seconds IS NULL)
    OR (claimed_by IS NOT NULL AND lease_expires_at_unix_seconds IS NOT NULL)
  ),
  CHECK (
    (state = 3 AND artifact_reference_id IS NOT NULL
      AND artifact_declared_bytes IS NOT NULL AND artifact_sha256 IS NOT NULL
      AND rejection_code IS NULL)
    OR (state = 4 AND artifact_reference_id IS NULL
      AND artifact_declared_bytes IS NULL AND artifact_sha256 IS NULL
      AND rejection_code IS NOT NULL)
    OR (state IN (1, 2) AND artifact_reference_id IS NULL
      AND artifact_declared_bytes IS NULL AND artifact_sha256 IS NULL
      AND rejection_code IS NULL)
  )
);

CREATE TABLE makosh_data.communications_export_items (
  export_id BYTEA NOT NULL REFERENCES
    makosh_data.communications_export_jobs (export_id),
  ordinal INTEGER NOT NULL CHECK (ordinal BETWEEN 0 AND 63),
  message_id BYTEA NOT NULL CHECK (octet_length(message_id) = 16),
  conversation_id BYTEA CHECK (
    conversation_id IS NULL OR octet_length(conversation_id) = 16
  ),
  evidence_id BYTEA CHECK (
    evidence_id IS NULL OR octet_length(evidence_id) = 16
  ),
  evidence_revision BIGINT CHECK (
    evidence_revision IS NULL OR evidence_revision > 0
  ),
  direction SMALLINT CHECK (
    direction IS NULL OR direction BETWEEN 1 AND 3
  ),
  occurred_at_unix_seconds BIGINT,
  observed_at_unix_seconds BIGINT,
  participant_display_label TEXT CHECK (
    participant_display_label IS NULL
    OR char_length(participant_display_label) BETWEEN 1 AND 512
  ),
  body_state SMALLINT CHECK (
    body_state IS NULL OR body_state IN (1, 2)
  ),
  source_reference_id BYTEA CHECK (
    source_reference_id IS NULL
    OR octet_length(source_reference_id) = 16
  ),
  source_declared_bytes BIGINT CHECK (
    source_declared_bytes IS NULL
    OR source_declared_bytes BETWEEN 1 AND 16777216
  ),
  source_sha256 BYTEA CHECK (
    source_sha256 IS NULL OR octet_length(source_sha256) = 32
  ),
  source_custody_proof BYTEA CHECK (
    source_custody_proof IS NULL
    OR octet_length(source_custody_proof) BETWEEN 1 AND 2048
  ),
  target_reference_id BYTEA CHECK (
    target_reference_id IS NULL
    OR octet_length(target_reference_id) = 16
  ),
  target_sha256 BYTEA CHECK (
    target_sha256 IS NULL OR octet_length(target_sha256) = 32
  ),
  PRIMARY KEY (export_id, ordinal),
  UNIQUE (export_id, message_id),
  CHECK (
    (body_state IS NULL AND conversation_id IS NULL AND evidence_id IS NULL
      AND evidence_revision IS NULL AND direction IS NULL
      AND occurred_at_unix_seconds IS NULL AND observed_at_unix_seconds IS NULL)
    OR (body_state IS NOT NULL AND conversation_id IS NOT NULL
      AND evidence_id IS NOT NULL AND evidence_revision IS NOT NULL
      AND direction IS NOT NULL AND occurred_at_unix_seconds IS NOT NULL
      AND observed_at_unix_seconds IS NOT NULL)
  ),
  CHECK (
    (body_state = 1 AND source_reference_id IS NOT NULL
      AND source_declared_bytes IS NOT NULL AND source_sha256 IS NOT NULL
      AND source_custody_proof IS NOT NULL)
    OR (body_state = 2 AND source_reference_id IS NULL
      AND source_declared_bytes IS NULL AND source_sha256 IS NULL
      AND source_custody_proof IS NULL)
    OR body_state IS NULL
  ),
  CHECK (
    (target_reference_id IS NULL AND target_sha256 IS NULL)
    OR (target_reference_id IS NOT NULL AND target_sha256 IS NOT NULL)
  )
);

CREATE INDEX communications_export_jobs_pending_idx
  ON makosh_data.communications_export_jobs (
    updated_at_unix_seconds,
    export_id
  )
  WHERE state IN (1, 2);

CREATE INDEX communications_export_outbox_pending_idx
  ON makosh_data.communications_export_outbox (
    created_at_unix_seconds,
    message_id
  )
  WHERE published_at_unix_seconds IS NULL;
