CREATE TABLE makosh_data.search_projection_control (
  logical_owner_id TEXT NOT NULL,
  active_projection_generation BIGINT NOT NULL,
  next_projection_generation BIGINT NOT NULL,
  rebuilt_at_unix_millis BIGINT NOT NULL,
  PRIMARY KEY (logical_owner_id),
  CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
  CHECK (active_projection_generation >= 0),
  CHECK (next_projection_generation > active_projection_generation),
  CHECK (rebuilt_at_unix_millis > 0)
);

CREATE TABLE makosh_data.search_projection_documents (
  logical_owner_id TEXT NOT NULL,
  projection_generation BIGINT NOT NULL,
  source_owner TEXT NOT NULL,
  entity_kind TEXT NOT NULL,
  entity_id BYTEA NOT NULL,
  source_revision BIGINT NOT NULL,
  lifecycle_state TEXT NOT NULL,
  occurred_at_unix_millis BIGINT NOT NULL,
  deleted_at BIGINT,
  PRIMARY KEY (logical_owner_id, projection_generation, source_owner, entity_kind, entity_id),
  CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
  CHECK (projection_generation > 0 AND source_revision > 0),
  CHECK (length(source_owner) BETWEEN 1 AND 128),
  CHECK (length(entity_kind) BETWEEN 1 AND 128),
  CHECK (length(entity_id) = 16),
  CHECK (length(lifecycle_state) <= 128),
  CHECK (occurred_at_unix_millis > 0),
  CHECK (deleted_at IS NULL OR (deleted_at >= occurred_at_unix_millis AND lifecycle_state = ''))
);

CREATE TABLE makosh_data.search_projection_tokens (
  logical_owner_id TEXT NOT NULL,
  projection_generation BIGINT NOT NULL,
  source_owner TEXT NOT NULL,
  entity_kind TEXT NOT NULL,
  entity_id BYTEA NOT NULL,
  token_digest BYTEA NOT NULL,
  PRIMARY KEY (logical_owner_id, projection_generation, source_owner, entity_kind, entity_id, token_digest),
  FOREIGN KEY (logical_owner_id, projection_generation, source_owner, entity_kind, entity_id)
    REFERENCES makosh_data.search_projection_documents
      (logical_owner_id, projection_generation, source_owner, entity_kind, entity_id)
    ON DELETE CASCADE,
  CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
  CHECK (projection_generation > 0),
  CHECK (length(token_digest) = 32)
);

CREATE TABLE makosh_data.search_projection_inbox (
  logical_owner_id TEXT NOT NULL,
  message_id BYTEA NOT NULL,
  envelope_sha256 BYTEA NOT NULL,
  envelope_bytes BYTEA NOT NULL,
  source_owner TEXT NOT NULL,
  source_revision BIGINT NOT NULL,
  completed_at_unix_millis BIGINT NOT NULL,
  PRIMARY KEY (logical_owner_id, message_id),
  CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
  CHECK (length(message_id) = 16 AND length(envelope_sha256) = 32),
  CHECK (octet_length(envelope_bytes) BETWEEN 1 AND 65536),
  CHECK (length(source_owner) BETWEEN 1 AND 128),
  CHECK (source_revision > 0 AND completed_at_unix_millis > 0)
);

CREATE TABLE makosh_data.search_projection_rebuilds (
  logical_owner_id TEXT NOT NULL,
  projection_generation BIGINT NOT NULL,
  state SMALLINT NOT NULL,
  expected_source_count BIGINT NOT NULL,
  applied_source_count BIGINT NOT NULL,
  started_at_unix_millis BIGINT NOT NULL,
  completed_at_unix_millis BIGINT,
  PRIMARY KEY (logical_owner_id, projection_generation),
  CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
  CHECK (projection_generation > 0),
  CHECK (state BETWEEN 1 AND 3),
  CHECK (expected_source_count >= 0),
  CHECK (applied_source_count BETWEEN 0 AND expected_source_count),
  CHECK (started_at_unix_millis > 0),
  CHECK (completed_at_unix_millis IS NULL OR completed_at_unix_millis >= started_at_unix_millis)
);

CREATE INDEX search_projection_token_lookup_idx
  ON makosh_data.search_projection_tokens
  (logical_owner_id, projection_generation, token_digest, source_owner, entity_kind, entity_id);
CREATE INDEX search_projection_document_order_idx
  ON makosh_data.search_projection_documents
  (logical_owner_id, projection_generation, occurred_at_unix_millis DESC, source_owner, entity_kind, entity_id)
  WHERE deleted_at IS NULL;

ALTER TABLE makosh_data.search_projection_control ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.search_projection_control FORCE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.search_projection_documents ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.search_projection_documents FORCE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.search_projection_tokens ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.search_projection_tokens FORCE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.search_projection_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.search_projection_inbox FORCE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.search_projection_rebuilds ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.search_projection_rebuilds FORCE ROW LEVEL SECURITY;

CREATE POLICY search_projection_control_owner ON makosh_data.search_projection_control
USING (logical_owner_id=current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id=current_setting('makosh.logical_owner_id', true));
CREATE POLICY search_projection_documents_owner ON makosh_data.search_projection_documents
USING (logical_owner_id=current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id=current_setting('makosh.logical_owner_id', true));
CREATE POLICY search_projection_tokens_owner ON makosh_data.search_projection_tokens
USING (logical_owner_id=current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id=current_setting('makosh.logical_owner_id', true));
CREATE POLICY search_projection_inbox_owner ON makosh_data.search_projection_inbox
USING (logical_owner_id=current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id=current_setting('makosh.logical_owner_id', true));
CREATE POLICY search_projection_rebuilds_owner ON makosh_data.search_projection_rebuilds
USING (logical_owner_id=current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id=current_setting('makosh.logical_owner_id', true));
