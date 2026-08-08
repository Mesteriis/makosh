#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    MailAddressBookEnvelopeBuildErrorV1, MailAddressBookEnvelopeContextV1,
    MailAddressBookResultEnvelopeContextV1, build_fetch_mail_address_book_page_command_v1,
    build_mail_address_book_entry_observed_v1,
    build_mail_address_book_entry_upsert_rejected_result_v1,
    build_mail_address_book_entry_upserted_result_v1,
    build_mail_address_book_page_completed_result_v1,
    build_mail_address_book_page_rejected_result_v1,
    build_upsert_mail_address_book_entry_command_v1,
};

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-mail-address-book-contract";
pub const MAIL_OWNER_ID_V1: &str = "mail";
pub const MAIL_RUNTIME_MODULE_ID_V1: &str = "makosh-mail-runtime";
pub const MAIL_ADDRESS_BOOK_COMMAND_SOURCE_MODULE_ID_V1: &str = "makosh-mail-contacts-sync-runtime";
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
