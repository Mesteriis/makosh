CREATE TABLE IF NOT EXISTS makosh_data.telemost_accounts (
 logical_owner_id TEXT NOT NULL,
 account_cursor_sha256 BYTEA NOT NULL,
 mapping_revision BIGINT NOT NULL,
 lifecycle_state SMALLINT NOT NULL,
 updated_at_unix_millis BIGINT NOT NULL,
 PRIMARY KEY(logical_owner_id,account_cursor_sha256),
 CHECK(octet_length(account_cursor_sha256)=32),
 CHECK(mapping_revision>0),
 CHECK(lifecycle_state BETWEEN 1 AND 3),
 CHECK(updated_at_unix_millis>0)
);
CREATE TABLE IF NOT EXISTS makosh_data.telemost_observation_inbox (
 logical_owner_id TEXT NOT NULL,
 message_id BYTEA NOT NULL,
 envelope_sha256 BYTEA NOT NULL,
 envelope_bytes BYTEA NOT NULL,
 account_cursor_sha256 BYTEA NOT NULL,
 source_revision BIGINT NOT NULL,
 completed_at_unix_millis BIGINT NOT NULL,
 PRIMARY KEY(logical_owner_id,message_id),
 CHECK(octet_length(message_id)=16),
 CHECK(octet_length(envelope_sha256)=32),
 CHECK(octet_length(envelope_bytes) BETWEEN 1 AND 524288),
 CHECK(octet_length(account_cursor_sha256)=32),
 CHECK(source_revision>0),
 CHECK(completed_at_unix_millis>0)
);
CREATE TABLE IF NOT EXISTS makosh_data.telemost_call_evidence_outbox (
 sequence BIGINT GENERATED ALWAYS AS IDENTITY,
 logical_owner_id TEXT NOT NULL,
 message_id BYTEA NOT NULL,
 envelope_sha256 BYTEA NOT NULL,
 envelope_bytes BYTEA NOT NULL,
 published_at_unix_millis BIGINT,
 PRIMARY KEY(sequence),
 UNIQUE(logical_owner_id,message_id),
 CHECK(octet_length(message_id)=16),
 CHECK(octet_length(envelope_sha256)=32),
 CHECK(octet_length(envelope_bytes) BETWEEN 1 AND 524288),
 CHECK(published_at_unix_millis IS NULL OR published_at_unix_millis>0)
);
CREATE INDEX telemost_outbox_pending_idx ON makosh_data.telemost_call_evidence_outbox(sequence) WHERE published_at_unix_millis IS NULL;
ALTER TABLE makosh_data.telemost_accounts ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.telemost_accounts FORCE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.telemost_observation_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.telemost_observation_inbox FORCE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.telemost_call_evidence_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.telemost_call_evidence_outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY telemost_accounts_owner ON makosh_data.telemost_accounts USING (logical_owner_id=current_setting('makosh.logical_owner_id',true)) WITH CHECK (logical_owner_id=current_setting('makosh.logical_owner_id',true));
CREATE POLICY telemost_inbox_owner ON makosh_data.telemost_observation_inbox USING (logical_owner_id=current_setting('makosh.logical_owner_id',true)) WITH CHECK (logical_owner_id=current_setting('makosh.logical_owner_id',true));
CREATE POLICY telemost_outbox_owner ON makosh_data.telemost_call_evidence_outbox USING (logical_owner_id=current_setting('makosh.logical_owner_id',true)) WITH CHECK (logical_owner_id=current_setting('makosh.logical_owner_id',true));
