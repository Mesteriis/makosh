use makosh_organizations_core::{
    OrganizationDraftV1, OrganizationRecordV1, OrganizationStateV1, OrganizationTimestampV1,
};
use sha2::{Digest, Sha256};

pub const ORGANIZATIONS_MAX_CLIENT_MESSAGE_BYTES_V1: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrganizationOutboxRecordV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OrganizationLifecycleMutationV1 {
    Create(OrganizationDraftV1),
    Update {
        operation_id: [u8; 16],
        organization_id: [u8; 16],
        expected_revision: u64,
        display_name: Option<String>,
        legal_name: Option<String>,
        description: Option<String>,
        website: Option<String>,
        industry: Option<String>,
        country_code: Option<String>,
        changed_at: OrganizationTimestampV1,
    },
    SetState {
        operation_id: [u8; 16],
        organization_id: [u8; 16],
        expected_revision: u64,
        state: OrganizationStateV1,
        changed_at: OrganizationTimestampV1,
    },
    AddSource {
        operation_id: [u8; 16],
        organization_id: [u8; 16],
        expected_revision: u64,
        source_owner_id: String,
        source_record_id: String,
        source_revision: u64,
        evidence_digest: [u8; 32],
        changed_at: OrganizationTimestampV1,
    },
    RemoveSource {
        operation_id: [u8; 16],
        organization_id: [u8; 16],
        expected_revision: u64,
        source_id: [u8; 16],
        changed_at: OrganizationTimestampV1,
    },
}

impl OrganizationLifecycleMutationV1 {
    #[must_use]
    pub fn operation_id(&self) -> [u8; 16] {
        match self {
            Self::Create(value) => value.operation_id,
            Self::Update { operation_id, .. }
            | Self::SetState { operation_id, .. }
            | Self::AddSource { operation_id, .. }
            | Self::RemoveSource { operation_id, .. } => *operation_id,
        }
    }

    #[must_use]
    pub fn operation_kind(&self) -> i16 {
        match self {
            Self::Create(_) => 1,
            Self::Update { .. } => 2,
            Self::SetState { .. } => 3,
            Self::AddSource { .. } => 4,
            Self::RemoveSource { .. } => 5,
        }
    }

    #[must_use]
    pub fn organization_id(&self) -> Option<[u8; 16]> {
        match self {
            Self::Create(_) => None,
            Self::Update {
                organization_id, ..
            }
            | Self::SetState {
                organization_id, ..
            }
            | Self::AddSource {
                organization_id, ..
            }
            | Self::RemoveSource {
                organization_id, ..
            } => Some(*organization_id),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrganizationLifecycleOperationV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; 16],
    pub request_sha256: [u8; 32],
    pub request_bytes: Vec<u8>,
    pub received_at_unix_millis: i64,
    pub mutation: OrganizationLifecycleMutationV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrganizationLifecycleCommitV1 {
    pub response_sha256: [u8; 32],
    pub response_bytes: Vec<u8>,
    pub lifecycle_event: OrganizationOutboxRecordV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OrganizationLifecycleOperationOutcomeV1 {
    Applied {
        organization: Box<OrganizationRecordV1>,
        response_bytes: Vec<u8>,
    },
    Replayed {
        response_bytes: Vec<u8>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizationsPersistenceErrorV1 {
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

pub(crate) fn valid_operation(value: &OrganizationLifecycleOperationV1) -> bool {
    value.operation_id == value.mutation.operation_id()
        && valid_owner(&value.logical_owner_id)
        && nonzero(&value.operation_id)
        && nonzero(&value.request_sha256)
        && !value.request_bytes.is_empty()
        && value.request_bytes.len() <= ORGANIZATIONS_MAX_CLIENT_MESSAGE_BYTES_V1
        && Sha256::digest(&value.request_bytes).as_slice() == value.request_sha256
        && value.received_at_unix_millis > 0
}

pub(crate) fn valid_commit(value: &OrganizationLifecycleCommitV1) -> bool {
    nonzero(&value.response_sha256)
        && !value.response_bytes.is_empty()
        && value.response_bytes.len() <= ORGANIZATIONS_MAX_CLIENT_MESSAGE_BYTES_V1
        && Sha256::digest(&value.response_bytes).as_slice() == value.response_sha256
        && nonzero(&value.lifecycle_event.message_id)
        && nonzero(&value.lifecycle_event.envelope_sha256)
        && !value.lifecycle_event.envelope_bytes.is_empty()
        && value.lifecycle_event.envelope_bytes.len() <= ORGANIZATIONS_MAX_CLIENT_MESSAGE_BYTES_V1
        && Sha256::digest(&value.lifecycle_event.envelope_bytes).as_slice()
            == value.lifecycle_event.envelope_sha256
}

pub(crate) fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn i64_value(value: u64) -> Result<i64, OrganizationsPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| OrganizationsPersistenceErrorV1::InvalidInput)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_hashes_and_bounds_are_required() {
        let request = b"request".to_vec();
        let response = b"response".to_vec();
        let event = b"event".to_vec();
        let operation = OrganizationLifecycleOperationV1 {
            logical_owner_id: "owner-1".to_owned(),
            operation_id: [1; 16],
            request_sha256: Sha256::digest(&request).into(),
            request_bytes: request,
            received_at_unix_millis: 1,
            mutation: OrganizationLifecycleMutationV1::Create(OrganizationDraftV1 {
                operation_id: [1; 16],
                logical_owner_id: "owner-1".to_owned(),
                display_name: "Organization".to_owned(),
                legal_name: String::new(),
                description: String::new(),
                website: String::new(),
                industry: String::new(),
                country_code: String::new(),
                created_at: OrganizationTimestampV1 {
                    unix_seconds: 1,
                    nanos: 0,
                },
            }),
        };
        assert!(valid_operation(&operation));
        let commit = OrganizationLifecycleCommitV1 {
            response_sha256: Sha256::digest(&response).into(),
            response_bytes: response,
            lifecycle_event: OrganizationOutboxRecordV1 {
                message_id: [2; 16],
                envelope_sha256: Sha256::digest(&event).into(),
                envelope_bytes: event,
            },
        };
        assert!(valid_commit(&commit));
        let mut drift = operation;
        drift.request_bytes.push(0);
        assert!(!valid_operation(&drift));
    }
}
