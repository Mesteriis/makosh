#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    DecisionsEnvelopeBuildErrorV1, DecisionsEnvelopeContextV1,
    build_decision_changed_outbox_record_v1,
};

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-decisions-api";
pub const DECISIONS_OWNER_ID_V1: &str = "decisions";
pub const DECISIONS_MODULE_ID_V1: &str = "makosh-decisions-runtime";
pub const DECISIONS_CLIENT_CAPABILITY_ID_V1: &str = "decisions.client.v1";
pub const DECISIONS_LIFECYCLE_EVENT_CAPABILITY_ID_V1: &str = "decisions.lifecycle.event.v1";
pub const DECISIONS_STORAGE_CAPABILITY_ID_V1: &str = "decisions.storage.v1";
pub const DECISIONS_CLIENT_CONTRACT_MAJOR_V1: u32 = 1;
pub const DECISIONS_CLIENT_CONTRACT_REVISION_V1: u32 = 1;

pub mod client_wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.decisions.client.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/decisions_client_schema.rs"));
pub const DECISIONS_CLIENT_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/decisions-client-v1.bin"));

macro_rules! route {
    ($constant:ident, $service:literal, $method:literal) => {
        pub const $constant: &str = concat!("/makosh.decisions.client.v1.", $service, "/", $method);
    };
}

route!(
    DECISIONS_CREATE_CONNECT_PATH_V1,
    "DecisionsCommandService",
    "Create"
);
route!(
    DECISIONS_UPDATE_CONNECT_PATH_V1,
    "DecisionsCommandService",
    "Update"
);
route!(
    DECISIONS_ADD_ALTERNATIVE_CONNECT_PATH_V1,
    "DecisionsCommandService",
    "AddAlternative"
);
route!(
    DECISIONS_UPDATE_ALTERNATIVE_CONNECT_PATH_V1,
    "DecisionsCommandService",
    "UpdateAlternative"
);
route!(
    DECISIONS_REMOVE_ALTERNATIVE_CONNECT_PATH_V1,
    "DecisionsCommandService",
    "RemoveAlternative"
);
route!(
    DECISIONS_ADD_EVIDENCE_CONNECT_PATH_V1,
    "DecisionsCommandService",
    "AddEvidence"
);
route!(
    DECISIONS_REMOVE_EVIDENCE_CONNECT_PATH_V1,
    "DecisionsCommandService",
    "RemoveEvidence"
);
route!(
    DECISIONS_DECIDE_CONNECT_PATH_V1,
    "DecisionsCommandService",
    "Decide"
);
route!(
    DECISIONS_SUPERSEDE_CONNECT_PATH_V1,
    "DecisionsCommandService",
    "Supersede"
);
route!(
    DECISIONS_CANCEL_CONNECT_PATH_V1,
    "DecisionsCommandService",
    "Cancel"
);
route!(
    DECISIONS_GET_CONNECT_PATH_V1,
    "DecisionsQueryService",
    "Get"
);
route!(
    DECISIONS_LIST_CONNECT_PATH_V1,
    "DecisionsQueryService",
    "List"
);
route!(
    DECISIONS_LIST_ALTERNATIVES_CONNECT_PATH_V1,
    "DecisionsQueryService",
    "ListAlternatives"
);
route!(
    DECISIONS_LIST_EVIDENCE_CONNECT_PATH_V1,
    "DecisionsQueryService",
    "ListEvidence"
);

fn contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: DECISIONS_OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: DECISIONS_CLIENT_CONTRACT_MAJOR_V1,
        revision: DECISIONS_CLIENT_CONTRACT_REVISION_V1,
        schema_sha256: DECISIONS_CLIENT_SCHEMA_SHA256_V1.to_vec(),
    }
}

#[must_use]
pub fn decisions_client_routes_v1() -> [(ContractReferenceV1, &'static str); 14] {
    [
        (
            contract("decisions_client_create"),
            DECISIONS_CREATE_CONNECT_PATH_V1,
        ),
        (
            contract("decisions_client_update"),
            DECISIONS_UPDATE_CONNECT_PATH_V1,
        ),
        (
            contract("decisions_client_add_alternative"),
            DECISIONS_ADD_ALTERNATIVE_CONNECT_PATH_V1,
        ),
        (
            contract("decisions_client_update_alternative"),
            DECISIONS_UPDATE_ALTERNATIVE_CONNECT_PATH_V1,
        ),
        (
            contract("decisions_client_remove_alternative"),
            DECISIONS_REMOVE_ALTERNATIVE_CONNECT_PATH_V1,
        ),
        (
            contract("decisions_client_add_evidence"),
            DECISIONS_ADD_EVIDENCE_CONNECT_PATH_V1,
        ),
        (
            contract("decisions_client_remove_evidence"),
            DECISIONS_REMOVE_EVIDENCE_CONNECT_PATH_V1,
        ),
        (
            contract("decisions_client_decide"),
            DECISIONS_DECIDE_CONNECT_PATH_V1,
        ),
        (
            contract("decisions_client_supersede"),
            DECISIONS_SUPERSEDE_CONNECT_PATH_V1,
        ),
        (
            contract("decisions_client_cancel"),
            DECISIONS_CANCEL_CONNECT_PATH_V1,
        ),
        (
            contract("decisions_client_get"),
            DECISIONS_GET_CONNECT_PATH_V1,
        ),
        (
            contract("decisions_client_list"),
            DECISIONS_LIST_CONNECT_PATH_V1,
        ),
        (
            contract("decisions_client_list_alternatives"),
            DECISIONS_LIST_ALTERNATIVES_CONNECT_PATH_V1,
        ),
        (
            contract("decisions_client_list_evidence"),
            DECISIONS_LIST_EVIDENCE_CONNECT_PATH_V1,
        ),
    ]
}

#[must_use]
pub fn decisions_lifecycle_event_contract_reference_v1() -> ContractReferenceV1 {
    contract("decision_changed")
}

#[must_use]
pub fn decisions_lifecycle_event_publish_request_v1() -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: DurableEnvelopeKindV1::Event as i32,
            contract: Some(decisions_lifecycle_event_contract_reference_v1()),
            direction: EventRouteDirectionV1::Publish as i32,
            max_in_flight: 32,
            subscription_requirement: EventSubscriptionRequirementV1::Unspecified as i32,
            max_deliver: 0,
            ack_wait_millis: 0,
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_public_surface_is_typed_and_provider_private_free() {
        assert_eq!(decisions_client_routes_v1().len(), 14);
        assert_eq!(
            [
                DECISIONS_CLIENT_CAPABILITY_ID_V1,
                DECISIONS_LIFECYCLE_EVENT_CAPABILITY_ID_V1,
                DECISIONS_STORAGE_CAPABILITY_ID_V1,
            ],
            [
                "decisions.client.v1",
                "decisions.lifecycle.event.v1",
                "decisions.storage.v1",
            ]
        );
        let descriptor = String::from_utf8_lossy(DECISIONS_CLIENT_DESCRIPTOR_SET_V1);
        for forbidden in [
            "provider_payload",
            "credential",
            "private_locator",
            "arbitrary_json",
        ] {
            assert!(!descriptor.contains(forbidden), "{forbidden}");
        }
    }
}
