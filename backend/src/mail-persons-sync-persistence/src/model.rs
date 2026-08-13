use std::collections::BTreeSet;

use makosh_events_protocol::delivery::OutboxRecordV1;
use sha2::{Digest, Sha256};

use crate::MailPersonsSyncPersistenceErrorV1;

pub const MAIL_PERSONS_SYNC_MAX_ENVELOPE_BYTES_V1: usize = 256 * 1024;
pub const MAIL_PERSONS_SYNC_OUTBOX_READ_LIMIT_V1: i64 = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum MailPersonsSyncAccountLifecycleKindV1 {
    Ready = 1,
    Retired = 2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyMailPersonsSyncAccountLifecycleV1 {
    pub logical_owner_id: String,
    pub integration_public_id: [u8; 16],
    pub account_public_id: [u8; 16],
    pub mapping_revision: u64,
    pub kind: MailPersonsSyncAccountLifecycleKindV1,
    pub lifecycle: MailPersonsSyncEnvelopeRecordV1,
    pub processed_at_unix_millis: i64,
}

impl ApplyMailPersonsSyncAccountLifecycleV1 {
    pub fn validate(&self) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
        self.lifecycle.validate()?;
        if valid_owner(&self.logical_owner_id)
            && nonzero(&self.integration_public_id)
            && nonzero(&self.account_public_id)
            && self.mapping_revision > 0
            && self.processed_at_unix_millis > 0
        {
            Ok(())
        } else {
            Err(MailPersonsSyncPersistenceErrorV1::InvalidInput)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonsSyncScheduleControlOutboxRecordV1 {
    pub record: MailPersonsSyncEnvelopeRecordV1,
    pub outbox_sequence: u64,
    pub account_public_id: [u8; 16],
    pub mapping_revision: u64,
    pub schedule_revision: u64,
    pub kind: MailPersonsSyncAccountLifecycleKindV1,
    pub created_at_unix_millis: i64,
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonsSyncEnvelopeRecordV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

impl MailPersonsSyncEnvelopeRecordV1 {
    pub fn new(
        message_id: [u8; 16],
        envelope_bytes: Vec<u8>,
    ) -> Result<Self, MailPersonsSyncPersistenceErrorV1> {
        let record = Self {
            message_id,
            envelope_sha256: Sha256::digest(&envelope_bytes).into(),
            envelope_bytes,
        };
        record.validate()?;
        Ok(record)
    }

    pub fn validate(&self) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
        if self.message_id.iter().all(|byte| *byte == 0)
            || self.envelope_bytes.is_empty()
            || self.envelope_bytes.len() > MAIL_PERSONS_SYNC_MAX_ENVELOPE_BYTES_V1
        {
            return Err(MailPersonsSyncPersistenceErrorV1::InvalidInput);
        }
        if <[u8; 32]>::from(Sha256::digest(&self.envelope_bytes)) != self.envelope_sha256 {
            return Err(MailPersonsSyncPersistenceErrorV1::HashMismatch);
        }
        let accepted = OutboxRecordV1::accept(self.envelope_bytes.clone())
            .map_err(|_| MailPersonsSyncPersistenceErrorV1::InvalidInput)?;
        if accepted.message_id() != &self.message_id
            || accepted.envelope_sha256() != &self.envelope_sha256
        {
            return Err(MailPersonsSyncPersistenceErrorV1::InvalidInput);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BeginMailPersonsSyncRunV1 {
    pub logical_owner_id: String,
    pub account_public_id: [u8; 16],
    pub run_id: [u8; 16],
    pub run_fingerprint: [u8; 32],
    pub scheduler_command: MailPersonsSyncEnvelopeRecordV1,
    pub scheduler_acceptance: MailPersonsSyncEnvelopeRecordV1,
    pub initial_fetch: MailPersonsSyncEnvelopeRecordV1,
    pub lease_epoch: u64,
    pub lease_expires_at_unix_millis: i64,
    pub received_at_unix_millis: i64,
}

impl BeginMailPersonsSyncRunV1 {
    pub fn validate(&self) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
        self.scheduler_command.validate()?;
        self.scheduler_acceptance.validate()?;
        self.initial_fetch.validate()?;
        if valid_owner(&self.logical_owner_id)
            && nonzero(&self.account_public_id)
            && nonzero(&self.run_id)
            && nonzero(&self.run_fingerprint)
            && self.scheduler_command.message_id != self.scheduler_acceptance.message_id
            && self.scheduler_command.message_id != self.initial_fetch.message_id
            && self.scheduler_acceptance.message_id != self.initial_fetch.message_id
            && self.lease_epoch > 0
            && self.received_at_unix_millis > 0
            && self.lease_expires_at_unix_millis > self.received_at_unix_millis
        {
            Ok(())
        } else {
            Err(MailPersonsSyncPersistenceErrorV1::InvalidInput)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RejectMailPersonsSyncAccountBusyV1 {
    pub begin: BeginMailPersonsSyncRunV1,
    pub scheduler_terminal: MailPersonsSyncEnvelopeRecordV1,
}

impl RejectMailPersonsSyncAccountBusyV1 {
    pub fn validate(&self) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
        self.begin.validate()?;
        self.scheduler_terminal.validate()?;
        if self.scheduler_terminal.message_id == self.begin.scheduler_command.message_id
            || self.scheduler_terminal.message_id == self.begin.scheduler_acceptance.message_id
            || self.scheduler_terminal.message_id == self.begin.initial_fetch.message_id
        {
            Err(MailPersonsSyncPersistenceErrorV1::InvalidInput)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageMailPersonsSyncSourceV1 {
    pub logical_owner_id: String,
    pub account_public_id: [u8; 16],
    pub run_id: [u8; 16],
    pub page_sequence: u64,
    pub observation: MailPersonsSyncEnvelopeRecordV1,
    pub integration_public_id: [u8; 16],
    pub provider_source_contact_public_id: [u8; 16],
    pub change_kind: u8,
    pub source_revision: u64,
    pub source_digest: [u8; 32],
    pub persons_command_id: [u8; 16],
    pub persons_command_fingerprint: [u8; 32],
    pub persons_command: MailPersonsSyncEnvelopeRecordV1,
    pub received_at_unix_millis: i64,
}

impl StageMailPersonsSyncSourceV1 {
    pub fn validate(&self) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
        self.observation.validate()?;
        self.persons_command.validate()?;
        if valid_owner(&self.logical_owner_id)
            && nonzero(&self.account_public_id)
            && nonzero(&self.run_id)
            && (1..=4_096).contains(&self.page_sequence)
            && nonzero(&self.integration_public_id)
            && nonzero(&self.provider_source_contact_public_id)
            && (1..=3).contains(&self.change_kind)
            && self.source_revision > 0
            && nonzero(&self.source_digest)
            && nonzero(&self.persons_command_id)
            && nonzero(&self.persons_command_fingerprint)
            && self.received_at_unix_millis > 0
        {
            Ok(())
        } else {
            Err(MailPersonsSyncPersistenceErrorV1::InvalidInput)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteMailPersonsSyncPageV1 {
    pub logical_owner_id: String,
    pub account_public_id: [u8; 16],
    pub run_id: [u8; 16],
    pub page_sequence: u64,
    pub completion: MailPersonsSyncEnvelopeRecordV1,
    pub page_digest: [u8; 32],
    pub observed_sources: u32,
    pub updated_sources: u32,
    pub removed_sources: u32,
    pub has_more: bool,
    pub page_receipt: MailPersonsSyncEnvelopeRecordV1,
    pub rejection_code: Option<MailPersonsSyncStoredRejectCodeV1>,
    pub continuation: MailPersonsSyncPageContinuationV1,
    pub completed_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum MailPersonsSyncStoredRejectCodeV1 {
    InvalidRequest = 1,
    Conflict = 2,
    SourceUnavailable = 3,
    Policy = 4,
}

impl TryFrom<i16> for MailPersonsSyncStoredRejectCodeV1 {
    type Error = MailPersonsSyncPersistenceErrorV1;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::InvalidRequest),
            2 => Ok(Self::Conflict),
            3 => Ok(Self::SourceUnavailable),
            4 => Ok(Self::Policy),
            _ => Err(MailPersonsSyncPersistenceErrorV1::StateConflict),
        }
    }
}

impl From<MailPersonsSyncStoredRejectCodeV1> for i16 {
    fn from(value: MailPersonsSyncStoredRejectCodeV1) -> Self {
        value as Self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailPersonsSyncPageContinuationV1 {
    NextPage {
        next_fetch: MailPersonsSyncEnvelopeRecordV1,
    },
    Finished {
        run_result: MailPersonsSyncEnvelopeRecordV1,
        scheduler_terminal: MailPersonsSyncEnvelopeRecordV1,
    },
    AwaitingPersons,
}

impl MailPersonsSyncPageContinuationV1 {
    fn validate(&self, has_more: bool) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
        match self {
            Self::NextPage { next_fetch } if has_more => next_fetch.validate(),
            Self::Finished {
                run_result,
                scheduler_terminal,
            } if !has_more => {
                run_result.validate()?;
                scheduler_terminal.validate()
            }
            Self::AwaitingPersons if !has_more => Ok(()),
            _ => Err(MailPersonsSyncPersistenceErrorV1::InvalidInput),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MailPersonsSyncReplayOutcomeV1 {
    pub replayed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordMailPersonsSyncPersonsTerminalV1 {
    pub logical_owner_id: String,
    pub account_public_id: [u8; 16],
    pub run_id: [u8; 16],
    pub page_sequence: u64,
    pub persons_command_id: [u8; 16],
    pub result: MailPersonsSyncEnvelopeRecordV1,
    pub outcome: u8,
    pub result_completed_at_unix_millis: i64,
    pub received_at_unix_millis: i64,
}

impl RecordMailPersonsSyncPersonsTerminalV1 {
    pub fn validate(&self) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
        self.result.validate()?;
        if valid_owner(&self.logical_owner_id)
            && nonzero(&self.account_public_id)
            && nonzero(&self.run_id)
            && (1..=4_096).contains(&self.page_sequence)
            && nonzero(&self.persons_command_id)
            && (1..=2).contains(&self.outcome)
            && self.result_completed_at_unix_millis > 0
            && self.result_completed_at_unix_millis <= self.received_at_unix_millis
            && self.received_at_unix_millis > 0
        {
            Ok(())
        } else {
            Err(MailPersonsSyncPersistenceErrorV1::InvalidInput)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonsSyncOutboxRecordV1 {
    pub record: MailPersonsSyncEnvelopeRecordV1,
    pub run_id: [u8; 16],
    pub page_sequence: u64,
    pub semantic_kind: MailPersonsSyncSemanticKindV1,
    pub semantic_order_key: Vec<u8>,
    pub source_ordinal: u16,
    pub created_at_unix_millis: i64,
    pub published_at_unix_millis: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonsSyncRunContextV1 {
    pub account_public_id: [u8; 16],
    pub run_id: [u8; 16],
    pub state: u8,
    pub next_page_sequence: u64,
    pub processed_pages: u64,
    pub processed_sources: u64,
    pub rejection_code: Option<MailPersonsSyncStoredRejectCodeV1>,
    pub scheduler_message_id: [u8; 16],
    pub lease_epoch: u64,
    pub lease_expires_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonsSyncExpiredRunContextV1 {
    pub logical_owner_id: String,
    pub account_public_id: [u8; 16],
    pub run_id: [u8; 16],
    pub scheduler_message_id: [u8; 16],
    pub lease_epoch: u64,
    pub lease_expires_at_unix_millis: i64,
    pub next_page_sequence: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MailPersonsSyncSourceCommandContextV1 {
    pub account_public_id: [u8; 16],
    pub run_id: [u8; 16],
    pub page_sequence: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MailPersonsSyncPageFinalizationContextV1 {
    pub account_public_id: [u8; 16],
    pub completion_message_id: [u8; 16],
    pub observed_sources: u32,
    pub updated_sources: u32,
    pub removed_sources: u32,
    pub rejected: bool,
}

impl CompleteMailPersonsSyncPageV1 {
    pub fn validate(&self) -> Result<(), MailPersonsSyncPersistenceErrorV1> {
        self.completion.validate()?;
        self.page_receipt.validate()?;
        self.continuation.validate(self.has_more)?;
        let count = self
            .observed_sources
            .checked_add(self.updated_sources)
            .and_then(|count| count.checked_add(self.removed_sources));
        if valid_owner(&self.logical_owner_id)
            && nonzero(&self.account_public_id)
            && nonzero(&self.run_id)
            && (1..=4_096).contains(&self.page_sequence)
            && nonzero(&self.page_digest)
            && count.is_some_and(|count| count <= 500)
            && self.rejection_code.is_none_or(|_| {
                !self.has_more
                    && count == Some(0)
                    && matches!(
                        self.continuation,
                        MailPersonsSyncPageContinuationV1::Finished { .. }
                    )
            })
            && self.completed_at_unix_millis > 0
        {
            Ok(())
        } else {
            Err(MailPersonsSyncPersistenceErrorV1::InvalidInput)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum MailPersonsSyncSemanticKindV1 {
    SchedulerAcceptance = 1,
    MailFetch = 2,
    PersonsCommand = 3,
    PageReceipt = 4,
    NextMailFetch = 5,
    RunResult = 6,
    SchedulerTerminal = 7,
}

pub fn mail_persons_sync_semantic_order_key_v1(
    page_sequence: u64,
    kind: MailPersonsSyncSemanticKindV1,
    public_source_id: Option<[u8; 16]>,
    ordinal: u16,
) -> Result<Vec<u8>, MailPersonsSyncPersistenceErrorV1> {
    if page_sequence > 4_096 || ordinal > 502 {
        return Err(MailPersonsSyncPersistenceErrorV1::InvalidInput);
    }
    let requires_source = kind == MailPersonsSyncSemanticKindV1::PersonsCommand;
    if requires_source != public_source_id.is_some() {
        return Err(MailPersonsSyncPersistenceErrorV1::InvalidInput);
    }
    let mut key = Vec::with_capacity(27);
    key.extend_from_slice(&page_sequence.to_be_bytes());
    key.push(kind as u8);
    key.extend_from_slice(&public_source_id.unwrap_or([0; 16]));
    key.extend_from_slice(&ordinal.to_be_bytes());
    Ok(key)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedSourceV1 {
    pub public_source_id: [u8; 16],
    pub observed: u32,
    pub updated: u32,
    pub removed: u32,
}

pub fn validate_page_promotion_v1(
    expected_observed: u32,
    expected_updated: u32,
    expected_removed: u32,
    staged: &[StagedSourceV1],
) -> Result<Vec<StagedSourceV1>, MailPersonsSyncPersistenceErrorV1> {
    let expected_total = expected_observed
        .checked_add(expected_updated)
        .and_then(|count| count.checked_add(expected_removed))
        .ok_or(MailPersonsSyncPersistenceErrorV1::InvalidInput)?;
    if expected_total > 500 || staged.len() > 500 {
        return Err(MailPersonsSyncPersistenceErrorV1::InvalidInput);
    }
    let mut unique = BTreeSet::new();
    let mut actual = [0_u32; 3];
    for source in staged {
        if source.public_source_id.iter().all(|byte| *byte == 0)
            || source
                .observed
                .checked_add(source.updated)
                .and_then(|count| count.checked_add(source.removed))
                != Some(1)
            || !unique.insert(source.public_source_id)
        {
            return Err(MailPersonsSyncPersistenceErrorV1::StateConflict);
        }
        actual[0] = actual[0]
            .checked_add(source.observed)
            .ok_or(MailPersonsSyncPersistenceErrorV1::InvalidInput)?;
        actual[1] = actual[1]
            .checked_add(source.updated)
            .ok_or(MailPersonsSyncPersistenceErrorV1::InvalidInput)?;
        actual[2] = actual[2]
            .checked_add(source.removed)
            .ok_or(MailPersonsSyncPersistenceErrorV1::InvalidInput)?;
    }
    if actual != [expected_observed, expected_updated, expected_removed] {
        return Err(MailPersonsSyncPersistenceErrorV1::PageIncomplete);
    }
    let mut ordered = staged.to_vec();
    ordered.sort_by_key(|source| source.public_source_id);
    Ok(ordered)
}
