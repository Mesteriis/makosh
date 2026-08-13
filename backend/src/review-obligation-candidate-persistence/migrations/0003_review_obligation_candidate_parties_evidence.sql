ALTER TABLE makosh_data.review_obligation_candidate_state
    ADD COLUMN due_at_unix_seconds BIGINT,
    ADD COLUMN due_at_nanos INTEGER,
    ADD COLUMN obligated_party_id BYTEA NOT NULL,
    ADD COLUMN beneficiary_party_id BYTEA,
    DROP COLUMN due_text_hint;

ALTER TABLE makosh_data.review_obligation_candidate_state
    ADD CONSTRAINT review_obligation_candidate_state_due_at_check CHECK (
        (due_at_unix_seconds IS NULL AND due_at_nanos IS NULL)
        OR (due_at_unix_seconds > 0 AND due_at_nanos BETWEEN 0 AND 999999999)
    ),
    ADD CONSTRAINT review_obligation_candidate_state_obligated_party_check
        CHECK (length(obligated_party_id) = 16),
    ADD CONSTRAINT review_obligation_candidate_state_beneficiary_party_check
        CHECK (beneficiary_party_id IS NULL OR length(beneficiary_party_id) = 16);

CREATE TABLE makosh_data.review_obligation_candidate_evidence (
    logical_owner_id TEXT NOT NULL,
    review_id BYTEA NOT NULL,
    evidence_link_id BYTEA NOT NULL,
    evidence_owner_id TEXT NOT NULL,
    evidence_record_id BYTEA NOT NULL,
    evidence_revision BIGINT NOT NULL,
    evidence_digest BYTEA NOT NULL,
    PRIMARY KEY (logical_owner_id, review_id, evidence_link_id),
    FOREIGN KEY (logical_owner_id, review_id)
        REFERENCES makosh_data.review_obligation_candidate_state (logical_owner_id, review_id)
        ON DELETE CASCADE,
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(review_id) = 16),
    CHECK (length(evidence_link_id) = 16),
    CHECK (length(evidence_owner_id) BETWEEN 1 AND 128),
    CHECK (length(evidence_record_id) = 16),
    CHECK (evidence_revision > 0),
    CHECK (length(evidence_digest) = 32)
);

ALTER TABLE makosh_data.review_obligation_candidate_evidence ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.review_obligation_candidate_evidence FORCE ROW LEVEL SECURITY;
CREATE POLICY review_obligation_candidate_evidence_owner_policy
ON makosh_data.review_obligation_candidate_evidence
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
