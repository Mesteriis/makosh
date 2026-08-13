#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    RelationshipsEnvelopeBuildErrorV1, RelationshipsEnvelopeContextV1,
    build_relationship_changed_outbox_record_v1,
};

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-relationships-api";
pub const RELATIONSHIPS_OWNER_ID_V1: &str = "relationships";
pub const RELATIONSHIPS_MODULE_ID_V1: &str = "makosh-relationships-runtime";
pub const RELATIONSHIPS_CLIENT_CAPABILITY_ID_V1: &str = "relationships.client.v1";
pub const RELATIONSHIPS_LIFECYCLE_EVENT_CAPABILITY_ID_V1: &str = "relationships.lifecycle.event.v1";
pub const RELATIONSHIPS_STORAGE_CAPABILITY_ID_V1: &str = "relationships.storage.v1";
pub const RELATIONSHIPS_CONTRACT_MAJOR_V1: u32 = 1;
pub const RELATIONSHIPS_CONTRACT_REVISION_V1: u32 = 1;

pub const RELATIONSHIPS_CREATE_CONNECT_PATH_V1: &str =
    "/makosh.relationships.client.v1.RelationshipsCommandService/Create";
pub const RELATIONSHIPS_UPDATE_VALIDITY_CONNECT_PATH_V1: &str =
    "/makosh.relationships.client.v1.RelationshipsCommandService/UpdateValidity";
pub const RELATIONSHIPS_END_CONNECT_PATH_V1: &str =
    "/makosh.relationships.client.v1.RelationshipsCommandService/End";
pub const RELATIONSHIPS_REACTIVATE_CONNECT_PATH_V1: &str =
    "/makosh.relationships.client.v1.RelationshipsCommandService/Reactivate";
pub const RELATIONSHIPS_ADD_EVIDENCE_CONNECT_PATH_V1: &str =
    "/makosh.relationships.client.v1.RelationshipsCommandService/AddEvidence";
pub const RELATIONSHIPS_REMOVE_EVIDENCE_CONNECT_PATH_V1: &str =
    "/makosh.relationships.client.v1.RelationshipsCommandService/RemoveEvidence";
pub const RELATIONSHIPS_GET_CONNECT_PATH_V1: &str =
    "/makosh.relationships.client.v1.RelationshipsQueryService/Get";
pub const RELATIONSHIPS_LIST_FOR_PARTICIPANT_CONNECT_PATH_V1: &str =
    "/makosh.relationships.client.v1.RelationshipsQueryService/ListForParticipant";
pub const RELATIONSHIPS_LIST_EVIDENCE_CONNECT_PATH_V1: &str =
    "/makosh.relationships.client.v1.RelationshipsQueryService/ListEvidence";

pub mod client_wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.relationships.client.v1.rs"
    ));
}

include!(concat!(env!("OUT_DIR"), "/relationships_client_schema.rs"));
pub const RELATIONSHIPS_CLIENT_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/relationships-client-v1.bin"));

fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: RELATIONSHIPS_OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: RELATIONSHIPS_CONTRACT_MAJOR_V1,
        revision: RELATIONSHIPS_CONTRACT_REVISION_V1,
        schema_sha256: RELATIONSHIPS_CLIENT_SCHEMA_SHA256_V1.to_vec(),
    }
}

macro_rules! reference {
    ($function:ident, $name:literal) => {
        #[must_use]
        pub fn $function() -> ContractReferenceV1 {
            contract($name)
        }
    };
}

reference!(
    relationships_client_create_contract_reference_v1,
    "relationships_client_create"
);
reference!(
    relationships_client_update_validity_contract_reference_v1,
    "relationships_client_update_validity"
);
reference!(
    relationships_client_end_contract_reference_v1,
    "relationships_client_end"
);
reference!(
    relationships_client_reactivate_contract_reference_v1,
    "relationships_client_reactivate"
);
reference!(
    relationships_client_add_evidence_contract_reference_v1,
    "relationships_client_add_evidence"
);
reference!(
    relationships_client_remove_evidence_contract_reference_v1,
    "relationships_client_remove_evidence"
);
reference!(
    relationships_client_get_contract_reference_v1,
    "relationships_client_get"
);
reference!(
    relationships_client_list_for_participant_contract_reference_v1,
    "relationships_client_list_for_participant"
);
reference!(
    relationships_client_list_evidence_contract_reference_v1,
    "relationships_client_list_evidence"
);
reference!(
    relationships_lifecycle_event_contract_reference_v1,
    "relationship_changed"
);

#[must_use]
pub fn relationships_lifecycle_event_publish_request_v1() -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: DurableEnvelopeKindV1::Event as i32,
            contract: Some(relationships_lifecycle_event_contract_reference_v1()),
            direction: EventRouteDirectionV1::Publish as i32,
            max_in_flight: 32,
            subscription_requirement: EventSubscriptionRequirementV1::Unspecified as i32,
            max_deliver: 0,
            ack_wait_millis: 0,
        })),
    }
}

#[must_use]
pub fn relationships_client_routes_v1() -> [(ContractReferenceV1, &'static str); 9] {
    [
        (
            relationships_client_create_contract_reference_v1(),
            RELATIONSHIPS_CREATE_CONNECT_PATH_V1,
        ),
        (
            relationships_client_update_validity_contract_reference_v1(),
            RELATIONSHIPS_UPDATE_VALIDITY_CONNECT_PATH_V1,
        ),
        (
            relationships_client_end_contract_reference_v1(),
            RELATIONSHIPS_END_CONNECT_PATH_V1,
        ),
        (
            relationships_client_reactivate_contract_reference_v1(),
            RELATIONSHIPS_REACTIVATE_CONNECT_PATH_V1,
        ),
        (
            relationships_client_add_evidence_contract_reference_v1(),
            RELATIONSHIPS_ADD_EVIDENCE_CONNECT_PATH_V1,
        ),
        (
            relationships_client_remove_evidence_contract_reference_v1(),
            RELATIONSHIPS_REMOVE_EVIDENCE_CONNECT_PATH_V1,
        ),
        (
            relationships_client_get_contract_reference_v1(),
            RELATIONSHIPS_GET_CONNECT_PATH_V1,
        ),
        (
            relationships_client_list_for_participant_contract_reference_v1(),
            RELATIONSHIPS_LIST_FOR_PARTICIPANT_CONNECT_PATH_V1,
        ),
        (
            relationships_client_list_evidence_contract_reference_v1(),
            RELATIONSHIPS_LIST_EVIDENCE_CONNECT_PATH_V1,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_is_closed_provider_neutral_and_exact() {
        assert_eq!(relationships_client_routes_v1().len(), 9);
        assert_eq!(
            [
                RELATIONSHIPS_CLIENT_CAPABILITY_ID_V1,
                RELATIONSHIPS_LIFECYCLE_EVENT_CAPABILITY_ID_V1,
                RELATIONSHIPS_STORAGE_CAPABILITY_ID_V1,
            ],
            [
                "relationships.client.v1",
                "relationships.lifecycle.event.v1",
                "relationships.storage.v1",
            ]
        );
        let descriptor = String::from_utf8_lossy(RELATIONSHIPS_CLIENT_DESCRIPTOR_SET_V1);
        for forbidden in [
            "credential",
            "raw_payload",
            "private_locator",
            "confidence",
            "trust_score",
        ] {
            assert!(!descriptor.contains(forbidden), "{forbidden}");
        }
    }
}
