#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    MailAddressBookEnvelopeBuildErrorV1, MailAddressBookEnvelopeContextV1,
    MailAddressBookResultEnvelopeContextV1, build_fetch_mail_address_book_page_command_v1,
    build_fetch_mail_person_source_page_command_v1, build_mail_address_book_entry_observed_v1,
    build_mail_address_book_entry_upsert_rejected_result_v1,
    build_mail_address_book_entry_upserted_result_v1,
    build_mail_address_book_page_completed_result_v1,
    build_mail_address_book_page_rejected_result_v1, build_mail_person_source_account_ready_v1,
    build_mail_person_source_account_retired_v1, build_mail_person_source_observed_v1,
    build_mail_person_source_page_completed_v1, build_mail_person_source_page_rejected_v1,
    build_mail_person_source_removed_v1, build_mail_person_source_updated_v1,
    build_upsert_mail_address_book_entry_command_v1,
};

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};
use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-mail-address-book-contract";
pub const MAIL_OWNER_ID_V1: &str = "mail";
pub const MAIL_RUNTIME_MODULE_ID_V1: &str = "makosh-mail-runtime";
pub const MAIL_PERSON_SOURCE_COMMAND_SOURCE_MODULE_ID_V1: &str = "makosh-mail-persons-sync-runtime";
pub const MAIL_ADDRESS_BOOK_CAPABILITY_ID_V1: &str = "mail.address-book.provider.v1";
pub const MAIL_ADDRESS_BOOK_CONTRACT_MAJOR_V1: u32 = 1;
pub const MAIL_ADDRESS_BOOK_CONTRACT_REVISION_V1: u32 = 3;
pub const MAIL_ADDRESS_BOOK_MAX_PAGE_SIZE_V1: u32 = 500;
pub const MAIL_ADDRESS_BOOK_MAX_CURSOR_BYTES_V1: usize = 4096;
pub const MAIL_ADDRESS_BOOK_MAX_SNAPSHOT_TICKET_BYTES_V1: usize = 4096;
pub const MAIL_ADDRESS_BOOK_MAX_IN_FLIGHT_V1: u32 = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAddressBookContractV1 {
    FetchPageCommand,
    EntryObserved,
    PageCompleted,
    PageRejected,
    UpsertEntryCommand,
    EntryUpserted,
    EntryUpsertRejected,
}

impl MailAddressBookContractV1 {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::FetchPageCommand => "mail_address_book_fetch_page",
            Self::EntryObserved => "mail_address_book_entry_observed",
            Self::PageCompleted => "mail_address_book_page_completed",
            Self::PageRejected => "mail_address_book_page_rejected",
            Self::UpsertEntryCommand => "mail_address_book_upsert_entry",
            Self::EntryUpserted => "mail_address_book_entry_upserted",
            Self::EntryUpsertRejected => "mail_address_book_entry_upsert_rejected",
        }
    }

    #[must_use]
    pub const fn envelope_kind(self) -> DurableEnvelopeKindV1 {
        match self {
            Self::FetchPageCommand | Self::UpsertEntryCommand => DurableEnvelopeKindV1::Command,
            Self::EntryObserved => DurableEnvelopeKindV1::Observation,
            Self::PageCompleted
            | Self::PageRejected
            | Self::EntryUpserted
            | Self::EntryUpsertRejected => DurableEnvelopeKindV1::Result,
        }
    }

    #[must_use]
    pub fn reference(self) -> ContractReferenceV1 {
        ContractReferenceV1 {
            owner: MAIL_OWNER_ID_V1.to_owned(),
            name: self.name().to_owned(),
            major: MAIL_ADDRESS_BOOK_CONTRACT_MAJOR_V1,
            revision: MAIL_ADDRESS_BOOK_CONTRACT_REVISION_V1,
            schema_sha256: MAIL_ADDRESS_BOOK_SCHEMA_SHA256_V1.to_vec(),
        }
    }

    #[must_use]
    pub fn publish_request(self) -> CapabilityRequestV1 {
        event_request(self, EventRouteDirectionV1::Publish)
    }

    #[must_use]
    pub fn consume_request(self) -> CapabilityRequestV1 {
        event_request(self, EventRouteDirectionV1::Consume)
    }
}

fn event_request(
    contract: MailAddressBookContractV1,
    direction: EventRouteDirectionV1,
) -> CapabilityRequestV1 {
    let consumes = direction == EventRouteDirectionV1::Consume;
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: contract.envelope_kind() as i32,
            contract: Some(contract.reference()),
            direction: direction as i32,
            max_in_flight: MAIL_ADDRESS_BOOK_MAX_IN_FLIGHT_V1,
            subscription_requirement: if consumes {
                EventSubscriptionRequirementV1::Required as i32
            } else {
                EventSubscriptionRequirementV1::Unspecified as i32
            },
            max_deliver: if consumes { 10 } else { 0 },
            ack_wait_millis: if consumes { 30_000 } else { 0 },
        })),
    }
}

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.mail.address_book.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/mail_address_book_schema.rs"));

pub mod wire_person_source {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.mail.address_book.person_source.v1.rs"
    ));
}

include!(concat!(env!("OUT_DIR"), "/mail_person_source_schema.rs"));

pub const MAIL_PERSON_SOURCE_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/mail-person-source-v1.bin"));
pub const MAIL_PERSON_SOURCE_CONTRACT_MAJOR_V1: u32 = 1;
pub const MAIL_PERSON_SOURCE_CONTRACT_REVISION_V1: u32 = 1;
pub const MAIL_PERSON_SOURCE_MAX_PAGE_SIZE_V1: u32 = 500;
pub const MAIL_PERSON_SOURCE_MAX_DISPLAY_NAME_CHARS_V1: usize = 240;
pub const MAIL_PERSON_SOURCE_MAX_EMAILS_V1: usize = 32;
pub const MAIL_PERSON_SOURCE_MAX_PHONES_V1: usize = 32;
pub const MAIL_PERSON_SOURCE_CAPABILITY_ID_V1: &str = "mail.person-source.provider.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonSourceContractV1 {
    FetchPageCommand,
    AccountReady,
    AccountRetired,
    SourceObserved,
    SourceUpdated,
    SourceRemoved,
    PageCompleted,
    PageRejected,
}

impl MailPersonSourceContractV1 {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::FetchPageCommand => "mail_person_source_fetch_page",
            Self::AccountReady => "mail_person_source_account_ready",
            Self::AccountRetired => "mail_person_source_account_retired",
            Self::SourceObserved => "mail_person_source_observed",
            Self::SourceUpdated => "mail_person_source_updated",
            Self::SourceRemoved => "mail_person_source_removed",
            Self::PageCompleted => "mail_person_source_page_completed",
            Self::PageRejected => "mail_person_source_page_rejected",
        }
    }

    #[must_use]
    pub const fn envelope_kind(self) -> DurableEnvelopeKindV1 {
        match self {
            Self::FetchPageCommand => DurableEnvelopeKindV1::Command,
            Self::AccountReady
            | Self::AccountRetired
            | Self::SourceObserved
            | Self::SourceUpdated
            | Self::SourceRemoved => DurableEnvelopeKindV1::Observation,
            Self::PageCompleted | Self::PageRejected => DurableEnvelopeKindV1::Result,
        }
    }

    #[must_use]
    pub fn reference(self) -> ContractReferenceV1 {
        ContractReferenceV1 {
            owner: MAIL_OWNER_ID_V1.to_owned(),
            name: self.name().to_owned(),
            major: MAIL_PERSON_SOURCE_CONTRACT_MAJOR_V1,
            revision: MAIL_PERSON_SOURCE_CONTRACT_REVISION_V1,
            schema_sha256: MAIL_PERSON_SOURCE_SCHEMA_SHA256_V1.to_vec(),
        }
    }

    #[must_use]
    pub fn publish_request(self) -> CapabilityRequestV1 {
        person_source_event_request(self, EventRouteDirectionV1::Publish)
    }

    #[must_use]
    pub fn consume_request(self) -> CapabilityRequestV1 {
        person_source_event_request(self, EventRouteDirectionV1::Consume)
    }
}

fn person_source_event_request(
    contract: MailPersonSourceContractV1,
    direction: EventRouteDirectionV1,
) -> CapabilityRequestV1 {
    let consumes = direction == EventRouteDirectionV1::Consume;
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: contract.envelope_kind() as i32,
            contract: Some(contract.reference()),
            direction: direction as i32,
            max_in_flight: MAIL_ADDRESS_BOOK_MAX_IN_FLIGHT_V1,
            subscription_requirement: if consumes {
                EventSubscriptionRequirementV1::Required as i32
            } else {
                EventSubscriptionRequirementV1::Unspecified as i32
            },
            max_deliver: if consumes { 10 } else { 0 },
            ack_wait_millis: if consumes { 30_000 } else { 0 },
        })),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonSourceValidationErrorV1 {
    InvalidPayload,
}

pub fn validate_mail_person_source_owner_v1(
    value: &str,
) -> Result<(), MailPersonSourceValidationErrorV1> {
    if !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        Ok(())
    } else {
        Err(MailPersonSourceValidationErrorV1::InvalidPayload)
    }
}

pub fn validate_mail_person_source_account_ready_v1(
    value: &wire_person_source::MailPersonSourceAccountReadyV1,
) -> Result<(), MailPersonSourceValidationErrorV1> {
    validate_account_lifecycle(
        &value.account_event_id,
        &value.logical_owner_id,
        &value.integration_public_id,
        &value.account_public_id,
        value.mapping_revision,
        value.observed_at.as_ref(),
    )
}

pub fn validate_mail_person_source_account_retired_v1(
    value: &wire_person_source::MailPersonSourceAccountRetiredV1,
) -> Result<(), MailPersonSourceValidationErrorV1> {
    validate_account_lifecycle(
        &value.account_event_id,
        &value.logical_owner_id,
        &value.integration_public_id,
        &value.account_public_id,
        value.mapping_revision,
        value.retired_at.as_ref(),
    )
}

fn validate_account_lifecycle(
    account_event_id: &[u8],
    logical_owner_id: &str,
    integration_public_id: &[u8],
    account_public_id: &[u8],
    mapping_revision: u64,
    occurred_at: Option<&prost_types::Timestamp>,
) -> Result<(), MailPersonSourceValidationErrorV1> {
    validate_mail_person_source_owner_v1(logical_owner_id)?;
    if valid_id16(account_event_id)
        && valid_id16(integration_public_id)
        && valid_id16(account_public_id)
        && mapping_revision > 0
        && valid_time(occurred_at)
    {
        Ok(())
    } else {
        Err(MailPersonSourceValidationErrorV1::InvalidPayload)
    }
}

pub fn validate_mail_person_source_claims_v1(
    value: &wire_person_source::MailPersonSourceClaimsV1,
) -> Result<(), MailPersonSourceValidationErrorV1> {
    let display_valid = value.display_name.as_deref().is_none_or(|display| {
        !display.is_empty()
            && display.chars().count() <= MAIL_PERSON_SOURCE_MAX_DISPLAY_NAME_CHARS_V1
            && display.trim() == display
            && !display.chars().any(char::is_control)
    });
    if display_valid
        && value.normalized_emails.len() <= MAIL_PERSON_SOURCE_MAX_EMAILS_V1
        && value.normalized_phones.len() <= MAIL_PERSON_SOURCE_MAX_PHONES_V1
        && sorted_unique(&value.normalized_emails)
        && sorted_unique(&value.normalized_phones)
        && value
            .normalized_emails
            .iter()
            .all(|value| valid_email(value))
        && value
            .normalized_phones
            .iter()
            .all(|value| valid_phone(value))
        && (value.display_name.is_some()
            || !value.normalized_emails.is_empty()
            || !value.normalized_phones.is_empty())
    {
        Ok(())
    } else {
        Err(MailPersonSourceValidationErrorV1::InvalidPayload)
    }
}

const MAIL_PERSON_SOURCE_CLAIMS_DIGEST_DOMAIN_V1: &[u8] =
    b"makosh.mail.person-source.public-claims.v1";
const MAIL_PERSON_SOURCE_TOMBSTONE_DIGEST_DOMAIN_V1: &[u8] =
    b"makosh.mail.person-source.public-tombstone.v1";

pub fn mail_person_source_claims_digest_v1(
    source: &wire_person_source::MailPersonSourceIdentityV1,
    claims: &wire_person_source::MailPersonSourceClaimsV1,
) -> Result<[u8; 32], MailPersonSourceValidationErrorV1> {
    validate_public_source(Some(source))?;
    validate_mail_person_source_claims_v1(claims)?;
    let mut digest = public_source_digest(MAIL_PERSON_SOURCE_CLAIMS_DIGEST_DOMAIN_V1, source);
    match claims.display_name.as_deref() {
        None => digest.update([0]),
        Some(display_name) => {
            digest.update([1]);
            digest_part(&mut digest, display_name.as_bytes());
        }
    }
    digest_sequence(&mut digest, &claims.normalized_emails);
    digest_sequence(&mut digest, &claims.normalized_phones);
    Ok(digest.finalize().into())
}

pub fn mail_person_source_tombstone_digest_v1(
    source: &wire_person_source::MailPersonSourceIdentityV1,
) -> Result<[u8; 32], MailPersonSourceValidationErrorV1> {
    validate_public_source(Some(source))?;
    Ok(
        public_source_digest(MAIL_PERSON_SOURCE_TOMBSTONE_DIGEST_DOMAIN_V1, source)
            .finalize()
            .into(),
    )
}

fn public_source_digest(
    domain: &[u8],
    source: &wire_person_source::MailPersonSourceIdentityV1,
) -> Sha256 {
    let mut digest = Sha256::new();
    digest_part(&mut digest, domain);
    digest_part(&mut digest, &source.integration_public_id);
    digest_part(&mut digest, &source.account_public_id);
    digest_part(&mut digest, &source.provider_source_contact_public_id);
    digest
}

fn digest_sequence(digest: &mut Sha256, values: &[String]) {
    digest.update((values.len() as u64).to_be_bytes());
    for value in values {
        digest_part(digest, value.as_bytes());
    }
}

fn digest_part(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

pub fn validate_mail_person_source_observed_v1(
    value: &wire_person_source::MailPersonSourceObservedV1,
) -> Result<(), MailPersonSourceValidationErrorV1> {
    validate_source_change(
        &value.observation_id,
        &value.run_id,
        &value.logical_owner_id,
        value.page_sequence,
        value.source.as_ref(),
        value.claims.as_ref(),
        value.provenance.as_ref(),
    )
}

pub fn validate_mail_person_source_updated_v1(
    value: &wire_person_source::MailPersonSourceUpdatedV1,
) -> Result<(), MailPersonSourceValidationErrorV1> {
    validate_source_change(
        &value.observation_id,
        &value.run_id,
        &value.logical_owner_id,
        value.page_sequence,
        value.source.as_ref(),
        value.claims.as_ref(),
        value.provenance.as_ref(),
    )
}

pub fn validate_mail_person_source_removed_v1(
    value: &wire_person_source::MailPersonSourceRemovedV1,
) -> Result<(), MailPersonSourceValidationErrorV1> {
    validate_public_header(
        &value.observation_id,
        &value.run_id,
        &value.logical_owner_id,
        value.page_sequence,
    )?;
    let source = value
        .source
        .as_ref()
        .ok_or(MailPersonSourceValidationErrorV1::InvalidPayload)?;
    validate_public_source(Some(source))?;
    let provenance = value
        .provenance
        .as_ref()
        .ok_or(MailPersonSourceValidationErrorV1::InvalidPayload)?;
    validate_public_provenance(Some(provenance))?;
    if provenance.source_digest == mail_person_source_tombstone_digest_v1(source)? {
        Ok(())
    } else {
        Err(MailPersonSourceValidationErrorV1::InvalidPayload)
    }
}

pub fn validate_fetch_mail_person_source_page_v1(
    value: &wire_person_source::FetchMailPersonSourcePageCommandV1,
) -> Result<(), MailPersonSourceValidationErrorV1> {
    validate_public_header(
        &value.command_id,
        &value.run_id,
        &value.logical_owner_id,
        value.page_sequence,
    )?;
    if valid_id16(&value.account_public_id)
        && (1..=MAIL_PERSON_SOURCE_MAX_PAGE_SIZE_V1).contains(&value.page_size)
    {
        Ok(())
    } else {
        Err(MailPersonSourceValidationErrorV1::InvalidPayload)
    }
}

pub fn validate_mail_person_source_page_completed_v1(
    value: &wire_person_source::MailPersonSourcePageCompletedV1,
) -> Result<(), MailPersonSourceValidationErrorV1> {
    validate_public_header(
        &value.command_id,
        &value.run_id,
        &value.logical_owner_id,
        value.page_sequence,
    )?;
    let sources = value
        .observed_sources
        .checked_add(value.updated_sources)
        .and_then(|count| count.checked_add(value.removed_sources));
    if valid_id16(&value.account_public_id)
        && sources.is_some_and(|count| count <= MAIL_PERSON_SOURCE_MAX_PAGE_SIZE_V1)
        && valid_id32(&value.page_digest)
        && valid_time(value.completed_at.as_ref())
    {
        Ok(())
    } else {
        Err(MailPersonSourceValidationErrorV1::InvalidPayload)
    }
}

pub fn validate_mail_person_source_page_rejected_v1(
    value: &wire_person_source::MailPersonSourcePageRejectedV1,
) -> Result<(), MailPersonSourceValidationErrorV1> {
    use wire_person_source::MailPersonSourceRejectCodeV1;

    validate_public_header(
        &value.command_id,
        &value.run_id,
        &value.logical_owner_id,
        value.page_sequence,
    )?;
    let code = MailPersonSourceRejectCodeV1::try_from(value.code)
        .map_err(|_| MailPersonSourceValidationErrorV1::InvalidPayload)?;
    let retryable =
        code == MailPersonSourceRejectCodeV1::MailPersonSourceRejectCodeSourceUnavailable;
    if valid_id16(&value.account_public_id)
        && code != MailPersonSourceRejectCodeV1::MailPersonSourceRejectCodeUnspecified
        && value.retryable == retryable
        && valid_time(value.rejected_at.as_ref())
    {
        Ok(())
    } else {
        Err(MailPersonSourceValidationErrorV1::InvalidPayload)
    }
}

fn validate_source_change(
    observation_id: &[u8],
    run_id: &[u8],
    logical_owner_id: &str,
    page_sequence: u64,
    source: Option<&wire_person_source::MailPersonSourceIdentityV1>,
    claims: Option<&wire_person_source::MailPersonSourceClaimsV1>,
    provenance: Option<&wire_person_source::MailPersonSourceProvenanceV1>,
) -> Result<(), MailPersonSourceValidationErrorV1> {
    validate_public_header(observation_id, run_id, logical_owner_id, page_sequence)?;
    let source = source.ok_or(MailPersonSourceValidationErrorV1::InvalidPayload)?;
    validate_public_source(Some(source))?;
    let claims = claims.ok_or(MailPersonSourceValidationErrorV1::InvalidPayload)?;
    validate_public_claims(Some(claims))?;
    let provenance = provenance.ok_or(MailPersonSourceValidationErrorV1::InvalidPayload)?;
    validate_public_provenance(Some(provenance))?;
    if provenance.source_digest == mail_person_source_claims_digest_v1(source, claims)? {
        Ok(())
    } else {
        Err(MailPersonSourceValidationErrorV1::InvalidPayload)
    }
}

fn validate_public_header(
    observation_id: &[u8],
    run_id: &[u8],
    logical_owner_id: &str,
    page_sequence: u64,
) -> Result<(), MailPersonSourceValidationErrorV1> {
    if !valid_id16(observation_id) || !valid_id16(run_id) || page_sequence == 0 {
        return Err(MailPersonSourceValidationErrorV1::InvalidPayload);
    }
    validate_mail_person_source_owner_v1(logical_owner_id)
}

fn validate_public_source(
    value: Option<&wire_person_source::MailPersonSourceIdentityV1>,
) -> Result<(), MailPersonSourceValidationErrorV1> {
    if value.is_some_and(|value| {
        valid_id16(&value.integration_public_id)
            && valid_id16(&value.account_public_id)
            && valid_id16(&value.provider_source_contact_public_id)
    }) {
        Ok(())
    } else {
        Err(MailPersonSourceValidationErrorV1::InvalidPayload)
    }
}

fn validate_public_claims(
    value: Option<&wire_person_source::MailPersonSourceClaimsV1>,
) -> Result<(), MailPersonSourceValidationErrorV1> {
    value
        .ok_or(MailPersonSourceValidationErrorV1::InvalidPayload)
        .and_then(validate_mail_person_source_claims_v1)
}

fn validate_public_provenance(
    value: Option<&wire_person_source::MailPersonSourceProvenanceV1>,
) -> Result<(), MailPersonSourceValidationErrorV1> {
    if value.is_some_and(|value| {
        value.source_revision > 0
            && valid_id32(&value.source_digest)
            && valid_time(value.observed_at.as_ref())
    }) {
        Ok(())
    } else {
        Err(MailPersonSourceValidationErrorV1::InvalidPayload)
    }
}

fn valid_time(value: Option<&prost_types::Timestamp>) -> bool {
    value.is_some_and(|value| value.seconds > 0 && (0..1_000_000_000).contains(&value.nanos))
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn valid_email(value: &str) -> bool {
    if value.is_empty()
        || value.len() > 320
        || value.trim() != value
        || value.to_lowercase() != value
        || value.matches('@').count() != 1
        || value.chars().any(char::is_control)
    {
        return false;
    }
    value.split_once('@').is_some_and(|(local, domain)| {
        !local.is_empty()
            && !domain.is_empty()
            && domain.contains('.')
            && !domain.starts_with('.')
            && !domain.ends_with('.')
            && !local.chars().any(char::is_whitespace)
            && !domain.chars().any(char::is_whitespace)
    })
}

fn valid_phone(value: &str) -> bool {
    value.strip_prefix('+').is_some_and(|digits| {
        (7..=15).contains(&digits.len())
            && !digits.starts_with('0')
            && digits.bytes().all(|byte| byte.is_ascii_digit())
    })
}

#[cfg(test)]
mod person_source_tests {
    use super::*;

    fn public_source() -> wire_person_source::MailPersonSourceIdentityV1 {
        wire_person_source::MailPersonSourceIdentityV1 {
            integration_public_id: vec![3; 16],
            account_public_id: vec![4; 16],
            provider_source_contact_public_id: vec![5; 16],
        }
    }

    #[test]
    fn public_account_lifecycle_is_bounded_and_provider_private_free() {
        let ready = wire_person_source::MailPersonSourceAccountReadyV1 {
            account_event_id: vec![0x11; 16],
            logical_owner_id: "owner-1".to_owned(),
            integration_public_id: vec![0x12; 16],
            account_public_id: vec![0x13; 16],
            mapping_revision: 1,
            observed_at: Some(prost_types::Timestamp {
                seconds: 1_700_000_000,
                nanos: 0,
            }),
        };
        validate_mail_person_source_account_ready_v1(&ready).expect("valid ready");
        let retired = wire_person_source::MailPersonSourceAccountRetiredV1 {
            account_event_id: vec![0x21; 16],
            logical_owner_id: ready.logical_owner_id.clone(),
            integration_public_id: ready.integration_public_id.clone(),
            account_public_id: ready.account_public_id.clone(),
            mapping_revision: 2,
            retired_at: Some(prost_types::Timestamp {
                seconds: 1_700_000_001,
                nanos: 0,
            }),
        };
        validate_mail_person_source_account_retired_v1(&retired).expect("valid retired");

        for mutation in [
            |value: &mut wire_person_source::MailPersonSourceAccountReadyV1| {
                value.account_event_id.clear();
            },
            |value: &mut wire_person_source::MailPersonSourceAccountReadyV1| {
                value.logical_owner_id = "OWNER:private".to_owned();
            },
            |value: &mut wire_person_source::MailPersonSourceAccountReadyV1| {
                value.integration_public_id = vec![0x12; 15];
            },
            |value: &mut wire_person_source::MailPersonSourceAccountReadyV1| {
                value.account_public_id = vec![0; 16];
            },
            |value: &mut wire_person_source::MailPersonSourceAccountReadyV1| {
                value.mapping_revision = 0;
            },
        ] {
            let mut invalid = ready.clone();
            mutation(&mut invalid);
            assert!(validate_mail_person_source_account_ready_v1(&invalid).is_err());
        }

        let proto =
            include_str!("../proto/makosh/mail/address_book/person_source/v1/person_source.proto");
        let lifecycle = proto
            .split("message MailPersonSourceAccountReadyV1")
            .nth(1)
            .expect("ready")
            .split("message MailPersonSourceObservedV1")
            .next()
            .expect("lifecycle messages");
        assert!(!lifecycle.contains("provider_account"));
        assert!(!lifecycle.contains("private_account"));
        assert!(!lifecycle.contains("credential"));
        assert!(!lifecycle.contains("cursor"));
        assert!(!lifecycle.contains("locator"));
    }

    fn public_claims() -> wire_person_source::MailPersonSourceClaimsV1 {
        wire_person_source::MailPersonSourceClaimsV1 {
            display_name: Some("Ada Lovelace".to_owned()),
            normalized_emails: vec!["ada@example.test".to_owned()],
            normalized_phones: vec!["+34910000000".to_owned()],
        }
    }

    fn provenance(source_digest: [u8; 32]) -> wire_person_source::MailPersonSourceProvenanceV1 {
        wire_person_source::MailPersonSourceProvenanceV1 {
            source_revision: 1,
            source_digest: source_digest.to_vec(),
            observed_at: Some(prost_types::Timestamp {
                seconds: 1,
                nanos: 0,
            }),
        }
    }

    #[test]
    fn public_source_schema_is_separate_bounded_and_provider_private_negative() {
        let source =
            include_str!("../proto/makosh/mail/address_book/person_source/v1/person_source.proto");
        for message in [
            "FetchMailPersonSourcePageCommandV1",
            "MailPersonSourceObservedV1",
            "MailPersonSourceUpdatedV1",
            "MailPersonSourceRemovedV1",
            "MailPersonSourcePageCompletedV1",
            "MailPersonSourcePageRejectedV1",
        ] {
            assert!(source.contains(&format!("message {message}")), "{message}");
        }
        for forbidden in [
            "provider_entry_id",
            "provider_etag",
            "continuation_cursor",
            "credential",
            "private_locator",
            "raw_payload",
            "error_detail",
        ] {
            assert!(!source.to_lowercase().contains(forbidden), "{forbidden}");
        }
        assert_eq!(MAIL_PERSON_SOURCE_MAX_PAGE_SIZE_V1, 500);
        assert_eq!(
            MailPersonSourceContractV1::SourceObserved.reference().owner,
            "mail"
        );
    }

    #[test]
    fn observed_contract_rejects_unbounded_unsorted_or_duplicate_claims() {
        let source = public_source();
        let claims = public_claims();
        let valid = wire_person_source::MailPersonSourceObservedV1 {
            observation_id: vec![1; 16],
            run_id: vec![2; 16],
            logical_owner_id: "owner-1".to_owned(),
            page_sequence: 1,
            source: Some(source.clone()),
            claims: Some(claims.clone()),
            provenance: Some(provenance(
                mail_person_source_claims_digest_v1(&source, &claims).expect("canonical digest"),
            )),
        };
        validate_mail_person_source_observed_v1(&valid).expect("valid public observation");

        let mut duplicate = valid.clone();
        duplicate
            .claims
            .as_mut()
            .expect("claims")
            .normalized_emails
            .push("ada@example.test".to_owned());
        assert!(validate_mail_person_source_observed_v1(&duplicate).is_err());

        let mut unbounded = valid;
        unbounded.claims.as_mut().expect("claims").display_name = Some("x".repeat(241));
        assert!(
            validate_mail_person_source_claims_v1(unbounded.claims.as_ref().expect("claims"))
                .is_err()
        );
    }

    #[test]
    fn owner_and_display_are_exactly_persons_compatible() {
        for invalid_owner in [
            "Owner-1".to_owned(),
            "owner:1".to_owned(),
            "owner/1".to_owned(),
            "x".repeat(129),
        ] {
            assert!(validate_mail_person_source_owner_v1(&invalid_owner).is_err());
        }
        for valid_owner in ["owner-1", "owner_1", "owner.1", "a"] {
            validate_mail_person_source_owner_v1(valid_owner).expect("Persons-compatible owner");
        }

        let source = public_source();
        let mut claims = public_claims();
        claims.display_name = Some("Ж".repeat(240));
        let valid = wire_person_source::MailPersonSourceObservedV1 {
            observation_id: vec![1; 16],
            run_id: vec![2; 16],
            logical_owner_id: "owner-1".to_owned(),
            page_sequence: 1,
            source: Some(source.clone()),
            claims: Some(claims.clone()),
            provenance: Some(provenance(
                mail_person_source_claims_digest_v1(&source, &claims).expect("digest"),
            )),
        };
        validate_mail_person_source_observed_v1(&valid).expect("240 Unicode chars");

        claims.display_name = Some("Ж".repeat(241));
        assert!(validate_mail_person_source_claims_v1(&claims).is_err());
    }

    #[test]
    fn source_digest_is_canonical_public_only_and_bound_to_exact_claims() {
        use sha2::{Digest, Sha256};

        let source = public_source();
        let claims = public_claims();
        let digest = mail_person_source_claims_digest_v1(&source, &claims).expect("digest");
        assert_eq!(
            digest,
            mail_person_source_claims_digest_v1(&source, &claims).expect("same digest")
        );

        let mut changed = claims.clone();
        changed.display_name = Some("Ada Byron".to_owned());
        assert_ne!(
            digest,
            mail_person_source_claims_digest_v1(&source, &changed).expect("changed digest")
        );

        let mut observation = wire_person_source::MailPersonSourceObservedV1 {
            observation_id: vec![1; 16],
            run_id: vec![2; 16],
            logical_owner_id: "owner-1".to_owned(),
            page_sequence: 1,
            source: Some(source.clone()),
            claims: Some(changed),
            provenance: Some(provenance(digest)),
        };
        assert!(validate_mail_person_source_observed_v1(&observation).is_err());

        let mut private_material = Sha256::new();
        private_material.update(digest);
        private_material.update(b"provider-private-record-id-or-etag");
        observation.claims = Some(claims);
        observation
            .provenance
            .as_mut()
            .expect("provenance")
            .source_digest = private_material.finalize().to_vec();
        assert!(validate_mail_person_source_observed_v1(&observation).is_err());
    }

    #[test]
    fn removed_source_requires_the_separate_canonical_tombstone_digest() {
        let source = public_source();
        let claims = public_claims();
        let tombstone = mail_person_source_tombstone_digest_v1(&source).expect("tombstone digest");
        assert_ne!(
            tombstone,
            mail_person_source_claims_digest_v1(&source, &claims).expect("claims digest")
        );
        let mut removed = wire_person_source::MailPersonSourceRemovedV1 {
            observation_id: vec![1; 16],
            run_id: vec![2; 16],
            logical_owner_id: "owner-1".to_owned(),
            page_sequence: 1,
            source: Some(source.clone()),
            provenance: Some(provenance(tombstone)),
        };
        validate_mail_person_source_removed_v1(&removed).expect("canonical tombstone");
        removed
            .provenance
            .as_mut()
            .expect("provenance")
            .source_digest = mail_person_source_claims_digest_v1(&source, &claims)
            .expect("claims digest")
            .to_vec();
        assert!(validate_mail_person_source_removed_v1(&removed).is_err());
    }

    #[test]
    fn fetch_completion_rejection_and_every_route_are_exact_and_bounded() {
        use wire_person_source::MailPersonSourceRejectCodeV1;

        let fetch = wire_person_source::FetchMailPersonSourcePageCommandV1 {
            command_id: vec![1; 16],
            run_id: vec![2; 16],
            logical_owner_id: "owner-1".to_owned(),
            account_public_id: vec![3; 16],
            page_sequence: 1,
            page_size: MAIL_PERSON_SOURCE_MAX_PAGE_SIZE_V1,
        };
        validate_fetch_mail_person_source_page_v1(&fetch).expect("bounded fetch");
        let mut unbounded_fetch = fetch;
        unbounded_fetch.page_size += 1;
        assert!(validate_fetch_mail_person_source_page_v1(&unbounded_fetch).is_err());

        let completed = wire_person_source::MailPersonSourcePageCompletedV1 {
            command_id: vec![1; 16],
            run_id: vec![2; 16],
            logical_owner_id: "owner-1".to_owned(),
            account_public_id: vec![3; 16],
            page_sequence: 1,
            observed_sources: 200,
            updated_sources: 200,
            removed_sources: 100,
            has_more: true,
            page_digest: vec![4; 32],
            completed_at: Some(prost_types::Timestamp {
                seconds: 1,
                nanos: 0,
            }),
        };
        validate_mail_person_source_page_completed_v1(&completed).expect("bounded completion");
        let mut overflow = completed;
        overflow.removed_sources = 101;
        assert!(validate_mail_person_source_page_completed_v1(&overflow).is_err());

        let rejected = wire_person_source::MailPersonSourcePageRejectedV1 {
            command_id: vec![1; 16],
            run_id: vec![2; 16],
            logical_owner_id: "owner-1".to_owned(),
            account_public_id: vec![3; 16],
            page_sequence: 1,
            code: MailPersonSourceRejectCodeV1::MailPersonSourceRejectCodeSourceUnavailable as i32,
            retryable: true,
            rejected_at: Some(prost_types::Timestamp {
                seconds: 1,
                nanos: 0,
            }),
        };
        validate_mail_person_source_page_rejected_v1(&rejected).expect("typed rejection");
        let mut wrong_retry = rejected;
        wrong_retry.retryable = false;
        assert!(validate_mail_person_source_page_rejected_v1(&wrong_retry).is_err());

        let contracts = [
            MailPersonSourceContractV1::FetchPageCommand,
            MailPersonSourceContractV1::AccountReady,
            MailPersonSourceContractV1::AccountRetired,
            MailPersonSourceContractV1::SourceObserved,
            MailPersonSourceContractV1::SourceUpdated,
            MailPersonSourceContractV1::SourceRemoved,
            MailPersonSourceContractV1::PageCompleted,
            MailPersonSourceContractV1::PageRejected,
        ];
        let names = contracts.map(MailPersonSourceContractV1::name);
        assert_eq!(
            names
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            8
        );
        for contract in contracts {
            assert_eq!(contract.reference().owner, MAIL_OWNER_ID_V1);
            assert!(contract.publish_request().request.is_some());
            assert!(contract.consume_request().request.is_some());
        }
    }
}

pub fn validate_mail_address_book_entry_upserted_v1(
    payload: &wire::MailAddressBookEntryUpsertedV1,
) -> Result<(), MailAddressBookEnvelopeBuildErrorV1> {
    let provider = wire::MailAddressBookProviderKindV1::try_from(payload.provider_kind)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?;
    if valid_id16(&payload.command_id)
        && valid_id16(&payload.run_id)
        && valid_ascii(&payload.provider_entry_id, 512)
        && valid_ascii(&payload.provider_etag, 512)
        && payload.applied_contact_revision > 0
        && provider != wire::MailAddressBookProviderKindV1::MailAddressBookProviderKindUnspecified
    {
        Ok(())
    } else {
        Err(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)
    }
}

pub fn validate_mail_address_book_entry_observed_v1(
    payload: &wire::MailAddressBookEntryObservedV1,
) -> Result<(), MailAddressBookEnvelopeBuildErrorV1> {
    use wire::MailAddressBookProviderKindV1;

    let provider = MailAddressBookProviderKindV1::try_from(payload.provider_kind)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?;
    let observed_at = payload
        .observed_at
        .as_ref()
        .ok_or(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?;
    if valid_id16(&payload.observation_id)
        && valid_id16(&payload.run_id)
        && valid_identity(&payload.logical_owner_id, 128)
        && valid_identity(&payload.account_id, 256)
        && provider != MailAddressBookProviderKindV1::MailAddressBookProviderKindUnspecified
        && valid_ascii(&payload.provider_entry_id, 512)
        && payload
            .provider_etag
            .as_deref()
            .is_none_or(|value| valid_ascii(value, 512))
        && valid_private_text(&payload.display_name)
        && payload.email_addresses.len() <= 32
        && payload.phone_numbers.len() <= 32
        && (!payload.email_addresses.is_empty()
            || !payload.phone_numbers.is_empty()
            || !payload.display_name.is_empty())
        && payload
            .email_addresses
            .iter()
            .all(|value| valid_private_text(value))
        && payload
            .phone_numbers
            .iter()
            .all(|value| valid_private_text(value))
        && observed_at.seconds > 0
        && (0..1_000_000_000).contains(&observed_at.nanos)
        && payload.source_revision > 0
        && valid_id32(&payload.entry_digest)
        && payload.page_sequence > 0
    {
        Ok(())
    } else {
        Err(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)
    }
}

pub fn validate_mail_address_book_page_completed_v1(
    payload: &wire::MailAddressBookPageCompletedV1,
) -> Result<(), MailAddressBookEnvelopeBuildErrorV1> {
    if valid_id16(&payload.command_id)
        && valid_id16(&payload.run_id)
        && payload.page_sequence > 0
        && payload.observed_entries <= MAIL_ADDRESS_BOOK_MAX_PAGE_SIZE_V1
        && payload
            .next_continuation_cursor
            .as_ref()
            .is_none_or(|cursor| {
                !cursor.is_empty() && cursor.len() <= MAIL_ADDRESS_BOOK_MAX_CURSOR_BYTES_V1
            })
    {
        Ok(())
    } else {
        Err(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)
    }
}

pub fn validate_mail_address_book_page_rejected_v1(
    payload: &wire::MailAddressBookPageRejectedV1,
) -> Result<(), MailAddressBookEnvelopeBuildErrorV1> {
    use wire::MailAddressBookRejectCodeV1;

    let code = MailAddressBookRejectCodeV1::try_from(payload.code)
        .map_err(|_| MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)?;
    let retryable = matches!(
        code,
        MailAddressBookRejectCodeV1::MailAddressBookRejectCodeProviderUnavailable
            | MailAddressBookRejectCodeV1::MailAddressBookRejectCodeCredentialUnavailable
    );
    if valid_id16(&payload.command_id)
        && valid_id16(&payload.run_id)
        && code != MailAddressBookRejectCodeV1::MailAddressBookRejectCodeUnspecified
        && payload.retryable == retryable
    {
        Ok(())
    } else {
        Err(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)
    }
}

pub fn validate_mail_address_book_entry_upsert_rejected_v1(
    payload: &wire::MailAddressBookEntryUpsertRejectedV1,
) -> Result<(), MailAddressBookEnvelopeBuildErrorV1> {
    use wire::MailAddressBookRejectCodeV1;

    let Ok(code) = MailAddressBookRejectCodeV1::try_from(payload.code) else {
        return Err(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload);
    };
    let outcome_unknown =
        code == MailAddressBookRejectCodeV1::MailAddressBookRejectCodeOutcomeUnknown;
    if valid_id16(&payload.command_id)
        && valid_id16(&payload.run_id)
        && code != MailAddressBookRejectCodeV1::MailAddressBookRejectCodeUnspecified
        && payload.outcome_unknown == outcome_unknown
    {
        Ok(())
    } else {
        Err(MailAddressBookEnvelopeBuildErrorV1::InvalidPayload)
    }
}

fn valid_id16(value: &[u8]) -> bool {
    value.len() == 16 && value.iter().any(|byte| *byte != 0)
}

fn valid_id32(value: &[u8]) -> bool {
    value.len() == 32 && value.iter().any(|byte| *byte != 0)
}

fn valid_identity(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
}

fn valid_private_text(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 2_048
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn valid_ascii(value: &str, max: usize) -> bool {
    !value.is_empty() && value.len() <= max && value.is_ascii() && value.trim() == value
}

pub const MAIL_ADDRESS_BOOK_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/mail-address-book-v1.bin"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_keeps_provider_protocol_in_mail_and_payloads_bounded() {
        let source = include_str!("../proto/makosh/mail/address_book/v1/address_book.proto");
        let fetch_command = message_source(source, "FetchMailAddressBookPageCommandV1");
        let observed_entry = message_source(source, "MailAddressBookEntryObservedV1");
        let upsert_command = message_source(source, "UpsertMailAddressBookEntryCommandV1");
        assert!(source.contains("GOOGLE_PEOPLE"));
        assert!(source.contains("ICLOUD_CARDDAV"));
        assert!(!upsert_command.contains("provider_kind"));
        assert!(!upsert_command.contains("provider_entry_id"));
        assert!(!upsert_command.contains("provider_etag"));
        assert!(source.contains("outcome_unknown"));
        assert!(!fetch_command.contains("provider_kind"));
        assert!(observed_entry.contains("provider_kind"));
        for forbidden in [
            "password",
            "access_token",
            "refresh_token",
            "cookie",
            "map<",
            "raw_json",
            "raw_xml",
        ] {
            assert!(!source.contains(forbidden), "forbidden field: {forbidden}");
        }
    }

    fn message_source<'a>(source: &'a str, name: &str) -> &'a str {
        let start = source
            .find(&format!("message {name} {{"))
            .expect("message start");
        let tail = &source[start..];
        let end = tail.find("\n}").expect("message end") + 2;
        &tail[..end]
    }

    #[test]
    fn descriptor_and_limits_are_non_empty() {
        assert!(!MAIL_ADDRESS_BOOK_DESCRIPTOR_SET_V1.is_empty());
        assert_ne!(MAIL_ADDRESS_BOOK_SCHEMA_SHA256_V1, [0; 32]);
        assert_eq!(MAIL_ADDRESS_BOOK_MAX_PAGE_SIZE_V1, 500);
    }

    #[test]
    fn event_contracts_have_exact_mail_owner_kind_and_complementary_routes() {
        use makosh_runtime_protocol::v1::{
            DurableEnvelopeKindV1, EventRouteDirectionV1, capability_request_v1::Request,
        };

        let contracts = [
            MailAddressBookContractV1::FetchPageCommand,
            MailAddressBookContractV1::EntryObserved,
            MailAddressBookContractV1::PageCompleted,
            MailAddressBookContractV1::PageRejected,
            MailAddressBookContractV1::UpsertEntryCommand,
            MailAddressBookContractV1::EntryUpserted,
            MailAddressBookContractV1::EntryUpsertRejected,
        ];
        assert_eq!(
            contracts.map(MailAddressBookContractV1::name).as_slice(),
            [
                "mail_address_book_fetch_page",
                "mail_address_book_entry_observed",
                "mail_address_book_page_completed",
                "mail_address_book_page_rejected",
                "mail_address_book_upsert_entry",
                "mail_address_book_entry_upserted",
                "mail_address_book_entry_upsert_rejected",
            ]
        );
        assert_eq!(
            MailAddressBookContractV1::EntryObserved.envelope_kind(),
            DurableEnvelopeKindV1::Observation
        );
        for contract in contracts {
            assert_eq!(contract.reference().owner, MAIL_OWNER_ID_V1);
            let Some(Request::EventRoute(publish)) = contract.publish_request().request else {
                panic!("publish route");
            };
            let Some(Request::EventRoute(consume)) = contract.consume_request().request else {
                panic!("consume route");
            };
            assert_eq!(publish.direction, EventRouteDirectionV1::Publish as i32);
            assert_eq!(consume.direction, EventRouteDirectionV1::Consume as i32);
            assert_eq!(publish.contract, consume.contract);
        }
    }
}
