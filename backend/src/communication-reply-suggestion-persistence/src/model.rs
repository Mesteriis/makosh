use makosh_communication_reply_suggestion_core::{
    ReplySuggestionDraftV1, ReplySuggestionRejectionCodeV1, ReplySuggestionStatusV1,
    ReplySuggestionTransitionV1,
};
use sha2::{Digest, Sha256};

pub const REPLY_SUGGESTION_RECOVERY_LIMIT_V1: u16 = 128;
pub const REPLY_SUGGESTION_REALTIME_LIMIT_V1: u16 = 1_024;
pub const REPLY_SUGGESTION_OUTBOX_LIMIT_V1: u16 = 128;
pub const REPLY_SUGGESTION_MAX_EVENT_BYTES_V1: usize = 64 * 1024;
pub const REPLY_SUGGESTION_MAX_INFERENCE_REQUEST_BYTES_V1: usize = 16 * 1024;
pub const REPLY_SUGGESTION_MAX_CUSTODY_PROOF_BYTES_V1: usize = 2_048;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplySuggestionBlobCleanupV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateReplySuggestionRunV1 {
    pub logical_owner_id: String,
    pub draft: ReplySuggestionDraftV1,
    pub source_prepare_message_id: [u8; 16],
    pub source_prepare_envelope_sha256: [u8; 32],
    pub source_prepare_envelope_bytes: Vec<u8>,
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedReplySuggestionRunV1 {
    pub logical_owner_id: String,
    pub draft: ReplySuggestionDraftV1,
    pub request_fingerprint: [u8; 32],
    pub status: ReplySuggestionStatusV1,
    pub inference_request_bytes: Option<Vec<u8>>,
    pub source_cleanup: Option<ReplySuggestionBlobCleanupV1>,
    pub cleanup_completed_at_unix_millis: Option<i64>,
    pub created_at_unix_millis: i64,
    pub updated_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreateReplySuggestionOutcomeV1 {
    Created(PersistedReplySuggestionRunV1),
    Existing(PersistedReplySuggestionRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplySuggestionSourceResultV1 {
    pub result_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub logical_owner_id: String,
    pub run_id: [u8; 16],
    pub transition: ReplySuggestionTransitionV1,
    pub inference_request_bytes: Option<Vec<u8>>,
    pub source_cleanup: Option<ReplySuggestionBlobCleanupV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplySuggestionInboxResultV1 {
    Applied(PersistedReplySuggestionRunV1),
    Duplicate(PersistedReplySuggestionRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnpublishedReplySuggestionEventV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplySuggestionPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    RequestConflict,
    InboxConflict,
    RevisionConflict,
    InvalidTransition,
    NotFound,
}

pub(crate) fn request_fingerprint(draft: &ReplySuggestionDraftV1) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.communication_reply_suggestion.start.v1\0");
    hash.update(draft.source_message_id);
    hash.update(draft.expected_source_revision.to_be_bytes());
    hash.update([tone_code(draft.tone)]);
    hash.update([language_code(draft.language)]);
    hash.finalize().into()
}

pub(crate) const fn tone_code(
    value: makosh_communication_reply_suggestion_core::ReplySuggestionToneV1,
) -> u8 {
    use makosh_communication_reply_suggestion_core::ReplySuggestionToneV1;
    match value {
        ReplySuggestionToneV1::Professional => 1,
        ReplySuggestionToneV1::Friendly => 2,
        ReplySuggestionToneV1::Concise => 3,
        ReplySuggestionToneV1::Formal => 4,
    }
}

pub(crate) const fn language_code(
    value: makosh_communication_reply_suggestion_core::ReplySuggestionLanguageV1,
) -> u8 {
    use makosh_communication_reply_suggestion_core::ReplySuggestionLanguageV1;
    match value {
        ReplySuggestionLanguageV1::Source => 1,
        ReplySuggestionLanguageV1::English => 2,
        ReplySuggestionLanguageV1::Russian => 3,
        ReplySuggestionLanguageV1::Spanish => 4,
    }
}

pub(crate) const fn rejection_code(value: ReplySuggestionRejectionCodeV1) -> i16 {
    match value {
        ReplySuggestionRejectionCodeV1::InvalidRequest => 1,
        ReplySuggestionRejectionCodeV1::SourceRejected => 2,
        ReplySuggestionRejectionCodeV1::InferenceRejected => 3,
        ReplySuggestionRejectionCodeV1::Policy => 4,
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
    use makosh_communication_reply_suggestion_core::{
        ReplySuggestionLanguageV1, ReplySuggestionToneV1,
    };

    use super::*;

    #[test]
    fn request_fingerprint_is_stable_and_excludes_run_identity() {
        let draft = ReplySuggestionDraftV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            source_message_id: [3; 16],
            expected_source_revision: 4,
            tone: ReplySuggestionToneV1::Formal,
            language: ReplySuggestionLanguageV1::Spanish,
        };
        let mut replay = draft.clone();
        replay.run_id = [9; 16];
        assert_eq!(request_fingerprint(&draft), request_fingerprint(&replay));
        replay.expected_source_revision += 1;
        assert_ne!(request_fingerprint(&draft), request_fingerprint(&replay));
    }
}
