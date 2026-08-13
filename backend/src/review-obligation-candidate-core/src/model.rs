use sha2::{Digest, Sha256};

use crate::{
    DIGEST_BYTES_V1, MAX_HINT_CHARS_V1, MAX_LOGICAL_OWNER_ID_BYTES_V1, MAX_STATEMENT_CHARS_V1,
    STABLE_ID_BYTES_V1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewObligationCandidateStateV1 {
    Pending,
    Approved,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewObligationCandidatePromotionStatusV1 {
    NotRequested,
    Pending,
    Succeeded,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewObligationCandidateDecisionV1 {
    Approve,
    Reject,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewObligationCandidatePromotionResultV1 {
    Succeeded {
        obligation_id: [u8; STABLE_ID_BYTES_V1],
    },
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewObligationCandidateTimestampV1 {
    pub unix_seconds: i64,
    pub nanos: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewObligationEvidenceLinkV1 {
    pub evidence_link_id: [u8; STABLE_ID_BYTES_V1],
    pub evidence_owner_id: String,
    pub evidence_record_id: [u8; STABLE_ID_BYTES_V1],
    pub evidence_revision: u64,
    pub evidence_digest: [u8; DIGEST_BYTES_V1],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewObligationCandidateDraftV1 {
    pub logical_owner_id: String,
    pub candidate_id: [u8; STABLE_ID_BYTES_V1],
    pub candidate_digest: [u8; DIGEST_BYTES_V1],
    pub source_evidence_id: [u8; STABLE_ID_BYTES_V1],
    pub source_evidence_revision: u64,
    pub statement: String,
    pub due_at: Option<ReviewObligationCandidateTimestampV1>,
    pub condition: Option<String>,
    pub obligated_party_id: [u8; STABLE_ID_BYTES_V1],
    pub beneficiary_party_id: Option<[u8; STABLE_ID_BYTES_V1]>,
    pub evidence_links: Vec<ReviewObligationEvidenceLinkV1>,
    pub submitted_at: ReviewObligationCandidateTimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewObligationCandidateV1 {
    pub review_id: [u8; STABLE_ID_BYTES_V1],
    pub logical_owner_id: String,
    pub candidate_id: [u8; STABLE_ID_BYTES_V1],
    pub candidate_digest: [u8; DIGEST_BYTES_V1],
    pub source_evidence_id: [u8; STABLE_ID_BYTES_V1],
    pub source_evidence_revision: u64,
    pub statement: String,
    pub due_at: Option<ReviewObligationCandidateTimestampV1>,
    pub condition: Option<String>,
    pub obligated_party_id: [u8; STABLE_ID_BYTES_V1],
    pub beneficiary_party_id: Option<[u8; STABLE_ID_BYTES_V1]>,
    pub evidence_links: Vec<ReviewObligationEvidenceLinkV1>,
    pub state: ReviewObligationCandidateStateV1,
    pub promotion_status: ReviewObligationCandidatePromotionStatusV1,
    pub review_revision: u64,
    pub decided_by_owner_device_id: Option<[u8; STABLE_ID_BYTES_V1]>,
    pub decided_at: Option<ReviewObligationCandidateTimestampV1>,
    pub promoted_obligation_id: Option<[u8; STABLE_ID_BYTES_V1]>,
    pub updated_at: ReviewObligationCandidateTimestampV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewObligationCandidateValidationErrorV1 {
    InvalidOwner,
    InvalidReviewId,
    InvalidCandidateId,
    InvalidCandidateDigest,
    InvalidSourceEvidence,
    InvalidSourceRevision,
    InvalidStatement,
    InvalidParty,
    InvalidEvidence,
    InvalidCondition,
    InvalidTimestamp,
    InvalidRevision,
    InvalidDecisionEvidence,
    InvalidPromotionState,
}

pub fn derive_review_obligation_candidate_id_v1(
    logical_owner_id: &str,
    candidate_id: &[u8; STABLE_ID_BYTES_V1],
    candidate_digest: &[u8; DIGEST_BYTES_V1],
) -> Result<[u8; STABLE_ID_BYTES_V1], ReviewObligationCandidateValidationErrorV1> {
    if !valid_owner(logical_owner_id) {
        return Err(ReviewObligationCandidateValidationErrorV1::InvalidOwner);
    }
    if !nonzero(candidate_id) {
        return Err(ReviewObligationCandidateValidationErrorV1::InvalidCandidateId);
    }
    if !nonzero(candidate_digest) {
        return Err(ReviewObligationCandidateValidationErrorV1::InvalidCandidateDigest);
    }
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.review.obligation-candidate.v1");
    hasher.update([0]);
    hasher.update(logical_owner_id.as_bytes());
    hasher.update([0]);
    hasher.update(candidate_id);
    hasher.update(candidate_digest);
    hasher.finalize()[..STABLE_ID_BYTES_V1]
        .try_into()
        .map_err(|_| ReviewObligationCandidateValidationErrorV1::InvalidReviewId)
}

pub fn validate_review_obligation_candidate_v1(
    review: &ReviewObligationCandidateV1,
) -> Result<(), ReviewObligationCandidateValidationErrorV1> {
    let expected = derive_review_obligation_candidate_id_v1(
        &review.logical_owner_id,
        &review.candidate_id,
        &review.candidate_digest,
    )?;
    if review.review_id != expected || !nonzero(&review.review_id) {
        return Err(ReviewObligationCandidateValidationErrorV1::InvalidReviewId);
    }
    validate_source_and_content(
        &review.source_evidence_id,
        review.source_evidence_revision,
        &review.statement,
        review.due_at,
        review.condition.as_deref(),
        &review.obligated_party_id,
        review.beneficiary_party_id.as_ref(),
        &review.evidence_links,
        review.updated_at,
    )?;
    if review.review_revision == 0 {
        return Err(ReviewObligationCandidateValidationErrorV1::InvalidRevision);
    }
    match review.state {
        ReviewObligationCandidateStateV1::Pending => {
            if review.promotion_status != ReviewObligationCandidatePromotionStatusV1::NotRequested
                || review.decided_by_owner_device_id.is_some()
                || review.decided_at.is_some()
                || review.promoted_obligation_id.is_some()
            {
                return Err(ReviewObligationCandidateValidationErrorV1::InvalidDecisionEvidence);
            }
        }
        ReviewObligationCandidateStateV1::Rejected => {
            if review.promotion_status != ReviewObligationCandidatePromotionStatusV1::NotRequested
                || !valid_decision_evidence(review)
                || review.promoted_obligation_id.is_some()
            {
                return Err(ReviewObligationCandidateValidationErrorV1::InvalidDecisionEvidence);
            }
        }
        ReviewObligationCandidateStateV1::Approved => {
            if review.promotion_status == ReviewObligationCandidatePromotionStatusV1::NotRequested
                || !valid_decision_evidence(review)
            {
                return Err(ReviewObligationCandidateValidationErrorV1::InvalidDecisionEvidence);
            }
            let has_obligation = review.promoted_obligation_id.as_ref().is_some_and(nonzero);
            if has_obligation
                != (review.promotion_status
                    == ReviewObligationCandidatePromotionStatusV1::Succeeded)
            {
                return Err(ReviewObligationCandidateValidationErrorV1::InvalidPromotionState);
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_draft(
    draft: &ReviewObligationCandidateDraftV1,
) -> Result<(), ReviewObligationCandidateValidationErrorV1> {
    derive_review_obligation_candidate_id_v1(
        &draft.logical_owner_id,
        &draft.candidate_id,
        &draft.candidate_digest,
    )?;
    validate_source_and_content(
        &draft.source_evidence_id,
        draft.source_evidence_revision,
        &draft.statement,
        draft.due_at,
        draft.condition.as_deref(),
        &draft.obligated_party_id,
        draft.beneficiary_party_id.as_ref(),
        &draft.evidence_links,
        draft.submitted_at,
    )
}

pub(crate) const fn valid_timestamp(timestamp: ReviewObligationCandidateTimestampV1) -> bool {
    timestamp.unix_seconds > 0 && timestamp.nanos >= 0 && timestamp.nanos < 1_000_000_000
}

pub(crate) fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

#[allow(clippy::too_many_arguments)]
fn validate_source_and_content(
    source_evidence_id: &[u8; STABLE_ID_BYTES_V1],
    source_evidence_revision: u64,
    statement: &str,
    due_at: Option<ReviewObligationCandidateTimestampV1>,
    condition: Option<&str>,
    obligated_party_id: &[u8; STABLE_ID_BYTES_V1],
    beneficiary_party_id: Option<&[u8; STABLE_ID_BYTES_V1]>,
    evidence_links: &[ReviewObligationEvidenceLinkV1],
    timestamp: ReviewObligationCandidateTimestampV1,
) -> Result<(), ReviewObligationCandidateValidationErrorV1> {
    if !nonzero(source_evidence_id) {
        return Err(ReviewObligationCandidateValidationErrorV1::InvalidSourceEvidence);
    }
    if source_evidence_revision == 0 {
        return Err(ReviewObligationCandidateValidationErrorV1::InvalidSourceRevision);
    }
    if !valid_text(statement, MAX_STATEMENT_CHARS_V1) {
        return Err(ReviewObligationCandidateValidationErrorV1::InvalidStatement);
    }
    if due_at.is_some_and(|value| !valid_timestamp(value)) {
        return Err(ReviewObligationCandidateValidationErrorV1::InvalidTimestamp);
    }
    if condition.is_some_and(|value| !valid_text(value, MAX_HINT_CHARS_V1)) {
        return Err(ReviewObligationCandidateValidationErrorV1::InvalidCondition);
    }
    if !nonzero(obligated_party_id) || beneficiary_party_id.is_some_and(|value| !nonzero(value)) {
        return Err(ReviewObligationCandidateValidationErrorV1::InvalidParty);
    }
    if evidence_links.iter().any(|value| {
        !nonzero(&value.evidence_link_id)
            || !valid_owner(&value.evidence_owner_id)
            || !nonzero(&value.evidence_record_id)
            || value.evidence_revision == 0
            || !nonzero(&value.evidence_digest)
    }) || evidence_links
        .windows(2)
        .any(|pair| pair[0].evidence_link_id >= pair[1].evidence_link_id)
    {
        return Err(ReviewObligationCandidateValidationErrorV1::InvalidEvidence);
    }
    if !valid_timestamp(timestamp) {
        return Err(ReviewObligationCandidateValidationErrorV1::InvalidTimestamp);
    }
    Ok(())
}

fn valid_decision_evidence(review: &ReviewObligationCandidateV1) -> bool {
    review
        .decided_by_owner_device_id
        .as_ref()
        .is_some_and(nonzero)
        && review.decided_at.is_some_and(valid_timestamp)
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_LOGICAL_OWNER_ID_BYTES_V1
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn valid_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(|character| character.is_control())
}
