use makosh_attachment_translation_core::{
    AttachmentTranslationDraftV1, AttachmentTranslationLanguageV1,
    AttachmentTranslationRejectionCodeV1, AttachmentTranslationStatusV1,
    AttachmentTranslationTransitionV1,
};
use sha2::{Digest, Sha256};

pub const ATTACHMENT_TRANSLATION_RECOVERY_LIMIT_V1: u16 = 128;
pub const ATTACHMENT_TRANSLATION_REALTIME_LIMIT_V1: u16 = 1_024;
pub const ATTACHMENT_TRANSLATION_OUTBOX_LIMIT_V1: u16 = 128;
pub const ATTACHMENT_TRANSLATION_MAX_EVENT_BYTES_V1: usize = 64 * 1024;
pub const ATTACHMENT_TRANSLATION_MAX_INFERENCE_REQUEST_BYTES_V1: usize = 16 * 1024;
pub const ATTACHMENT_TRANSLATION_MAX_CUSTODY_PROOF_BYTES_V1: usize = 2_048;
pub const ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1: u64 = 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentTranslationSourceAuthorityV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateAttachmentTranslationRunV1 {
    pub logical_owner_id: String,
    pub draft: AttachmentTranslationDraftV1,
    pub source_prepare_message_id: [u8; 16],
    pub source_prepare_envelope_sha256: [u8; 32],
    pub source_prepare_envelope_bytes: Vec<u8>,
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedAttachmentTranslationRunV1 {
    pub logical_owner_id: String,
    pub draft: AttachmentTranslationDraftV1,
    pub request_fingerprint: [u8; 32],
    pub status: AttachmentTranslationStatusV1,
    pub inference_request_bytes: Option<Vec<u8>>,
    pub source_authority: Option<AttachmentTranslationSourceAuthorityV1>,
    pub cleanup_completed_at_unix_millis: Option<i64>,
    pub created_at_unix_millis: i64,
    pub updated_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreateAttachmentTranslationOutcomeV1 {
    Created(PersistedAttachmentTranslationRunV1),
    Existing(PersistedAttachmentTranslationRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentTranslationSourceResultV1 {
    pub result_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub logical_owner_id: String,
    pub run_id: [u8; 16],
    pub transition: AttachmentTranslationTransitionV1,
    pub inference_request_bytes: Option<Vec<u8>>,
    pub source_authority: Option<AttachmentTranslationSourceAuthorityV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentTranslationInferenceResultV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub logical_owner_id: String,
    pub run_id: [u8; 16],
    pub transition: AttachmentTranslationTransitionV1,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentTranslationMaterializationResultV1 {
    pub message_id: [u8; 16],
    pub result_sha256: [u8; 32],
    pub logical_owner_id: String,
    pub run_id: [u8; 16],
    pub transition: AttachmentTranslationTransitionV1,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IssueAttachmentTranslationTicketV1 {
    pub ticket_sha256: [u8; 32],
    pub device_actor_sha256: [u8; 32],
    pub run_id: [u8; 16],
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub now_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IssuedAttachmentTranslationTicketV1 {
    pub run_id: [u8; 16],
    pub expires_at_unix_seconds: i64,
    pub translated_size_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RedeemedAttachmentTranslationTicketV1 {
    pub run_id: [u8; 16],
    pub artifact_reference_id: [u8; 16],
    pub artifact_receipt_sha256: [u8; 32],
    pub translated_size_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttachmentTranslationInboxResultV1 {
    Applied(PersistedAttachmentTranslationRunV1),
    Duplicate(PersistedAttachmentTranslationRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnpublishedAttachmentTranslationEventV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTranslationPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    RequestConflict,
    InboxConflict,
    RevisionConflict,
    InvalidTransition,
    NotFound,
    TicketExpired,
    TicketUsed,
    StaleFence,
}

pub(crate) fn request_fingerprint(draft: &AttachmentTranslationDraftV1) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.attachment_translation.start.v1\0");
    hash.update(draft.source_extraction_run_id);
    hash.update(draft.expected_source_revision.to_be_bytes());
    hash.update([target_language_code(draft.target_language)]);
    hash.finalize().into()
}

pub(crate) const fn target_language_code(value: AttachmentTranslationLanguageV1) -> u8 {
    match value {
        AttachmentTranslationLanguageV1::English => 1,
        AttachmentTranslationLanguageV1::Russian => 2,
        AttachmentTranslationLanguageV1::Spanish => 3,
    }
}

pub(crate) const fn rejection_code(value: AttachmentTranslationRejectionCodeV1) -> i16 {
    match value {
        AttachmentTranslationRejectionCodeV1::InvalidRequest => 1,
        AttachmentTranslationRejectionCodeV1::SourceRejected => 2,
        AttachmentTranslationRejectionCodeV1::InferenceRejected => 3,
        AttachmentTranslationRejectionCodeV1::ResultRejected => 4,
        AttachmentTranslationRejectionCodeV1::Policy => 5,
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
    fn request_fingerprint_is_stable_excludes_run_identity_and_binds_target_language() {
        let draft = AttachmentTranslationDraftV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            source_extraction_run_id: [3; 16],
            expected_source_revision: 4,
            target_language: AttachmentTranslationLanguageV1::Spanish,
        };
        let mut replay = draft.clone();
        replay.run_id = [9; 16];
        assert_eq!(request_fingerprint(&draft), request_fingerprint(&replay));
        replay.target_language = AttachmentTranslationLanguageV1::Russian;
        assert_ne!(request_fingerprint(&draft), request_fingerprint(&replay));
    }
}
