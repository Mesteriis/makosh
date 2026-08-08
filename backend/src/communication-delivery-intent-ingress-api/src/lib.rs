#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    CommunicationDeliveryIntentIngressEnvelopeBuildErrorV1,
    CommunicationDeliveryIntentIngressEnvelopeContextV1,
    build_communication_delivery_intent_rejected_outbox_record_v1,
    build_communication_delivery_intent_submit_outbox_record_v1,
    build_communication_delivery_intent_submitted_outbox_record_v1,
    communication_delivery_intent_rejected_message_id_v1,
    communication_delivery_intent_submit_message_id_v1,
    communication_delivery_intent_submitted_message_id_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-communication-delivery-intent-ingress-api";
pub const COMMUNICATION_DELIVERY_INTENT_INGRESS_OWNER_V1: &str = "communication_delivery_intent";
pub const COMMUNICATION_DELIVERY_INTENT_SUBMIT_CONTRACT_NAME_V1: &str =
    "communication_delivery_intent_submit";
pub const COMMUNICATION_DELIVERY_INTENT_SUBMITTED_CONTRACT_NAME_V1: &str =
    "communication_delivery_intent_submitted";
pub const COMMUNICATION_DELIVERY_INTENT_REJECTED_CONTRACT_NAME_V1: &str =
    "communication_delivery_intent_rejected";
pub const COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_MAJOR_V1: u32 = 1;
pub const COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_REVISION_V1: u32 = 1;
pub const COMMUNICATION_DELIVERY_INTENT_INGRESS_MAX_BYTES_V1: u64 = 16 * 1024 * 1024;
pub const COMMUNICATION_DELIVERY_INTENT_INGRESS_MAX_PROOF_BYTES_V1: usize = 2_048;
pub const COMMUNICATION_DELIVERY_INTENT_INGRESS_MAX_IN_FLIGHT_V1: u32 = 32;
pub const COMMUNICATION_DELIVERY_INTENT_INGRESS_COMMAND_CAPABILITY_ID_V1: &str =
    "communication_delivery_intent.event-ingress.v1";
pub const COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_OWNER_ID_V1: &str =
    "communication_delivery_intent";
pub const COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_MODULE_ID_V1: &str =
    "makosh-communication-delivery-intent-runtime";
pub const COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_CAPABILITY_ID_V1: &str =
    "communication_delivery_intent.blob.v1";

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communication_delivery_intent.ingress.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/communication_delivery_intent_ingress_schema.rs"
));

pub const COMMUNICATION_DELIVERY_INTENT_INGRESS_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/communication-delivery-intent-ingress-v1.bin"
));

#[must_use]
pub fn communication_delivery_intent_submit_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(COMMUNICATION_DELIVERY_INTENT_SUBMIT_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communication_delivery_intent_submitted_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(COMMUNICATION_DELIVERY_INTENT_SUBMITTED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communication_delivery_intent_rejected_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(COMMUNICATION_DELIVERY_INTENT_REJECTED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communication_delivery_intent_submit_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        communication_delivery_intent_submit_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn communication_delivery_intent_submit_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        communication_delivery_intent_submit_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn communication_delivery_intent_submitted_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        communication_delivery_intent_submitted_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn communication_delivery_intent_submitted_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        communication_delivery_intent_submitted_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn communication_delivery_intent_rejected_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        communication_delivery_intent_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn communication_delivery_intent_rejected_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        communication_delivery_intent_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

fn contract_reference(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATION_DELIVERY_INTENT_INGRESS_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_MAJOR_V1,
        revision: COMMUNICATION_DELIVERY_INTENT_INGRESS_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATION_DELIVERY_INTENT_INGRESS_SCHEMA_SHA256.to_vec(),
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
            max_in_flight: COMMUNICATION_DELIVERY_INTENT_INGRESS_MAX_IN_FLIGHT_V1,
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
    fn ingress_routes_are_exact_and_directional() {
        let publish = communication_delivery_intent_submit_publish_request_v1();
        let consume = communication_delivery_intent_submit_consume_request_v1();
        let Some(Request::EventRoute(publish)) = publish.request else {
            panic!("publish route");
        };
        let Some(Request::EventRoute(consume)) = consume.request else {
            panic!("consume route");
        };
        assert_eq!(
            publish.contract,
            Some(communication_delivery_intent_submit_contract_reference_v1())
        );
        assert_eq!(publish.direction, EventRouteDirectionV1::Publish as i32);
        assert_eq!(consume.direction, EventRouteDirectionV1::Consume as i32);
        assert_eq!(
            consume.subscription_requirement,
            EventSubscriptionRequirementV1::Required as i32
        );
        assert_eq!(
            [
                COMMUNICATION_DELIVERY_INTENT_SUBMIT_CONTRACT_NAME_V1,
                COMMUNICATION_DELIVERY_INTENT_SUBMITTED_CONTRACT_NAME_V1,
                COMMUNICATION_DELIVERY_INTENT_REJECTED_CONTRACT_NAME_V1,
            ],
            [
                "communication_delivery_intent_submit",
                "communication_delivery_intent_submitted",
                "communication_delivery_intent_rejected",
            ]
        );
    }

    #[test]
    fn blob_target_is_exact_delivery_intent_runtime() {
        assert_eq!(
            COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_OWNER_ID_V1,
            "communication_delivery_intent"
        );
        assert_eq!(
            COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_MODULE_ID_V1,
            "makosh-communication-delivery-intent-runtime"
        );
        assert_eq!(
            COMMUNICATION_DELIVERY_INTENT_BLOB_TARGET_CAPABILITY_ID_V1,
            "communication_delivery_intent.blob.v1"
        );
    }
}
