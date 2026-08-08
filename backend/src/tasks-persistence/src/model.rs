use makosh_tasks_core::{ReviewedCandidateTaskDraftV1, TaskV1, task_creation_fingerprint_v1};
use sha2::{Digest, Sha256};

pub const TASKS_RECOVERY_LIMIT_V1: u16 = 128;
pub const TASKS_OUTBOX_LIMIT_V1: u16 = 128;
pub const TASKS_MAX_EVENT_BYTES_V1: usize = 64 * 1024;
pub const TASKS_MAX_BLOB_BYTES_V1: u64 = 16 * 1024;
pub const TASKS_MAX_CUSTODY_PROOF_BYTES_V1: usize = 2_048;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TasksBlobReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TasksBlobCleanupV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TasksOutboxRecordV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReserveReviewedCandidateCommandV1 {
    pub logical_owner_id: String,
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub command_id: [u8; 16],
    pub approved_candidate_id: [u8; 16],
    pub candidate_digest: [u8; 32],
    pub source_evidence_id: [u8; 16],
    pub source_evidence_revision: u64,
    pub review_id: [u8; 16],
    pub decision_revision: u64,
    pub decided_by_owner_device_id: [u8; 16],
    pub candidate_content: TasksBlobReceiptV1,
    pub received_at_unix_millis: i64,
}

impl ReserveReviewedCandidateCommandV1 {
    pub fn command_fingerprint(&self) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(b"makosh.tasks.reviewed-candidate.command.v1\0");
        hash.update(self.command_id);
        hash.update(self.approved_candidate_id);
        hash.update(self.candidate_digest);
        hash.update(self.source_evidence_id);
        hash.update(self.source_evidence_revision.to_be_bytes());
        hash.update(self.review_id);
        hash.update(self.decision_revision.to_be_bytes());
        hash.update(self.decided_by_owner_device_id);
        hash.update(self.candidate_content.reference_id);
        hash.update(self.candidate_content.declared_bytes.to_be_bytes());
        hash.update(self.candidate_content.sha256);
        hash.finalize().into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedReviewedCandidateCommandV1 {
    pub logical_owner_id: String,
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub command_id: [u8; 16],
    pub command_fingerprint: [u8; 32],
    pub approved_candidate_id: [u8; 16],
    pub candidate_digest: [u8; 32],
    pub source_evidence_id: [u8; 16],
    pub source_evidence_revision: u64,
    pub review_id: [u8; 16],
    pub decision_revision: u64,
    pub decided_by_owner_device_id: [u8; 16],
    pub candidate_content: TasksBlobReceiptV1,
    pub materialization: Option<TasksBlobCleanupV1>,
    pub cleanup_completed_at_unix_millis: Option<i64>,
    pub completed: bool,
    pub rejected: bool,
    pub task_id: Option<[u8; 16]>,
    pub received_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReserveReviewedCandidateCommandOutcomeV1 {
    Reserved(PersistedReviewedCandidateCommandV1),
    Existing(PersistedReviewedCandidateCommandV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistReviewedCandidateMaterializationV1 {
    pub logical_owner_id: String,
    pub command_message_id: [u8; 16],
    pub materialization: TasksBlobCleanupV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteReviewedCandidateTaskV1 {
    pub logical_owner_id: String,
    pub command_message_id: [u8; 16],
    pub draft: ReviewedCandidateTaskDraftV1,
    pub created_result: TasksOutboxRecordV1,
    pub occurred_at_unix_millis: i64,
}

impl CompleteReviewedCandidateTaskV1 {
    pub fn creation_fingerprint(&self) -> Result<[u8; 32], TasksPersistenceErrorV1> {
        task_creation_fingerprint_v1(&self.draft).map_err(|_| TasksPersistenceErrorV1::InvalidInput)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RejectReviewedCandidateTaskV1 {
    pub logical_owner_id: String,
    pub command_message_id: [u8; 16],
    pub rejected_result: TasksOutboxRecordV1,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TasksPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    CommandConflict,
    InboxConflict,
    TaskConflict,
    NotFound,
}

pub(crate) fn valid_reservation(value: &ReserveReviewedCandidateCommandV1) -> bool {
    valid_identity(&value.logical_owner_id)
        && nonzero(&value.command_message_id)
        && nonzero(&value.command_envelope_sha256)
        && nonzero(&value.command_id)
        && nonzero(&value.approved_candidate_id)
        && nonzero(&value.candidate_digest)
        && nonzero(&value.source_evidence_id)
        && value.source_evidence_revision > 0
        && nonzero(&value.review_id)
        && value.decision_revision > 0
        && nonzero(&value.decided_by_owner_device_id)
        && valid_blob(&value.candidate_content)
        && value.received_at_unix_millis > 0
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

pub(crate) fn valid_outbox(value: &TasksOutboxRecordV1) -> bool {
    nonzero(&value.message_id)
        && nonzero(&value.envelope_sha256)
        && !value.envelope_bytes.is_empty()
        && value.envelope_bytes.len() <= TASKS_MAX_EVENT_BYTES_V1
        && Sha256::digest(&value.envelope_bytes).as_slice() == value.envelope_sha256
}

pub(crate) fn valid_blob(value: &TasksBlobReceiptV1) -> bool {
    nonzero(&value.reference_id)
        && (1..=TASKS_MAX_BLOB_BYTES_V1).contains(&value.declared_bytes)
        && nonzero(&value.sha256)
        && !value.custody_transfer_source_proof.is_empty()
        && value.custody_transfer_source_proof.len() <= TASKS_MAX_CUSTODY_PROOF_BYTES_V1
}

pub(crate) fn valid_cleanup(value: &TasksBlobCleanupV1) -> bool {
    nonzero(&value.reference_id)
        && (1..=TASKS_MAX_BLOB_BYTES_V1).contains(&value.declared_bytes)
        && nonzero(&value.sha256)
        && !value.custody_proof.is_empty()
        && value.custody_proof.len() <= TASKS_MAX_CUSTODY_PROOF_BYTES_V1
}

pub(crate) fn valid_task(value: &TaskV1) -> bool {
    makosh_tasks_core::validate_task_v1(value).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_fingerprint_binds_decision_and_candidate() {
        let mut input = reservation();
        let first = input.command_fingerprint();
        input.decision_revision += 1;
        assert_ne!(first, input.command_fingerprint());
        input.decision_revision -= 1;
        input.candidate_digest = [9; 32];
        assert_ne!(first, input.command_fingerprint());
    }

    #[test]
    fn exact_outbox_hash_is_required() {
        let bytes = vec![7; 32];
        let record = TasksOutboxRecordV1 {
            message_id: [1; 16],
            envelope_sha256: Sha256::digest(&bytes).into(),
            envelope_bytes: bytes,
        };
        assert!(valid_outbox(&record));
        let mut invalid = record;
        invalid.envelope_sha256 = [9; 32];
        assert!(!valid_outbox(&invalid));
    }

    fn reservation() -> ReserveReviewedCandidateCommandV1 {
        ReserveReviewedCandidateCommandV1 {
            logical_owner_id: "owner-1".to_owned(),
            command_message_id: [1; 16],
            command_envelope_sha256: [2; 32],
            command_id: [3; 16],
            approved_candidate_id: [4; 16],
            candidate_digest: [5; 32],
            source_evidence_id: [6; 16],
            source_evidence_revision: 7,
            review_id: [8; 16],
            decision_revision: 9,
            decided_by_owner_device_id: [10; 16],
            candidate_content: TasksBlobReceiptV1 {
                reference_id: [11; 16],
                declared_bytes: 12,
                sha256: [13; 32],
                custody_transfer_source_proof: vec![14; 32],
            },
            received_at_unix_millis: 1_800_000_000_000,
        }
    }
}
