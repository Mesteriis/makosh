use makosh_communication_explanation_core::{
    COMMUNICATION_EXPLANATION_MAX_REASON_TEXT_BYTES_V1, COMMUNICATION_EXPLANATION_MAX_REASONS_V1,
    CommunicationExplanationDraftV1, CommunicationExplanationReasonKindV1,
    CommunicationExplanationReasonV1, CommunicationExplanationRejectionCodeV1,
    CommunicationExplanationSourceBasisV1, CommunicationExplanationStatusV1,
    CommunicationExplanationTransitionV1,
};
use sha2::{Digest, Sha256};

pub const COMMUNICATION_EXPLANATION_RECOVERY_LIMIT_V1: u16 = 128;
pub const COMMUNICATION_EXPLANATION_REALTIME_LIMIT_V1: u16 = 1_024;
pub const COMMUNICATION_EXPLANATION_OUTBOX_LIMIT_V1: u16 = 128;
pub const COMMUNICATION_EXPLANATION_MAX_EVENT_BYTES_V1: usize = 64 * 1024;
pub const COMMUNICATION_EXPLANATION_MAX_INFERENCE_REQUEST_BYTES_V1: usize = 16 * 1024;
pub const COMMUNICATION_EXPLANATION_MAX_CUSTODY_PROOF_BYTES_V1: usize = 2_048;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationExplanationBlobCleanupV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateCommunicationExplanationRunV1 {
    pub logical_owner_id: String,
    pub draft: CommunicationExplanationDraftV1,
    pub source_prepare_message_id: [u8; 16],
    pub source_prepare_envelope_sha256: [u8; 32],
    pub source_prepare_envelope_bytes: Vec<u8>,
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedCommunicationExplanationRunV1 {
    pub logical_owner_id: String,
    pub draft: CommunicationExplanationDraftV1,
    pub request_fingerprint: [u8; 32],
    pub status: CommunicationExplanationStatusV1,
    pub inference_request_bytes: Option<Vec<u8>>,
    pub source_cleanup: Option<CommunicationExplanationBlobCleanupV1>,
    pub cleanup_completed_at_unix_millis: Option<i64>,
    pub created_at_unix_millis: i64,
    pub updated_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreateCommunicationExplanationOutcomeV1 {
    Created(PersistedCommunicationExplanationRunV1),
    Existing(PersistedCommunicationExplanationRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationExplanationSourceResultV1 {
    pub result_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub logical_owner_id: String,
    pub run_id: [u8; 16],
    pub transition: CommunicationExplanationTransitionV1,
    pub inference_request_bytes: Option<Vec<u8>>,
    pub source_cleanup: Option<CommunicationExplanationBlobCleanupV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunicationExplanationInboxResultV1 {
    Applied(PersistedCommunicationExplanationRunV1),
    Duplicate(PersistedCommunicationExplanationRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnpublishedCommunicationExplanationEventV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationExplanationPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    RequestConflict,
    InboxConflict,
    RevisionConflict,
    InvalidTransition,
    NotFound,
}

pub(crate) fn request_fingerprint(draft: &CommunicationExplanationDraftV1) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.communication_explanation.start.v1\0");
    hash.update(draft.source_message_id);
    hash.update(draft.expected_source_revision.to_be_bytes());
    hash.finalize().into()
}

pub(crate) fn encode_reasons(
    reasons: &[CommunicationExplanationReasonV1],
) -> Result<Vec<u8>, CommunicationExplanationPersistenceErrorV1> {
    if reasons.len() > COMMUNICATION_EXPLANATION_MAX_REASONS_V1 {
        return Err(CommunicationExplanationPersistenceErrorV1::InvalidInput);
    }
    let mut bytes = Vec::with_capacity(2 + reasons.len() * 8);
    bytes.push(1);
    bytes.push(
        u8::try_from(reasons.len())
            .map_err(|_| CommunicationExplanationPersistenceErrorV1::InvalidInput)?,
    );
    for reason in reasons {
        if reason.explanation_utf8.is_empty()
            || reason.explanation_utf8.len() > COMMUNICATION_EXPLANATION_MAX_REASON_TEXT_BYTES_V1
            || std::str::from_utf8(&reason.explanation_utf8).is_err()
            || reason.confidence_basis_points > 10_000
        {
            return Err(CommunicationExplanationPersistenceErrorV1::InvalidInput);
        }
        bytes.push(reason_kind_code(reason.kind));
        bytes.push(source_basis_code(reason.source_basis));
        bytes.extend_from_slice(&reason.confidence_basis_points.to_be_bytes());
        bytes.extend_from_slice(
            &u16::try_from(reason.explanation_utf8.len())
                .map_err(|_| CommunicationExplanationPersistenceErrorV1::InvalidInput)?
                .to_be_bytes(),
        );
        bytes.extend_from_slice(&reason.explanation_utf8);
    }
    Ok(bytes)
}

pub(crate) fn decode_reasons(
    bytes: &[u8],
) -> Result<Vec<CommunicationExplanationReasonV1>, CommunicationExplanationPersistenceErrorV1> {
    if bytes.len() < 2 || bytes[0] != 1 {
        return Err(CommunicationExplanationPersistenceErrorV1::InvalidRow);
    }
    let count = usize::from(bytes[1]);
    if count > COMMUNICATION_EXPLANATION_MAX_REASONS_V1 {
        return Err(CommunicationExplanationPersistenceErrorV1::InvalidRow);
    }
    let mut offset = 2;
    let mut reasons = Vec::with_capacity(count);
    for _ in 0..count {
        let header = bytes
            .get(offset..offset + 8)
            .ok_or(CommunicationExplanationPersistenceErrorV1::InvalidRow)?;
        let confidence = u32::from_be_bytes(
            header[2..6]
                .try_into()
                .map_err(|_| CommunicationExplanationPersistenceErrorV1::InvalidRow)?,
        );
        let length =
            usize::from(u16::from_be_bytes(header[6..8].try_into().map_err(
                |_| CommunicationExplanationPersistenceErrorV1::InvalidRow,
            )?));
        offset += 8;
        let explanation = bytes
            .get(offset..offset + length)
            .ok_or(CommunicationExplanationPersistenceErrorV1::InvalidRow)?
            .to_vec();
        offset += length;
        if explanation.is_empty()
            || explanation.len() > COMMUNICATION_EXPLANATION_MAX_REASON_TEXT_BYTES_V1
            || std::str::from_utf8(&explanation).is_err()
            || confidence > 10_000
        {
            return Err(CommunicationExplanationPersistenceErrorV1::InvalidRow);
        }
        reasons.push(CommunicationExplanationReasonV1 {
            kind: reason_kind_from_code(header[0])?,
            explanation_utf8: explanation,
            source_basis: source_basis_from_code(header[1])?,
            confidence_basis_points: confidence,
        });
    }
    if offset != bytes.len() {
        return Err(CommunicationExplanationPersistenceErrorV1::InvalidRow);
    }
    Ok(reasons)
}

const fn reason_kind_code(value: CommunicationExplanationReasonKindV1) -> u8 {
    match value {
        CommunicationExplanationReasonKindV1::Urgency => 1,
        CommunicationExplanationReasonKindV1::FinancialAttention => 2,
        CommunicationExplanationReasonKindV1::LegalOrContractual => 3,
        CommunicationExplanationReasonKindV1::ReplyRequested => 4,
        CommunicationExplanationReasonKindV1::Deadline => 5,
        CommunicationExplanationReasonKindV1::AttachmentReference => 6,
        CommunicationExplanationReasonKindV1::MarketingOrBulk => 7,
        CommunicationExplanationReasonKindV1::OtherAttention => 8,
    }
}

fn reason_kind_from_code(
    value: u8,
) -> Result<CommunicationExplanationReasonKindV1, CommunicationExplanationPersistenceErrorV1> {
    match value {
        1 => Ok(CommunicationExplanationReasonKindV1::Urgency),
        2 => Ok(CommunicationExplanationReasonKindV1::FinancialAttention),
        3 => Ok(CommunicationExplanationReasonKindV1::LegalOrContractual),
        4 => Ok(CommunicationExplanationReasonKindV1::ReplyRequested),
        5 => Ok(CommunicationExplanationReasonKindV1::Deadline),
        6 => Ok(CommunicationExplanationReasonKindV1::AttachmentReference),
        7 => Ok(CommunicationExplanationReasonKindV1::MarketingOrBulk),
        8 => Ok(CommunicationExplanationReasonKindV1::OtherAttention),
        _ => Err(CommunicationExplanationPersistenceErrorV1::InvalidRow),
    }
}

const fn source_basis_code(value: CommunicationExplanationSourceBasisV1) -> u8 {
    match value {
        CommunicationExplanationSourceBasisV1::Subject => 1,
        CommunicationExplanationSourceBasisV1::Body => 2,
        CommunicationExplanationSourceBasisV1::CanonicalMetadata => 3,
        CommunicationExplanationSourceBasisV1::Combined => 4,
    }
}

fn source_basis_from_code(
    value: u8,
) -> Result<CommunicationExplanationSourceBasisV1, CommunicationExplanationPersistenceErrorV1> {
    match value {
        1 => Ok(CommunicationExplanationSourceBasisV1::Subject),
        2 => Ok(CommunicationExplanationSourceBasisV1::Body),
        3 => Ok(CommunicationExplanationSourceBasisV1::CanonicalMetadata),
        4 => Ok(CommunicationExplanationSourceBasisV1::Combined),
        _ => Err(CommunicationExplanationPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) const fn rejection_code(value: CommunicationExplanationRejectionCodeV1) -> i16 {
    match value {
        CommunicationExplanationRejectionCodeV1::InvalidRequest => 1,
        CommunicationExplanationRejectionCodeV1::SourceRejected => 2,
        CommunicationExplanationRejectionCodeV1::InferenceRejected => 3,
        CommunicationExplanationRejectionCodeV1::Policy => 4,
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
        let draft = CommunicationExplanationDraftV1 {
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
    fn reason_codec_preserves_typed_order_and_empty_candidate() {
        let reasons = vec![CommunicationExplanationReasonV1 {
            kind: CommunicationExplanationReasonKindV1::Deadline,
            explanation_utf8: b"Deadline tomorrow".to_vec(),
            source_basis: CommunicationExplanationSourceBasisV1::Body,
            confidence_basis_points: 8_500,
        }];
        let encoded = encode_reasons(&reasons).expect("encode");
        assert_eq!(decode_reasons(&encoded), Ok(reasons));
        assert_eq!(
            decode_reasons(&encode_reasons(&[]).expect("empty")),
            Ok(vec![])
        );
    }
}
