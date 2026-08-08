use makosh_communication_translation_core::{
    CommunicationTranslationDraftV1, CommunicationTranslationLanguageV1,
    CommunicationTranslationRejectionCodeV1, CommunicationTranslationStatusV1,
    CommunicationTranslationTransitionV1,
};
use sha2::{Digest, Sha256};

pub const COMMUNICATION_TRANSLATION_RECOVERY_LIMIT_V1: u16 = 128;
pub const COMMUNICATION_TRANSLATION_REALTIME_LIMIT_V1: u16 = 1_024;
pub const COMMUNICATION_TRANSLATION_OUTBOX_LIMIT_V1: u16 = 128;
pub const COMMUNICATION_TRANSLATION_MAX_EVENT_BYTES_V1: usize = 64 * 1024;
pub const COMMUNICATION_TRANSLATION_MAX_INFERENCE_REQUEST_BYTES_V1: usize = 16 * 1024;
pub const COMMUNICATION_TRANSLATION_MAX_CUSTODY_PROOF_BYTES_V1: usize = 2_048;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationTranslationBlobCleanupV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateCommunicationTranslationRunV1 {
    pub logical_owner_id: String,
    pub draft: CommunicationTranslationDraftV1,
    pub source_prepare_message_id: [u8; 16],
    pub source_prepare_envelope_sha256: [u8; 32],
    pub source_prepare_envelope_bytes: Vec<u8>,
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedCommunicationTranslationRunV1 {
    pub logical_owner_id: String,
    pub draft: CommunicationTranslationDraftV1,
    pub request_fingerprint: [u8; 32],
    pub status: CommunicationTranslationStatusV1,
    pub inference_request_bytes: Option<Vec<u8>>,
    pub source_cleanup: Option<CommunicationTranslationBlobCleanupV1>,
    pub cleanup_completed_at_unix_millis: Option<i64>,
    pub created_at_unix_millis: i64,
    pub updated_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreateCommunicationTranslationOutcomeV1 {
    Created(PersistedCommunicationTranslationRunV1),
    Existing(PersistedCommunicationTranslationRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationTranslationSourceResultV1 {
    pub result_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub logical_owner_id: String,
    pub run_id: [u8; 16],
    pub transition: CommunicationTranslationTransitionV1,
    pub inference_request_bytes: Option<Vec<u8>>,
    pub source_cleanup: Option<CommunicationTranslationBlobCleanupV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunicationTranslationInboxResultV1 {
    Applied(PersistedCommunicationTranslationRunV1),
    Duplicate(PersistedCommunicationTranslationRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnpublishedCommunicationTranslationEventV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationTranslationPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    RequestConflict,
    InboxConflict,
    RevisionConflict,
    InvalidTransition,
    NotFound,
}

pub(crate) fn request_fingerprint(draft: &CommunicationTranslationDraftV1) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.communication_translation.start.v1\0");
    hash.update(draft.source_message_id);
    hash.update(draft.expected_source_revision.to_be_bytes());
    hash.update([target_language_code(draft.target_language)]);
    hash.finalize().into()
}

pub(crate) const fn target_language_code(value: CommunicationTranslationLanguageV1) -> u8 {
    match value {
        CommunicationTranslationLanguageV1::English => 1,
        CommunicationTranslationLanguageV1::Russian => 2,
        CommunicationTranslationLanguageV1::Spanish => 3,
    }
}

pub(crate) const fn rejection_code(value: CommunicationTranslationRejectionCodeV1) -> i16 {
    match value {
        CommunicationTranslationRejectionCodeV1::InvalidRequest => 1,
        CommunicationTranslationRejectionCodeV1::SourceRejected => 2,
        CommunicationTranslationRejectionCodeV1::InferenceRejected => 3,
        CommunicationTranslationRejectionCodeV1::Policy => 4,
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
        let draft = CommunicationTranslationDraftV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            source_message_id: [3; 16],
            expected_source_revision: 4,
            target_language: CommunicationTranslationLanguageV1::Spanish,
        };
        let mut replay = draft.clone();
        replay.run_id = [9; 16];
        assert_eq!(request_fingerprint(&draft), request_fingerprint(&replay));
        replay.target_language = CommunicationTranslationLanguageV1::Russian;
        assert_ne!(request_fingerprint(&draft), request_fingerprint(&replay));
    }
}
