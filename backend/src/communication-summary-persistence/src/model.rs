use makosh_communication_summary_core::{
    CommunicationSummaryDraftV1, CommunicationSummaryRejectionCodeV1, CommunicationSummaryStatusV1,
    CommunicationSummaryTransitionV1,
};
use sha2::{Digest, Sha256};

pub const COMMUNICATION_SUMMARY_RECOVERY_LIMIT_V1: u16 = 128;
pub const COMMUNICATION_SUMMARY_REALTIME_LIMIT_V1: u16 = 1_024;
pub const COMMUNICATION_SUMMARY_OUTBOX_LIMIT_V1: u16 = 128;
pub const COMMUNICATION_SUMMARY_MAX_EVENT_BYTES_V1: usize = 64 * 1024;
pub const COMMUNICATION_SUMMARY_MAX_INFERENCE_REQUEST_BYTES_V1: usize = 16 * 1024;
pub const COMMUNICATION_SUMMARY_MAX_CUSTODY_PROOF_BYTES_V1: usize = 2_048;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationSummaryBlobCleanupV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateCommunicationSummaryRunV1 {
    pub logical_owner_id: String,
    pub draft: CommunicationSummaryDraftV1,
    pub source_prepare_message_id: [u8; 16],
    pub source_prepare_envelope_sha256: [u8; 32],
    pub source_prepare_envelope_bytes: Vec<u8>,
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedCommunicationSummaryRunV1 {
    pub logical_owner_id: String,
    pub draft: CommunicationSummaryDraftV1,
    pub request_fingerprint: [u8; 32],
    pub status: CommunicationSummaryStatusV1,
    pub inference_request_bytes: Option<Vec<u8>>,
    pub source_cleanup: Option<CommunicationSummaryBlobCleanupV1>,
    pub cleanup_completed_at_unix_millis: Option<i64>,
    pub created_at_unix_millis: i64,
    pub updated_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreateCommunicationSummaryOutcomeV1 {
    Created(PersistedCommunicationSummaryRunV1),
    Existing(PersistedCommunicationSummaryRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationSummarySourceResultV1 {
    pub result_message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub logical_owner_id: String,
    pub run_id: [u8; 16],
    pub transition: CommunicationSummaryTransitionV1,
    pub inference_request_bytes: Option<Vec<u8>>,
    pub source_cleanup: Option<CommunicationSummaryBlobCleanupV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunicationSummaryInboxResultV1 {
    Applied(PersistedCommunicationSummaryRunV1),
    Duplicate(PersistedCommunicationSummaryRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnpublishedCommunicationSummaryEventV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationSummaryPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    RequestConflict,
    InboxConflict,
    RevisionConflict,
    InvalidTransition,
    NotFound,
}

pub(crate) fn request_fingerprint(draft: &CommunicationSummaryDraftV1) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.communication_summary.start.v1\0");
    hash.update(draft.source_message_id);
    hash.update(draft.expected_source_revision.to_be_bytes());
    hash.update([length_code(draft.length)]);
    hash.update([language_code(draft.language)]);
    hash.finalize().into()
}

pub(crate) const fn length_code(
    value: makosh_communication_summary_core::CommunicationSummaryLengthV1,
) -> u8 {
    use makosh_communication_summary_core::CommunicationSummaryLengthV1;
    match value {
        CommunicationSummaryLengthV1::Short => 1,
        CommunicationSummaryLengthV1::Standard => 2,
        CommunicationSummaryLengthV1::Detailed => 3,
    }
}

pub(crate) const fn language_code(
    value: makosh_communication_summary_core::CommunicationSummaryLanguageV1,
) -> u8 {
    use makosh_communication_summary_core::CommunicationSummaryLanguageV1;
    match value {
        CommunicationSummaryLanguageV1::Auto => 1,
        CommunicationSummaryLanguageV1::English => 2,
        CommunicationSummaryLanguageV1::Russian => 3,
        CommunicationSummaryLanguageV1::Spanish => 4,
    }
}

pub(crate) const fn rejection_code(value: CommunicationSummaryRejectionCodeV1) -> i16 {
    match value {
        CommunicationSummaryRejectionCodeV1::InvalidRequest => 1,
        CommunicationSummaryRejectionCodeV1::SourceRejected => 2,
        CommunicationSummaryRejectionCodeV1::InferenceRejected => 3,
        CommunicationSummaryRejectionCodeV1::Policy => 4,
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
    use makosh_communication_summary_core::{
        CommunicationSummaryLanguageV1, CommunicationSummaryLengthV1,
    };

    use super::*;

    #[test]
    fn request_fingerprint_is_stable_and_excludes_run_identity() {
        let draft = CommunicationSummaryDraftV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            source_message_id: [3; 16],
            expected_source_revision: 4,
            length: CommunicationSummaryLengthV1::Detailed,
            language: CommunicationSummaryLanguageV1::Spanish,
        };
        let mut replay = draft.clone();
        replay.run_id = [9; 16];
        assert_eq!(request_fingerprint(&draft), request_fingerprint(&replay));
        replay.expected_source_revision += 1;
        assert_ne!(request_fingerprint(&draft), request_fingerprint(&replay));
    }
}
