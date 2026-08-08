ALTER TABLE makosh_data.communications_messages
  ADD COLUMN canonical_body_state SMALLINT NOT NULL DEFAULT 1
  CHECK (canonical_body_state IN (1, 2, 3, 4));
