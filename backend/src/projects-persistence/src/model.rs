use makosh_projects_core::{
    ProjectDraftV1, ProjectOutcomeStateV1, ProjectRecordV1, ProjectReferenceKindV1, ProjectStateV1,
    ProjectTimestampV1,
};
use sha2::{Digest, Sha256};

pub const PROJECTS_MAX_CLIENT_MESSAGE_BYTES_V1: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectOutboxRecordV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectLifecycleMutationV1 {
    Create(ProjectDraftV1),
    Update {
        operation_id: [u8; 16],
        project_id: [u8; 16],
        expected_revision: u64,
        name: Option<String>,
        description: Option<String>,
        start_at: Option<ProjectTimestampV1>,
        target_at: Option<ProjectTimestampV1>,
        changed_at: ProjectTimestampV1,
    },
    SetState {
        operation_id: [u8; 16],
        project_id: [u8; 16],
        expected_revision: u64,
        state: ProjectStateV1,
        changed_at: ProjectTimestampV1,
    },
    AddOutcome {
        operation_id: [u8; 16],
        project_id: [u8; 16],
        expected_revision: u64,
        title: String,
        description: String,
        target_at: Option<ProjectTimestampV1>,
        changed_at: ProjectTimestampV1,
    },
    UpdateOutcome {
        operation_id: [u8; 16],
        project_id: [u8; 16],
        expected_revision: u64,
        outcome_id: [u8; 16],
        expected_outcome_revision: u64,
        title: Option<String>,
        description: Option<String>,
        target_at: Option<ProjectTimestampV1>,
        changed_at: ProjectTimestampV1,
    },
    SetOutcomeState {
        operation_id: [u8; 16],
        project_id: [u8; 16],
        expected_revision: u64,
        outcome_id: [u8; 16],
        expected_outcome_revision: u64,
        state: ProjectOutcomeStateV1,
        changed_at: ProjectTimestampV1,
    },
    RemoveOutcome {
        operation_id: [u8; 16],
        project_id: [u8; 16],
        expected_revision: u64,
        outcome_id: [u8; 16],
        expected_outcome_revision: u64,
        changed_at: ProjectTimestampV1,
    },
    AddReference {
        operation_id: [u8; 16],
        project_id: [u8; 16],
        expected_revision: u64,
        kind: ProjectReferenceKindV1,
        public_id: [u8; 16],
        label: String,
        changed_at: ProjectTimestampV1,
    },
    RemoveReference {
        operation_id: [u8; 16],
        project_id: [u8; 16],
        expected_revision: u64,
        reference_id: [u8; 16],
        changed_at: ProjectTimestampV1,
    },
}

impl ProjectLifecycleMutationV1 {
    #[must_use]
    pub fn operation_id(&self) -> [u8; 16] {
        match self {
            Self::Create(value) => value.operation_id,
            Self::Update { operation_id, .. }
            | Self::SetState { operation_id, .. }
            | Self::AddOutcome { operation_id, .. }
            | Self::UpdateOutcome { operation_id, .. }
            | Self::SetOutcomeState { operation_id, .. }
            | Self::RemoveOutcome { operation_id, .. }
            | Self::AddReference { operation_id, .. }
            | Self::RemoveReference { operation_id, .. } => *operation_id,
        }
    }

    #[must_use]
    pub fn operation_kind(&self) -> i16 {
        match self {
            Self::Create(_) => 1,
            Self::Update { .. } => 2,
            Self::SetState { .. } => 3,
            Self::AddOutcome { .. } => 4,
            Self::UpdateOutcome { .. } => 5,
            Self::SetOutcomeState { .. } => 6,
            Self::RemoveOutcome { .. } => 7,
            Self::AddReference { .. } => 8,
            Self::RemoveReference { .. } => 9,
        }
    }

    #[must_use]
    pub fn project_id(&self) -> Option<[u8; 16]> {
        match self {
            Self::Create(_) => None,
            Self::Update { project_id, .. }
            | Self::SetState { project_id, .. }
            | Self::AddOutcome { project_id, .. }
            | Self::UpdateOutcome { project_id, .. }
            | Self::SetOutcomeState { project_id, .. }
            | Self::RemoveOutcome { project_id, .. }
            | Self::AddReference { project_id, .. }
            | Self::RemoveReference { project_id, .. } => Some(*project_id),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectLifecycleOperationV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; 16],
    pub request_sha256: [u8; 32],
    pub request_bytes: Vec<u8>,
    pub received_at_unix_millis: i64,
    pub mutation: ProjectLifecycleMutationV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectLifecycleCommitV1 {
    pub response_sha256: [u8; 32],
    pub response_bytes: Vec<u8>,
    pub lifecycle_event: ProjectOutboxRecordV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectLifecycleOperationOutcomeV1 {
    Applied {
        project: Box<ProjectRecordV1>,
        response_bytes: Vec<u8>,
    },
    Replayed {
        response_bytes: Vec<u8>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectsPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    NotFound,
    OperationConflict,
    RevisionConflict,
    OutboxConflict,
}

pub(crate) fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

pub(crate) fn valid_operation(value: &ProjectLifecycleOperationV1) -> bool {
    value.operation_id == value.mutation.operation_id()
        && valid_owner(&value.logical_owner_id)
        && nonzero(&value.operation_id)
        && nonzero(&value.request_sha256)
        && !value.request_bytes.is_empty()
        && value.request_bytes.len() <= PROJECTS_MAX_CLIENT_MESSAGE_BYTES_V1
        && Sha256::digest(&value.request_bytes).as_slice() == value.request_sha256
        && value.received_at_unix_millis > 0
}

pub(crate) fn valid_commit(value: &ProjectLifecycleCommitV1) -> bool {
    nonzero(&value.response_sha256)
        && !value.response_bytes.is_empty()
        && value.response_bytes.len() <= PROJECTS_MAX_CLIENT_MESSAGE_BYTES_V1
        && Sha256::digest(&value.response_bytes).as_slice() == value.response_sha256
        && nonzero(&value.lifecycle_event.message_id)
        && nonzero(&value.lifecycle_event.envelope_sha256)
        && !value.lifecycle_event.envelope_bytes.is_empty()
        && value.lifecycle_event.envelope_bytes.len() <= PROJECTS_MAX_CLIENT_MESSAGE_BYTES_V1
        && Sha256::digest(&value.lifecycle_event.envelope_bytes).as_slice()
            == value.lifecycle_event.envelope_sha256
}

pub(crate) fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn i64_value(value: u64) -> Result<i64, ProjectsPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| ProjectsPersistenceErrorV1::InvalidInput)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_hashes_and_nine_operation_kinds_are_required() {
        let request = b"request".to_vec();
        let operation = ProjectLifecycleOperationV1 {
            logical_owner_id: "owner-1".to_owned(),
            operation_id: [1; 16],
            request_sha256: Sha256::digest(&request).into(),
            request_bytes: request,
            received_at_unix_millis: 1,
            mutation: ProjectLifecycleMutationV1::Create(ProjectDraftV1 {
                operation_id: [1; 16],
                logical_owner_id: "owner-1".to_owned(),
                name: "Project".to_owned(),
                description: String::new(),
                start_at: None,
                target_at: None,
                created_at: ProjectTimestampV1 {
                    unix_seconds: 1,
                    nanos: 0,
                },
            }),
        };
        assert!(valid_operation(&operation));
        assert_eq!(operation.mutation.operation_kind(), 1);
    }
}
