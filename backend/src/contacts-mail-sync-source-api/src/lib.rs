#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    ContactsMailSyncSourceEnvelopeBuildErrorV1, ContactsMailSyncSourceEnvelopeContextV1,
    build_contact_changed_for_mail_sync_outbox_record_caused_by_v1,
    build_contact_changed_for_mail_sync_outbox_record_v1,
    build_contact_mail_sync_source_prepare_outbox_record_v1,
    build_contact_mail_sync_source_prepared_outbox_record_v1,
    build_contact_mail_sync_source_rejected_outbox_record_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-contacts-mail-sync-source-api";
pub const CONTACTS_MAIL_SYNC_SOURCE_OWNER_V1: &str = "contacts";
pub const CONTACT_CHANGED_FOR_MAIL_SYNC_CONTRACT_NAME_V1: &str = "contact_changed_for_mail_sync";
pub const CONTACT_MAIL_SYNC_SOURCE_PREPARE_CONTRACT_NAME_V1: &str =
    "contact_mail_sync_source_prepare";
pub const CONTACT_MAIL_SYNC_SOURCE_PREPARED_CONTRACT_NAME_V1: &str =
    "contact_mail_sync_source_prepared";
pub const CONTACT_MAIL_SYNC_SOURCE_REJECTED_CONTRACT_NAME_V1: &str =
    "contact_mail_sync_source_rejected";
pub const CONTACTS_MAIL_SYNC_SOURCE_CONTRACT_MAJOR_V1: u32 = 1;
pub const CONTACTS_MAIL_SYNC_SOURCE_CONTRACT_REVISION_V1: u32 = 1;
pub const CONTACT_MAIL_SYNC_SOURCE_MAX_BYTES_V1: u64 = 32 * 1024;
pub const CONTACT_MAIL_SYNC_SOURCE_MAX_PROOF_BYTES_V1: usize = 2_048;
pub const CONTACT_MAIL_SYNC_SOURCE_MAX_IN_FLIGHT_V1: u32 = 32;
pub const CONTACTS_MAIL_SYNC_SOURCE_CAPABILITY_ID_V1: &str = "contacts.mail-sync-source.v1";
pub const CONTACT_MAIL_SYNC_SOURCE_REQUESTER_MODULE_ID_V1: &str =
    "makosh-mail-contacts-sync-runtime";
pub const CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_OWNER_ID_V1: &str = "mail";
pub const CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_MODULE_ID_V1: &str = "makosh-mail-runtime";
pub const CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1: &str =
    "mail.address-book.contact-source.blob.v1";

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.contacts.mail_sync_source.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/contacts_mail_sync_source_schema.rs"
));

pub const CONTACTS_MAIL_SYNC_SOURCE_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/contacts-mail-sync-source-v1.bin"
));

#[must_use]
pub fn contact_changed_for_mail_sync_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(CONTACT_CHANGED_FOR_MAIL_SYNC_CONTRACT_NAME_V1)
}

#[must_use]
pub fn contact_mail_sync_source_prepare_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(CONTACT_MAIL_SYNC_SOURCE_PREPARE_CONTRACT_NAME_V1)
}

#[must_use]
pub fn contact_mail_sync_source_prepared_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(CONTACT_MAIL_SYNC_SOURCE_PREPARED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn contact_mail_sync_source_rejected_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(CONTACT_MAIL_SYNC_SOURCE_REJECTED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn contact_changed_for_mail_sync_publish_request_v1() -> CapabilityRequestV1 {
    route(
        DurableEnvelopeKindV1::Event,
        contact_changed_for_mail_sync_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn contact_changed_for_mail_sync_consume_request_v1() -> CapabilityRequestV1 {
    route(
        DurableEnvelopeKindV1::Event,
        contact_changed_for_mail_sync_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn contact_mail_sync_source_prepare_publish_request_v1() -> CapabilityRequestV1 {
    route(
        DurableEnvelopeKindV1::Command,
        contact_mail_sync_source_prepare_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn contact_mail_sync_source_prepare_consume_request_v1() -> CapabilityRequestV1 {
    route(
        DurableEnvelopeKindV1::Command,
        contact_mail_sync_source_prepare_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn contact_mail_sync_source_prepared_publish_request_v1() -> CapabilityRequestV1 {
    route(
        DurableEnvelopeKindV1::Result,
        contact_mail_sync_source_prepared_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn contact_mail_sync_source_prepared_consume_request_v1() -> CapabilityRequestV1 {
    route(
        DurableEnvelopeKindV1::Result,
        contact_mail_sync_source_prepared_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn contact_mail_sync_source_rejected_publish_request_v1() -> CapabilityRequestV1 {
    route(
        DurableEnvelopeKindV1::Result,
        contact_mail_sync_source_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn contact_mail_sync_source_rejected_consume_request_v1() -> CapabilityRequestV1 {
    route(
        DurableEnvelopeKindV1::Result,
        contact_mail_sync_source_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

fn contract_reference(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: CONTACTS_MAIL_SYNC_SOURCE_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: CONTACTS_MAIL_SYNC_SOURCE_CONTRACT_MAJOR_V1,
        revision: CONTACTS_MAIL_SYNC_SOURCE_CONTRACT_REVISION_V1,
        schema_sha256: CONTACTS_MAIL_SYNC_SOURCE_SCHEMA_SHA256_V1.to_vec(),
    }
}

fn route(
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
            max_in_flight: CONTACT_MAIL_SYNC_SOURCE_MAX_IN_FLIGHT_V1,
            subscription_requirement: subscription_requirement as i32,
            max_deliver: if direction == EventRouteDirectionV1::Consume {
                10
            } else {
                0
            },
            ack_wait_millis: if direction == EventRouteDirectionV1::Consume {
                30_000
            } else {
                0
            },
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_port_is_exact_target_bound_and_private_fields_stay_out_of_changed_event() {
        assert_eq!(CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_OWNER_ID_V1, "mail");
        assert_eq!(
            CONTACT_MAIL_SYNC_SOURCE_BLOB_TARGET_MODULE_ID_V1,
            "makosh-mail-runtime"
        );
        let source =
            include_str!("../proto/makosh/contacts/mail_sync_source/v1/mail_sync_source.proto");
        let changed = source
            .split("message ContactChangedForMailSyncV1")
            .nth(1)
            .expect("changed")
            .split('}')
            .next()
            .expect("body");
        for forbidden in [
            "display_name",
            "email_addresses",
            "phone_numbers",
            "provider_entry_id",
            "provider_etag",
            "provider_kind",
        ] {
            assert!(
                !changed.contains(forbidden),
                "private changed-event field {forbidden}"
            );
        }
        assert!(source.contains("ContactMailSyncSourceContentV1"));
        assert!(!source.contains("map<"));
    }
}
