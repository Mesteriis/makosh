#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    ObligationsCommandEnvelopeBuildErrorV1, ObligationsCommandEnvelopeContextV1,
    build_create_obligation_from_reviewed_candidate_outbox_record_v1,
    build_obligation_changed_outbox_record_v1,
    build_obligation_created_from_reviewed_candidate_outbox_record_v1,
    build_obligation_creation_from_reviewed_candidate_rejected_outbox_record_v1,
};

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-obligations-api";
pub const OBLIGATIONS_OWNER_ID_V1: &str = "obligations";
pub const OBLIGATIONS_MODULE_ID_V1: &str = "makosh-obligations-runtime";
pub const OBLIGATIONS_REVIEWED_CANDIDATE_COMMAND_CAPABILITY_ID_V1: &str =
    "obligations.reviewed-candidate.command.v1";
pub const OBLIGATIONS_REVIEWED_CANDIDATE_BLOB_CAPABILITY_ID_V1: &str =
    "obligations.reviewed-candidate.blob.v1";
pub const CREATE_OBLIGATION_FROM_REVIEWED_CANDIDATE_CONTRACT_NAME_V1: &str =
    "obligations_create_from_reviewed_candidate";
pub const OBLIGATION_CREATED_FROM_REVIEWED_CANDIDATE_CONTRACT_NAME_V1: &str =
    "obligations_created_from_reviewed_candidate";
pub const OBLIGATION_CREATION_FROM_REVIEWED_CANDIDATE_REJECTED_CONTRACT_NAME_V1: &str =
    "obligations_creation_from_reviewed_candidate_rejected";
pub const OBLIGATIONS_COMMAND_CONTRACT_MAJOR_V1: u32 = 1;
pub const OBLIGATIONS_COMMAND_CONTRACT_REVISION_V1: u32 = 1;
pub const OBLIGATIONS_REVIEWED_CANDIDATE_MAX_BLOB_BYTES_V1: u64 = 16 * 1024;
pub const OBLIGATIONS_REVIEWED_CANDIDATE_MAX_PROOF_BYTES_V1: usize = 2_048;
pub const OBLIGATIONS_REVIEWED_CANDIDATE_MAX_IN_FLIGHT_V1: u32 = 32;
pub const OBLIGATIONS_CLIENT_CAPABILITY_ID_V1: &str = "obligations.client.v1";
pub const OBLIGATIONS_LIFECYCLE_EVENT_CAPABILITY_ID_V1: &str = "obligations.lifecycle.event.v1";
pub const OBLIGATIONS_LIFECYCLE_EVENT_CONTRACT_NAME_V1: &str = "obligations_lifecycle_changed";
pub const OBLIGATIONS_CLIENT_CONTRACT_MAJOR_V1: u32 = 1;
pub const OBLIGATIONS_CLIENT_CONTRACT_REVISION_V1: u32 = 1;

pub const OBLIGATIONS_UPDATE_CONNECT_PATH_V1: &str =
    "/makosh.obligations.client.v1.ObligationsCommandService/Update";
pub const OBLIGATIONS_SET_STATE_CONNECT_PATH_V1: &str =
    "/makosh.obligations.client.v1.ObligationsCommandService/SetState";
pub const OBLIGATIONS_ADD_EVIDENCE_CONNECT_PATH_V1: &str =
    "/makosh.obligations.client.v1.ObligationsCommandService/AddEvidence";
pub const OBLIGATIONS_REMOVE_EVIDENCE_CONNECT_PATH_V1: &str =
    "/makosh.obligations.client.v1.ObligationsCommandService/RemoveEvidence";
pub const OBLIGATIONS_GET_CONNECT_PATH_V1: &str =
    "/makosh.obligations.client.v1.ObligationsQueryService/Get";
pub const OBLIGATIONS_LIST_CONNECT_PATH_V1: &str =
    "/makosh.obligations.client.v1.ObligationsQueryService/List";
pub const OBLIGATIONS_LIST_EVIDENCE_CONNECT_PATH_V1: &str =
    "/makosh.obligations.client.v1.ObligationsQueryService/ListEvidence";

pub const OBLIGATIONS_CLIENT_UPDATE_CONTRACT_NAME_V1: &str = "obligations_client_update";
pub const OBLIGATIONS_CLIENT_SET_STATE_CONTRACT_NAME_V1: &str = "obligations_client_set_state";
pub const OBLIGATIONS_CLIENT_ADD_EVIDENCE_CONTRACT_NAME_V1: &str =
    "obligations_client_add_evidence";
pub const OBLIGATIONS_CLIENT_REMOVE_EVIDENCE_CONTRACT_NAME_V1: &str =
    "obligations_client_remove_evidence";
pub const OBLIGATIONS_CLIENT_GET_CONTRACT_NAME_V1: &str = "obligations_client_get";
pub const OBLIGATIONS_CLIENT_LIST_CONTRACT_NAME_V1: &str = "obligations_client_list";
pub const OBLIGATIONS_CLIENT_LIST_EVIDENCE_CONTRACT_NAME_V1: &str =
    "obligations_client_list_evidence";

pub mod wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.obligations.command.v1.rs"
    ));
}

pub mod client_wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.obligations.client.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/obligations_schema.rs"));
include!(concat!(env!("OUT_DIR"), "/obligations_client_schema.rs"));

pub const OBLIGATIONS_COMMAND_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/obligations-command-v1.bin"));
pub const OBLIGATIONS_CLIENT_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/obligations-client-v1.bin"));

fn client_contract_reference(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: OBLIGATIONS_OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: OBLIGATIONS_CLIENT_CONTRACT_MAJOR_V1,
        revision: OBLIGATIONS_CLIENT_CONTRACT_REVISION_V1,
        schema_sha256: OBLIGATIONS_CLIENT_SCHEMA_SHA256_V1.to_vec(),
    }
}

macro_rules! client_contract {
    ($function:ident, $name:ident) => {
        #[must_use]
        pub fn $function() -> ContractReferenceV1 {
            client_contract_reference($name)
        }
    };
}

client_contract!(
    obligations_client_update_contract_reference_v1,
    OBLIGATIONS_CLIENT_UPDATE_CONTRACT_NAME_V1
);
client_contract!(
    obligations_client_set_state_contract_reference_v1,
    OBLIGATIONS_CLIENT_SET_STATE_CONTRACT_NAME_V1
);
client_contract!(
    obligations_client_add_evidence_contract_reference_v1,
    OBLIGATIONS_CLIENT_ADD_EVIDENCE_CONTRACT_NAME_V1
);
client_contract!(
    obligations_client_remove_evidence_contract_reference_v1,
    OBLIGATIONS_CLIENT_REMOVE_EVIDENCE_CONTRACT_NAME_V1
);
client_contract!(
    obligations_client_get_contract_reference_v1,
    OBLIGATIONS_CLIENT_GET_CONTRACT_NAME_V1
);
client_contract!(
    obligations_client_list_contract_reference_v1,
    OBLIGATIONS_CLIENT_LIST_CONTRACT_NAME_V1
);
client_contract!(
    obligations_client_list_evidence_contract_reference_v1,
    OBLIGATIONS_CLIENT_LIST_EVIDENCE_CONTRACT_NAME_V1
);

#[must_use]
pub fn obligations_lifecycle_event_contract_reference_v1() -> ContractReferenceV1 {
    client_contract_reference(OBLIGATIONS_LIFECYCLE_EVENT_CONTRACT_NAME_V1)
}

#[must_use]
pub fn obligations_client_routes_v1() -> [(ContractReferenceV1, &'static str); 7] {
    [
        (
            obligations_client_update_contract_reference_v1(),
            OBLIGATIONS_UPDATE_CONNECT_PATH_V1,
        ),
        (
            obligations_client_set_state_contract_reference_v1(),
            OBLIGATIONS_SET_STATE_CONNECT_PATH_V1,
        ),
        (
            obligations_client_add_evidence_contract_reference_v1(),
            OBLIGATIONS_ADD_EVIDENCE_CONNECT_PATH_V1,
        ),
        (
            obligations_client_remove_evidence_contract_reference_v1(),
            OBLIGATIONS_REMOVE_EVIDENCE_CONNECT_PATH_V1,
        ),
        (
            obligations_client_get_contract_reference_v1(),
            OBLIGATIONS_GET_CONNECT_PATH_V1,
        ),
        (
            obligations_client_list_contract_reference_v1(),
            OBLIGATIONS_LIST_CONNECT_PATH_V1,
        ),
        (
            obligations_client_list_evidence_contract_reference_v1(),
            OBLIGATIONS_LIST_EVIDENCE_CONNECT_PATH_V1,
        ),
    ]
}

#[must_use]
pub fn obligations_lifecycle_event_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Event,
        obligations_lifecycle_event_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn create_obligation_from_reviewed_candidate_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(CREATE_OBLIGATION_FROM_REVIEWED_CANDIDATE_CONTRACT_NAME_V1)
}

#[must_use]
pub fn obligation_created_from_reviewed_candidate_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(OBLIGATION_CREATED_FROM_REVIEWED_CANDIDATE_CONTRACT_NAME_V1)
}

#[must_use]
pub fn obligation_creation_from_reviewed_candidate_rejected_contract_reference_v1()
-> ContractReferenceV1 {
    contract_reference(OBLIGATION_CREATION_FROM_REVIEWED_CANDIDATE_REJECTED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn create_obligation_from_reviewed_candidate_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        create_obligation_from_reviewed_candidate_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn create_obligation_from_reviewed_candidate_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        create_obligation_from_reviewed_candidate_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn obligation_created_from_reviewed_candidate_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        obligation_created_from_reviewed_candidate_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn obligation_created_from_reviewed_candidate_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        obligation_created_from_reviewed_candidate_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn obligation_creation_from_reviewed_candidate_rejected_publish_request_v1()
-> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        obligation_creation_from_reviewed_candidate_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn obligation_creation_from_reviewed_candidate_rejected_consume_request_v1()
-> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        obligation_creation_from_reviewed_candidate_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

fn contract_reference(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: OBLIGATIONS_OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: OBLIGATIONS_COMMAND_CONTRACT_MAJOR_V1,
        revision: OBLIGATIONS_COMMAND_CONTRACT_REVISION_V1,
        schema_sha256: OBLIGATIONS_COMMAND_SCHEMA_SHA256_V1.to_vec(),
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
            max_in_flight: OBLIGATIONS_REVIEWED_CANDIDATE_MAX_IN_FLIGHT_V1,
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
    fn exact_obligations_is_target_owned() {
        assert_eq!(OBLIGATIONS_OWNER_ID_V1, "obligations");
        assert_eq!(OBLIGATIONS_MODULE_ID_V1, "makosh-obligations-runtime");
        let Some(Request::EventRoute(route)) =
            create_obligation_from_reviewed_candidate_consume_request_v1().request
        else {
            panic!("command route");
        };
        assert_eq!(route.direction, EventRouteDirectionV1::Consume as i32);
        assert_eq!(
            route.subscription_requirement,
            EventSubscriptionRequirementV1::Required as i32
        );
    }

    #[test]
    fn durable_messages_exclude_candidate_presentation_text() {
        let source = include_str!("../proto/makosh/obligations/command/v1/obligations.proto");
        let command = source
            .split("message CreateObligationFromReviewedCandidateCommandV1")
            .nth(1)
            .and_then(|value| {
                value
                    .split("message ObligationCreatedFromReviewedCandidateV1")
                    .next()
            })
            .expect("command section");
        assert!(!command.contains("string statement"));
        assert!(!command.contains("due_text_hint"));
        assert!(!command.contains("condition"));
        assert!(!source.contains("provider_id"));
        assert!(!source.contains("project_id"));
        assert!(!source.contains("calendar"));
    }

    #[test]
    fn lifecycle_client_and_public_event_contracts_are_exact() {
        assert_eq!(OBLIGATIONS_CLIENT_CAPABILITY_ID_V1, "obligations.client.v1");
        assert_eq!(
            OBLIGATIONS_LIFECYCLE_EVENT_CAPABILITY_ID_V1,
            "obligations.lifecycle.event.v1"
        );
        assert_eq!(obligations_client_routes_v1().len(), 7);
        assert_eq!(
            obligations_client_routes_v1()
                .iter()
                .map(|(_, path)| *path)
                .collect::<Vec<_>>(),
            vec![
                "/makosh.obligations.client.v1.ObligationsCommandService/Update",
                "/makosh.obligations.client.v1.ObligationsCommandService/SetState",
                "/makosh.obligations.client.v1.ObligationsCommandService/AddEvidence",
                "/makosh.obligations.client.v1.ObligationsCommandService/RemoveEvidence",
                "/makosh.obligations.client.v1.ObligationsQueryService/Get",
                "/makosh.obligations.client.v1.ObligationsQueryService/List",
                "/makosh.obligations.client.v1.ObligationsQueryService/ListEvidence",
            ]
        );
        assert_ne!(OBLIGATIONS_CLIENT_SCHEMA_SHA256_V1, [0; 32]);
        assert_eq!(
            obligations_lifecycle_event_contract_reference_v1().owner,
            OBLIGATIONS_OWNER_ID_V1
        );
    }
}
