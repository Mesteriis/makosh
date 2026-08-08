use makosh_contacts_core::ContactProviderKindV1;

use crate::ContactsOutboxRecordV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindMailProviderLinkCommandV1 {
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub command_id: [u8; 16],
    pub logical_owner_id: String,
    pub contact_id: [u8; 16],
    pub expected_contact_revision: u64,
    pub source_account_id: String,
    pub provider_kind: ContactProviderKindV1,
    pub provider_entry_id: String,
    pub provider_etag: Option<String>,
    pub received_at_unix_millis: i64,
    pub completed_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i16)]
pub enum ContactProviderLinkBindRejectCodeV1 {
    InvalidRequest = 1,
    ContactMissing = 2,
    StaleContactRevision = 3,
    ProviderLinkConflict = 4,
    Policy = 5,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactProviderLinkBindOutcomeV1 {
    Bound { contact_revision: u64 },
    Rejected(ContactProviderLinkBindRejectCodeV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedMailProviderLinkCommandV1 {
    pub outcome: ContactProviderLinkBindOutcomeV1,
    pub terminal_result: ContactsOutboxRecordV1,
    pub replayed: bool,
}
