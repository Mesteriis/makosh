CREATE TABLE makosh_data.communications_body_custody_transfers (
  evidence_id BYTEA PRIMARY KEY REFERENCES makosh_data.communications_evidence_summaries (
    observation_id
  ) ON DELETE CASCADE CHECK (octet_length(evidence_id) = 16),
  envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
  source_blob_ref TEXT NOT NULL CHECK (
    octet_length(source_blob_ref) BETWEEN 1 AND 512
  ),
  source_reference_id BYTEA NOT NULL CHECK (octet_length(source_reference_id) = 16),
  declared_bytes BIGINT NOT NULL CHECK (declared_bytes BETWEEN 1 AND 67108864),
  plaintext_sha256 BYTEA NOT NULL CHECK (octet_length(plaintext_sha256) = 32),
  source_custody_proof BYTEA NOT NULL CHECK (
    octet_length(source_custody_proof) BETWEEN 1 AND 2048
  ),
  state SMALLINT NOT NULL CHECK (state = 1)
);
