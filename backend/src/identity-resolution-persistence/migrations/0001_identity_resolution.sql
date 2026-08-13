CREATE TABLE makosh_data.identity_resolution_candidates (
  logical_owner_id TEXT NOT NULL,
  candidate_id BYTEA NOT NULL,
  evidence_event_id BYTEA NOT NULL,
  first_person_id BYTEA NOT NULL,
  second_person_id BYTEA NOT NULL,
  first_integration_public_id BYTEA NOT NULL,
  first_account_public_id BYTEA NOT NULL,
  first_source_public_id BYTEA NOT NULL,
  second_integration_public_id BYTEA NOT NULL,
  second_account_public_id BYTEA NOT NULL,
  second_source_public_id BYTEA NOT NULL,
  match_kind SMALLINT NOT NULL,
  observed_at_unix_millis BIGINT NOT NULL,
  resulting_owner_revision BIGINT NOT NULL,
  proposal_message_id BYTEA NOT NULL,
  proposal_sha256 BYTEA NOT NULL,
  proposal_bytes BYTEA NOT NULL,
  updated_at_unix_millis BIGINT NOT NULL,
  PRIMARY KEY (logical_owner_id, candidate_id),
  CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
  CHECK (length(candidate_id)=16 AND length(evidence_event_id)=16),
  CHECK (length(first_person_id)=16 AND length(second_person_id)=16 AND first_person_id<>second_person_id),
  CHECK (length(first_integration_public_id)=16 AND length(first_account_public_id)=16 AND length(first_source_public_id)=16),
  CHECK (length(second_integration_public_id)=16 AND length(second_account_public_id)=16 AND length(second_source_public_id)=16),
  CHECK (match_kind BETWEEN 1 AND 2),
  CHECK (observed_at_unix_millis>0 AND resulting_owner_revision>0),
  CHECK (length(proposal_message_id)=16 AND length(proposal_sha256)=32),
  CHECK (octet_length(proposal_bytes) BETWEEN 1 AND 65536),
  CHECK (updated_at_unix_millis>=observed_at_unix_millis)
);

CREATE TABLE makosh_data.identity_resolution_inbox (
  logical_owner_id TEXT NOT NULL,
  message_id BYTEA NOT NULL,
  envelope_sha256 BYTEA NOT NULL,
  envelope_bytes BYTEA NOT NULL,
  candidate_id BYTEA NOT NULL,
  proposal_message_id BYTEA NOT NULL,
  completed_at_unix_millis BIGINT NOT NULL,
  PRIMARY KEY (logical_owner_id, message_id),
  CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
  CHECK (length(message_id)=16 AND length(envelope_sha256)=32),
  CHECK (octet_length(envelope_bytes) BETWEEN 1 AND 65536),
  CHECK (length(candidate_id)=16 AND length(proposal_message_id)=16),
  CHECK (completed_at_unix_millis>0)
);

CREATE TABLE makosh_data.identity_resolution_outbox (
  outbox_sequence BIGINT GENERATED ALWAYS AS IDENTITY,
  logical_owner_id TEXT NOT NULL,
  message_id BYTEA NOT NULL,
  envelope_sha256 BYTEA NOT NULL,
  envelope_bytes BYTEA NOT NULL,
  candidate_id BYTEA NOT NULL,
  created_at_unix_millis BIGINT NOT NULL,
  published_at_unix_millis BIGINT,
  PRIMARY KEY (logical_owner_id, message_id),
  UNIQUE (logical_owner_id, outbox_sequence),
  CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
  CHECK (length(message_id)=16 AND length(envelope_sha256)=32),
  CHECK (octet_length(envelope_bytes) BETWEEN 1 AND 65536),
  CHECK (length(candidate_id)=16 AND created_at_unix_millis>0),
  CHECK (published_at_unix_millis IS NULL OR published_at_unix_millis>=created_at_unix_millis)
);

CREATE INDEX identity_resolution_outbox_pending_idx ON makosh_data.identity_resolution_outbox
  (logical_owner_id, outbox_sequence) WHERE published_at_unix_millis IS NULL;

ALTER TABLE makosh_data.identity_resolution_candidates ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.identity_resolution_candidates FORCE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.identity_resolution_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.identity_resolution_inbox FORCE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.identity_resolution_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.identity_resolution_outbox FORCE ROW LEVEL SECURITY;

CREATE POLICY identity_resolution_candidates_owner ON makosh_data.identity_resolution_candidates
USING (logical_owner_id=current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id=current_setting('makosh.logical_owner_id', true));
CREATE POLICY identity_resolution_inbox_owner ON makosh_data.identity_resolution_inbox
USING (logical_owner_id=current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id=current_setting('makosh.logical_owner_id', true));
CREATE POLICY identity_resolution_outbox_owner ON makosh_data.identity_resolution_outbox
USING (logical_owner_id=current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id=current_setting('makosh.logical_owner_id', true));
