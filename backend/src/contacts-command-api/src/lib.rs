#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    ContactsCommandEnvelopeBuildErrorV1, ContactsCommandEnvelopeContextV1,
    build_bind_mail_address_book_provider_link_command_outbox_record_v1,
    build_bind_mail_address_book_provider_link_rejected_outbox_record_v1,
    build_contact_upsert_rejected_outbox_record_v1, build_contact_upserted_outbox_record_v1,
    build_mail_address_book_provider_link_bound_outbox_record_v1,
    build_upsert_contact_command_outbox_record_v1,
};

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-contacts-command-api";
pub const CONTACTS_OWNER_ID_V1: &str = "contacts";
pub const CONTACTS_MODULE_ID_V1: &str = "makosh-contacts-runtime";
pub const CONTACTS_MAIL_IDENTITY_COMMAND_CAPABILITY_ID_V1: &str =
    "contacts.mail-identity.command.v1";
pub const CONTACTS_MAIL_PROVIDER_LINK_COMMAND_CAPABILITY_ID_V1: &str =
    "contacts.mail-provider-link.command.v1";
pub const UPSERT_CONTACT_FROM_MAIL_ENTRY_CONTRACT_NAME_V1: &str =
    "contacts_upsert_from_mail_address_book_entry";
pub const CONTACT_UPSERTED_FROM_MAIL_ENTRY_CONTRACT_NAME_V1: &str =
    "contacts_upserted_from_mail_address_book_entry";
pub const CONTACT_UPSERT_FROM_MAIL_ENTRY_REJECTED_CONTRACT_NAME_V1: &str =
    "contacts_upsert_from_mail_address_book_entry_rejected";
pub const BIND_MAIL_ADDRESS_BOOK_PROVIDER_LINK_CONTRACT_NAME_V1: &str =
    "contacts_bind_mail_address_book_provider_link";
pub const MAIL_ADDRESS_BOOK_PROVIDER_LINK_BOUND_CONTRACT_NAME_V1: &str =
    "contacts_mail_address_book_provider_link_bound";
pub const BIND_MAIL_ADDRESS_BOOK_PROVIDER_LINK_REJECTED_CONTRACT_NAME_V1: &str =
    "contacts_bind_mail_address_book_provider_link_rejected";
pub const CONTACTS_COMMAND_CONTRACT_MAJOR_V1: u32 = 1;
pub const CONTACTS_COMMAND_CONTRACT_REVISION_V1: u32 = 1;
pub const CONTACTS_MAIL_IDENTITY_MAX_IN_FLIGHT_V1: u32 = 32;

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.contacts.command.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/contacts_command_schema.rs"));

pub const CONTACTS_COMMAND_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/contacts-command-v1.bin"));

#[must_use]
pub fn upsert_contact_command_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(UPSERT_CONTACT_FROM_MAIL_ENTRY_CONTRACT_NAME_V1)
}

#[must_use]
pub fn contact_upserted_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(CONTACT_UPSERTED_FROM_MAIL_ENTRY_CONTRACT_NAME_V1)
}

#[must_use]
pub fn contact_upsert_rejected_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(CONTACT_UPSERT_FROM_MAIL_ENTRY_REJECTED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn bind_mail_address_book_provider_link_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(BIND_MAIL_ADDRESS_BOOK_PROVIDER_LINK_CONTRACT_NAME_V1)
}

#[must_use]
pub fn mail_address_book_provider_link_bound_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(MAIL_ADDRESS_BOOK_PROVIDER_LINK_BOUND_CONTRACT_NAME_V1)
}

#[must_use]
pub fn bind_mail_address_book_provider_link_rejected_contract_reference_v1() -> ContractReferenceV1
{
    contract_reference(BIND_MAIL_ADDRESS_BOOK_PROVIDER_LINK_REJECTED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn upsert_contact_command_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        upsert_contact_command_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn upsert_contact_command_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        upsert_contact_command_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn contact_upserted_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        contact_upserted_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn contact_upsert_rejected_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        contact_upsert_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn bind_mail_address_book_provider_link_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        bind_mail_address_book_provider_link_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn bind_mail_address_book_provider_link_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        bind_mail_address_book_provider_link_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn mail_address_book_provider_link_bound_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        mail_address_book_provider_link_bound_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn mail_address_book_provider_link_bound_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        mail_address_book_provider_link_bound_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn bind_mail_address_book_provider_link_rejected_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        bind_mail_address_book_provider_link_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn bind_mail_address_book_provider_link_rejected_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        bind_mail_address_book_provider_link_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

fn contract_reference(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: CONTACTS_OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: CONTACTS_COMMAND_CONTRACT_MAJOR_V1,
        revision: CONTACTS_COMMAND_CONTRACT_REVISION_V1,
        schema_sha256: CONTACTS_COMMAND_SCHEMA_SHA256_V1.to_vec(),
    }
}

fn event_route(
    envelope_kind: DurableEnvelopeKindV1,
    contract: ContractReferenceV1,
    direction: EventRouteDirectionV1,
    subscription_requirement: EventSubscriptionRequirementV1,
) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: envelope_kind as i32,
            contract: Some(contract),
            direction: direction as i32,
            max_in_flight: CONTACTS_MAIL_IDENTITY_MAX_IN_FLIGHT_V1,
            subscription_requirement: subscription_requirement as i32,
            max_deliver: u32::from(direction == EventRouteDirectionV1::Consume) * 10,
            ack_wait_millis: u32::from(direction == EventRouteDirectionV1::Consume) * 30_000,
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_route_is_contacts_owned_and_required() {
        let Some(Request::EventRoute(route)) = upsert_contact_command_consume_request_v1().request
        else {
            panic!("event route");
        };
        assert_eq!(route.direction, EventRouteDirectionV1::Consume as i32);
        assert_eq!(
            route.subscription_requirement,
            EventSubscriptionRequirementV1::Required as i32
        );
        assert_eq!(
            route.contract.expect("contract").owner,
            CONTACTS_OWNER_ID_V1
        );
    }

    #[test]
    fn contract_has_no_provider_secret_or_generic_payload_fields() {
        let source = include_str!("../proto/makosh/contacts/command/v1/contacts_command.proto");
        for forbidden in [
            "token",
            "credential",
            "password",
            "cookie",
            "map<",
            "bytes payload",
        ] {
            assert!(!source.contains(forbidden), "forbidden field: {forbidden}");
        }
    }
}
