use makosh_call_transcription_core::{
    CallTranscriptionDraftV1, CallTranscriptionRejectionV1, CallTranscriptionStatusV1,
    CallTranscriptionTransitionV1, RecordingSourceV1,
};
use sha2::{Digest, Sha256};

pub const CALL_TRANSCRIPTION_RECOVERY_LIMIT_V1: u32 = 128;
pub const CALL_TRANSCRIPTION_REALTIME_LIMIT_V1: u32 = 256;
pub const CALL_TRANSCRIPTION_OUTBOX_LIMIT_V1: u32 = 128;
pub const CALL_TRANSCRIPTION_MAX_EVENT_BYTES_V1: usize = 64 * 1024;
pub const CALL_TRANSCRIPTION_MAX_ATTEMPTS_V1: u32 = 5;
pub const CALL_TRANSCRIPTION_MAX_LEASE_MILLIS_V1: u64 = 300_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateCallTranscriptionRunV1 {
    pub logical_owner_id: String,
    pub draft: CallTranscriptionDraftV1,
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreateCallTranscriptionRunOutcomeV1 {
    Created(PersistedCallTranscriptionRunV1),
    Existing(PersistedCallTranscriptionRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedRecordingSourceV1 {
    pub source: RecordingSourceV1,
    pub source_receipt_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PersistedTranscriptBlobV1 {
    pub reference_id: [u8; 16],
    pub receipt_sha256: [u8; 32],
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedCallTranscriptionRunV1 {
    pub logical_owner_id: String,
    pub run_id: [u8; 16],
    pub request_fingerprint: [u8; 32],
    pub draft: CallTranscriptionDraftV1,
    pub status: CallTranscriptionStatusV1,
    pub recording_source: Option<PersistedRecordingSourceV1>,
    pub source_cleanup_completed_at_unix_millis: Option<i64>,
    pub stt_request_id: Option<[u8; 16]>,
    pub stt_result_receipt_sha256: Option<[u8; 32]>,
    pub artifact_blob: Option<PersistedTranscriptBlobV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableOutboxRecordV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordingIngressOutcomeV1 {
    Ready {
        source: Box<RecordingSourceV1>,
        source_receipt_sha256: [u8; 32],
        stt_request_id: [u8; 16],
        stt_request_digest: [u8; 32],
    },
    Rejected(CallTranscriptionRejectionV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistRecordingIngressV1 {
    pub logical_owner_id: String,
    pub run_id: [u8; 16],
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub outcome: RecordingIngressOutcomeV1,
    pub outbox: Option<DurableOutboxRecordV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallTranscriptionInboxOutcomeV1 {
    Applied(PersistedCallTranscriptionRunV1),
    Duplicate(PersistedCallTranscriptionRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallTranscriptionJobLeaseV1 {
    pub worker_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub lease_fence: u64,
    pub lease_expires_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimedCallTranscriptionJobV1 {
    pub logical_owner_id: String,
    pub job_id: [u8; 16],
    pub run_id: [u8; 16],
    pub stt_request_id: [u8; 16],
    pub stt_request_digest: [u8; 32],
    pub draft: CallTranscriptionDraftV1,
    pub recording_source: PersistedRecordingSourceV1,
    pub attempt_count: u32,
    pub max_attempts: u32,
    pub lease: CallTranscriptionJobLeaseV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistSttResultV1 {
    pub logical_owner_id: String,
    pub job_id: [u8; 16],
    pub lease: CallTranscriptionJobLeaseV1,
    pub transition: CallTranscriptionTransitionV1,
    pub result_receipt_sha256: Option<[u8; 32]>,
    pub outbox: Option<DurableOutboxRecordV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaterializeTranscriptV1 {
    pub logical_owner_id: String,
    pub job_id: [u8; 16],
    pub run_id: [u8; 16],
    pub artifact_id: [u8; 16],
    pub artifact_reference_id: [u8; 16],
    pub artifact_receipt_sha256: [u8; 32],
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub outbox: Option<DurableOutboxRecordV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RebindTranscriptMaterializationV1 {
    pub run_id: [u8; 16],
    pub job_id: [u8; 16],
    pub transcript_reference_id: [u8; 16],
    pub transcript_receipt_sha256: [u8; 32],
    pub stt_result_receipt_sha256: [u8; 32],
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub rebound_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompleteSourceCleanupV1 {
    pub run_id: [u8; 16],
    pub source_reference_id: [u8; 16],
    pub source_receipt_sha256: [u8; 32],
    pub completed_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IssueCallTranscriptTicketV1 {
    pub ticket_sha256: [u8; 32],
    pub device_actor_sha256: [u8; 32],
    pub client_session_sha256: [u8; 32],
    pub run_id: [u8; 16],
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub now_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IssuedCallTranscriptTicketV1 {
    pub run_id: [u8; 16],
    pub expires_at_unix_seconds: i64,
    pub transcript_size_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RedeemedCallTranscriptTicketV1 {
    pub run_id: [u8; 16],
    pub artifact_reference_id: [u8; 16],
    pub artifact_receipt_sha256: [u8; 32],
    pub transcript_size_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnpublishedCallTranscriptionEventV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CallTranscriptionRealtimeTransitionV1 {
    pub sequence: u64,
    pub run_id: [u8; 16],
    pub state: makosh_call_transcription_core::CallTranscriptionStateV1,
    pub state_revision: u64,
    pub rejection: Option<CallTranscriptionRejectionV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallTranscriptionPersistenceErrorV1 {
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

#[must_use]
pub fn call_transcription_job_id_v1(run_id: [u8; 16], request_id: [u8; 16]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.call-transcription.job.v1\0");
    digest.update(run_id);
    digest.update(request_id);
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

pub(crate) fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

pub(crate) fn valid_worker(value: &str) -> bool {
    valid_owner(value)
}

pub(crate) fn valid_id16(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn valid_sha256(value: &[u8; 32]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn valid_timestamp_millis(value: i64) -> bool {
    value > 0
}

pub(crate) fn valid_outbox(value: &DurableOutboxRecordV1) -> bool {
    valid_id16(&value.message_id)
        && valid_sha256(&value.envelope_sha256)
        && !value.envelope_bytes.is_empty()
        && value.envelope_bytes.len() <= CALL_TRANSCRIPTION_MAX_EVENT_BYTES_V1
        && Sha256::digest(&value.envelope_bytes).as_slice() == value.envelope_sha256
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_identity_is_stable_and_binds_request() {
        let first = call_transcription_job_id_v1([1; 16], [2; 16]);
        assert_eq!(first, call_transcription_job_id_v1([1; 16], [2; 16]));
        assert_ne!(first, call_transcription_job_id_v1([1; 16], [3; 16]));
    }
}
