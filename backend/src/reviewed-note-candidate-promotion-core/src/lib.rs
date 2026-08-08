#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-reviewed-note-candidate-promotion-core";
pub const REVIEWED_NOTE_CANDIDATE_PROMOTION_OWNER_V1: &str = "reviewed_note_candidate_promotion";
pub const REVIEWED_NOTE_CANDIDATE_PROMOTION_MODULE_ID_V1: &str =
    "makosh-reviewed-note-candidate-promotion-runtime";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewedNoteCandidatePromotionIdentityErrorV1 {
    InvalidApprovalMessageId,
    InvalidKnowledgeResultMessageId,
    InvalidReviewId,
    InvalidCandidateId,
    InvalidDecisionRevision,
}

pub fn derive_reviewed_note_candidate_command_id_v1(
    approval_message_id: [u8; 16],
    review_id: [u8; 16],
    candidate_id: [u8; 16],
    decision_revision: u64,
) -> Result<[u8; 16], ReviewedNoteCandidatePromotionIdentityErrorV1> {
    if !nonzero(&approval_message_id) {
        return Err(ReviewedNoteCandidatePromotionIdentityErrorV1::InvalidApprovalMessageId);
    }
    if !nonzero(&review_id) {
        return Err(ReviewedNoteCandidatePromotionIdentityErrorV1::InvalidReviewId);
    }
    if !nonzero(&candidate_id) {
        return Err(ReviewedNoteCandidatePromotionIdentityErrorV1::InvalidCandidateId);
    }
    if decision_revision == 0 {
        return Err(ReviewedNoteCandidatePromotionIdentityErrorV1::InvalidDecisionRevision);
    }
    Ok(digest(
        b"makosh.reviewed-note-candidate-promotion.command.v1",
        &[
            approval_message_id.as_slice(),
            review_id.as_slice(),
            candidate_id.as_slice(),
            &decision_revision.to_be_bytes(),
        ],
    ))
}

pub fn derive_reviewed_note_candidate_result_id_v1(
    knowledge_result_message_id: [u8; 16],
    command_id: [u8; 16],
    review_id: [u8; 16],
) -> Result<[u8; 16], ReviewedNoteCandidatePromotionIdentityErrorV1> {
    if !nonzero(&knowledge_result_message_id) {
        return Err(ReviewedNoteCandidatePromotionIdentityErrorV1::InvalidKnowledgeResultMessageId);
    }
    if !nonzero(&command_id) {
        return Err(ReviewedNoteCandidatePromotionIdentityErrorV1::InvalidApprovalMessageId);
    }
    if !nonzero(&review_id) {
        return Err(ReviewedNoteCandidatePromotionIdentityErrorV1::InvalidReviewId);
    }
    Ok(digest(
        b"makosh.reviewed-note-candidate-promotion.result.v1",
        &[
            knowledge_result_message_id.as_slice(),
            command_id.as_slice(),
            review_id.as_slice(),
        ],
    ))
}

fn digest(label: &[u8], fields: &[&[u8]]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(label);
    for field in fields {
        hash.update([0]);
        hash.update(field);
    }
    hash.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_identity_is_stable_and_binds_decision_revision() {
        let first = derive_reviewed_note_candidate_command_id_v1([1; 16], [2; 16], [3; 16], 4)
            .expect("command ID");
        assert_eq!(
            first,
            derive_reviewed_note_candidate_command_id_v1([1; 16], [2; 16], [3; 16], 4)
                .expect("replayed command ID")
        );
        assert_ne!(
            first,
            derive_reviewed_note_candidate_command_id_v1([1; 16], [2; 16], [3; 16], 5)
                .expect("next decision command ID")
        );
    }

    #[test]
    fn result_identity_is_knowledge_result_and_review_bound() {
        let first = derive_reviewed_note_candidate_result_id_v1([4; 16], [5; 16], [6; 16])
            .expect("result ID");
        assert_ne!(
            first,
            derive_reviewed_note_candidate_result_id_v1([7; 16], [5; 16], [6; 16])
                .expect("different result ID")
        );
    }
}
