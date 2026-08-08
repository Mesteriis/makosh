CREATE TABLE makosh_data.communications_derived_index_failures (
  evidence_id BYTEA PRIMARY KEY CHECK (octet_length(evidence_id) = 16),
  message_id BYTEA NOT NULL CHECK (octet_length(message_id) = 16),
  projection_revision INTEGER NOT NULL CHECK (projection_revision > 0),
  observed_at_unix_seconds BIGINT NOT NULL,
  failure_code SMALLINT NOT NULL CHECK (failure_code IN (6)),
  recorded_at_unix_seconds BIGINT NOT NULL
);
