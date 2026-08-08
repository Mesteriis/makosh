ALTER TABLE makosh_data.communications_messages
  ADD COLUMN canonical_revision BIGINT NOT NULL DEFAULT 1
  CHECK (canonical_revision > 0);
