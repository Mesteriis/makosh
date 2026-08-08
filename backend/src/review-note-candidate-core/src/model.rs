use sha2::{Digest, Sha256};

use crate::{
    DIGEST_BYTES_V1, MAX_EXCERPT_CHARS_V1, MAX_LOGICAL_OWNER_ID_BYTES_V1, MAX_TITLE_CHARS_V1,
    MAX_TOPIC_HINTS_V1, STABLE_ID_BYTES_V1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewNoteSourceBasisV1 {
    Subject,
    Body,
    Combined,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ReviewNoteTopicHintV1 {
    Financial,
    Legal,
    DecisionStatement,
    DeadlineStatement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewNoteCandidateStateV1 {
    Pending,
    Approved,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewNoteCandidatePromotionStatusV1 {
    NotRequested,
    Pending,
    Succeeded,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewNoteCandidateDecisionV1 {
    Approve,
    Reject,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewNoteCandidatePromotionResultV1 {
    Succeeded { note_id: [u8; STABLE_ID_BYTES_V1] },
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewNoteCandidateTimestampV1 {
    pub unix_seconds: i64,
    pub nanos: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewNoteCandidateDraftV1 {
    pub logical_owner_id: String,
    pub candidate_id: [u8; STABLE_ID_BYTES_V1],
    pub candidate_digest: [u8; DIGEST_BYTES_V1],
    pub source_evidence_id: [u8; STABLE_ID_BYTES_V1],
    pub source_evidence_revision: u64,
    pub title: String,
    pub excerpt: String,
    pub topic_hints: Vec<ReviewNoteTopicHintV1>,
    pub source_basis: ReviewNoteSourceBasisV1,
    pub confidence_basis_points: u32,
    pub submitted_at: ReviewNoteCandidateTimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewNoteCandidateV1 {
    pub review_id: [u8; STABLE_ID_BYTES_V1],
    pub logical_owner_id: String,
    pub candidate_id: [u8; STABLE_ID_BYTES_V1],
    pub candidate_digest: [u8; DIGEST_BYTES_V1],
    pub source_evidence_id: [u8; STABLE_ID_BYTES_V1],
    pub source_evidence_revision: u64,
    pub title: String,
    pub excerpt: String,
    pub topic_hints: Vec<ReviewNoteTopicHintV1>,
    pub source_basis: ReviewNoteSourceBasisV1,
    pub confidence_basis_points: u32,
    pub state: ReviewNoteCandidateStateV1,
    pub promotion_status: ReviewNoteCandidatePromotionStatusV1,
    pub review_revision: u64,
    pub decided_by_owner_device_id: Option<[u8; STABLE_ID_BYTES_V1]>,
    pub decided_at: Option<ReviewNoteCandidateTimestampV1>,
    pub promoted_note_id: Option<[u8; STABLE_ID_BYTES_V1]>,
    pub updated_at: ReviewNoteCandidateTimestampV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewNoteCandidateValidationErrorV1 {
    InvalidOwner,
    InvalidReviewId,
    InvalidCandidateId,
    InvalidCandidateDigest,
    InvalidSourceEvidence,
    InvalidSourceRevision,
    InvalidTitle,
    InvalidExcerpt,
    InvalidTopicHints,
    InvalidConfidence,
    InvalidTimestamp,
    InvalidRevision,
    InvalidDecisionEvidence,
    InvalidPromotionState,
}

pub fn derive_review_note_candidate_id_v1(
    logical_owner_id: &str,
    candidate_id: &[u8; STABLE_ID_BYTES_V1],
    candidate_digest: &[u8; DIGEST_BYTES_V1],
) -> Result<[u8; STABLE_ID_BYTES_V1], ReviewNoteCandidateValidationErrorV1> {
    if !valid_owner(logical_owner_id) {
        return Err(ReviewNoteCandidateValidationErrorV1::InvalidOwner);
    }
    if !nonzero(candidate_id) {
        return Err(ReviewNoteCandidateValidationErrorV1::InvalidCandidateId);
    }
    if !nonzero(candidate_digest) {
        return Err(ReviewNoteCandidateValidationErrorV1::InvalidCandidateDigest);
    }
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.review.note-candidate.v1");
    hasher.update([0]);
    hasher.update(logical_owner_id.as_bytes());
    hasher.update([0]);
    hasher.update(candidate_id);
    hasher.update(candidate_digest);
    hasher.finalize()[..STABLE_ID_BYTES_V1]
        .try_into()
        .map_err(|_| ReviewNoteCandidateValidationErrorV1::InvalidReviewId)
}

pub fn validate_review_note_candidate_v1(
    review: &ReviewNoteCandidateV1,
) -> Result<(), ReviewNoteCandidateValidationErrorV1> {
    let expected = derive_review_note_candidate_id_v1(
        &review.logical_owner_id,
        &review.candidate_id,
        &review.candidate_digest,
    )?;
    if review.review_id != expected || !nonzero(&review.review_id) {
        return Err(ReviewNoteCandidateValidationErrorV1::InvalidReviewId);
    }
    validate_source_and_content(
        &review.source_evidence_id,
        review.source_evidence_revision,
        &review.title,
        &review.excerpt,
        &review.topic_hints,
        review.confidence_basis_points,
        review.updated_at,
    )?;
    if review.review_revision == 0 {
        return Err(ReviewNoteCandidateValidationErrorV1::InvalidRevision);
    }
    match review.state {
        ReviewNoteCandidateStateV1::Pending => {
            if review.promotion_status != ReviewNoteCandidatePromotionStatusV1::NotRequested
                || review.decided_by_owner_device_id.is_some()
                || review.decided_at.is_some()
                || review.promoted_note_id.is_some()
            {
                return Err(ReviewNoteCandidateValidationErrorV1::InvalidDecisionEvidence);
            }
        }
        ReviewNoteCandidateStateV1::Rejected => {
            if review.promotion_status != ReviewNoteCandidatePromotionStatusV1::NotRequested
                || !valid_decision_evidence(review)
                || review.promoted_note_id.is_some()
            {
                return Err(ReviewNoteCandidateValidationErrorV1::InvalidDecisionEvidence);
            }
        }
        ReviewNoteCandidateStateV1::Approved => {
            if review.promotion_status == ReviewNoteCandidatePromotionStatusV1::NotRequested
                || !valid_decision_evidence(review)
            {
                return Err(ReviewNoteCandidateValidationErrorV1::InvalidDecisionEvidence);
            }
            let has_note = review.promoted_note_id.as_ref().is_some_and(nonzero);
            if has_note
                != (review.promotion_status == ReviewNoteCandidatePromotionStatusV1::Succeeded)
            {
                return Err(ReviewNoteCandidateValidationErrorV1::InvalidPromotionState);
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_draft(
    draft: &ReviewNoteCandidateDraftV1,
) -> Result<(), ReviewNoteCandidateValidationErrorV1> {
    derive_review_note_candidate_id_v1(
        &draft.logical_owner_id,
        &draft.candidate_id,
        &draft.candidate_digest,
    )?;
    validate_source_and_content(
        &draft.source_evidence_id,
        draft.source_evidence_revision,
        &draft.title,
        &draft.excerpt,
        &draft.topic_hints,
        draft.confidence_basis_points,
        draft.submitted_at,
    )
}

pub(crate) const fn valid_timestamp(timestamp: ReviewNoteCandidateTimestampV1) -> bool {
    timestamp.unix_seconds > 0 && timestamp.nanos >= 0 && timestamp.nanos < 1_000_000_000
}

pub(crate) fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn validate_source_and_content(
    source_evidence_id: &[u8; STABLE_ID_BYTES_V1],
    source_evidence_revision: u64,
    title: &str,
    excerpt: &str,
    topic_hints: &[ReviewNoteTopicHintV1],
    confidence_basis_points: u32,
    timestamp: ReviewNoteCandidateTimestampV1,
) -> Result<(), ReviewNoteCandidateValidationErrorV1> {
    if !nonzero(source_evidence_id) {
        return Err(ReviewNoteCandidateValidationErrorV1::InvalidSourceEvidence);
    }
    if source_evidence_revision == 0 {
        return Err(ReviewNoteCandidateValidationErrorV1::InvalidSourceRevision);
    }
    if !valid_text(title, MAX_TITLE_CHARS_V1) {
        return Err(ReviewNoteCandidateValidationErrorV1::InvalidTitle);
    }
    if !valid_excerpt(excerpt) {
        return Err(ReviewNoteCandidateValidationErrorV1::InvalidExcerpt);
    }
    if topic_hints.is_empty()
        || topic_hints.len() > MAX_TOPIC_HINTS_V1
        || !topic_hints.windows(2).all(|pair| pair[0] < pair[1])
    {
        return Err(ReviewNoteCandidateValidationErrorV1::InvalidTopicHints);
    }
    if !(1..=10_000).contains(&confidence_basis_points) {
        return Err(ReviewNoteCandidateValidationErrorV1::InvalidConfidence);
    }
    if !valid_timestamp(timestamp) {
        return Err(ReviewNoteCandidateValidationErrorV1::InvalidTimestamp);
    }
    Ok(())
}

fn valid_decision_evidence(review: &ReviewNoteCandidateV1) -> bool {
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

fn valid_excerpt(value: &str) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= MAX_EXCERPT_CHARS_V1
        && !value
            .chars()
            .any(|character| character.is_control() && character != '\n')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> ReviewNoteCandidateDraftV1 {
        ReviewNoteCandidateDraftV1 {
            logical_owner_id: "owner-1".to_owned(),
            candidate_id: [1; 16],
            candidate_digest: [2; 32],
            source_evidence_id: [3; 16],
            source_evidence_revision: 4,
            title: "Contract approved".to_owned(),
            excerpt: "Invoice amount\nPayment by Friday".to_owned(),
            topic_hints: vec![
                ReviewNoteTopicHintV1::Financial,
                ReviewNoteTopicHintV1::DeadlineStatement,
            ],
            source_basis: ReviewNoteSourceBasisV1::Combined,
            confidence_basis_points: 8_300,
            submitted_at: ReviewNoteCandidateTimestampV1 {
                unix_seconds: 1_800_000_000,
                nanos: 7,
            },
        }
    }

    #[test]
    fn extraction_excerpt_newlines_remain_valid_review_content() {
        assert_eq!(validate_draft(&draft()), Ok(()));
    }

    #[test]
    fn topic_hints_are_nonempty_ordered_and_unique() {
        let mut invalid = draft();
        invalid.topic_hints = vec![
            ReviewNoteTopicHintV1::Legal,
            ReviewNoteTopicHintV1::Financial,
        ];
        assert_eq!(
            validate_draft(&invalid),
            Err(ReviewNoteCandidateValidationErrorV1::InvalidTopicHints)
        );

        invalid.topic_hints = vec![ReviewNoteTopicHintV1::Financial; 2];
        assert_eq!(
            validate_draft(&invalid),
            Err(ReviewNoteCandidateValidationErrorV1::InvalidTopicHints)
        );
    }

    #[test]
    fn empty_excerpt_and_out_of_range_confidence_are_rejected() {
        let mut invalid = draft();
        invalid.excerpt.clear();
        assert_eq!(
            validate_draft(&invalid),
            Err(ReviewNoteCandidateValidationErrorV1::InvalidExcerpt)
        );

        invalid = draft();
        invalid.confidence_basis_points = 10_001;
        assert_eq!(
            validate_draft(&invalid),
            Err(ReviewNoteCandidateValidationErrorV1::InvalidConfidence)
        );
    }
}
