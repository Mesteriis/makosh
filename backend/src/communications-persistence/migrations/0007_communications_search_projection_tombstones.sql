CREATE TABLE makosh_data.communications_derived_index_tombstones (
  message_id BYTEA PRIMARY KEY CHECK (octet_length(message_id) = 16),
  evidence_id BYTEA NOT NULL CHECK (octet_length(evidence_id) = 16),
  observed_at_unix_seconds BIGINT NOT NULL,
  projection_revision INTEGER NOT NULL CHECK (projection_revision > 0),
  removed_at_unix_seconds BIGINT NOT NULL
);
