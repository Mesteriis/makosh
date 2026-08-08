#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    CommunicationNoteSourceEnvelopeBuildErrorV1, CommunicationNoteSourceEnvelopeContextV1,
    build_communication_note_source_prepare_outbox_record_v1,
    build_communication_note_source_prepared_outbox_record_v1,
    build_communication_note_source_rejected_outbox_record_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-communications-note-source-api";
pub const COMMUNICATIONS_NOTE_SOURCE_OWNER_V1: &str = "communications";
pub const COMMUNICATION_NOTE_SOURCE_PREPARE_CONTRACT_NAME_V1: &str =
    "communication_note_source_prepare";
pub const COMMUNICATION_NOTE_SOURCE_PREPARED_CONTRACT_NAME_V1: &str =
    "communication_note_source_prepared";
pub const COMMUNICATION_NOTE_SOURCE_REJECTED_CONTRACT_NAME_V1: &str =
    "communication_note_source_rejected";
pub const COMMUNICATIONS_NOTE_SOURCE_CONTRACT_MAJOR_V1: u32 = 1;
pub const COMMUNICATIONS_NOTE_SOURCE_CONTRACT_REVISION_V1: u32 = 1;
pub const COMMUNICATION_NOTE_SOURCE_MAX_BYTES_V1: u64 = 256 * 1024;
pub const COMMUNICATION_NOTE_SOURCE_MAX_PROOF_BYTES_V1: usize = 2_048;
pub const COMMUNICATION_NOTE_SOURCE_MAX_IN_FLIGHT_V1: u32 = 32;
pub const COMMUNICATIONS_NOTE_SOURCE_CAPABILITY_ID_V1: &str = "communications.note-source.v1";
pub const COMMUNICATION_NOTE_SOURCE_BLOB_TARGET_OWNER_ID_V1: &str =
    "communication_note_candidate_extraction";
pub const COMMUNICATION_NOTE_SOURCE_BLOB_TARGET_MODULE_ID_V1: &str =
    "makosh-communication-note-candidate-runtime";
pub const COMMUNICATION_NOTE_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1: &str =
    "communication_note_candidate_extraction.source.blob.v1";

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.communications.note_source.v1.rs"
    ));
}

include!(concat!(
    env!("OUT_DIR"),
    "/communications_note_source_schema.rs"
));

pub const COMMUNICATIONS_NOTE_SOURCE_DESCRIPTOR_SET_V1: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/communications-note-source-v1.bin"
));

#[must_use]
pub fn communication_note_source_prepare_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(COMMUNICATION_NOTE_SOURCE_PREPARE_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communication_note_source_prepared_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(COMMUNICATION_NOTE_SOURCE_PREPARED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communication_note_source_rejected_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(COMMUNICATION_NOTE_SOURCE_REJECTED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communication_note_source_prepare_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        communication_note_source_prepare_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn communication_note_source_prepare_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        communication_note_source_prepare_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn communication_note_source_prepared_publish_request_v1() -> CapabilityRequestV1 {
    result_route(
        communication_note_source_prepared_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn communication_note_source_prepared_consume_request_v1() -> CapabilityRequestV1 {
    result_route(
        communication_note_source_prepared_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn communication_note_source_rejected_publish_request_v1() -> CapabilityRequestV1 {
    result_route(
        communication_note_source_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn communication_note_source_rejected_consume_request_v1() -> CapabilityRequestV1 {
    result_route(
        communication_note_source_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

fn contract_reference(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATIONS_NOTE_SOURCE_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: COMMUNICATIONS_NOTE_SOURCE_CONTRACT_MAJOR_V1,
        revision: COMMUNICATIONS_NOTE_SOURCE_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATIONS_NOTE_SOURCE_SCHEMA_SHA256.to_vec(),
    }
}

fn result_route(
    contract: ContractReferenceV1,
    direction: EventRouteDirectionV1,
    subscription_requirement: EventSubscriptionRequirementV1,
) -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        contract,
        direction,
        subscription_requirement,
    )
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
            max_in_flight: COMMUNICATION_NOTE_SOURCE_MAX_IN_FLIGHT_V1,
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
    fn source_routes_are_exact_directional_and_workflow_specific() {
        let Some(Request::EventRoute(publish)) =
            communication_note_source_prepare_publish_request_v1().request
        else {
            panic!("publish route");
        };
        let Some(Request::EventRoute(consume)) =
            communication_note_source_prepare_consume_request_v1().request
        else {
            panic!("consume route");
        };
        assert_eq!(publish.direction, EventRouteDirectionV1::Publish as i32);
        assert_eq!(consume.direction, EventRouteDirectionV1::Consume as i32);
        assert_eq!(
            consume.subscription_requirement,
            EventSubscriptionRequirementV1::Required as i32
        );
        assert_eq!(
            COMMUNICATION_NOTE_SOURCE_BLOB_TARGET_OWNER_ID_V1,
            "communication_note_candidate_extraction"
        );
        assert_eq!(
            COMMUNICATION_NOTE_SOURCE_BLOB_TARGET_CAPABILITY_ID_V1,
            "communication_note_candidate_extraction.source.blob.v1"
        );
        let source =
            include_str!("../proto/makosh/communications/note_source/v1/note_source.proto");
        assert!(source.contains("CommunicationNoteSourceContentV1"));
        assert!(source.contains("subject_utf8"));
        assert!(source.contains("body_utf8"));
        for forbidden in ["provider_id", "account_id", "model_id", "prompt", "map<"] {
            assert!(
                !source.contains(forbidden),
                "forbidden source field {forbidden}"
            );
        }
    }
}
