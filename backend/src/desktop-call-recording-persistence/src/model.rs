use makosh_desktop_call_recording_core::RecordingStateV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewRecordingRunV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; 16],
    pub request_sha256: [u8; 32],
    pub call_evidence_id: [u8; 16],
    pub call_evidence_revision: u64,
    pub recording_evidence_id: [u8; 16],
    pub device_actor_sha256: [u8; 32],
    pub challenge_id: [u8; 16],
    pub challenge_expires_at_unix_ms: i64,
    pub maximum_duration_millis: u64,
    pub consent_policy_revision: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedRecordingRunV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; 16],
    pub request_sha256: [u8; 32],
    pub call_evidence_id: [u8; 16],
    pub call_evidence_revision: u64,
    pub recording_evidence_id: [u8; 16],
    pub recording_revision: u64,
    pub state: RecordingStateV1,
    pub device_actor_sha256: [u8; 32],
    pub challenge_id: [u8; 16],
    pub challenge_expires_at_unix_ms: i64,
    pub maximum_duration_millis: u64,
    pub consent_policy_revision: u32,
    pub started_at_unix_ms: Option<i64>,
    pub ended_at_unix_ms: Option<i64>,
    pub consent_receipt_id: Option<[u8; 16]>,
    pub source_reference_id: Option<[u8; 16]>,
    pub source_declared_bytes: Option<u64>,
    pub source_duration_millis: Option<u64>,
    pub source_sha256: Option<[u8; 32]>,
    pub public_error_code: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalRecordingMetadataV1 {
    pub ended_at_unix_ms: i64,
    pub consent_receipt_id: [u8; 16],
    pub source_reference_id: [u8; 16],
    pub source_declared_bytes: u64,
    pub source_duration_millis: u64,
    pub source_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureStartedWriteV1 {
    pub logical_owner_id: String,
    pub recording_evidence_id: [u8; 16],
    pub expected_revision: u64,
    pub started_at_unix_ms: i64,
    pub consent_receipt_id: [u8; 16],
    pub command_id: [u8; 16],
    pub claim_sha256: [u8; 32],
    pub realtime: RealtimeTransitionV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactOutboxRecordV1 {
    pub event_id: [u8; 16],
    pub contract_name: String,
    pub exact_envelope_bytes: Vec<u8>,
    pub envelope_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealtimeTransitionV1 {
    pub occurred_at_unix_ms: i64,
    pub payload_bytes: Vec<u8>,
    pub payload_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeasedHostCommandV1 {
    pub command_id: [u8; 16],
    pub logical_owner_id: String,
    pub recording_evidence_id: [u8; 16],
    pub command_kind: u16,
    pub command_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingOutboxV1 {
    pub sequence_id: i64,
    pub event_id: [u8; 16],
    pub logical_owner_id: String,
    pub recording_evidence_id: [u8; 16],
    pub contract_name: String,
    pub envelope_sha256: [u8; 32],
    pub exact_envelope_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingRealtimeV1 {
    pub sequence_id: i64,
    pub logical_owner_id: String,
    pub recording_evidence_id: [u8; 16],
    pub recording_revision: u64,
    pub occurred_at_unix_ms: i64,
    pub payload_bytes: Vec<u8>,
    pub payload_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RejectRecordingWriteV1 {
    pub logical_owner_id: String,
    pub recording_evidence_id: [u8; 16],
    pub expected_revision: u64,
    pub expected_state: RecordingStateV1,
    pub public_error_code: String,
    pub host_command_completion: Option<HostCommandCompletionV1>,
    pub outbox: ExactOutboxRecordV1,
    pub realtime: RealtimeTransitionV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostCommandCompletionV1 {
    pub command_id: [u8; 16],
    pub claim_sha256: [u8; 32],
    pub completed_at_unix_ms: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistenceErrorV1 {
    InvalidInput,
    StorageUnavailable,
    Conflict,
    InvalidRow,
}
