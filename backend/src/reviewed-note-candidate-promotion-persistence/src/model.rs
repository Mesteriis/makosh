use makosh_events_protocol::delivery::OutboxRecordV1;

pub(crate) const MAX_EVENT_BYTES_V1: usize = 65_536;
pub(crate) const MAX_OUTBOX_BATCH_V1: u16 = 128;
pub const REVIEWED_NOTE_CANDIDATE_PROMOTION_MAX_BLOB_BYTES_V1: u64 = 16 * 1024;
pub const REVIEWED_NOTE_CANDIDATE_PROMOTION_MAX_PROOF_BYTES_V1: usize = 2_048;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionBlobReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
    pub custody_proof: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReservePromotionApprovalV1 {
    pub logical_owner_id: String,
    pub approval_message_id: [u8; 16],
    pub approval_envelope_sha256: [u8; 32],
    pub review_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub decision_revision: u64,
    pub source_blob: PromotionBlobReceiptV1,
    pub knowledge_command_id: [u8; 16],
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedPromotionApprovalV1 {
    pub logical_owner_id: String,
    pub approval_message_id: [u8; 16],
    pub approval_envelope_sha256: [u8; 32],
    pub review_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub decision_revision: u64,
    pub source_blob: PromotionBlobReceiptV1,
    pub materialized_reference_id: Option<[u8; 16]>,
    pub cleanup_completed_at_unix_millis: Option<i64>,
    pub knowledge_command_id: [u8; 16],
    pub command_completed: bool,
    pub workflow_failure_result_id: Option<[u8; 16]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReservePromotionApprovalOutcomeV1 {
    Reserved(PersistedPromotionApprovalV1),
    Existing(PersistedPromotionApprovalV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistPromotionMaterializationV1 {
    pub logical_owner_id: String,
    pub approval_message_id: [u8; 16],
    pub materialized_reference_id: [u8; 16],
    pub materialized_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistPromotionApprovalV1 {
    pub logical_owner_id: String,
    pub approval_message_id: [u8; 16],
    pub approval_envelope_sha256: [u8; 32],
    pub review_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub decision_revision: u64,
    pub knowledge_command_id: [u8; 16],
    pub knowledge_command_outbox: OutboxRecordV1,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistPromotionApprovalOutcomeV1 {
    Applied,
    Duplicate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistPromotionWorkflowFailureV1 {
    pub logical_owner_id: String,
    pub approval_message_id: [u8; 16],
    pub review_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub knowledge_command_id: [u8; 16],
    pub failure_code: u16,
    pub review_result_outbox: OutboxRecordV1,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewedNoteCandidatePromotionOutcomeV1 {
    Succeeded { note_id: [u8; 16] },
    Failed { failure_code: u16 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistPromotionTerminalResultV1 {
    pub logical_owner_id: String,
    pub knowledge_result_message_id: [u8; 16],
    pub knowledge_result_envelope_sha256: [u8; 32],
    pub knowledge_command_id: [u8; 16],
    pub review_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub outcome: ReviewedNoteCandidatePromotionOutcomeV1,
    pub review_result_outbox: OutboxRecordV1,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistPromotionResultOutcomeV1 {
    Applied,
    Duplicate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PromotionCorrelationV1 {
    pub review_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub decision_revision: u64,
    pub completed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnpublishedPromotionEventV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

pub(crate) fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

pub(crate) fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn valid_timestamp(value: i64) -> bool {
    value > 0
}

pub(crate) fn valid_outbox(value: &OutboxRecordV1) -> bool {
    nonzero(value.message_id())
        && nonzero(value.envelope_sha256())
        && !value.exact_bytes().is_empty()
        && value.exact_bytes().len() <= MAX_EVENT_BYTES_V1
}

pub(crate) fn valid_blob(value: &PromotionBlobReceiptV1) -> bool {
    nonzero(&value.reference_id)
        && (1..=REVIEWED_NOTE_CANDIDATE_PROMOTION_MAX_BLOB_BYTES_V1).contains(&value.declared_bytes)
        && nonzero(&value.sha256)
        && !value.custody_proof.is_empty()
        && value.custody_proof.len() <= REVIEWED_NOTE_CANDIDATE_PROMOTION_MAX_PROOF_BYTES_V1
}
