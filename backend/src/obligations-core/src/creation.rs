use crate::model::validate_draft;
use crate::{
    ObligationStatusV1, ObligationV1, ObligationsValidationErrorV1,
    ReviewedCandidateObligationDraftV1, derive_obligation_id_v1, validate_obligation_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObligationCreationErrorV1 {
    InvalidDraft,
}

pub fn create_obligation_from_reviewed_candidate_v1(
    draft: ReviewedCandidateObligationDraftV1,
) -> Result<ObligationV1, ObligationCreationErrorV1> {
    validate_draft(&draft).map_err(invalid_draft)?;
    let obligation_id = derive_obligation_id_v1(
        &draft.logical_owner_id,
        &draft.provenance.approved_candidate_id,
    )
    .map_err(invalid_draft)?;
    let obligation = ObligationV1 {
        obligation_id,
        logical_owner_id: draft.logical_owner_id,
        statement: draft.statement,
        condition: draft.condition,
        due_at: draft.due_at,
        obligated_party_id: draft.obligated_party_id,
        beneficiary_party_id: draft.beneficiary_party_id,
        evidence_links: draft.evidence_links,
        status: ObligationStatusV1::Open,
        obligation_revision: 1,
        provenance: draft.provenance,
        created_at: draft.created_at,
        updated_at: draft.created_at,
    };
    validate_obligation_v1(&obligation).map_err(invalid_draft)?;
    Ok(obligation)
}

fn invalid_draft(_: ObligationsValidationErrorV1) -> ObligationCreationErrorV1 {
    ObligationCreationErrorV1::InvalidDraft
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ObligationProvenanceV1, ObligationTimestampV1, obligation_creation_fingerprint_v1,
    };

    fn draft() -> ReviewedCandidateObligationDraftV1 {
        ReviewedCandidateObligationDraftV1 {
            logical_owner_id: "owner-1".to_owned(),
            provenance: ObligationProvenanceV1 {
                approved_candidate_id: [1; 16],
                candidate_digest: [2; 32],
                source_evidence_id: [3; 16],
                source_evidence_revision: 4,
                review_id: [5; 16],
                decision_revision: 6,
                decided_by_owner_device_id: [7; 16],
            },
            statement: "Подготовить отчёт".to_owned(),
            condition: Some("я".to_owned()),
            due_at: Some(ObligationTimestampV1 {
                unix_seconds: 1_800_000_100,
                nanos: 0,
            }),
            obligated_party_id: [8; 16],
            beneficiary_party_id: Some([9; 16]),
            evidence_links: vec![crate::ObligationEvidenceLinkV1 {
                evidence_link_id: [10; 16],
                evidence_owner_id: "communications".to_owned(),
                evidence_record_id: [11; 16],
                evidence_revision: 1,
                evidence_digest: [12; 32],
            }],
            created_at: ObligationTimestampV1 {
                unix_seconds: 1_800_000_000,
                nanos: 3,
            },
        }
    }

    #[test]
    fn reviewed_candidate_creates_exactly_one_deterministic_open_obligation() {
        let first = create_obligation_from_reviewed_candidate_v1(draft()).expect("obligation");
        let second = create_obligation_from_reviewed_candidate_v1(draft()).expect("obligation");
        assert_eq!(first, second);
        assert_eq!(first.status, ObligationStatusV1::Open);
        assert_eq!(first.obligation_revision, 1);
    }

    #[test]
    fn fingerprint_detects_conflicting_candidate_content() {
        let first = obligation_creation_fingerprint_v1(&draft()).expect("fingerprint");
        let mut changed = draft();
        changed.statement = "Другой заголовок".to_owned();
        let second = obligation_creation_fingerprint_v1(&changed).expect("fingerprint");
        assert_ne!(first, second);
    }

    #[test]
    fn typed_parties_and_evidence_are_preserved() {
        let obligation = create_obligation_from_reviewed_candidate_v1(draft()).expect("obligation");
        assert_eq!(obligation.condition.as_deref(), Some("я"));
        assert_eq!(obligation.obligated_party_id, [8; 16]);
        assert_eq!(obligation.evidence_links.len(), 1);
    }

    #[test]
    fn missing_human_decision_evidence_is_rejected() {
        let mut invalid = draft();
        invalid.provenance.decided_by_owner_device_id = [0; 16];
        assert_eq!(
            create_obligation_from_reviewed_candidate_v1(invalid),
            Err(ObligationCreationErrorV1::InvalidDraft)
        );
    }
}
