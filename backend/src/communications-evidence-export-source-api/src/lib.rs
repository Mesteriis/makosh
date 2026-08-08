#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    EvidenceExportEnvelopeBuildErrorV1, EvidenceExportEnvelopeContextV1,
    build_evidence_export_prepare_outbox_record_v1,
    build_evidence_export_prepared_outbox_record_v1,
    build_evidence_export_rejected_outbox_record_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-communications-evidence-export-source-api";
pub const EVIDENCE_EXPORT_SOURCE_OWNER_V1: &str = "communications";
pub const EVIDENCE_EXPORT_PREPARE_CONTRACT_NAME_V1: &str = "evidence_export_prepare";
pub const EVIDENCE_EXPORT_PREPARED_CONTRACT_NAME_V1: &str = "evidence_export_prepared";
pub const EVIDENCE_EXPORT_REJECTED_CONTRACT_NAME_V1: &str = "evidence_export_rejected";
pub const EVIDENCE_EXPORT_SOURCE_CONTRACT_MAJOR_V1: u32 = 1;
pub const EVIDENCE_EXPORT_SOURCE_CONTRACT_REVISION_V1: u32 = 1;
pub const EVIDENCE_EXPORT_MAX_MESSAGES_V1: usize = 64;
pub const EVIDENCE_EXPORT_MAX_SOURCE_BYTES_V1: u64 = 16 * 1024 * 1024;
pub const EVIDENCE_EXPORT_MAX_SOURCE_PROOF_BYTES_V1: usize = 2_048;
pub const EVIDENCE_EXPORT_MAX_IN_FLIGHT_V1: u32 = 32;
pub const COMMUNICATIONS_EXPORT_SOURCE_BLOB_TARGET_OWNER_ID_V1: &str = "communications_export";
pub const COMMUNICATIONS_EXPORT_SOURCE_BLOB_TARGET_MODULE_ID_V1: &str =
    "makosh-communications-export-runtime";
pub const COMMUNICATIONS_EXPORT_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1: &str =
    "communications_export.blob.v1";

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communications.evidence_export_source.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/communications_evidence_export_source_schema.rs"
));

pub const COMMUNICATIONS_EVIDENCE_EXPORT_SOURCE_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/communications-evidence-export-source-v1.bin"
));

#[must_use]
pub fn evidence_export_prepare_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(EVIDENCE_EXPORT_PREPARE_CONTRACT_NAME_V1)
}

#[must_use]
pub fn evidence_export_prepared_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(EVIDENCE_EXPORT_PREPARED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn evidence_export_rejected_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(EVIDENCE_EXPORT_REJECTED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn evidence_export_prepare_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        evidence_export_prepare_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn evidence_export_prepare_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        evidence_export_prepare_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn evidence_export_prepared_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        evidence_export_prepared_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn evidence_export_prepared_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        evidence_export_prepared_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn evidence_export_rejected_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        evidence_export_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn evidence_export_rejected_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        evidence_export_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

fn contract_reference(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: EVIDENCE_EXPORT_SOURCE_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: EVIDENCE_EXPORT_SOURCE_CONTRACT_MAJOR_V1,
        revision: EVIDENCE_EXPORT_SOURCE_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATIONS_EVIDENCE_EXPORT_SOURCE_SCHEMA_SHA256.to_vec(),
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
            max_in_flight: EVIDENCE_EXPORT_MAX_IN_FLIGHT_V1,
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
        let publish = evidence_export_prepare_publish_request_v1();
        let consume = evidence_export_prepare_consume_request_v1();
        let Some(Request::EventRoute(publish)) = publish.request else {
            panic!("publish route");
        };
        let Some(Request::EventRoute(consume)) = consume.request else {
            panic!("consume route");
        };
        assert_eq!(
            publish.contract,
            Some(evidence_export_prepare_contract_reference_v1())
        );
        assert_eq!(publish.direction, EventRouteDirectionV1::Publish as i32);
        assert_eq!(consume.direction, EventRouteDirectionV1::Consume as i32);
        assert_eq!(
            consume.subscription_requirement,
            EventSubscriptionRequirementV1::Required as i32
        );
        assert_eq!(
            [
                EVIDENCE_EXPORT_PREPARE_CONTRACT_NAME_V1,
                EVIDENCE_EXPORT_PREPARED_CONTRACT_NAME_V1,
                EVIDENCE_EXPORT_REJECTED_CONTRACT_NAME_V1,
            ],
            [
                "evidence_export_prepare",
                "evidence_export_prepared",
                "evidence_export_rejected",
            ]
        );
    }

    #[test]
    fn target_is_exact_export_workflow_not_a_generic_recipient() {
        assert_eq!(
            COMMUNICATIONS_EXPORT_SOURCE_BLOB_TARGET_OWNER_ID_V1,
            "communications_export"
        );
        assert_eq!(
            COMMUNICATIONS_EXPORT_SOURCE_BLOB_TARGET_MODULE_ID_V1,
            "makosh-communications-export-runtime"
        );
        assert_eq!(
            COMMUNICATIONS_EXPORT_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1,
            "communications_export.blob.v1"
        );
    }
}
