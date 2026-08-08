use makosh_events_protocol::delivery::OutboxRecordV1;

pub(crate) const MAX_EVENT_BYTES_V1: usize = 65_536;
pub(crate) const MAX_OUTBOX_BATCH_V1: u16 = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistPromotionApprovalV1 {
    pub logical_owner_id: String,
    pub approval_message_id: [u8; 16],
    pub approval_envelope_sha256: [u8; 32],
    pub review_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub decision_revision: u64,
    pub tasks_command_id: [u8; 16],
    pub tasks_command_outbox: OutboxRecordV1,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistPromotionApprovalOutcomeV1 {
    Applied,
    Duplicate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewedTaskCandidatePromotionOutcomeV1 {
    Succeeded { task_id: [u8; 16] },
    Failed { failure_code: u16 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistPromotionTerminalResultV1 {
    pub logical_owner_id: String,
    pub tasks_result_message_id: [u8; 16],
    pub tasks_result_envelope_sha256: [u8; 32],
    pub tasks_command_id: [u8; 16],
    pub review_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub outcome: ReviewedTaskCandidatePromotionOutcomeV1,
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
