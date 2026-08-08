CREATE TABLE makosh_data.communications_derived_index_projections (
  message_id BYTEA PRIMARY KEY REFERENCES makosh_data.communications_messages (
    message_id
  ) ON DELETE CASCADE CHECK (octet_length(message_id) = 16),
  evidence_id BYTEA NOT NULL CHECK (octet_length(evidence_id) = 16),
  conversation_id BYTEA NOT NULL REFERENCES makosh_data.communications_conversations (
    conversation_id
  ) CHECK (octet_length(conversation_id) = 16),
  observed_at_unix_seconds BIGINT NOT NULL,
  projection_revision INTEGER NOT NULL CHECK (projection_revision > 0),
  indexed_at_unix_seconds BIGINT NOT NULL
);

CREATE TABLE makosh_data.communications_derived_index_token_digests (
  message_id BYTEA NOT NULL REFERENCES makosh_data.communications_derived_index_projections (
    message_id
  ) ON DELETE CASCADE CHECK (octet_length(message_id) = 16),
  token_digest BYTEA NOT NULL CHECK (octet_length(token_digest) = 32),
  PRIMARY KEY (message_id, token_digest)
);

CREATE INDEX communications_derived_index_token_digests_lookup
  ON makosh_data.communications_derived_index_token_digests (token_digest, message_id);
