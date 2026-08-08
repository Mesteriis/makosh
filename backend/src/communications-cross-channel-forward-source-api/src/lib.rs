#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    CrossChannelForwardSourceEnvelopeBuildErrorV1, CrossChannelForwardSourceEnvelopeContextV1,
    build_cross_channel_forward_source_prepare_outbox_record_v1,
    build_cross_channel_forward_source_prepared_outbox_record_v1,
    build_cross_channel_forward_source_rejected_outbox_record_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-communications-cross-channel-forward-source-api";
pub const CROSS_CHANNEL_FORWARD_SOURCE_OWNER_V1: &str = "communications";
pub const CROSS_CHANNEL_FORWARD_SOURCE_PREPARE_CONTRACT_NAME_V1: &str =
    "cross_channel_forward_source_prepare";
pub const CROSS_CHANNEL_FORWARD_SOURCE_PREPARED_CONTRACT_NAME_V1: &str =
    "cross_channel_forward_source_prepared";
pub const CROSS_CHANNEL_FORWARD_SOURCE_REJECTED_CONTRACT_NAME_V1: &str =
    "cross_channel_forward_source_rejected";
pub const CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_MAJOR_V1: u32 = 1;
pub const CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_REVISION_V1: u32 = 1;
pub const CROSS_CHANNEL_FORWARD_SOURCE_MAX_BYTES_V1: u64 = 16 * 1024 * 1024;
pub const CROSS_CHANNEL_FORWARD_SOURCE_MAX_PROOF_BYTES_V1: usize = 2_048;
pub const CROSS_CHANNEL_FORWARD_SOURCE_MAX_IN_FLIGHT_V1: u32 = 32;
pub const CROSS_CHANNEL_FORWARD_SOURCE_COMMAND_CAPABILITY_ID_V1: &str =
    "communications.cross-channel-forward-source.v1";
pub const CROSS_CHANNEL_FORWARD_SOURCE_BLOB_TARGET_OWNER_ID_V1: &str =
    "communication_cross_channel_forward";
pub const CROSS_CHANNEL_FORWARD_SOURCE_BLOB_TARGET_MODULE_ID_V1: &str =
    "makosh-communication-cross-channel-forward-runtime";
pub const CROSS_CHANNEL_FORWARD_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1: &str =
    "communication_cross_channel_forward.blob.v1";

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communications.cross_channel_forward_source.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/communications_cross_channel_forward_source_schema.rs"
));

pub const COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(
        env!("OUT_DIR"),
        "/communications-cross-channel-forward-source-v1.bin"
    ));

#[must_use]
pub fn cross_channel_forward_source_prepare_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(CROSS_CHANNEL_FORWARD_SOURCE_PREPARE_CONTRACT_NAME_V1)
}

#[must_use]
pub fn cross_channel_forward_source_prepared_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(CROSS_CHANNEL_FORWARD_SOURCE_PREPARED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn cross_channel_forward_source_rejected_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(CROSS_CHANNEL_FORWARD_SOURCE_REJECTED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn cross_channel_forward_source_prepare_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        cross_channel_forward_source_prepare_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn cross_channel_forward_source_prepare_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        cross_channel_forward_source_prepare_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn cross_channel_forward_source_prepared_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        cross_channel_forward_source_prepared_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn cross_channel_forward_source_prepared_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        cross_channel_forward_source_prepared_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn cross_channel_forward_source_rejected_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        cross_channel_forward_source_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn cross_channel_forward_source_rejected_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        cross_channel_forward_source_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

fn contract_reference(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: CROSS_CHANNEL_FORWARD_SOURCE_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_MAJOR_V1,
        revision: CROSS_CHANNEL_FORWARD_SOURCE_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_SCHEMA_SHA256.to_vec(),
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
            max_in_flight: CROSS_CHANNEL_FORWARD_SOURCE_MAX_IN_FLIGHT_V1,
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
    fn source_routes_are_exact_and_directional() {
        let publish = cross_channel_forward_source_prepare_publish_request_v1();
        let consume = cross_channel_forward_source_prepare_consume_request_v1();
        let Some(Request::EventRoute(publish)) = publish.request else {
            panic!("publish route");
        };
        let Some(Request::EventRoute(consume)) = consume.request else {
            panic!("consume route");
        };
        assert_eq!(
            publish.contract,
            Some(cross_channel_forward_source_prepare_contract_reference_v1())
        );
        assert_eq!(publish.direction, EventRouteDirectionV1::Publish as i32);
        assert_eq!(consume.direction, EventRouteDirectionV1::Consume as i32);
        assert_eq!(
            consume.subscription_requirement,
            EventSubscriptionRequirementV1::Required as i32
        );
        assert_eq!(
            [
                CROSS_CHANNEL_FORWARD_SOURCE_PREPARE_CONTRACT_NAME_V1,
                CROSS_CHANNEL_FORWARD_SOURCE_PREPARED_CONTRACT_NAME_V1,
                CROSS_CHANNEL_FORWARD_SOURCE_REJECTED_CONTRACT_NAME_V1,
            ],
            [
                "cross_channel_forward_source_prepare",
                "cross_channel_forward_source_prepared",
                "cross_channel_forward_source_rejected",
            ]
        );
    }

    #[test]
    fn target_is_exact_forward_workflow_not_a_generic_recipient() {
        assert_eq!(
            CROSS_CHANNEL_FORWARD_SOURCE_BLOB_TARGET_OWNER_ID_V1,
            "communication_cross_channel_forward"
        );
        assert_eq!(
            CROSS_CHANNEL_FORWARD_SOURCE_BLOB_TARGET_MODULE_ID_V1,
            "makosh-communication-cross-channel-forward-runtime"
        );
        assert_eq!(
            CROSS_CHANNEL_FORWARD_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1,
            "communication_cross_channel_forward.blob.v1"
        );
    }
}
