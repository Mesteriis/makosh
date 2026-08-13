CREATE TABLE makosh_data.reviewed_person_match_candidate_promotion_requests (
    logical_owner_id TEXT NOT NULL,
    approval_message_id BYTEA NOT NULL,
    approval_envelope_sha256 BYTEA NOT NULL,
    approval_envelope_bytes BYTEA NOT NULL,
    review_id BYTEA NOT NULL,
    candidate_id BYTEA NOT NULL,
    candidate_digest BYTEA NOT NULL,
    decision_id BYTEA NOT NULL,
    decision_revision BIGINT NOT NULL,
    approved_action_digest BYTEA NOT NULL,
    persons_command_id BYTEA,
    persons_command_fingerprint BYTEA,
    persons_command_message_id BYTEA,
    persons_result_message_id BYTEA,
    promotion_outcome SMALLINT,
    failure_code SMALLINT,
    created_at_unix_millis BIGINT NOT NULL,
    updated_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, approval_message_id),
    UNIQUE (logical_owner_id, review_id, decision_revision),
    UNIQUE (logical_owner_id, decision_id),
    UNIQUE (logical_owner_id, persons_command_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(approval_message_id)=16 AND length(approval_envelope_sha256)=32),
    CHECK (octet_length(approval_envelope_bytes) BETWEEN 1 AND 65536),
    CHECK (length(review_id)=16 AND length(candidate_id)=16 AND length(candidate_digest)=32),
    CHECK (length(decision_id)=16 AND decision_revision > 0 AND length(approved_action_digest)=32),
    CHECK ((persons_command_id IS NULL AND persons_command_fingerprint IS NULL AND persons_command_message_id IS NULL)
        OR (length(persons_command_id)=16 AND length(persons_command_fingerprint)=32
            AND length(persons_command_message_id)=16 AND persons_command_message_id=persons_command_id)),
    CHECK (created_at_unix_millis > 0 AND updated_at_unix_millis >= created_at_unix_millis),
    CHECK ((persons_command_id IS NOT NULL AND persons_result_message_id IS NULL AND promotion_outcome IS NULL AND failure_code IS NULL)
        OR (persons_command_id IS NOT NULL AND length(persons_result_message_id)=16 AND promotion_outcome=1 AND failure_code IS NULL)
        OR (persons_command_id IS NOT NULL AND length(persons_result_message_id)=16 AND promotion_outcome=2 AND failure_code=3)
        OR (persons_command_id IS NULL AND persons_result_message_id IS NULL AND promotion_outcome=2 AND failure_code=2))
);

CREATE TABLE makosh_data.reviewed_person_match_candidate_promotion_result_inbox (
    logical_owner_id TEXT NOT NULL,
    result_message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    envelope_bytes BYTEA NOT NULL,
    persons_command_id BYTEA NOT NULL,
    review_id BYTEA NOT NULL,
    processed_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, result_message_id),
    UNIQUE (logical_owner_id, persons_command_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(result_message_id)=16 AND length(envelope_sha256)=32),
    CHECK (octet_length(envelope_bytes) BETWEEN 1 AND 65536),
    CHECK (length(persons_command_id)=16 AND length(review_id)=16),
    CHECK (processed_at_unix_millis > 0)
);

CREATE TABLE makosh_data.reviewed_person_match_candidate_promotion_outbox (
    outbox_sequence BIGINT GENERATED ALWAYS AS IDENTITY,
    logical_owner_id TEXT NOT NULL,
    message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    envelope_bytes BYTEA NOT NULL,
    semantic_kind SMALLINT NOT NULL,
    created_at_unix_millis BIGINT NOT NULL,
    published_at_unix_millis BIGINT,
    PRIMARY KEY (logical_owner_id, message_id),
    UNIQUE (logical_owner_id, outbox_sequence),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(message_id)=16 AND length(envelope_sha256)=32),
    CHECK (octet_length(envelope_bytes) BETWEEN 1 AND 65536),
    CHECK (semantic_kind BETWEEN 1 AND 2),
    CHECK (created_at_unix_millis > 0),
    CHECK (published_at_unix_millis IS NULL OR published_at_unix_millis >= created_at_unix_millis)
);

CREATE INDEX reviewed_person_match_candidate_promotion_outbox_pending_idx
ON makosh_data.reviewed_person_match_candidate_promotion_outbox
  (logical_owner_id, outbox_sequence)
WHERE published_at_unix_millis IS NULL;

ALTER TABLE makosh_data.reviewed_person_match_candidate_promotion_requests ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.reviewed_person_match_candidate_promotion_requests FORCE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.reviewed_person_match_candidate_promotion_result_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.reviewed_person_match_candidate_promotion_result_inbox FORCE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.reviewed_person_match_candidate_promotion_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.reviewed_person_match_candidate_promotion_outbox FORCE ROW LEVEL SECURITY;

CREATE POLICY reviewed_person_match_candidate_promotion_requests_owner ON makosh_data.reviewed_person_match_candidate_promotion_requests
USING (logical_owner_id=current_setting('makosh.logical_owner_id', true)) WITH CHECK (logical_owner_id=current_setting('makosh.logical_owner_id', true));
CREATE POLICY reviewed_person_match_candidate_promotion_result_inbox_owner ON makosh_data.reviewed_person_match_candidate_promotion_result_inbox
USING (logical_owner_id=current_setting('makosh.logical_owner_id', true)) WITH CHECK (logical_owner_id=current_setting('makosh.logical_owner_id', true));
CREATE POLICY reviewed_person_match_candidate_promotion_outbox_owner ON makosh_data.reviewed_person_match_candidate_promotion_outbox
USING (logical_owner_id=current_setting('makosh.logical_owner_id', true)) WITH CHECK (logical_owner_id=current_setting('makosh.logical_owner_id', true));
