CREATE TABLE makosh_data.communications_body_custody_transfer_lifecycle (
  evidence_id BYTEA PRIMARY KEY REFERENCES makosh_data.communications_body_custody_transfers (
    evidence_id
  ) ON DELETE CASCADE CHECK (octet_length(evidence_id) = 16),
  state SMALLINT NOT NULL CHECK (state IN (1, 2, 3)),
  claimed_by TEXT,
  lease_expires_at_unix_seconds BIGINT,
  completed_at_unix_seconds BIGINT
);

CREATE INDEX communications_body_custody_transfer_lifecycle_pending_idx
  ON makosh_data.communications_body_custody_transfer_lifecycle (
    state, lease_expires_at_unix_seconds, evidence_id
  );
