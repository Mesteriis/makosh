use makosh_mail_contacts_sync_core::{
    MailContactsSyncDirectionV1, MailContactsSyncDraftV1, MailContactsSyncStatusV1,
    MailContactsSyncTransitionV1,
};
use sha2::{Digest, Sha256};

pub const MAX_ENVELOPE_BYTES_V1: usize = 64 * 1024;
pub const MAIL_CONTACTS_SYNC_OUTBOX_LIMIT_V1: u16 = 256;
pub const MAIL_CONTACTS_SYNC_REALTIME_LIMIT_V1: u16 = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxEnvelopeV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MailContactsSyncRealtimeTransitionV1 {
    pub sequence: u64,
    pub run_id: [u8; 16],
    pub state: makosh_mail_contacts_sync_core::MailContactsSyncStateV1,
    pub state_revision: u64,
    pub rejection: Option<makosh_mail_contacts_sync_core::MailContactsSyncRejectCodeV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateMailContactsSyncRunV1 {
    pub logical_owner_id: String,
    pub draft: MailContactsSyncDraftV1,
    pub initial_commands: Vec<OutboxEnvelopeV1>,
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptScheduledMailContactsSyncDueV1 {
    pub logical_owner_id: String,
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub scheduler_run_id: [u8; 16],
    pub lease_epoch: u64,
    pub lease_expires_at_unix_millis: u64,
    pub launch: Option<MailContactsSyncDraftV1>,
    pub durable_messages: Vec<OutboxEnvelopeV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncScheduledTerminalOutcomeV1 {
    Succeeded,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingMailContactsSyncScheduledTerminalV1 {
    pub run_id: [u8; 16],
    pub command_message_id: [u8; 16],
    pub lease_epoch: u64,
    pub lease_expires_at_unix_millis: u64,
    pub outcome: MailContactsSyncScheduledTerminalOutcomeV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueMailContactsSyncScheduledTerminalV1 {
    pub logical_owner_id: String,
    pub run_id: [u8; 16],
    pub terminal_receipt: OutboxEnvelopeV1,
    pub queued_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailContactsSyncTransitionInputV1 {
    pub logical_owner_id: String,
    pub run_id: [u8; 16],
    pub direction: MailContactsSyncDirectionV1,
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub transition: MailContactsSyncTransitionV1,
    pub next_command: Option<OutboxEnvelopeV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedMailContactsSyncRunV1 {
    pub logical_owner_id: String,
    pub draft: MailContactsSyncDraftV1,
    pub request_fingerprint: [u8; 32],
    pub status: MailContactsSyncStatusV1,
    pub created_at_unix_millis: i64,
    pub updated_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreateMailContactsSyncOutcomeV1 {
    Created(PersistedMailContactsSyncRunV1),
    Existing(PersistedMailContactsSyncRunV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AcceptScheduledMailContactsSyncDueOutcomeV1 {
    Launched(PersistedMailContactsSyncRunV1),
    Skipped,
    Duplicate(Option<PersistedMailContactsSyncRunV1>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailContactsSyncInboxOutcomeV1 {
    Applied(PersistedMailContactsSyncRunV1),
    Duplicate(PersistedMailContactsSyncRunV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncPersistenceOutcomeV1 {
    Applied,
    Duplicate,
    PendingPrerequisites,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncAdvanceOutcomeV1 {
    Applied,
    Idle,
    PendingContacts,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvanceMailContactsSyncPageV1 {
    pub logical_owner_id: String,
    pub run_id: [u8; 16],
    pub next_page_command: Option<OutboxEnvelopeV1>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailContactsSyncEntryInputV1 {
    pub logical_owner_id: String,
    pub run_id: [u8; 16],
    pub page_sequence: u64,
    pub observation_message_id: [u8; 16],
    pub observation_envelope_sha256: [u8; 32],
    pub contact_command_id: [u8; 16],
    pub entry_digest: [u8; 32],
    pub contact_command: OutboxEnvelopeV1,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailContactsSyncPageResultInputV1 {
    pub logical_owner_id: String,
    pub run_id: [u8; 16],
    pub page_sequence: u64,
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub observed_entries: u32,
    pub next_continuation_cursor: Option<Vec<u8>>,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncContactOutcomeV1 {
    Created,
    Updated,
    Unchanged,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailContactsSyncEntryOutcomeInputV1 {
    pub logical_owner_id: String,
    pub contact_command_id: [u8; 16],
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub outcome: MailContactsSyncContactOutcomeV1,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailContactsSyncPageProgressV1 {
    pub run_id: [u8; 16],
    pub page_sequence: u64,
    pub expected_entries: u32,
    pub recorded_entries: u32,
    pub accounted_entries: u32,
    pub next_continuation_cursor: Option<Vec<u8>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    RequestConflict,
    InboxConflict,
    RevisionConflict,
    InvalidTransition,
    NotFound,
}

pub(crate) fn request_fingerprint(draft: &MailContactsSyncDraftV1) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.mail_contacts_sync.start.v1\0");
    hash.update(draft.account_id.as_bytes());
    hash.update([direction_code(draft.direction) as u8]);
    hash.update([trigger_code(draft.trigger) as u8]);
    hash.finalize().into()
}

pub(crate) const fn direction_code(value: MailContactsSyncDirectionV1) -> i16 {
    match value {
        MailContactsSyncDirectionV1::ProviderToContacts => 1,
        MailContactsSyncDirectionV1::Bidirectional => 2,
    }
}

pub(crate) const fn trigger_code(
    value: makosh_mail_contacts_sync_core::MailContactsSyncTriggerV1,
) -> i16 {
    match value {
        makosh_mail_contacts_sync_core::MailContactsSyncTriggerV1::Manual => 1,
        makosh_mail_contacts_sync_core::MailContactsSyncTriggerV1::Scheduled => 2,
    }
}

pub(crate) fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

pub(crate) fn valid_envelope(value: &OutboxEnvelopeV1) -> bool {
    nonzero(&value.message_id)
        && nonzero(&value.envelope_sha256)
        && !value.envelope_bytes.is_empty()
        && value.envelope_bytes.len() <= MAX_ENVELOPE_BYTES_V1
        && Sha256::digest(&value.envelope_bytes).as_slice() == value.envelope_sha256
}

pub(crate) fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}
