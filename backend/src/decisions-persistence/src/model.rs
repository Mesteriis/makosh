use makosh_decisions_core::{DecisionEvidenceLinkV1, DecisionRecordV1, DecisionTimestampV1};

pub const DECISIONS_MAX_CLIENT_MESSAGE_BYTES_V1: usize = 65_536;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecisionLifecycleMutationV1 {
    Create {
        owner: String,
        operation_id: [u8; 16],
        title: String,
        question: String,
        created_at: DecisionTimestampV1,
    },
    Update {
        decision_id: [u8; 16],
        expected_revision: u64,
        title: Option<String>,
        question: Option<String>,
        changed_at: DecisionTimestampV1,
    },
    AddAlternative {
        decision_id: [u8; 16],
        expected_revision: u64,
        operation_id: [u8; 16],
        title: String,
        description: String,
        changed_at: DecisionTimestampV1,
    },
    UpdateAlternative {
        decision_id: [u8; 16],
        expected_revision: u64,
        alternative_id: [u8; 16],
        expected_alternative_revision: u64,
        title: Option<String>,
        description: Option<String>,
        changed_at: DecisionTimestampV1,
    },
    RemoveAlternative {
        decision_id: [u8; 16],
        expected_revision: u64,
        alternative_id: [u8; 16],
        expected_alternative_revision: u64,
        changed_at: DecisionTimestampV1,
    },
    AddEvidence {
        decision_id: [u8; 16],
        expected_revision: u64,
        evidence: DecisionEvidenceLinkV1,
        changed_at: DecisionTimestampV1,
    },
    RemoveEvidence {
        decision_id: [u8; 16],
        expected_revision: u64,
        evidence_link_id: [u8; 16],
        changed_at: DecisionTimestampV1,
    },
    Decide {
        decision_id: [u8; 16],
        expected_revision: u64,
        selected_alternative_id: [u8; 16],
        rationale: String,
        changed_at: DecisionTimestampV1,
    },
    Supersede {
        decision_id: [u8; 16],
        expected_revision: u64,
        replacement_decision_id: [u8; 16],
        changed_at: DecisionTimestampV1,
    },
    Cancel {
        decision_id: [u8; 16],
        expected_revision: u64,
        changed_at: DecisionTimestampV1,
    },
}

impl DecisionLifecycleMutationV1 {
    #[must_use]
    pub fn operation_kind(&self) -> i16 {
        match self {
            Self::Create { .. } => 1,
            Self::Update { .. } => 2,
            Self::AddAlternative { .. } => 3,
            Self::UpdateAlternative { .. } => 4,
            Self::RemoveAlternative { .. } => 5,
            Self::AddEvidence { .. } => 6,
            Self::RemoveEvidence { .. } => 7,
            Self::Decide { .. } => 8,
            Self::Supersede { .. } => 9,
            Self::Cancel { .. } => 10,
        }
    }

    #[must_use]
    pub fn decision_id(&self) -> Option<[u8; 16]> {
        match self {
            Self::Create { .. } => None,
            Self::Update { decision_id, .. }
            | Self::AddAlternative { decision_id, .. }
            | Self::UpdateAlternative { decision_id, .. }
            | Self::RemoveAlternative { decision_id, .. }
            | Self::AddEvidence { decision_id, .. }
            | Self::RemoveEvidence { decision_id, .. }
            | Self::Decide { decision_id, .. }
            | Self::Supersede { decision_id, .. }
            | Self::Cancel { decision_id, .. } => Some(*decision_id),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionLifecycleOperationV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; 16],
    pub request_sha256: [u8; 32],
    pub request_bytes: Vec<u8>,
    pub received_at_unix_millis: i64,
    pub mutation: DecisionLifecycleMutationV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionOutboxRecordV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionLifecycleCommitV1 {
    pub response_sha256: [u8; 32],
    pub response_bytes: Vec<u8>,
    pub lifecycle_event: DecisionOutboxRecordV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecisionLifecycleOperationOutcomeV1 {
    Applied {
        decision: Box<DecisionRecordV1>,
        response_bytes: Vec<u8>,
    },
    Replayed {
        response_bytes: Vec<u8>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionPendingOutboxV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionsPersistenceErrorV1 {
    InvalidInput,
    NotFound,
    RevisionConflict,
    StateConflict,
    OperationConflict,
    OutboxConflict,
    StorageUnavailable,
}

pub(crate) fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

pub(crate) fn valid_operation(value: &DecisionLifecycleOperationV1) -> bool {
    valid_owner(&value.logical_owner_id)
        && value.operation_id.iter().any(|byte| *byte != 0)
        && value.request_sha256.iter().any(|byte| *byte != 0)
        && !value.request_bytes.is_empty()
        && value.request_bytes.len() <= DECISIONS_MAX_CLIENT_MESSAGE_BYTES_V1
        && value.received_at_unix_millis > 0
}

pub(crate) fn valid_commit(value: &DecisionLifecycleCommitV1) -> bool {
    value.response_sha256.iter().any(|byte| *byte != 0)
        && !value.response_bytes.is_empty()
        && value.response_bytes.len() <= DECISIONS_MAX_CLIENT_MESSAGE_BYTES_V1
        && value
            .lifecycle_event
            .message_id
            .iter()
            .any(|byte| *byte != 0)
        && value
            .lifecycle_event
            .envelope_sha256
            .iter()
            .any(|byte| *byte != 0)
        && !value.lifecycle_event.envelope_bytes.is_empty()
        && value.lifecycle_event.envelope_bytes.len() <= DECISIONS_MAX_CLIENT_MESSAGE_BYTES_V1
}
