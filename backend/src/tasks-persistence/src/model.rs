use makosh_tasks_core::{
    ManualTaskDraftV1, ReviewedCandidateTaskDraftV1, TaskLifecycleStateV1, TaskPriorityV1,
    TaskRecordV1, TaskTimestampV1, TaskV1, task_creation_fingerprint_v1,
};
use sha2::{Digest, Sha256};

pub const TASKS_RECOVERY_LIMIT_V1: u16 = 128;
pub const TASKS_MAX_EVENT_BYTES_V1: usize = 64 * 1024;
pub const TASKS_MAX_BLOB_BYTES_V1: u64 = 16 * 1024;
pub const TASKS_MAX_CUSTODY_PROOF_BYTES_V1: usize = 2_048;
pub const TASKS_MAX_CLIENT_MESSAGE_BYTES_V1: usize = 64 * 1024;

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
    OperationConflict,
    RevisionConflict,
    DependencyCycle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TasksLifecycleMutationV1 {
    Create(ManualTaskDraftV1),
    Update {
        operation_id: [u8; 16],
        task_id: [u8; 16],
        expected_revision: u64,
        title: Option<String>,
        description: Option<Option<String>>,
        due_at: Option<Option<TaskTimestampV1>>,
        changed_at: TaskTimestampV1,
    },
    SetState {
        operation_id: [u8; 16],
        task_id: [u8; 16],
        expected_revision: u64,
        state: TaskLifecycleStateV1,
        changed_at: TaskTimestampV1,
    },
    SetPriority {
        operation_id: [u8; 16],
        task_id: [u8; 16],
        expected_revision: u64,
        priority: TaskPriorityV1,
        changed_at: TaskTimestampV1,
    },
    AddDependency {
        operation_id: [u8; 16],
        task_id: [u8; 16],
        expected_revision: u64,
        dependency_id: [u8; 16],
        depends_on_task_id: [u8; 16],
        changed_at: TaskTimestampV1,
    },
    RemoveDependency {
        operation_id: [u8; 16],
        task_id: [u8; 16],
        expected_revision: u64,
        dependency_id: [u8; 16],
        changed_at: TaskTimestampV1,
    },
    AddChecklistItem {
        operation_id: [u8; 16],
        task_id: [u8; 16],
        expected_revision: u64,
        checklist_item_id: [u8; 16],
        label: String,
        position: u32,
        changed_at: TaskTimestampV1,
    },
    UpdateChecklistItem {
        operation_id: [u8; 16],
        task_id: [u8; 16],
        expected_revision: u64,
        checklist_item_id: [u8; 16],
        label: Option<String>,
        completed: Option<bool>,
        position: Option<u32>,
        changed_at: TaskTimestampV1,
    },
    RemoveChecklistItem {
        operation_id: [u8; 16],
        task_id: [u8; 16],
        expected_revision: u64,
        checklist_item_id: [u8; 16],
        changed_at: TaskTimestampV1,
    },
}

impl TasksLifecycleMutationV1 {
    #[must_use]
    pub fn operation_kind(&self) -> i16 {
        match self {
            Self::Create(_) => 1,
            Self::Update { .. } => 2,
            Self::SetState { .. } => 3,
            Self::SetPriority { .. } => 4,
            Self::AddDependency { .. } => 5,
            Self::RemoveDependency { .. } => 6,
            Self::AddChecklistItem { .. } => 7,
            Self::UpdateChecklistItem { .. } => 8,
            Self::RemoveChecklistItem { .. } => 9,
        }
    }

    #[must_use]
    pub fn operation_id(&self) -> [u8; 16] {
        match self {
            Self::Create(value) => value.operation_id,
            Self::Update { operation_id, .. }
            | Self::SetState { operation_id, .. }
            | Self::SetPriority { operation_id, .. }
            | Self::AddDependency { operation_id, .. }
            | Self::RemoveDependency { operation_id, .. }
            | Self::AddChecklistItem { operation_id, .. }
            | Self::UpdateChecklistItem { operation_id, .. }
            | Self::RemoveChecklistItem { operation_id, .. } => *operation_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TasksLifecycleOperationV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; 16],
    pub request_sha256: [u8; 32],
    pub request_bytes: Vec<u8>,
    pub received_at_unix_millis: i64,
    pub mutation: TasksLifecycleMutationV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TasksLifecycleCommitV1 {
    pub response_sha256: [u8; 32],
    pub response_bytes: Vec<u8>,
    pub lifecycle_event: TasksOutboxRecordV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TasksLifecycleOperationOutcomeV1 {
    Applied {
        task: Box<TaskRecordV1>,
        response_bytes: Vec<u8>,
    },
    Replayed {
        response_bytes: Vec<u8>,
    },
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

pub(crate) fn valid_lifecycle_operation(value: &TasksLifecycleOperationV1) -> bool {
    valid_identity(&value.logical_owner_id)
        && nonzero(&value.operation_id)
        && nonzero(&value.request_sha256)
        && !value.request_bytes.is_empty()
        && value.request_bytes.len() <= TASKS_MAX_CLIENT_MESSAGE_BYTES_V1
        && Sha256::digest(&value.request_bytes).as_slice() == value.request_sha256
        && value.received_at_unix_millis > 0
        && value.mutation.operation_id() == value.operation_id
}

pub(crate) fn valid_lifecycle_commit(value: &TasksLifecycleCommitV1) -> bool {
    nonzero(&value.response_sha256)
        && !value.response_bytes.is_empty()
        && value.response_bytes.len() <= TASKS_MAX_CLIENT_MESSAGE_BYTES_V1
        && Sha256::digest(&value.response_bytes).as_slice() == value.response_sha256
        && valid_outbox(&value.lifecycle_event)
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

    #[test]
    fn lifecycle_operation_binds_exact_request_bytes() {
        let request_bytes = vec![7; 32];
        let operation = TasksLifecycleOperationV1 {
            logical_owner_id: "owner-1".to_owned(),
            operation_id: [4; 16],
            request_sha256: Sha256::digest(&request_bytes).into(),
            request_bytes,
            received_at_unix_millis: 1_800_000_000_000,
            mutation: TasksLifecycleMutationV1::SetPriority {
                operation_id: [4; 16],
                task_id: [5; 16],
                expected_revision: 2,
                priority: makosh_tasks_core::TaskPriorityV1::High,
                changed_at: makosh_tasks_core::TaskTimestampV1 {
                    unix_seconds: 1_800_000_000,
                    nanos: 0,
                },
            },
        };
        assert!(valid_lifecycle_operation(&operation));
        let mut changed = operation;
        changed.request_bytes.push(8);
        assert!(!valid_lifecycle_operation(&changed));
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
