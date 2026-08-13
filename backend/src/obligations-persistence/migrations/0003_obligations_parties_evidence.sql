ALTER TABLE makosh_data.obligations_state
    ADD COLUMN obligated_party_id BYTEA NOT NULL,
    ADD COLUMN beneficiary_party_id BYTEA,
    DROP COLUMN due_text_hint,
    DROP COLUMN priority;

ALTER TABLE makosh_data.obligations_state
    ADD CONSTRAINT obligations_state_obligated_party_check
        CHECK (length(obligated_party_id) = 16),
    ADD CONSTRAINT obligations_state_beneficiary_party_check
        CHECK (beneficiary_party_id IS NULL OR length(beneficiary_party_id) = 16);

DROP TABLE makosh_data.obligations_dependencies;
DROP TABLE makosh_data.obligations_checklist;

ALTER TABLE makosh_data.obligations_client_operations
    DROP CONSTRAINT obligations_client_operations_operation_kind_check,
    ADD CONSTRAINT obligations_client_operations_operation_kind_check
        CHECK (operation_kind BETWEEN 1 AND 4);

CREATE TABLE makosh_data.obligations_evidence (
    logical_owner_id TEXT NOT NULL,
    obligation_id BYTEA NOT NULL,
    evidence_link_id BYTEA NOT NULL,
    evidence_owner_id TEXT NOT NULL,
    evidence_record_id BYTEA NOT NULL,
    evidence_revision BIGINT NOT NULL,
    evidence_digest BYTEA NOT NULL,
    PRIMARY KEY (logical_owner_id, obligation_id, evidence_link_id),
    FOREIGN KEY (logical_owner_id, obligation_id)
        REFERENCES makosh_data.obligations_state (logical_owner_id, obligation_id) ON DELETE CASCADE,
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(obligation_id) = 16),
    CHECK (length(evidence_link_id) = 16),
    CHECK (length(evidence_owner_id) BETWEEN 1 AND 128),
    CHECK (length(evidence_record_id) = 16),
    CHECK (evidence_revision > 0),
    CHECK (length(evidence_digest) = 32)
);

ALTER TABLE makosh_data.obligations_evidence ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.obligations_evidence FORCE ROW LEVEL SECURITY;
CREATE POLICY obligations_evidence_owner_policy
ON makosh_data.obligations_evidence
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));
