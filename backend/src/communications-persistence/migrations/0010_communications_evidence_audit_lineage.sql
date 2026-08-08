CREATE TABLE makosh_data.communications_evidence_audit_lineage (
  evidence_id BYTEA PRIMARY KEY REFERENCES makosh_data.communications_evidence_summaries (
    observation_id
  ) CHECK (octet_length(evidence_id) = 16),
  causation_message_id BYTEA CHECK (
    causation_message_id IS NULL OR octet_length(causation_message_id) = 16
  ),
  correlation_id BYTEA NOT NULL CHECK (octet_length(correlation_id) = 16),
  recorded_at_unix_seconds BIGINT NOT NULL,
  recorded_at_nanos INTEGER NOT NULL CHECK (
    recorded_at_nanos BETWEEN 0 AND 999999999
  )
);
