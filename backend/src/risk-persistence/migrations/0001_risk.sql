CREATE TABLE makosh_data.risk_projection_control (
  logical_owner_id TEXT PRIMARY KEY,
  active_projection_generation BIGINT NOT NULL CHECK (active_projection_generation >= 0),
  next_projection_generation BIGINT NOT NULL CHECK (next_projection_generation > active_projection_generation),
  rebuilt_at_unix_millis BIGINT NOT NULL CHECK (rebuilt_at_unix_millis > 0),
  CHECK (length(logical_owner_id) BETWEEN 1 AND 128)
);
CREATE TABLE makosh_data.risk_projection_entries (
  logical_owner_id TEXT NOT NULL,
  projection_generation BIGINT NOT NULL,
  event_id BYTEA NOT NULL,
  source_owner TEXT NOT NULL,
  entity_kind TEXT NOT NULL,
  entity_id BYTEA NOT NULL,
  source_revision BIGINT NOT NULL,
  reason_code TEXT NOT NULL,
  severity SMALLINT NOT NULL,
  occurred_at_unix_millis BIGINT NOT NULL,
  expires_at_unix_millis BIGINT NOT NULL,
  deleted_at BIGINT,
  PRIMARY KEY (logical_owner_id, projection_generation, event_id),
  UNIQUE (logical_owner_id, projection_generation, source_owner, entity_kind, entity_id, source_revision),
  CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
  CHECK (projection_generation > 0 AND source_revision > 0),
  CHECK (length(event_id)=16 AND length(entity_id)=16),
  CHECK (length(source_owner) BETWEEN 1 AND 128 AND length(entity_kind) BETWEEN 1 AND 128),
  CHECK (length(reason_code) <= 128 AND occurred_at_unix_millis > 0),
  CHECK ((deleted_at IS NULL AND length(reason_code) > 0 AND severity BETWEEN 1 AND 5 AND expires_at_unix_millis > occurred_at_unix_millis)
    OR (deleted_at IS NOT NULL AND deleted_at >= occurred_at_unix_millis AND reason_code='' AND severity=0 AND expires_at_unix_millis=0))
);
CREATE TABLE makosh_data.risk_projection_inbox (
  logical_owner_id TEXT NOT NULL,
  message_id BYTEA NOT NULL,
  envelope_sha256 BYTEA NOT NULL,
  envelope_bytes BYTEA NOT NULL,
  source_owner TEXT NOT NULL,
  source_revision BIGINT NOT NULL,
  completed_at_unix_millis BIGINT NOT NULL,
  PRIMARY KEY (logical_owner_id,message_id),
  CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
  CHECK (length(message_id)=16 AND length(envelope_sha256)=32),
  CHECK (octet_length(envelope_bytes) BETWEEN 1 AND 65536),
  CHECK (source_revision > 0 AND completed_at_unix_millis > 0)
);
CREATE TABLE makosh_data.risk_projection_rebuilds (
  logical_owner_id TEXT NOT NULL,
  projection_generation BIGINT NOT NULL,
  state SMALLINT NOT NULL,
  expected_source_count BIGINT NOT NULL,
  applied_source_count BIGINT NOT NULL,
  started_at_unix_millis BIGINT NOT NULL,
  completed_at_unix_millis BIGINT,
  PRIMARY KEY (logical_owner_id,projection_generation),
  CHECK (projection_generation > 0 AND state BETWEEN 1 AND 3),
  CHECK (expected_source_count >= 0 AND applied_source_count BETWEEN 0 AND expected_source_count),
  CHECK (started_at_unix_millis > 0),
  CHECK (completed_at_unix_millis IS NULL OR completed_at_unix_millis >= started_at_unix_millis)
);
CREATE INDEX risk_projection_order_idx ON makosh_data.risk_projection_entries
  (logical_owner_id,projection_generation,occurred_at_unix_millis DESC,source_owner,entity_kind,entity_id,source_revision,event_id);

ALTER TABLE makosh_data.risk_projection_control ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.risk_projection_control FORCE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.risk_projection_entries ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.risk_projection_entries FORCE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.risk_projection_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.risk_projection_inbox FORCE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.risk_projection_rebuilds ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.risk_projection_rebuilds FORCE ROW LEVEL SECURITY;
CREATE POLICY risk_projection_control_owner ON makosh_data.risk_projection_control USING (logical_owner_id=current_setting('makosh.logical_owner_id',true)) WITH CHECK (logical_owner_id=current_setting('makosh.logical_owner_id',true));
CREATE POLICY risk_projection_entries_owner ON makosh_data.risk_projection_entries USING (logical_owner_id=current_setting('makosh.logical_owner_id',true)) WITH CHECK (logical_owner_id=current_setting('makosh.logical_owner_id',true));
CREATE POLICY risk_projection_inbox_owner ON makosh_data.risk_projection_inbox USING (logical_owner_id=current_setting('makosh.logical_owner_id',true)) WITH CHECK (logical_owner_id=current_setting('makosh.logical_owner_id',true));
CREATE POLICY risk_projection_rebuilds_owner ON makosh_data.risk_projection_rebuilds USING (logical_owner_id=current_setting('makosh.logical_owner_id',true)) WITH CHECK (logical_owner_id=current_setting('makosh.logical_owner_id',true));
