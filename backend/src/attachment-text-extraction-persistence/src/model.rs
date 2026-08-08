use makosh_attachment_text_extraction_core::{
    AttachmentTextCustodyDelegationIntentV1, AttachmentTextExtractionErrorV1,
    AttachmentTextExtractionRequestV1, AttachmentTextExtractionStateV1,
    AttachmentTextExtractionStatusV1, AttachmentTextFormatV1,
};
use sha2::{Digest, Sha256};

use crate::AttachmentTextExtractionPersistenceErrorV1;

pub const ATTACHMENT_TEXT_EXTRACTION_REALTIME_LIMIT_V1: u32 = 512;
pub const ATTACHMENT_TEXT_EXTRACTION_MAX_ATTEMPTS_V1: u32 = 8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateAttachmentTextExtractionRunV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedAttachmentTextExtractionRunV1 {
    pub logical_owner_id: String,
    pub request: AttachmentTextExtractionRequestV1,
    pub request_fingerprint: [u8; 32],
    pub status: AttachmentTextExtractionStatusV1,
    pub created_at_unix_millis: i64,
    pub updated_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreateAttachmentTextExtractionRunOutcomeV1 {
    Created(PersistedAttachmentTextExtractionRunV1),
    Replayed(PersistedAttachmentTextExtractionRunV1),
    OperationCollision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistAttachmentTextFactOutcomeV1 {
    Recorded { transitioned_runs: u32 },
    Replayed,
    Conflict { rejected_runs: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingAttachmentTextCustodyDelegationV1 {
    pub intent: AttachmentTextCustodyDelegationIntentV1,
    pub candidate_envelope_sha256: [u8; 32],
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnpublishedAttachmentTextCustodyDelegationV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub exact_envelope_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistAttachmentTextCustodyDelegationV1 {
    pub request_id: [u8; 16],
    pub run_id: [u8; 16],
    pub candidate_message_id: [u8; 16],
    pub safety_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub exact_envelope_bytes: Vec<u8>,
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistAttachmentTextCustodyResultOutcomeV1 {
    Recorded,
    Replayed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextExtractionLeaseV1 {
    pub worker_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub lease_fence: u64,
    pub lease_expires_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextExtractionTargetBlobReceiptV1 {
    pub reference_id: [u8; 16],
    pub receipt_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimedAttachmentTextExtractionJobV1 {
    pub logical_owner_id: String,
    pub job_id: [u8; 16],
    pub request: AttachmentTextExtractionRequestV1,
    pub delegation_request_id: [u8; 16],
    pub delegation_result_message_id: [u8; 16],
    pub delegation_result_envelope_sha256: [u8; 32],
    pub candidate_message_id: [u8; 16],
    pub safety_message_id: [u8; 16],
    pub source_reference_id: [u8; 16],
    pub target_blob_receipt: Option<TextExtractionTargetBlobReceiptV1>,
    pub source_receipt_sha256: [u8; 32],
    pub source_declared_size: u64,
    pub custody_transfer_source_proof: Vec<u8>,
    pub attempt_count: u32,
    pub max_attempts: u32,
    pub lease: TextExtractionLeaseV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PersistedAttachmentTextArtifactV1 {
    pub run_id: [u8; 16],
    pub derived_reference_id: [u8; 16],
    pub derived_receipt_sha256: [u8; 32],
    pub source_receipt_sha256: [u8; 32],
    pub parser_identity_sha256: [u8; 32],
    pub format: AttachmentTextFormatV1,
    pub extracted_size_bytes: u64,
    pub extraction_truncated: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextExtractionRealtimeTransitionV1 {
    pub sequence: u64,
    pub run_id: [u8; 16],
    pub state: AttachmentTextExtractionStateV1,
    pub state_revision: u64,
    pub format: Option<AttachmentTextFormatV1>,
    pub extracted_size_bytes: u64,
    pub extraction_truncated: bool,
    pub error: Option<AttachmentTextExtractionErrorV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TranslationSourceSnapshotV1 {
    pub source_revision: u64,
    pub artifact: PersistedAttachmentTextArtifactV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TranslationSourceSnapshotOutcomeV1 {
    Ready(TranslationSourceSnapshotV1),
    NotReady,
    StaleRevision,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistTranslationSourceResultV1 {
    pub request_message_id: [u8; 16],
    pub request_envelope_sha256: [u8; 32],
    pub request_id: [u8; 16],
    pub translation_run_id: [u8; 16],
    pub source_extraction_run_id: [u8; 16],
    pub expected_source_revision: u64,
    pub result_message_id: [u8; 16],
    pub result_envelope_sha256: [u8; 32],
    pub exact_result_envelope_bytes: Vec<u8>,
    pub processed_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistTranslationSourceResultOutcomeV1 {
    Recorded,
    Replayed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnpublishedTranslationSourceResultV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub exact_envelope_bytes: Vec<u8>,
}

#[must_use]
pub fn attachment_text_extraction_run_id_v1(
    logical_owner_id: &str,
    operation_id: [u8; 16],
) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.attachment-text-extraction.run.v1\0");
    hasher.update(logical_owner_id.as_bytes());
    hasher.update(operation_id);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

#[must_use]
pub fn attachment_text_extraction_request_fingerprint_v1(
    attachment_anchor_id: [u8; 16],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.attachment-text-extraction.request.v1\0");
    hasher.update(attachment_anchor_id);
    hasher.finalize().into()
}

#[must_use]
pub fn attachment_text_extraction_job_id_v1(
    request: &AttachmentTextExtractionRequestV1,
    delegation_request_id: [u8; 16],
    delegation_result_message_id: [u8; 16],
) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.attachment-text-extraction.job.v1\0");
    hasher.update(request.run_id);
    hasher.update(request.operation_id);
    hasher.update(request.attachment_anchor_id);
    hasher.update(delegation_request_id);
    hasher.update(delegation_result_message_id);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

pub(crate) fn validate_create(
    create: &CreateAttachmentTextExtractionRunV1,
) -> Result<(), AttachmentTextExtractionPersistenceErrorV1> {
    if !valid_owner(&create.logical_owner_id)
        || !valid_id16(&create.operation_id)
        || !valid_id16(&create.attachment_anchor_id)
        || create.created_at_unix_millis <= 0
    {
        return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

pub(crate) const fn state_code(value: AttachmentTextExtractionStateV1) -> i16 {
    match value {
        AttachmentTextExtractionStateV1::Accepted => 1,
        AttachmentTextExtractionStateV1::AwaitingEvidence => 2,
        AttachmentTextExtractionStateV1::Extracting => 3,
        AttachmentTextExtractionStateV1::Ready => 4,
        AttachmentTextExtractionStateV1::Unsupported => 5,
        AttachmentTextExtractionStateV1::Rejected => 6,
    }
}

pub(crate) fn state_from_code(
    value: i16,
) -> Result<AttachmentTextExtractionStateV1, AttachmentTextExtractionPersistenceErrorV1> {
    match value {
        1 => Ok(AttachmentTextExtractionStateV1::Accepted),
        2 => Ok(AttachmentTextExtractionStateV1::AwaitingEvidence),
        3 => Ok(AttachmentTextExtractionStateV1::Extracting),
        4 => Ok(AttachmentTextExtractionStateV1::Ready),
        5 => Ok(AttachmentTextExtractionStateV1::Unsupported),
        6 => Ok(AttachmentTextExtractionStateV1::Rejected),
        _ => Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) const fn format_code(value: AttachmentTextFormatV1) -> i16 {
    match value {
        AttachmentTextFormatV1::PlainUtf8 => 1,
        AttachmentTextFormatV1::Pdf => 2,
        AttachmentTextFormatV1::Docx => 3,
        AttachmentTextFormatV1::Ocr => 4,
    }
}

pub(crate) fn format_from_code(
    value: i16,
) -> Result<AttachmentTextFormatV1, AttachmentTextExtractionPersistenceErrorV1> {
    match value {
        1 => Ok(AttachmentTextFormatV1::PlainUtf8),
        2 => Ok(AttachmentTextFormatV1::Pdf),
        3 => Ok(AttachmentTextFormatV1::Docx),
        4 => Ok(AttachmentTextFormatV1::Ocr),
        _ => Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) const fn error_code(value: AttachmentTextExtractionErrorV1) -> i16 {
    match value {
        AttachmentTextExtractionErrorV1::NotSafe => 1,
        AttachmentTextExtractionErrorV1::Unsupported => 2,
        AttachmentTextExtractionErrorV1::SourceTooLarge => 3,
        AttachmentTextExtractionErrorV1::InvalidContent => 4,
        AttachmentTextExtractionErrorV1::ParserUnavailable => 5,
        AttachmentTextExtractionErrorV1::ParserFailed => 6,
        AttachmentTextExtractionErrorV1::CustodyRejected => 7,
        AttachmentTextExtractionErrorV1::Unavailable => 8,
    }
}

pub(crate) fn error_from_code(
    value: i16,
) -> Result<AttachmentTextExtractionErrorV1, AttachmentTextExtractionPersistenceErrorV1> {
    match value {
        1 => Ok(AttachmentTextExtractionErrorV1::NotSafe),
        2 => Ok(AttachmentTextExtractionErrorV1::Unsupported),
        3 => Ok(AttachmentTextExtractionErrorV1::SourceTooLarge),
        4 => Ok(AttachmentTextExtractionErrorV1::InvalidContent),
        5 => Ok(AttachmentTextExtractionErrorV1::ParserUnavailable),
        6 => Ok(AttachmentTextExtractionErrorV1::ParserFailed),
        7 => Ok(AttachmentTextExtractionErrorV1::CustodyRejected),
        8 => Ok(AttachmentTextExtractionErrorV1::Unavailable),
        _ => Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) fn valid_owner(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128
}

pub(crate) fn valid_worker(value: &str) -> bool {
    valid_owner(value)
}

pub(crate) const fn valid_timestamp_millis(value: i64) -> bool {
    value > 0
}

pub(crate) fn valid_id16(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn valid_sha256(value: &[u8; 32]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) const fn safety_state_code(
    value: makosh_attachment_text_extraction_core::AttachmentTextSafetyStateV1,
) -> i16 {
    use makosh_attachment_text_extraction_core::AttachmentTextSafetyStateV1;
    match value {
        AttachmentTextSafetyStateV1::DescriptorOnly => 1,
        AttachmentTextSafetyStateV1::BlobPending => 2,
        AttachmentTextSafetyStateV1::BlobAdmitted => 3,
        AttachmentTextSafetyStateV1::Quarantined => 4,
        AttachmentTextSafetyStateV1::SafeForDelivery => 5,
        AttachmentTextSafetyStateV1::Rejected => 6,
    }
}

pub(crate) fn safety_state_from_code(
    value: i16,
) -> Result<
    makosh_attachment_text_extraction_core::AttachmentTextSafetyStateV1,
    AttachmentTextExtractionPersistenceErrorV1,
> {
    use makosh_attachment_text_extraction_core::AttachmentTextSafetyStateV1;
    match value {
        1 => Ok(AttachmentTextSafetyStateV1::DescriptorOnly),
        2 => Ok(AttachmentTextSafetyStateV1::BlobPending),
        3 => Ok(AttachmentTextSafetyStateV1::BlobAdmitted),
        4 => Ok(AttachmentTextSafetyStateV1::Quarantined),
        5 => Ok(AttachmentTextSafetyStateV1::SafeForDelivery),
        6 => Ok(AttachmentTextSafetyStateV1::Rejected),
        _ => Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_id_is_owner_scoped_and_request_fingerprint_is_anchor_scoped() {
        let operation = [7; 16];
        assert_ne!(
            attachment_text_extraction_run_id_v1("alice", operation),
            attachment_text_extraction_run_id_v1("bob", operation)
        );
        assert_ne!(
            attachment_text_extraction_request_fingerprint_v1([1; 16]),
            attachment_text_extraction_request_fingerprint_v1([2; 16])
        );
    }

    #[test]
    fn job_identity_binds_request_and_custody_result() {
        let request = AttachmentTextExtractionRequestV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            attachment_anchor_id: [3; 16],
        };
        let first = attachment_text_extraction_job_id_v1(&request, [4; 16], [5; 16]);
        assert_eq!(
            first,
            attachment_text_extraction_job_id_v1(&request, [4; 16], [5; 16])
        );
        assert_ne!(
            first,
            attachment_text_extraction_job_id_v1(&request, [4; 16], [6; 16])
        );
    }
}
