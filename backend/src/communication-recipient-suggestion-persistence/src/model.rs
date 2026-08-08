use makosh_communication_recipient_suggestion_core::{
    COMMUNICATION_RECIPIENT_MAX_CANDIDATES_V1,
    COMMUNICATION_RECIPIENT_MAX_CONFIDENCE_BASIS_POINTS_V1, CommunicationRecipientCandidateV1,
    CommunicationRecipientRationaleV1, CommunicationRecipientRoleV1,
    CommunicationRecipientSourceBasisV1, CommunicationRecipientSuggestionDraftV1,
    CommunicationRecipientSuggestionRejectionCodeV1, CommunicationRecipientSuggestionStatusV1,
    CommunicationRecipientSuggestionTransitionV1,
};
use sha2::{Digest, Sha256};

pub const COMMUNICATION_RECIPIENT_SUGGESTION_RECOVERY_LIMIT_V1: u16 = 128;
pub const COMMUNICATION_RECIPIENT_SUGGESTION_REALTIME_LIMIT_V1: u16 = 1_024;
pub const COMMUNICATION_RECIPIENT_SUGGESTION_OUTBOX_LIMIT_V1: u16 = 128;
pub const COMMUNICATION_RECIPIENT_SUGGESTION_MAX_EVENT_BYTES_V1: usize = 64 * 1024;
pub const COMMUNICATION_RECIPIENT_SUGGESTION_MAX_EVALUATION_RECEIPT_BYTES_V1: usize = 16 * 1024;
pub const COMMUNICATION_RECIPIENT_SUGGESTION_MAX_CUSTODY_PROOF_BYTES_V1: usize = 2_048;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationRecipientSuggestionBlobCleanupV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateCommunicationRecipientSuggestionRunV1 {
    pub logical_owner_id: String,
    pub draft: CommunicationRecipientSuggestionDraftV1,
    pub source_prepare_message_id: [u8; 16],
    pub source_prepare_envelope_sha256: [u8; 32],
    pub source_prepare_envelope_bytes: Vec<u8>,
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedCommunicationRecipientSuggestionRunV1 {
    pub logical_owner_id: String,
    pub draft: CommunicationRecipientSuggestionDraftV1,
    pub request_fingerprint: [u8; 32],
    pub status: CommunicationRecipientSuggestionStatusV1,
    pub evaluation_receipt_bytes: Option<Vec<u8>>,
    pub source_cleanup: Option<CommunicationRecipientSuggestionBlobCleanupV1>,
    pub cleanup_completed_at_unix_millis: Option<i64>,
    pub created_at_unix_millis: i64,
    pub updated_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreateCommunicationRecipientSuggestionOutcomeV1 {
    Created(PersistedCommunicationRecipientSuggestionRunV1),
    Existing(PersistedCommunicationRecipientSuggestionRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationRecipientSuggestionSourceResultV1 {
    pub result_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub logical_owner_id: String,
    pub run_id: [u8; 16],
    pub transition: CommunicationRecipientSuggestionTransitionV1,
    pub evaluation_receipt_bytes: Option<Vec<u8>>,
    pub source_cleanup: Option<CommunicationRecipientSuggestionBlobCleanupV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunicationRecipientSuggestionInboxResultV1 {
    Applied(PersistedCommunicationRecipientSuggestionRunV1),
    Duplicate(PersistedCommunicationRecipientSuggestionRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnpublishedCommunicationRecipientSuggestionEventV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationRecipientSuggestionPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    RequestConflict,
    InboxConflict,
    RevisionConflict,
    InvalidTransition,
    NotFound,
}

pub(crate) fn request_fingerprint(draft: &CommunicationRecipientSuggestionDraftV1) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.communication_recipient_suggestion.start.v1\0");
    hash.update(draft.source_message_id);
    hash.update(draft.expected_source_revision.to_be_bytes());
    hash.finalize().into()
}

pub(crate) fn encode_candidates(
    candidates: &[CommunicationRecipientCandidateV1],
) -> Result<Vec<u8>, CommunicationRecipientSuggestionPersistenceErrorV1> {
    if candidates.len() > COMMUNICATION_RECIPIENT_MAX_CANDIDATES_V1 {
        return Err(CommunicationRecipientSuggestionPersistenceErrorV1::InvalidInput);
    }
    let mut bytes = Vec::with_capacity(2 + candidates.len() * 7);
    bytes.push(1);
    bytes.push(
        u8::try_from(candidates.len())
            .map_err(|_| CommunicationRecipientSuggestionPersistenceErrorV1::InvalidInput)?,
    );
    for candidate in candidates {
        if !valid_candidate(candidate) {
            return Err(CommunicationRecipientSuggestionPersistenceErrorV1::InvalidInput);
        }
        bytes.push(role_code(candidate.role));
        bytes.push(rationale_code(candidate.rationale));
        bytes.push(source_basis_code(candidate.source_basis));
        bytes.extend_from_slice(&candidate.confidence_basis_points.to_be_bytes());
    }
    Ok(bytes)
}

pub(crate) fn decode_candidates(
    bytes: &[u8],
) -> Result<
    Vec<CommunicationRecipientCandidateV1>,
    CommunicationRecipientSuggestionPersistenceErrorV1,
> {
    if bytes.len() < 2 || bytes[0] != 1 || bytes.len() != 2 + usize::from(bytes[1]) * 7 {
        return Err(CommunicationRecipientSuggestionPersistenceErrorV1::InvalidRow);
    }
    let count = usize::from(bytes[1]);
    if count > COMMUNICATION_RECIPIENT_MAX_CANDIDATES_V1 {
        return Err(CommunicationRecipientSuggestionPersistenceErrorV1::InvalidRow);
    }
    let mut candidates = Vec::with_capacity(count);
    for header in bytes[2..].chunks_exact(7) {
        let candidate = CommunicationRecipientCandidateV1 {
            role: role_from_code(header[0])?,
            rationale: rationale_from_code(header[1])?,
            source_basis: source_basis_from_code(header[2])?,
            confidence_basis_points: u32::from_be_bytes(
                header[3..7]
                    .try_into()
                    .map_err(|_| CommunicationRecipientSuggestionPersistenceErrorV1::InvalidRow)?,
            ),
        };
        if !valid_candidate(&candidate) {
            return Err(CommunicationRecipientSuggestionPersistenceErrorV1::InvalidRow);
        }
        candidates.push(candidate);
    }
    Ok(candidates)
}

const fn role_code(value: CommunicationRecipientRoleV1) -> u8 {
    match value {
        CommunicationRecipientRoleV1::AccountingOrBookkeeping => 1,
        CommunicationRecipientRoleV1::LegalCounsel => 2,
        CommunicationRecipientRoleV1::ProjectStakeholder => 3,
    }
}

fn role_from_code(
    value: u8,
) -> Result<CommunicationRecipientRoleV1, CommunicationRecipientSuggestionPersistenceErrorV1> {
    match value {
        1 => Ok(CommunicationRecipientRoleV1::AccountingOrBookkeeping),
        2 => Ok(CommunicationRecipientRoleV1::LegalCounsel),
        3 => Ok(CommunicationRecipientRoleV1::ProjectStakeholder),
        _ => Err(CommunicationRecipientSuggestionPersistenceErrorV1::InvalidRow),
    }
}

const fn rationale_code(value: CommunicationRecipientRationaleV1) -> u8 {
    match value {
        CommunicationRecipientRationaleV1::FinancialDocumentOrPayment => 1,
        CommunicationRecipientRationaleV1::LegalOrContractualReview => 2,
        CommunicationRecipientRationaleV1::ProjectStatusOrUpdate => 3,
    }
}

fn rationale_from_code(
    value: u8,
) -> Result<CommunicationRecipientRationaleV1, CommunicationRecipientSuggestionPersistenceErrorV1> {
    match value {
        1 => Ok(CommunicationRecipientRationaleV1::FinancialDocumentOrPayment),
        2 => Ok(CommunicationRecipientRationaleV1::LegalOrContractualReview),
        3 => Ok(CommunicationRecipientRationaleV1::ProjectStatusOrUpdate),
        _ => Err(CommunicationRecipientSuggestionPersistenceErrorV1::InvalidRow),
    }
}

const fn source_basis_code(value: CommunicationRecipientSourceBasisV1) -> u8 {
    match value {
        CommunicationRecipientSourceBasisV1::Body => 1,
    }
}

fn source_basis_from_code(
    value: u8,
) -> Result<CommunicationRecipientSourceBasisV1, CommunicationRecipientSuggestionPersistenceErrorV1>
{
    match value {
        1 => Ok(CommunicationRecipientSourceBasisV1::Body),
        _ => Err(CommunicationRecipientSuggestionPersistenceErrorV1::InvalidRow),
    }
}

fn valid_candidate(candidate: &CommunicationRecipientCandidateV1) -> bool {
    candidate.confidence_basis_points <= COMMUNICATION_RECIPIENT_MAX_CONFIDENCE_BASIS_POINTS_V1
        && matches!(
            candidate.source_basis,
            CommunicationRecipientSourceBasisV1::Body
        )
        && matches!(
            (candidate.role, candidate.rationale),
            (
                CommunicationRecipientRoleV1::AccountingOrBookkeeping,
                CommunicationRecipientRationaleV1::FinancialDocumentOrPayment
            ) | (
                CommunicationRecipientRoleV1::LegalCounsel,
                CommunicationRecipientRationaleV1::LegalOrContractualReview
            ) | (
                CommunicationRecipientRoleV1::ProjectStakeholder,
                CommunicationRecipientRationaleV1::ProjectStatusOrUpdate
            )
        )
}

pub(crate) const fn rejection_code(value: CommunicationRecipientSuggestionRejectionCodeV1) -> i16 {
    match value {
        CommunicationRecipientSuggestionRejectionCodeV1::InvalidRequest => 1,
        CommunicationRecipientSuggestionRejectionCodeV1::SourceRejected => 2,
        CommunicationRecipientSuggestionRejectionCodeV1::EvaluationRejected => 3,
        CommunicationRecipientSuggestionRejectionCodeV1::Policy => 4,
    }
}

pub(crate) fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

pub(crate) fn valid_timestamp(value: i64) -> bool {
    value > 0
}

pub(crate) fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_fingerprint_is_stable_and_excludes_run_identity() {
        let draft = CommunicationRecipientSuggestionDraftV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            source_message_id: [3; 16],
            expected_source_revision: 4,
        };
        let mut replay = draft.clone();
        replay.run_id = [9; 16];
        assert_eq!(request_fingerprint(&draft), request_fingerprint(&replay));
        replay.expected_source_revision = 5;
        assert_ne!(request_fingerprint(&draft), request_fingerprint(&replay));
    }

    #[test]
    fn candidate_codec_preserves_typed_order_and_empty_result() {
        let candidates = vec![CommunicationRecipientCandidateV1 {
            role: CommunicationRecipientRoleV1::LegalCounsel,
            rationale: CommunicationRecipientRationaleV1::LegalOrContractualReview,
            source_basis: CommunicationRecipientSourceBasisV1::Body,
            confidence_basis_points: 8_500,
        }];
        let encoded = encode_candidates(&candidates).expect("encode");
        assert_eq!(decode_candidates(&encoded), Ok(candidates));
        assert_eq!(
            decode_candidates(&encode_candidates(&[]).expect("empty")),
            Ok(vec![])
        );
    }
}
