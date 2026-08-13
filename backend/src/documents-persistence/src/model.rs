use makosh_documents_core::{DocumentStateV1, DocumentV1};
use sha2::{Digest, Sha256};

pub const DOCUMENTS_MAX_CLIENT_MESSAGE_BYTES_V1: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentOutboxRecordV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentLifecycleMutationV1 {
    Create {
        title: String,
        description: String,
        media_type: String,
        original_file_name: String,
        declared_size: u64,
        content_sha256: [u8; 32],
        created_at_unix_millis: i64,
    },
    Update {
        document_id: [u8; 16],
        expected_revision: u64,
        title: Option<String>,
        description: Option<String>,
        media_type: Option<String>,
        original_file_name: Option<String>,
        changed_at_unix_millis: i64,
    },
    SetState {
        document_id: [u8; 16],
        expected_revision: u64,
        state: DocumentStateV1,
        changed_at_unix_millis: i64,
    },
    AddSource {
        document_id: [u8; 16],
        expected_revision: u64,
        source_owner_id: String,
        source_record_id: String,
        source_revision: u64,
        evidence_digest: [u8; 32],
        changed_at_unix_millis: i64,
    },
    RemoveSource {
        document_id: [u8; 16],
        expected_revision: u64,
        source_id: [u8; 16],
        changed_at_unix_millis: i64,
    },
}

impl DocumentLifecycleMutationV1 {
    #[must_use]
    pub fn operation_kind(&self) -> i16 {
        match self {
            Self::Create { .. } => 1,
            Self::Update { .. } => 2,
            Self::SetState { .. } => 3,
            Self::AddSource { .. } => 6,
            Self::RemoveSource { .. } => 7,
        }
    }

    #[must_use]
    pub fn document_id(&self) -> Option<[u8; 16]> {
        match self {
            Self::Create { .. } => None,
            Self::Update { document_id, .. }
            | Self::SetState { document_id, .. }
            | Self::AddSource { document_id, .. }
            | Self::RemoveSource { document_id, .. } => Some(*document_id),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentLifecycleOperationV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; 16],
    pub request_sha256: [u8; 32],
    pub request_bytes: Vec<u8>,
    pub received_at_unix_millis: i64,
    pub mutation: DocumentLifecycleMutationV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentLifecycleCommitV1 {
    pub response_sha256: [u8; 32],
    pub response_bytes: Vec<u8>,
    pub lifecycle_event: DocumentOutboxRecordV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentLifecycleOperationOutcomeV1 {
    Applied {
        document: Box<DocumentV1>,
        response_bytes: Vec<u8>,
    },
    Replayed {
        response_bytes: Vec<u8>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentBlobOperationKindV1 {
    Attach,
    Release,
}

impl DocumentBlobOperationKindV1 {
    pub(crate) fn code(self) -> i16 {
        match self {
            Self::Attach => 4,
            Self::Release => 5,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentBlobOperationStartV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; 16],
    pub document_id: [u8; 16],
    pub expected_revision: u64,
    pub kind: DocumentBlobOperationKindV1,
    pub blob_reference_id: [u8; 16],
    pub declared_size: Option<u64>,
    pub content_sha256: Option<[u8; 32]>,
    pub changed_at_unix_millis: i64,
    pub custody_source_proof: Vec<u8>,
    pub source_evidence_id: Option<[u8; 16]>,
    pub source_evidence_envelope_sha256: Option<[u8; 32]>,
    pub client_request_sha256: [u8; 32],
    pub client_request_bytes: Vec<u8>,
    pub provider_request_sha256: [u8; 32],
    pub provider_request_bytes: Vec<u8>,
    pub received_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentBlobOperationStartOutcomeV1 {
    Pending,
    Replayed { response_bytes: Vec<u8> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteDocumentBlobOperationV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; 16],
    pub provider_receipt_sha256: [u8; 32],
    pub provider_receipt_bytes: Vec<u8>,
    pub resolved_blob_reference_id: [u8; 16],
    pub completed_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentBoundBlobCustodyV1 {
    pub blob_reference_id: [u8; 16],
    pub declared_size: u64,
    pub content_sha256: [u8; 32],
    pub custody_source_proof: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentsPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    NotFound,
    OperationConflict,
    RevisionConflict,
    StateConflict,
    OutboxConflict,
}

pub(crate) fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

pub(crate) fn valid_exact_bytes(bytes: &[u8], sha256: &[u8; 32]) -> bool {
    nonzero(sha256)
        && !bytes.is_empty()
        && bytes.len() <= DOCUMENTS_MAX_CLIENT_MESSAGE_BYTES_V1
        && Sha256::digest(bytes).as_slice() == sha256
}

pub(crate) fn valid_operation(value: &DocumentLifecycleOperationV1) -> bool {
    valid_owner(&value.logical_owner_id)
        && nonzero(&value.operation_id)
        && valid_exact_bytes(&value.request_bytes, &value.request_sha256)
        && value.received_at_unix_millis > 0
}

pub(crate) fn valid_commit(value: &DocumentLifecycleCommitV1) -> bool {
    valid_exact_bytes(&value.response_bytes, &value.response_sha256)
        && nonzero(&value.lifecycle_event.message_id)
        && valid_exact_bytes(
            &value.lifecycle_event.envelope_bytes,
            &value.lifecycle_event.envelope_sha256,
        )
}

pub(crate) fn valid_blob_start(value: &DocumentBlobOperationStartV1) -> bool {
    let shape = match value.kind {
        DocumentBlobOperationKindV1::Attach => {
            value.declared_size.is_some_and(|size| size > 0)
                && value.content_sha256.is_some_and(|digest| nonzero(&digest))
                && value.source_evidence_id.is_some_and(|id| nonzero(&id))
                && value
                    .source_evidence_envelope_sha256
                    .is_some_and(|digest| nonzero(&digest))
        }
        DocumentBlobOperationKindV1::Release => {
            value.declared_size.is_none()
                && value.content_sha256.is_none()
                && value.source_evidence_id.is_none()
                && value.source_evidence_envelope_sha256.is_none()
        }
    };
    valid_owner(&value.logical_owner_id)
        && nonzero(&value.operation_id)
        && nonzero(&value.document_id)
        && value.expected_revision > 0
        && nonzero(&value.blob_reference_id)
        && value.changed_at_unix_millis > 0
        && !value.custody_source_proof.is_empty()
        && value.custody_source_proof.len() <= 2_048
        && shape
        && valid_exact_bytes(&value.client_request_bytes, &value.client_request_sha256)
        && valid_exact_bytes(
            &value.provider_request_bytes,
            &value.provider_request_sha256,
        )
        && value.received_at_unix_millis > 0
}

pub(crate) fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn i64_value(value: u64) -> Result<i64, DocumentsPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| DocumentsPersistenceErrorV1::InvalidInput)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_client_and_provider_bytes_are_required() {
        let bytes = b"request".to_vec();
        let input = DocumentBlobOperationStartV1 {
            logical_owner_id: "owner-1".into(),
            operation_id: [1; 16],
            document_id: [2; 16],
            expected_revision: 1,
            kind: DocumentBlobOperationKindV1::Attach,
            blob_reference_id: [3; 16],
            declared_size: Some(10),
            content_sha256: Some([4; 32]),
            changed_at_unix_millis: 2,
            custody_source_proof: vec![5; 32],
            source_evidence_id: Some([6; 16]),
            source_evidence_envelope_sha256: Some([7; 32]),
            client_request_sha256: Sha256::digest(&bytes).into(),
            client_request_bytes: bytes.clone(),
            provider_request_sha256: Sha256::digest(&bytes).into(),
            provider_request_bytes: bytes,
            received_at_unix_millis: 1,
        };
        assert!(valid_blob_start(&input));
        let mut drift = input;
        drift.provider_request_bytes.push(0);
        assert!(!valid_blob_start(&drift));
    }
}
