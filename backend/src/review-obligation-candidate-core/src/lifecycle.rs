use crate::model::{nonzero, valid_timestamp, validate_draft};
use crate::{
    ReviewObligationCandidateDecisionV1, ReviewObligationCandidateDraftV1,
    ReviewObligationCandidatePromotionResultV1, ReviewObligationCandidatePromotionStatusV1,
    ReviewObligationCandidateStateV1, ReviewObligationCandidateTimestampV1,
    ReviewObligationCandidateV1, ReviewObligationCandidateValidationErrorV1, STABLE_ID_BYTES_V1,
    derive_review_obligation_candidate_id_v1, validate_review_obligation_candidate_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewObligationCandidateTransitionErrorV1 {
    InvalidRecord,
    InvalidActor,
    InvalidTimestamp,
    RevisionConflict,
    TerminalDecision,
    PromotionNotPending,
    RevisionOverflow,
}

pub fn create_review_obligation_candidate_v1(
    draft: ReviewObligationCandidateDraftV1,
) -> Result<ReviewObligationCandidateV1, ReviewObligationCandidateTransitionErrorV1> {
    validate_draft(&draft).map_err(invalid_record)?;
    let review_id = derive_review_obligation_candidate_id_v1(
        &draft.logical_owner_id,
        &draft.candidate_id,
        &draft.candidate_digest,
    )
    .map_err(invalid_record)?;
    let review = ReviewObligationCandidateV1 {
        review_id,
        logical_owner_id: draft.logical_owner_id,
        candidate_id: draft.candidate_id,
        candidate_digest: draft.candidate_digest,
        source_evidence_id: draft.source_evidence_id,
        source_evidence_revision: draft.source_evidence_revision,
        statement: draft.statement,
        due_at: draft.due_at,
        condition: draft.condition,
        obligated_party_id: draft.obligated_party_id,
        beneficiary_party_id: draft.beneficiary_party_id,
        evidence_links: draft.evidence_links,
        state: ReviewObligationCandidateStateV1::Pending,
        promotion_status: ReviewObligationCandidatePromotionStatusV1::NotRequested,
        review_revision: 1,
        decided_by_owner_device_id: None,
        decided_at: None,
        promoted_obligation_id: None,
        updated_at: draft.submitted_at,
    };
    validate_review_obligation_candidate_v1(&review).map_err(invalid_record)?;
    Ok(review)
}

pub fn decide_review_obligation_candidate_v1(
    current: &ReviewObligationCandidateV1,
    expected_review_revision: u64,
    decision: ReviewObligationCandidateDecisionV1,
    owner_device_id: [u8; STABLE_ID_BYTES_V1],
    decided_at: ReviewObligationCandidateTimestampV1,
) -> Result<ReviewObligationCandidateV1, ReviewObligationCandidateTransitionErrorV1> {
    validate_review_obligation_candidate_v1(current).map_err(invalid_record)?;
    if expected_review_revision != current.review_revision {
        return Err(ReviewObligationCandidateTransitionErrorV1::RevisionConflict);
    }
    if current.state != ReviewObligationCandidateStateV1::Pending {
        return Err(ReviewObligationCandidateTransitionErrorV1::TerminalDecision);
    }
    if !nonzero(&owner_device_id) {
        return Err(ReviewObligationCandidateTransitionErrorV1::InvalidActor);
    }
    if !valid_timestamp(decided_at) || decided_at.unix_seconds < current.updated_at.unix_seconds {
        return Err(ReviewObligationCandidateTransitionErrorV1::InvalidTimestamp);
    }
    let mut next = current.clone();
    next.review_revision = next
        .review_revision
        .checked_add(1)
        .ok_or(ReviewObligationCandidateTransitionErrorV1::RevisionOverflow)?;
    next.decided_by_owner_device_id = Some(owner_device_id);
    next.decided_at = Some(decided_at);
    next.updated_at = decided_at;
    match decision {
        ReviewObligationCandidateDecisionV1::Approve => {
            next.state = ReviewObligationCandidateStateV1::Approved;
            next.promotion_status = ReviewObligationCandidatePromotionStatusV1::Pending;
        }
        ReviewObligationCandidateDecisionV1::Reject => {
            next.state = ReviewObligationCandidateStateV1::Rejected;
            next.promotion_status = ReviewObligationCandidatePromotionStatusV1::NotRequested;
        }
    }
    validate_review_obligation_candidate_v1(&next).map_err(invalid_record)?;
    Ok(next)
}

pub fn record_review_obligation_candidate_promotion_v1(
    current: &ReviewObligationCandidateV1,
    expected_review_revision: u64,
    result: ReviewObligationCandidatePromotionResultV1,
    recorded_at: ReviewObligationCandidateTimestampV1,
) -> Result<ReviewObligationCandidateV1, ReviewObligationCandidateTransitionErrorV1> {
    validate_review_obligation_candidate_v1(current).map_err(invalid_record)?;
    if expected_review_revision != current.review_revision {
        return Err(ReviewObligationCandidateTransitionErrorV1::RevisionConflict);
    }
    if current.state != ReviewObligationCandidateStateV1::Approved
        || current.promotion_status != ReviewObligationCandidatePromotionStatusV1::Pending
    {
        return Err(ReviewObligationCandidateTransitionErrorV1::PromotionNotPending);
    }
    if !valid_timestamp(recorded_at) || recorded_at.unix_seconds < current.updated_at.unix_seconds {
        return Err(ReviewObligationCandidateTransitionErrorV1::InvalidTimestamp);
    }
    let mut next = current.clone();
    next.review_revision = next
        .review_revision
        .checked_add(1)
        .ok_or(ReviewObligationCandidateTransitionErrorV1::RevisionOverflow)?;
    next.updated_at = recorded_at;
    match result {
        ReviewObligationCandidatePromotionResultV1::Succeeded { obligation_id } => {
            if !nonzero(&obligation_id) {
                return Err(ReviewObligationCandidateTransitionErrorV1::InvalidRecord);
            }
            next.promotion_status = ReviewObligationCandidatePromotionStatusV1::Succeeded;
            next.promoted_obligation_id = Some(obligation_id);
        }
        ReviewObligationCandidatePromotionResultV1::Failed => {
            next.promotion_status = ReviewObligationCandidatePromotionStatusV1::Failed;
        }
    }
    validate_review_obligation_candidate_v1(&next).map_err(invalid_record)?;
    Ok(next)
}

fn invalid_record(
    _: ReviewObligationCandidateValidationErrorV1,
) -> ReviewObligationCandidateTransitionErrorV1 {
    ReviewObligationCandidateTransitionErrorV1::InvalidRecord
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp(seconds: i64) -> ReviewObligationCandidateTimestampV1 {
        ReviewObligationCandidateTimestampV1 {
            unix_seconds: seconds,
            nanos: 7,
        }
    }

    fn pending() -> ReviewObligationCandidateV1 {
        create_review_obligation_candidate_v1(ReviewObligationCandidateDraftV1 {
            logical_owner_id: "owner-1".to_owned(),
            candidate_id: [1; 16],
            candidate_digest: [2; 32],
            source_evidence_id: [3; 16],
            source_evidence_revision: 4,
            statement: "Подготовить ответ".to_owned(),
            due_at: Some(timestamp(1_800_000_100)),
            condition: None,
            obligated_party_id: [5; 16],
            beneficiary_party_id: Some([6; 16]),
            evidence_links: vec![crate::ReviewObligationEvidenceLinkV1 {
                evidence_link_id: [7; 16],
                evidence_owner_id: "communications".to_owned(),
                evidence_record_id: [8; 16],
                evidence_revision: 1,
                evidence_digest: [9; 32],
            }],
            submitted_at: timestamp(1_800_000_000),
        })
        .expect("pending review")
    }

    #[test]
    fn submission_creates_deterministic_pending_review() {
        let first = pending();
        let second = pending();
        assert_eq!(first, second);
        assert_eq!(first.review_revision, 1);
        assert_eq!(first.state, ReviewObligationCandidateStateV1::Pending);
        assert_eq!(
            first.promotion_status,
            ReviewObligationCandidatePromotionStatusV1::NotRequested
        );
    }

    #[test]
    fn approval_is_terminal_and_starts_separate_promotion() {
        let approved = decide_review_obligation_candidate_v1(
            &pending(),
            1,
            ReviewObligationCandidateDecisionV1::Approve,
            [4; 16],
            timestamp(1_800_000_001),
        )
        .expect("approve");
        assert_eq!(approved.state, ReviewObligationCandidateStateV1::Approved);
        assert_eq!(
            approved.promotion_status,
            ReviewObligationCandidatePromotionStatusV1::Pending
        );
        assert_eq!(
            decide_review_obligation_candidate_v1(
                &approved,
                2,
                ReviewObligationCandidateDecisionV1::Reject,
                [4; 16],
                timestamp(1_800_000_002),
            ),
            Err(ReviewObligationCandidateTransitionErrorV1::TerminalDecision)
        );
    }

    #[test]
    fn rejection_never_requests_promotion() {
        let rejected = decide_review_obligation_candidate_v1(
            &pending(),
            1,
            ReviewObligationCandidateDecisionV1::Reject,
            [4; 16],
            timestamp(1_800_000_001),
        )
        .expect("reject");
        assert_eq!(rejected.state, ReviewObligationCandidateStateV1::Rejected);
        assert_eq!(
            rejected.promotion_status,
            ReviewObligationCandidatePromotionStatusV1::NotRequested
        );
        assert_eq!(
            record_review_obligation_candidate_promotion_v1(
                &rejected,
                2,
                ReviewObligationCandidatePromotionResultV1::Failed,
                timestamp(1_800_000_002),
            ),
            Err(ReviewObligationCandidateTransitionErrorV1::PromotionNotPending)
        );
    }

    #[test]
    fn stale_revision_and_missing_human_actor_are_rejected() {
        assert_eq!(
            decide_review_obligation_candidate_v1(
                &pending(),
                2,
                ReviewObligationCandidateDecisionV1::Approve,
                [4; 16],
                timestamp(1_800_000_001),
            ),
            Err(ReviewObligationCandidateTransitionErrorV1::RevisionConflict)
        );
        assert_eq!(
            decide_review_obligation_candidate_v1(
                &pending(),
                1,
                ReviewObligationCandidateDecisionV1::Approve,
                [0; 16],
                timestamp(1_800_000_001),
            ),
            Err(ReviewObligationCandidateTransitionErrorV1::InvalidActor)
        );
    }

    #[test]
    fn terminal_obligation_result_is_distinct_from_approval() {
        let approved = decide_review_obligation_candidate_v1(
            &pending(),
            1,
            ReviewObligationCandidateDecisionV1::Approve,
            [4; 16],
            timestamp(1_800_000_001),
        )
        .expect("approve");
        assert_eq!(approved.promoted_obligation_id, None);
        let succeeded = record_review_obligation_candidate_promotion_v1(
            &approved,
            2,
            ReviewObligationCandidatePromotionResultV1::Succeeded {
                obligation_id: [5; 16],
            },
            timestamp(1_800_000_002),
        )
        .expect("promotion result");
        assert_eq!(
            succeeded.promotion_status,
            ReviewObligationCandidatePromotionStatusV1::Succeeded
        );
        assert_eq!(succeeded.promoted_obligation_id, Some([5; 16]));
    }
}
