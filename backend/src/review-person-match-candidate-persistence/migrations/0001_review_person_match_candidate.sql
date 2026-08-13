CREATE TABLE makosh_data.review_person_match_candidate_state (
    logical_owner_id TEXT NOT NULL,
    review_id BYTEA NOT NULL,
    evidence_event_id BYTEA NOT NULL,
    candidate_id BYTEA NOT NULL,
    candidate_digest BYTEA NOT NULL,
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
    state SMALLINT NOT NULL,
    promotion_status SMALLINT NOT NULL,
    review_revision BIGINT NOT NULL,
    decision_id BYTEA,
    decided_by_owner_device_id BYTEA,
    decided_at_unix_millis BIGINT,
    approved_action_kind SMALLINT,
    approved_action_bytes BYTEA,
    approved_action_digest BYTEA,
    updated_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, review_id),
    UNIQUE (logical_owner_id, candidate_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(review_id) = 16 AND length(evidence_event_id) = 16),
    CHECK (length(candidate_id) = 16 AND length(candidate_digest) = 32),
    CHECK (length(first_person_id) = 16 AND length(second_person_id) = 16),
    CHECK (first_person_id <> second_person_id),
    CHECK (length(first_integration_public_id) = 16),
    CHECK (length(first_account_public_id) = 16),
    CHECK (length(first_source_public_id) = 16),
    CHECK (length(second_integration_public_id) = 16),
    CHECK (length(second_account_public_id) = 16),
    CHECK (length(second_source_public_id) = 16),
    CHECK (match_kind BETWEEN 1 AND 2),
    CHECK (observed_at_unix_millis > 0 AND resulting_owner_revision > 0),
    CHECK (state BETWEEN 1 AND 3 AND promotion_status BETWEEN 1 AND 4),
    CHECK (review_revision > 0 AND updated_at_unix_millis >= observed_at_unix_millis),
    CHECK (
      (state=1 AND promotion_status=1 AND decision_id IS NULL
       AND decided_by_owner_device_id IS NULL AND decided_at_unix_millis IS NULL
       AND approved_action_kind IS NULL AND approved_action_bytes IS NULL
       AND approved_action_digest IS NULL)
      OR
      (state=2 AND promotion_status BETWEEN 2 AND 4
       AND length(decision_id)=16 AND length(decided_by_owner_device_id)=16
       AND decided_at_unix_millis >= observed_at_unix_millis
       AND approved_action_kind BETWEEN 1 AND 3
       AND octet_length(approved_action_bytes) BETWEEN 1 AND 2048
       AND length(approved_action_digest)=32)
      OR
      (state=3 AND promotion_status=1
       AND length(decision_id)=16 AND length(decided_by_owner_device_id)=16
       AND decided_at_unix_millis >= observed_at_unix_millis
       AND approved_action_kind IS NULL AND approved_action_bytes IS NULL
       AND approved_action_digest IS NULL)
    )
);

CREATE TABLE makosh_data.review_person_match_candidate_inbox (
    logical_owner_id TEXT NOT NULL,
    message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    envelope_bytes BYTEA NOT NULL,
    message_kind SMALLINT NOT NULL,
    request_fingerprint BYTEA NOT NULL,
    review_id BYTEA NOT NULL,
    resulting_review_revision BIGINT NOT NULL,
    received_at_unix_millis BIGINT NOT NULL,
    completed_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, message_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(message_id)=16 AND length(envelope_sha256)=32),
    CHECK (octet_length(envelope_bytes) BETWEEN 1 AND 65536),
    CHECK (message_kind BETWEEN 1 AND 3),
    CHECK (length(request_fingerprint)=32 AND length(review_id)=16),
    CHECK (resulting_review_revision > 0),
    CHECK (received_at_unix_millis > 0 AND completed_at_unix_millis >= received_at_unix_millis)
);

CREATE TABLE makosh_data.review_person_match_candidate_outbox (
    outbox_sequence BIGINT GENERATED ALWAYS AS IDENTITY,
    logical_owner_id TEXT NOT NULL,
    message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    envelope_bytes BYTEA NOT NULL,
    review_id BYTEA NOT NULL,
    review_revision BIGINT NOT NULL,
    semantic_kind SMALLINT NOT NULL,
    created_at_unix_millis BIGINT NOT NULL,
    published_at_unix_millis BIGINT,
    PRIMARY KEY (logical_owner_id, message_id),
    UNIQUE (logical_owner_id, outbox_sequence),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(message_id)=16 AND length(envelope_sha256)=32),
    CHECK (octet_length(envelope_bytes) BETWEEN 1 AND 65536),
    CHECK (length(review_id)=16 AND review_revision > 0),
    CHECK (semantic_kind BETWEEN 1 AND 4),
    CHECK (created_at_unix_millis > 0),
    CHECK (published_at_unix_millis IS NULL OR published_at_unix_millis >= created_at_unix_millis)
);

CREATE INDEX review_person_match_candidate_outbox_pending_idx
ON makosh_data.review_person_match_candidate_outbox
  (logical_owner_id, outbox_sequence)
WHERE published_at_unix_millis IS NULL;

ALTER TABLE makosh_data.review_person_match_candidate_state ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.review_person_match_candidate_state FORCE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.review_person_match_candidate_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.review_person_match_candidate_inbox FORCE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.review_person_match_candidate_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.review_person_match_candidate_outbox FORCE ROW LEVEL SECURITY;

CREATE POLICY review_person_match_candidate_state_owner
ON makosh_data.review_person_match_candidate_state
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
CREATE POLICY review_person_match_candidate_inbox_owner
ON makosh_data.review_person_match_candidate_inbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
CREATE POLICY review_person_match_candidate_outbox_owner
ON makosh_data.review_person_match_candidate_outbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
