use sha2::{Digest, Sha256};

use crate::{
    DIGEST_BYTES_V1, MAX_HINT_CHARS_V1, MAX_LOGICAL_OWNER_ID_BYTES_V1, MAX_TITLE_CHARS_V1,
    STABLE_ID_BYTES_V1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewTaskCandidateStateV1 {
    Pending,
    Approved,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewTaskCandidatePromotionStatusV1 {
    NotRequested,
    Pending,
    Succeeded,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewTaskCandidateDecisionV1 {
    Approve,
    Reject,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewTaskCandidatePromotionResultV1 {
    Succeeded { task_id: [u8; STABLE_ID_BYTES_V1] },
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewTaskCandidateTimestampV1 {
    pub unix_seconds: i64,
    pub nanos: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewTaskCandidateDraftV1 {
    pub logical_owner_id: String,
    pub candidate_id: [u8; STABLE_ID_BYTES_V1],
    pub candidate_digest: [u8; DIGEST_BYTES_V1],
    pub source_evidence_id: [u8; STABLE_ID_BYTES_V1],
    pub source_evidence_revision: u64,
    pub title: String,
    pub due_text_hint: Option<String>,
    pub assignee_label_hint: Option<String>,
    pub submitted_at: ReviewTaskCandidateTimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewTaskCandidateV1 {
    pub review_id: [u8; STABLE_ID_BYTES_V1],
    pub logical_owner_id: String,
    pub candidate_id: [u8; STABLE_ID_BYTES_V1],
    pub candidate_digest: [u8; DIGEST_BYTES_V1],
    pub source_evidence_id: [u8; STABLE_ID_BYTES_V1],
    pub source_evidence_revision: u64,
    pub title: String,
    pub due_text_hint: Option<String>,
    pub assignee_label_hint: Option<String>,
    pub state: ReviewTaskCandidateStateV1,
    pub promotion_status: ReviewTaskCandidatePromotionStatusV1,
    pub review_revision: u64,
    pub decided_by_owner_device_id: Option<[u8; STABLE_ID_BYTES_V1]>,
    pub decided_at: Option<ReviewTaskCandidateTimestampV1>,
    pub promoted_task_id: Option<[u8; STABLE_ID_BYTES_V1]>,
    pub updated_at: ReviewTaskCandidateTimestampV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewTaskCandidateValidationErrorV1 {
    InvalidOwner,
    InvalidReviewId,
    InvalidCandidateId,
    InvalidCandidateDigest,
    InvalidSourceEvidence,
    InvalidSourceRevision,
    InvalidTitle,
    InvalidDueTextHint,
    InvalidAssigneeLabelHint,
    InvalidTimestamp,
    InvalidRevision,
    InvalidDecisionEvidence,
    InvalidPromotionState,
}

pub fn derive_review_task_candidate_id_v1(
    logical_owner_id: &str,
    candidate_id: &[u8; STABLE_ID_BYTES_V1],
    candidate_digest: &[u8; DIGEST_BYTES_V1],
) -> Result<[u8; STABLE_ID_BYTES_V1], ReviewTaskCandidateValidationErrorV1> {
    if !valid_owner(logical_owner_id) {
        return Err(ReviewTaskCandidateValidationErrorV1::InvalidOwner);
    }
    if !nonzero(candidate_id) {
        return Err(ReviewTaskCandidateValidationErrorV1::InvalidCandidateId);
    }
    if !nonzero(candidate_digest) {
        return Err(ReviewTaskCandidateValidationErrorV1::InvalidCandidateDigest);
    }
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.review.task-candidate.v1");
    hasher.update([0]);
    hasher.update(logical_owner_id.as_bytes());
    hasher.update([0]);
    hasher.update(candidate_id);
    hasher.update(candidate_digest);
    hasher.finalize()[..STABLE_ID_BYTES_V1]
        .try_into()
        .map_err(|_| ReviewTaskCandidateValidationErrorV1::InvalidReviewId)
}

pub fn validate_review_task_candidate_v1(
    review: &ReviewTaskCandidateV1,
) -> Result<(), ReviewTaskCandidateValidationErrorV1> {
    let expected = derive_review_task_candidate_id_v1(
        &review.logical_owner_id,
        &review.candidate_id,
        &review.candidate_digest,
    )?;
    if review.review_id != expected || !nonzero(&review.review_id) {
        return Err(ReviewTaskCandidateValidationErrorV1::InvalidReviewId);
    }
    validate_source_and_content(
        &review.source_evidence_id,
        review.source_evidence_revision,
        &review.title,
        review.due_text_hint.as_deref(),
        review.assignee_label_hint.as_deref(),
        review.updated_at,
    )?;
    if review.review_revision == 0 {
        return Err(ReviewTaskCandidateValidationErrorV1::InvalidRevision);
    }
    match review.state {
        ReviewTaskCandidateStateV1::Pending => {
            if review.promotion_status != ReviewTaskCandidatePromotionStatusV1::NotRequested
                || review.decided_by_owner_device_id.is_some()
                || review.decided_at.is_some()
                || review.promoted_task_id.is_some()
            {
                return Err(ReviewTaskCandidateValidationErrorV1::InvalidDecisionEvidence);
            }
        }
        ReviewTaskCandidateStateV1::Rejected => {
            if review.promotion_status != ReviewTaskCandidatePromotionStatusV1::NotRequested
                || !valid_decision_evidence(review)
                || review.promoted_task_id.is_some()
            {
                return Err(ReviewTaskCandidateValidationErrorV1::InvalidDecisionEvidence);
            }
        }
        ReviewTaskCandidateStateV1::Approved => {
            if review.promotion_status == ReviewTaskCandidatePromotionStatusV1::NotRequested
                || !valid_decision_evidence(review)
            {
                return Err(ReviewTaskCandidateValidationErrorV1::InvalidDecisionEvidence);
            }
            let has_task = review.promoted_task_id.as_ref().is_some_and(nonzero);
            if has_task
                != (review.promotion_status == ReviewTaskCandidatePromotionStatusV1::Succeeded)
            {
                return Err(ReviewTaskCandidateValidationErrorV1::InvalidPromotionState);
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_draft(
    draft: &ReviewTaskCandidateDraftV1,
) -> Result<(), ReviewTaskCandidateValidationErrorV1> {
    derive_review_task_candidate_id_v1(
        &draft.logical_owner_id,
        &draft.candidate_id,
        &draft.candidate_digest,
    )?;
    validate_source_and_content(
        &draft.source_evidence_id,
        draft.source_evidence_revision,
        &draft.title,
        draft.due_text_hint.as_deref(),
        draft.assignee_label_hint.as_deref(),
        draft.submitted_at,
    )
}

pub(crate) const fn valid_timestamp(timestamp: ReviewTaskCandidateTimestampV1) -> bool {
    timestamp.unix_seconds > 0 && timestamp.nanos >= 0 && timestamp.nanos < 1_000_000_000
}

pub(crate) fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn validate_source_and_content(
    source_evidence_id: &[u8; STABLE_ID_BYTES_V1],
    source_evidence_revision: u64,
    title: &str,
    due_text_hint: Option<&str>,
    assignee_label_hint: Option<&str>,
    timestamp: ReviewTaskCandidateTimestampV1,
) -> Result<(), ReviewTaskCandidateValidationErrorV1> {
    if !nonzero(source_evidence_id) {
        return Err(ReviewTaskCandidateValidationErrorV1::InvalidSourceEvidence);
    }
    if source_evidence_revision == 0 {
        return Err(ReviewTaskCandidateValidationErrorV1::InvalidSourceRevision);
    }
    if !valid_text(title, MAX_TITLE_CHARS_V1) {
        return Err(ReviewTaskCandidateValidationErrorV1::InvalidTitle);
    }
    if due_text_hint.is_some_and(|value| !valid_text(value, MAX_HINT_CHARS_V1)) {
        return Err(ReviewTaskCandidateValidationErrorV1::InvalidDueTextHint);
    }
    if assignee_label_hint.is_some_and(|value| !valid_text(value, MAX_HINT_CHARS_V1)) {
        return Err(ReviewTaskCandidateValidationErrorV1::InvalidAssigneeLabelHint);
    }
    if !valid_timestamp(timestamp) {
        return Err(ReviewTaskCandidateValidationErrorV1::InvalidTimestamp);
    }
    Ok(())
}

fn valid_decision_evidence(review: &ReviewTaskCandidateV1) -> bool {
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
