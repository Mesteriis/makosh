use makosh_review_note_candidate_core::{
    ReviewNoteCandidateDecisionV1, ReviewNoteCandidateDraftV1,
    ReviewNoteCandidatePromotionResultV1, ReviewNoteCandidateStateV1,
    ReviewNoteCandidateTimestampV1, ReviewNoteCandidateV1,
};
use sha2::{Digest, Sha256};

pub const REVIEW_NOTE_CANDIDATE_RECOVERY_LIMIT_V1: u16 = 128;
pub const REVIEW_NOTE_CANDIDATE_REALTIME_LIMIT_V1: u16 = 1_024;
pub const REVIEW_NOTE_CANDIDATE_OUTBOX_LIMIT_V1: u16 = 128;
pub const REVIEW_NOTE_CANDIDATE_MAX_EVENT_BYTES_V1: usize = 64 * 1024;
pub const REVIEW_NOTE_CANDIDATE_MAX_BLOB_BYTES_V1: u64 = 16 * 1024;
pub const REVIEW_NOTE_CANDIDATE_MAX_CUSTODY_PROOF_BYTES_V1: usize = 2_048;
pub const REVIEW_NOTE_CANDIDATE_MAX_PAGE_SIZE_V1: u16 = 200;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ListReviewNoteCandidatesV1 {
    pub after_review_id: Option<[u8; 16]>,
    pub state: Option<ReviewNoteCandidateStateV1>,
    pub limit: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewNoteCandidatePageV1 {
    pub reviews: Vec<ReviewNoteCandidateV1>,
    pub next_after_review_id: Option<[u8; 16]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewNoteCandidateBlobReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewNoteCandidateBlobCleanupV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewNoteCandidateOutboxRecordV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReserveReviewNoteCandidateSubmissionV1 {
    pub logical_owner_id: String,
    pub submission_message_id: [u8; 16],
    pub submission_envelope_sha256: [u8; 32],
    pub submission_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub candidate_digest: [u8; 32],
    pub source_evidence_id: [u8; 16],
    pub source_evidence_revision: u64,
    pub candidate_content: ReviewNoteCandidateBlobReceiptV1,
    pub received_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedReviewNoteCandidateSubmissionV1 {
    pub logical_owner_id: String,
    pub submission_message_id: [u8; 16],
    pub submission_envelope_sha256: [u8; 32],
    pub submission_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub candidate_digest: [u8; 32],
    pub source_evidence_id: [u8; 16],
    pub source_evidence_revision: u64,
    pub candidate_content: ReviewNoteCandidateBlobReceiptV1,
    pub materialization: Option<ReviewNoteCandidateBlobCleanupV1>,
    pub cleanup_completed_at_unix_millis: Option<i64>,
    pub completed: bool,
    pub review_id: Option<[u8; 16]>,
    pub rejected: bool,
    pub received_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistReviewNoteCandidateMaterializationV1 {
    pub logical_owner_id: String,
    pub submission_message_id: [u8; 16],
    pub materialization: ReviewNoteCandidateBlobCleanupV1,
    pub materialized_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReserveReviewNoteCandidateSubmissionOutcomeV1 {
    Reserved(PersistedReviewNoteCandidateSubmissionV1),
    Existing(PersistedReviewNoteCandidateSubmissionV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteReviewNoteCandidateSubmissionV1 {
    pub logical_owner_id: String,
    pub submission_message_id: [u8; 16],
    pub draft: ReviewNoteCandidateDraftV1,
    pub submitted_result: ReviewNoteCandidateOutboxRecordV1,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RejectReviewNoteCandidateSubmissionV1 {
    pub logical_owner_id: String,
    pub submission_message_id: [u8; 16],
    pub rejected_result: ReviewNoteCandidateOutboxRecordV1,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecideReviewNoteCandidateOperationV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; 16],
    pub request_sha256: [u8; 32],
    pub review_id: [u8; 16],
    pub expected_review_revision: u64,
    pub decision: ReviewNoteCandidateDecisionV1,
    pub owner_device_id: [u8; 16],
    pub decided_at: ReviewNoteCandidateTimestampV1,
    pub approved_event: Option<ReviewNoteCandidateOutboxRecordV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckReviewNoteCandidateDecisionReplayV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; 16],
    pub request_sha256: [u8; 32],
    pub review_id: [u8; 16],
    pub expected_review_revision: u64,
    pub decision: ReviewNoteCandidateDecisionV1,
    pub owner_device_id: [u8; 16],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReviewNoteCandidateDecisionOutcomeV1 {
    Applied(ReviewNoteCandidateV1),
    Replayed(ReviewNoteCandidateV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistReviewNoteCandidatePromotionResultV1 {
    pub logical_owner_id: String,
    pub result_message_id: [u8; 16],
    pub result_envelope_sha256: [u8; 32],
    pub review_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub expected_review_revision: u64,
    pub result: ReviewNoteCandidatePromotionResultV1,
    pub occurred_at: ReviewNoteCandidateTimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReviewNoteCandidateInboxOutcomeV1 {
    Applied(ReviewNoteCandidateV1),
    Duplicate(ReviewNoteCandidateV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewNoteCandidateRealtimeTransitionV1 {
    pub sequence: u64,
    pub review_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub state: makosh_review_note_candidate_core::ReviewNoteCandidateStateV1,
    pub promotion_status: makosh_review_note_candidate_core::ReviewNoteCandidatePromotionStatusV1,
    pub review_revision: u64,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewNoteCandidatePersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    SubmissionConflict,
    OperationConflict,
    InboxConflict,
    RevisionConflict,
    InvalidTransition,
    NotFound,
}

pub(crate) fn decision_fingerprint(input: &DecideReviewNoteCandidateOperationV1) -> [u8; 32] {
    decision_fingerprint_fields(
        input.review_id,
        input.expected_review_revision,
        input.decision,
        input.owner_device_id,
    )
}

pub(crate) fn decision_replay_fingerprint(
    input: &CheckReviewNoteCandidateDecisionReplayV1,
) -> [u8; 32] {
    decision_fingerprint_fields(
        input.review_id,
        input.expected_review_revision,
        input.decision,
        input.owner_device_id,
    )
}

fn decision_fingerprint_fields(
    review_id: [u8; 16],
    expected_review_revision: u64,
    decision: ReviewNoteCandidateDecisionV1,
    owner_device_id: [u8; 16],
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.review.note-candidate.decision.v1\0");
    hash.update(review_id);
    hash.update(expected_review_revision.to_be_bytes());
    hash.update([match decision {
        ReviewNoteCandidateDecisionV1::Approve => 1,
        ReviewNoteCandidateDecisionV1::Reject => 2,
    }]);
    hash.update(owner_device_id);
    hash.finalize().into()
}

pub(crate) fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

pub(crate) fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn valid_outbox(value: &ReviewNoteCandidateOutboxRecordV1) -> bool {
    nonzero(&value.message_id)
        && nonzero(&value.envelope_sha256)
        && !value.envelope_bytes.is_empty()
        && value.envelope_bytes.len() <= REVIEW_NOTE_CANDIDATE_MAX_EVENT_BYTES_V1
        && Sha256::digest(&value.envelope_bytes).as_slice() == value.envelope_sha256
}

pub(crate) fn valid_blob(value: &ReviewNoteCandidateBlobReceiptV1) -> bool {
    nonzero(&value.reference_id)
        && (1..=REVIEW_NOTE_CANDIDATE_MAX_BLOB_BYTES_V1).contains(&value.declared_bytes)
        && nonzero(&value.sha256)
        && !value.custody_transfer_source_proof.is_empty()
        && value.custody_transfer_source_proof.len()
            <= REVIEW_NOTE_CANDIDATE_MAX_CUSTODY_PROOF_BYTES_V1
}

pub(crate) fn valid_cleanup(value: &ReviewNoteCandidateBlobCleanupV1) -> bool {
    nonzero(&value.reference_id)
        && (1..=REVIEW_NOTE_CANDIDATE_MAX_BLOB_BYTES_V1).contains(&value.declared_bytes)
        && nonzero(&value.sha256)
        && !value.custody_proof.is_empty()
        && value.custody_proof.len() <= REVIEW_NOTE_CANDIDATE_MAX_CUSTODY_PROOF_BYTES_V1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn operation(decision: ReviewNoteCandidateDecisionV1) -> DecideReviewNoteCandidateOperationV1 {
        DecideReviewNoteCandidateOperationV1 {
            logical_owner_id: "owner-1".to_owned(),
            operation_id: [1; 16],
            request_sha256: [2; 32],
            review_id: [3; 16],
            expected_review_revision: 4,
            decision,
            owner_device_id: [5; 16],
            decided_at: ReviewNoteCandidateTimestampV1 {
                unix_seconds: 1_800_000_000,
                nanos: 1,
            },
            approved_event: None,
        }
    }

    #[test]
    fn decision_fingerprint_binds_revision_decision_and_human_actor() {
        let approve = operation(ReviewNoteCandidateDecisionV1::Approve);
        let mut reject = operation(ReviewNoteCandidateDecisionV1::Reject);
        assert_ne!(
            decision_fingerprint(&approve),
            decision_fingerprint(&reject)
        );
        reject.decision = ReviewNoteCandidateDecisionV1::Approve;
        reject.owner_device_id = [6; 16];
        assert_ne!(
            decision_fingerprint(&approve),
            decision_fingerprint(&reject)
        );
    }

    #[test]
    fn exact_outbox_hash_is_required() {
        let bytes = vec![7; 32];
        let record = ReviewNoteCandidateOutboxRecordV1 {
            message_id: [1; 16],
            envelope_sha256: Sha256::digest(&bytes).into(),
            envelope_bytes: bytes,
        };
        assert!(valid_outbox(&record));
        let mut invalid = record;
        invalid.envelope_sha256 = [9; 32];
        assert!(!valid_outbox(&invalid));
    }
}
