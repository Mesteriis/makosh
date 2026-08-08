CREATE TABLE makosh_data.communications_derived_index_jobs (
  job_id BYTEA PRIMARY KEY CHECK (octet_length(job_id) = 16),
  operation SMALLINT NOT NULL CHECK (operation IN (1, 2)),
  evidence_id BYTEA NOT NULL CHECK (octet_length(evidence_id) = 16),
  message_id BYTEA NOT NULL CHECK (octet_length(message_id) = 16),
  conversation_id BYTEA CHECK (conversation_id IS NULL OR octet_length(conversation_id) = 16),
  blob_ref TEXT,
  blob_reference_id BYTEA CHECK (blob_reference_id IS NULL OR octet_length(blob_reference_id) = 16),
  blob_declared_bytes BIGINT CHECK (blob_declared_bytes IS NULL OR blob_declared_bytes BETWEEN 1 AND 262144),
  blob_sha256 BYTEA CHECK (blob_sha256 IS NULL OR octet_length(blob_sha256) = 32),
  projection_revision INTEGER NOT NULL CHECK (projection_revision > 0),
  observed_at_unix_seconds BIGINT NOT NULL,
  created_at_unix_seconds BIGINT NOT NULL,
  completed_at_unix_seconds BIGINT,
  CHECK (
    (operation = 1 AND conversation_id IS NOT NULL AND blob_ref IS NOT NULL AND blob_reference_id IS NOT NULL AND blob_declared_bytes IS NOT NULL AND blob_sha256 IS NOT NULL)
    OR (operation = 2 AND conversation_id IS NULL AND blob_ref IS NULL AND blob_reference_id IS NULL AND blob_declared_bytes IS NULL AND blob_sha256 IS NULL)
  )
);
CREATE INDEX communications_derived_index_jobs_pending
  ON makosh_data.communications_derived_index_jobs (created_at_unix_seconds, job_id)
  WHERE completed_at_unix_seconds IS NULL;
