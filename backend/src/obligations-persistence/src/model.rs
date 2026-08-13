use makosh_obligations_core::{
    ObligationEvidenceLinkV1, ObligationLifecycleStateV1, ObligationRecordV1,
    ObligationTimestampV1, ObligationV1, ReviewedCandidateObligationDraftV1,
    obligation_creation_fingerprint_v1,
};
use sha2::{Digest, Sha256};

pub const OBLIGATIONS_RECOVERY_LIMIT_V1: u16 = 128;
pub const OBLIGATIONS_MAX_EVENT_BYTES_V1: usize = 64 * 1024;
pub const OBLIGATIONS_MAX_BLOB_BYTES_V1: u64 = 16 * 1024;
pub const OBLIGATIONS_MAX_CUSTODY_PROOF_BYTES_V1: usize = 2_048;
pub const OBLIGATIONS_MAX_CLIENT_MESSAGE_BYTES_V1: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObligationsBlobReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_transfer_source_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObligationsBlobCleanupV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObligationsOutboxRecordV1 {
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
    pub candidate_content: ObligationsBlobReceiptV1,
    pub received_at_unix_millis: i64,
}

impl ReserveReviewedCandidateCommandV1 {
    pub fn command_fingerprint(&self) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(b"makosh.obligations.reviewed-candidate.command.v1\0");
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
    pub candidate_content: ObligationsBlobReceiptV1,
    pub materialization: Option<ObligationsBlobCleanupV1>,
    pub cleanup_completed_at_unix_millis: Option<i64>,
    pub completed: bool,
    pub rejected: bool,
    pub obligation_id: Option<[u8; 16]>,
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
    pub materialization: ObligationsBlobCleanupV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteReviewedCandidateObligationV1 {
    pub logical_owner_id: String,
    pub command_message_id: [u8; 16],
    pub draft: ReviewedCandidateObligationDraftV1,
    pub created_result: ObligationsOutboxRecordV1,
    pub occurred_at_unix_millis: i64,
}

impl CompleteReviewedCandidateObligationV1 {
    pub fn creation_fingerprint(&self) -> Result<[u8; 32], ObligationsPersistenceErrorV1> {
        obligation_creation_fingerprint_v1(&self.draft)
            .map_err(|_| ObligationsPersistenceErrorV1::InvalidInput)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RejectReviewedCandidateObligationV1 {
    pub logical_owner_id: String,
    pub command_message_id: [u8; 16],
    pub rejected_result: ObligationsOutboxRecordV1,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObligationsPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    CommandConflict,
    InboxConflict,
    ObligationConflict,
    NotFound,
    OperationConflict,
    RevisionConflict,
    DependencyCycle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObligationsLifecycleMutationV1 {
    Update {
        operation_id: [u8; 16],
        obligation_id: [u8; 16],
        expected_revision: u64,
        statement: Option<String>,
        condition: Option<Option<String>>,
        due_at: Option<Option<ObligationTimestampV1>>,
        obligated_party_id: Option<[u8; 16]>,
        beneficiary_party_id: Option<Option<[u8; 16]>>,
        changed_at: ObligationTimestampV1,
    },
    SetState {
        operation_id: [u8; 16],
        obligation_id: [u8; 16],
        expected_revision: u64,
        state: ObligationLifecycleStateV1,
        changed_at: ObligationTimestampV1,
    },
    AddEvidence {
        operation_id: [u8; 16],
        obligation_id: [u8; 16],
        expected_revision: u64,
        evidence: ObligationEvidenceLinkV1,
        changed_at: ObligationTimestampV1,
    },
    RemoveEvidence {
        operation_id: [u8; 16],
        obligation_id: [u8; 16],
        expected_revision: u64,
        evidence_link_id: [u8; 16],
        changed_at: ObligationTimestampV1,
    },
}

impl ObligationsLifecycleMutationV1 {
    #[must_use]
    pub fn operation_kind(&self) -> i16 {
        match self {
            Self::Update { .. } => 1,
            Self::SetState { .. } => 2,
            Self::AddEvidence { .. } => 3,
            Self::RemoveEvidence { .. } => 4,
        }
    }

    #[must_use]
    pub fn operation_id(&self) -> [u8; 16] {
        match self {
            Self::Update { operation_id, .. }
            | Self::SetState { operation_id, .. }
            | Self::AddEvidence { operation_id, .. }
            | Self::RemoveEvidence { operation_id, .. } => *operation_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObligationsLifecycleOperationV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; 16],
    pub request_sha256: [u8; 32],
    pub request_bytes: Vec<u8>,
    pub received_at_unix_millis: i64,
    pub mutation: ObligationsLifecycleMutationV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObligationsLifecycleCommitV1 {
    pub response_sha256: [u8; 32],
    pub response_bytes: Vec<u8>,
    pub lifecycle_event: ObligationsOutboxRecordV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObligationsLifecycleOperationOutcomeV1 {
    Applied {
        obligation: Box<ObligationRecordV1>,
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

pub(crate) fn valid_outbox(value: &ObligationsOutboxRecordV1) -> bool {
    nonzero(&value.message_id)
        && nonzero(&value.envelope_sha256)
        && !value.envelope_bytes.is_empty()
        && value.envelope_bytes.len() <= OBLIGATIONS_MAX_EVENT_BYTES_V1
        && Sha256::digest(&value.envelope_bytes).as_slice() == value.envelope_sha256
}

pub(crate) fn valid_blob(value: &ObligationsBlobReceiptV1) -> bool {
    nonzero(&value.reference_id)
        && (1..=OBLIGATIONS_MAX_BLOB_BYTES_V1).contains(&value.declared_bytes)
        && nonzero(&value.sha256)
        && !value.custody_transfer_source_proof.is_empty()
        && value.custody_transfer_source_proof.len() <= OBLIGATIONS_MAX_CUSTODY_PROOF_BYTES_V1
}

pub(crate) fn valid_cleanup(value: &ObligationsBlobCleanupV1) -> bool {
    nonzero(&value.reference_id)
        && (1..=OBLIGATIONS_MAX_BLOB_BYTES_V1).contains(&value.declared_bytes)
        && nonzero(&value.sha256)
        && !value.custody_proof.is_empty()
        && value.custody_proof.len() <= OBLIGATIONS_MAX_CUSTODY_PROOF_BYTES_V1
}

pub(crate) fn valid_obligation(value: &ObligationV1) -> bool {
    makosh_obligations_core::validate_obligation_v1(value).is_ok()
}

pub(crate) fn valid_lifecycle_operation(value: &ObligationsLifecycleOperationV1) -> bool {
    valid_identity(&value.logical_owner_id)
        && nonzero(&value.operation_id)
        && nonzero(&value.request_sha256)
        && !value.request_bytes.is_empty()
        && value.request_bytes.len() <= OBLIGATIONS_MAX_CLIENT_MESSAGE_BYTES_V1
        && Sha256::digest(&value.request_bytes).as_slice() == value.request_sha256
        && value.received_at_unix_millis > 0
        && value.mutation.operation_id() == value.operation_id
}

pub(crate) fn valid_lifecycle_commit(value: &ObligationsLifecycleCommitV1) -> bool {
    nonzero(&value.response_sha256)
        && !value.response_bytes.is_empty()
        && value.response_bytes.len() <= OBLIGATIONS_MAX_CLIENT_MESSAGE_BYTES_V1
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
        let record = ObligationsOutboxRecordV1 {
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
        let operation = ObligationsLifecycleOperationV1 {
            logical_owner_id: "owner-1".to_owned(),
            operation_id: [4; 16],
            request_sha256: Sha256::digest(&request_bytes).into(),
            request_bytes,
            received_at_unix_millis: 1_800_000_000_000,
            mutation: ObligationsLifecycleMutationV1::AddEvidence {
                operation_id: [4; 16],
                obligation_id: [5; 16],
                expected_revision: 2,
                evidence: ObligationEvidenceLinkV1 {
                    evidence_link_id: [6; 16],
                    evidence_owner_id: "communications".to_owned(),
                    evidence_record_id: [7; 16],
                    evidence_revision: 1,
                    evidence_digest: [8; 32],
                },
                changed_at: makosh_obligations_core::ObligationTimestampV1 {
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
            candidate_content: ObligationsBlobReceiptV1 {
                reference_id: [11; 16],
                declared_bytes: 12,
                sha256: [13; 32],
                custody_transfer_source_proof: vec![14; 32],
            },
            received_at_unix_millis: 1_800_000_000_000,
        }
    }
}
