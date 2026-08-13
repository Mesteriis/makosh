use makosh_relationships_core::{
    RelationshipParticipantV1, RelationshipTimestampV1, RelationshipTypeV1,
};
use sha2::{Digest, Sha256};

pub const MAX_CLIENT_MESSAGE_BYTES_V1: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipOutboxRecordV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationshipMutationV1 {
    Create {
        operation_id: [u8; 16],
        source: RelationshipParticipantV1,
        target: RelationshipParticipantV1,
        relationship_type: RelationshipTypeV1,
        valid_from: RelationshipTimestampV1,
        valid_until: Option<RelationshipTimestampV1>,
        evidence_source_owner_id: String,
        evidence_source_record_id: String,
        evidence_source_revision: u64,
        evidence_digest: [u8; 32],
        evidence_observed_at: RelationshipTimestampV1,
        created_at: RelationshipTimestampV1,
    },
    UpdateValidity {
        operation_id: [u8; 16],
        relationship_id: [u8; 16],
        expected_revision: u64,
        valid_from: RelationshipTimestampV1,
        valid_until: Option<RelationshipTimestampV1>,
        changed_at: RelationshipTimestampV1,
    },
    End {
        operation_id: [u8; 16],
        relationship_id: [u8; 16],
        expected_revision: u64,
        valid_until: RelationshipTimestampV1,
        changed_at: RelationshipTimestampV1,
    },
    Reactivate {
        operation_id: [u8; 16],
        relationship_id: [u8; 16],
        expected_revision: u64,
        valid_from: RelationshipTimestampV1,
        valid_until: Option<RelationshipTimestampV1>,
        changed_at: RelationshipTimestampV1,
    },
    AddEvidence {
        operation_id: [u8; 16],
        relationship_id: [u8; 16],
        expected_revision: u64,
        source_owner_id: String,
        source_record_id: String,
        source_revision: u64,
        evidence_digest: [u8; 32],
        observed_at: RelationshipTimestampV1,
        changed_at: RelationshipTimestampV1,
    },
    RemoveEvidence {
        operation_id: [u8; 16],
        relationship_id: [u8; 16],
        expected_revision: u64,
        evidence_id: [u8; 16],
        changed_at: RelationshipTimestampV1,
    },
}

impl RelationshipMutationV1 {
    pub fn operation_id(&self) -> [u8; 16] {
        match self {
            Self::Create { operation_id, .. }
            | Self::UpdateValidity { operation_id, .. }
            | Self::End { operation_id, .. }
            | Self::Reactivate { operation_id, .. }
            | Self::AddEvidence { operation_id, .. }
            | Self::RemoveEvidence { operation_id, .. } => *operation_id,
        }
    }
    pub fn operation_kind(&self) -> i16 {
        match self {
            Self::Create { .. } => 1,
            Self::UpdateValidity { .. } => 2,
            Self::End { .. } => 3,
            Self::Reactivate { .. } => 4,
            Self::AddEvidence { .. } => 5,
            Self::RemoveEvidence { .. } => 6,
        }
    }
    pub fn relationship_id(&self) -> Option<[u8; 16]> {
        match self {
            Self::Create { .. } => None,
            Self::UpdateValidity {
                relationship_id, ..
            }
            | Self::End {
                relationship_id, ..
            }
            | Self::Reactivate {
                relationship_id, ..
            }
            | Self::AddEvidence {
                relationship_id, ..
            }
            | Self::RemoveEvidence {
                relationship_id, ..
            } => Some(*relationship_id),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipOperationV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; 16],
    pub request_sha256: [u8; 32],
    pub request_bytes: Vec<u8>,
    pub received_at_unix_millis: i64,
    pub mutation: RelationshipMutationV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipCommitV1 {
    pub response_sha256: [u8; 32],
    pub response_bytes: Vec<u8>,
    pub lifecycle_event: RelationshipOutboxRecordV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationshipOperationOutcomeV1 {
    Applied { response_bytes: Vec<u8> },
    Replayed { response_bytes: Vec<u8> },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationshipsPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    NotFound,
    OperationConflict,
    RevisionConflict,
    StateConflict,
    EvidenceConflict,
    OutboxConflict,
}

pub(crate) fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}
pub(crate) fn valid_operation(value: &RelationshipOperationV1) -> bool {
    value.operation_id == value.mutation.operation_id()
        && valid_owner(&value.logical_owner_id)
        && nonzero(&value.operation_id)
        && nonzero(&value.request_sha256)
        && !value.request_bytes.is_empty()
        && value.request_bytes.len() <= MAX_CLIENT_MESSAGE_BYTES_V1
        && Sha256::digest(&value.request_bytes).as_slice() == value.request_sha256
        && value.received_at_unix_millis > 0
}
pub(crate) fn valid_commit(value: &RelationshipCommitV1) -> bool {
    nonzero(&value.response_sha256)
        && !value.response_bytes.is_empty()
        && value.response_bytes.len() <= MAX_CLIENT_MESSAGE_BYTES_V1
        && Sha256::digest(&value.response_bytes).as_slice() == value.response_sha256
        && nonzero(&value.lifecycle_event.message_id)
        && nonzero(&value.lifecycle_event.envelope_sha256)
        && !value.lifecycle_event.envelope_bytes.is_empty()
        && value.lifecycle_event.envelope_bytes.len() <= MAX_CLIENT_MESSAGE_BYTES_V1
        && Sha256::digest(&value.lifecycle_event.envelope_bytes).as_slice()
            == value.lifecycle_event.envelope_sha256
}
pub(crate) fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}
pub(crate) fn i64_value(value: u64) -> Result<i64, RelationshipsPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| RelationshipsPersistenceErrorV1::InvalidInput)
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_relationships_core::{RelationshipParticipantKindV1, RelationshipParticipantV1};

    #[test]
    fn exact_request_hash_is_required() {
        let bytes = b"request".to_vec();
        let mut input = RelationshipOperationV1 {
            logical_owner_id: "owner-1".to_owned(),
            operation_id: [1; 16],
            request_sha256: Sha256::digest(&bytes).into(),
            request_bytes: bytes,
            received_at_unix_millis: 1,
            mutation: RelationshipMutationV1::Create {
                operation_id: [1; 16],
                source: RelationshipParticipantV1 {
                    kind: RelationshipParticipantKindV1::Person,
                    public_id: [2; 16],
                },
                target: RelationshipParticipantV1 {
                    kind: RelationshipParticipantKindV1::Person,
                    public_id: [3; 16],
                },
                relationship_type: RelationshipTypeV1::Friend,
                valid_from: RelationshipTimestampV1 {
                    unix_seconds: 1,
                    nanos: 0,
                },
                valid_until: None,
                evidence_source_owner_id: "persons".to_owned(),
                evidence_source_record_id: "record-1".to_owned(),
                evidence_source_revision: 1,
                evidence_digest: [4; 32],
                evidence_observed_at: RelationshipTimestampV1 {
                    unix_seconds: 1,
                    nanos: 0,
                },
                created_at: RelationshipTimestampV1 {
                    unix_seconds: 1,
                    nanos: 0,
                },
            },
        };
        assert!(valid_operation(&input));
        input.request_bytes.push(0);
        assert!(!valid_operation(&input));
    }
}
