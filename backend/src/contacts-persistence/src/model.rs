use makosh_contacts_core::{ContactUpsertDraftV1, ContactUpsertOutcomeV1};
use sha2::{Digest, Sha256};

pub const CONTACTS_MAX_EVENT_BYTES_V1: usize = 64 * 1024;
pub const CONTACTS_OUTBOX_LIMIT_V1: u16 = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactsOutboxRecordV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactMutationOutboxV1 {
    pub terminal_result: ContactsOutboxRecordV1,
    pub changed_event: Option<ContactsOutboxRecordV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactMailSyncSourceLinkV1 {
    pub provider_entry_id: String,
    pub provider_etag: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactMailSyncSourceSnapshotV1 {
    pub contact_id: [u8; 16],
    pub contact_revision: u64,
    pub display_name: String,
    pub email_addresses: Vec<String>,
    pub phone_numbers: Vec<String>,
    pub target_account_link: Option<ContactMailSyncSourceLinkV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i16)]
pub enum ContactMailSyncSourceRejectCodeV1 {
    InvalidRequest = 1,
    ContactMissing = 2,
    StaleContactRevision = 3,
    ContentLimit = 4,
    Policy = 5,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReserveContactMailSyncSourceV1 {
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub operation_id: [u8; 16],
    pub contact_id: [u8; 16],
    pub expected_contact_revision: u64,
    pub target_mail_account_id: String,
    pub logical_owner_id: String,
    pub received_at_unix_millis: i64,
}

impl ReserveContactMailSyncSourceV1 {
    #[must_use]
    pub fn command_fingerprint(&self) -> [u8; 32] {
        source_command_fingerprint(
            self.command_envelope_sha256,
            self.operation_id,
            self.contact_id,
            self.expected_contact_revision,
            &self.target_mail_account_id,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistContactMailSyncSourceResultV1 {
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub operation_id: [u8; 16],
    pub contact_id: [u8; 16],
    pub expected_contact_revision: u64,
    pub target_mail_account_id: String,
    pub logical_owner_id: String,
    pub reject_code: Option<ContactMailSyncSourceRejectCodeV1>,
    pub terminal_result: ContactsOutboxRecordV1,
    pub received_at_unix_millis: i64,
    pub completed_at_unix_millis: i64,
}

impl PersistContactMailSyncSourceResultV1 {
    #[must_use]
    pub fn command_fingerprint(&self) -> [u8; 32] {
        source_command_fingerprint(
            self.command_envelope_sha256,
            self.operation_id,
            self.contact_id,
            self.expected_contact_revision,
            &self.target_mail_account_id,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactMailSyncSourceResultV1 {
    pub terminal_result: ContactsOutboxRecordV1,
    pub reject_code: Option<ContactMailSyncSourceRejectCodeV1>,
    pub replayed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyMailEntryCommandV1 {
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub command_id: [u8; 16],
    pub draft: ContactUpsertDraftV1,
    pub received_at_unix_millis: i64,
    pub completed_at_unix_millis: i64,
}

impl ApplyMailEntryCommandV1 {
    #[must_use]
    pub fn command_fingerprint(&self) -> [u8; 32] {
        command_fingerprint(
            self.command_envelope_sha256,
            self.command_id,
            self.draft.provenance.entry_digest,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i16)]
pub enum ContactMailEntryRejectCodeV1 {
    InvalidRequest = 1,
    IdentityAmbiguous = 2,
    ProviderLinkConflict = 3,
    StaleSource = 4,
    Policy = 5,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RejectMailEntryCommandV1 {
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub command_id: [u8; 16],
    pub logical_owner_id: String,
    pub entry_digest: [u8; 32],
    pub received_at_unix_millis: i64,
    pub completed_at_unix_millis: i64,
    pub code: ContactMailEntryRejectCodeV1,
    pub terminal_result: ContactsOutboxRecordV1,
}

impl RejectMailEntryCommandV1 {
    #[must_use]
    pub fn command_fingerprint(&self) -> [u8; 32] {
        command_fingerprint(
            self.command_envelope_sha256,
            self.command_id,
            self.entry_digest,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RejectedMailEntryCommandV1 {
    pub code: ContactMailEntryRejectCodeV1,
    pub terminal_result: ContactsOutboxRecordV1,
    pub replayed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedMailEntryCommandV1 {
    pub contact_id: [u8; 16],
    pub contact_revision: u64,
    pub outcome: ContactUpsertOutcomeV1,
    pub terminal_result: ContactsOutboxRecordV1,
    pub replayed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactsPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    CommandConflict,
    InboxConflict,
    IdentityAmbiguous,
    ProviderLinkConflict,
    StaleSource,
    PolicyRejected,
    NotFound,
}

pub(crate) fn valid_apply(value: &ApplyMailEntryCommandV1) -> bool {
    nonzero(&value.command_message_id)
        && nonzero(&value.command_envelope_sha256)
        && nonzero(&value.command_id)
        && makosh_contacts_core::upsert_fingerprint_v1(&value.draft).is_ok()
        && value.received_at_unix_millis > 0
        && value.completed_at_unix_millis >= value.received_at_unix_millis
}

pub(crate) fn valid_reject(value: &RejectMailEntryCommandV1) -> bool {
    nonzero(&value.command_message_id)
        && nonzero(&value.command_envelope_sha256)
        && nonzero(&value.command_id)
        && valid_owner(&value.logical_owner_id)
        && nonzero(&value.entry_digest)
        && value.received_at_unix_millis > 0
        && value.completed_at_unix_millis >= value.received_at_unix_millis
        && valid_outbox(&value.terminal_result)
}

pub(crate) fn valid_outbox(value: &ContactsOutboxRecordV1) -> bool {
    nonzero(&value.message_id)
        && nonzero(&value.envelope_sha256)
        && !value.envelope_bytes.is_empty()
        && value.envelope_bytes.len() <= CONTACTS_MAX_EVENT_BYTES_V1
        && Sha256::digest(&value.envelope_bytes).as_slice() == value.envelope_sha256
}

pub(crate) fn valid_mutation_outbox(value: &ContactMutationOutboxV1) -> bool {
    valid_outbox(&value.terminal_result)
        && value.changed_event.as_ref().is_none_or(|changed| {
            valid_outbox(changed) && changed.message_id != value.terminal_result.message_id
        })
}

pub(crate) fn valid_source_result(value: &PersistContactMailSyncSourceResultV1) -> bool {
    valid_source_command(
        value.command_message_id,
        value.command_envelope_sha256,
        value.operation_id,
        value.contact_id,
        value.expected_contact_revision,
        &value.target_mail_account_id,
        &value.logical_owner_id,
        value.received_at_unix_millis,
    ) && valid_outbox(&value.terminal_result)
        && value.completed_at_unix_millis >= value.received_at_unix_millis
}

pub(crate) fn valid_source_reservation(value: &ReserveContactMailSyncSourceV1) -> bool {
    valid_source_command(
        value.command_message_id,
        value.command_envelope_sha256,
        value.operation_id,
        value.contact_id,
        value.expected_contact_revision,
        &value.target_mail_account_id,
        &value.logical_owner_id,
        value.received_at_unix_millis,
    )
}

#[allow(clippy::too_many_arguments)]
fn valid_source_command(
    command_message_id: [u8; 16],
    command_envelope_sha256: [u8; 32],
    operation_id: [u8; 16],
    contact_id: [u8; 16],
    expected_contact_revision: u64,
    target_mail_account_id: &str,
    logical_owner_id: &str,
    received_at_unix_millis: i64,
) -> bool {
    nonzero(&command_message_id)
        && nonzero(&command_envelope_sha256)
        && nonzero(&operation_id)
        && nonzero(&contact_id)
        && expected_contact_revision > 0
        && valid_bounded_text(target_mail_account_id, 256)
        && valid_owner(logical_owner_id)
        && received_at_unix_millis > 0
}

fn source_command_fingerprint(
    command_envelope_sha256: [u8; 32],
    operation_id: [u8; 16],
    contact_id: [u8; 16],
    expected_contact_revision: u64,
    target_mail_account_id: &str,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.contacts.mail-sync-source.command.v1\0");
    hash.update(command_envelope_sha256);
    hash.update(operation_id);
    hash.update(contact_id);
    hash.update(expected_contact_revision.to_be_bytes());
    hash.update(target_mail_account_id.as_bytes());
    hash.finalize().into()
}

pub(crate) fn valid_bounded_text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
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

fn command_fingerprint(
    command_envelope_sha256: [u8; 32],
    command_id: [u8; 16],
    entry_digest: [u8; 32],
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.contacts.mail-entry.command.v1\0");
    hash.update(command_envelope_sha256);
    hash.update(command_id);
    hash.update(entry_digest);
    hash.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_contacts_core::{
        ContactProviderKindV1, ContactProviderProvenanceV1, ContactTimestampV1,
    };

    #[test]
    fn fingerprint_binds_command_and_canonical_input() {
        let mut input = sample();
        let first = input.command_fingerprint();
        input.command_envelope_sha256[0] ^= 1;
        assert_ne!(first, input.command_fingerprint());
    }

    #[test]
    fn source_reservation_and_completion_share_one_command_fingerprint() {
        let reservation = ReserveContactMailSyncSourceV1 {
            command_message_id: [1; 16],
            command_envelope_sha256: [2; 32],
            operation_id: [3; 16],
            contact_id: [4; 16],
            expected_contact_revision: 5,
            target_mail_account_id: "mail-1".to_owned(),
            logical_owner_id: "owner-1".to_owned(),
            received_at_unix_millis: 6,
        };
        let completion = PersistContactMailSyncSourceResultV1 {
            command_message_id: reservation.command_message_id,
            command_envelope_sha256: reservation.command_envelope_sha256,
            operation_id: reservation.operation_id,
            contact_id: reservation.contact_id,
            expected_contact_revision: reservation.expected_contact_revision,
            target_mail_account_id: reservation.target_mail_account_id.clone(),
            logical_owner_id: reservation.logical_owner_id.clone(),
            reject_code: None,
            terminal_result: ContactsOutboxRecordV1 {
                message_id: [7; 16],
                envelope_sha256: [8; 32],
                envelope_bytes: vec![9],
            },
            received_at_unix_millis: reservation.received_at_unix_millis,
            completed_at_unix_millis: 7,
        };

        assert_eq!(
            reservation.command_fingerprint(),
            completion.command_fingerprint()
        );
    }

    fn sample() -> ApplyMailEntryCommandV1 {
        ApplyMailEntryCommandV1 {
            command_message_id: [1; 16],
            command_envelope_sha256: [2; 32],
            command_id: [3; 16],
            draft: ContactUpsertDraftV1 {
                logical_owner_id: "owner-1".to_owned(),
                display_name: "Ada".to_owned(),
                email_addresses: vec!["ada@example.test".to_owned()],
                phone_numbers: Vec::new(),
                provenance: ContactProviderProvenanceV1 {
                    source_account_id: "mail-1".to_owned(),
                    provider_kind: ContactProviderKindV1::Gmail,
                    provider_entry_id: "people/c1".to_owned(),
                    provider_etag: Some("etag-1".to_owned()),
                    source_revision: 1,
                    entry_digest: [4; 32],
                    observed_at: ContactTimestampV1 {
                        unix_seconds: 1_800_000_000,
                        nanos: 0,
                    },
                },
            },
            received_at_unix_millis: 1_800_000_000_000,
            completed_at_unix_millis: 1_800_000_000_001,
        }
    }
}
