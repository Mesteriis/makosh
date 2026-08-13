#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    ProjectsEnvelopeBuildErrorV1, ProjectsEnvelopeContextV1, build_project_changed_outbox_record_v1,
};

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-projects-api";
pub const PROJECTS_OWNER_ID_V1: &str = "projects";
pub const PROJECTS_MODULE_ID_V1: &str = "makosh-projects-runtime";
pub const PROJECTS_CLIENT_CAPABILITY_ID_V1: &str = "projects.client.v1";
pub const PROJECTS_LIFECYCLE_EVENT_CAPABILITY_ID_V1: &str = "projects.lifecycle.event.v1";
pub const PROJECTS_STORAGE_CAPABILITY_ID_V1: &str = "projects.storage.v1";
pub const PROJECTS_LIFECYCLE_EVENT_CONTRACT_NAME_V1: &str = "project_changed";
pub const PROJECTS_CLIENT_CONTRACT_MAJOR_V1: u32 = 1;
pub const PROJECTS_CLIENT_CONTRACT_REVISION_V1: u32 = 1;

pub const PROJECTS_CREATE_CONNECT_PATH_V1: &str =
    "/makosh.projects.client.v1.ProjectsCommandService/Create";
pub const PROJECTS_UPDATE_CONNECT_PATH_V1: &str =
    "/makosh.projects.client.v1.ProjectsCommandService/Update";
pub const PROJECTS_SET_STATE_CONNECT_PATH_V1: &str =
    "/makosh.projects.client.v1.ProjectsCommandService/SetState";
pub const PROJECTS_ADD_OUTCOME_CONNECT_PATH_V1: &str =
    "/makosh.projects.client.v1.ProjectsCommandService/AddOutcome";
pub const PROJECTS_UPDATE_OUTCOME_CONNECT_PATH_V1: &str =
    "/makosh.projects.client.v1.ProjectsCommandService/UpdateOutcome";
pub const PROJECTS_SET_OUTCOME_STATE_CONNECT_PATH_V1: &str =
    "/makosh.projects.client.v1.ProjectsCommandService/SetOutcomeState";
pub const PROJECTS_REMOVE_OUTCOME_CONNECT_PATH_V1: &str =
    "/makosh.projects.client.v1.ProjectsCommandService/RemoveOutcome";
pub const PROJECTS_ADD_REFERENCE_CONNECT_PATH_V1: &str =
    "/makosh.projects.client.v1.ProjectsCommandService/AddReference";
pub const PROJECTS_REMOVE_REFERENCE_CONNECT_PATH_V1: &str =
    "/makosh.projects.client.v1.ProjectsCommandService/RemoveReference";
pub const PROJECTS_GET_CONNECT_PATH_V1: &str =
    "/makosh.projects.client.v1.ProjectsQueryService/Get";
pub const PROJECTS_LIST_CONNECT_PATH_V1: &str =
    "/makosh.projects.client.v1.ProjectsQueryService/List";
pub const PROJECTS_LIST_OUTCOMES_CONNECT_PATH_V1: &str =
    "/makosh.projects.client.v1.ProjectsQueryService/ListOutcomes";
pub const PROJECTS_LIST_REFERENCES_CONNECT_PATH_V1: &str =
    "/makosh.projects.client.v1.ProjectsQueryService/ListReferences";

pub mod client_wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.projects.client.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/projects_client_schema.rs"));

pub const PROJECTS_CLIENT_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/projects-client-v1.bin"));

fn contract_reference(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: PROJECTS_OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: PROJECTS_CLIENT_CONTRACT_MAJOR_V1,
        revision: PROJECTS_CLIENT_CONTRACT_REVISION_V1,
        schema_sha256: PROJECTS_CLIENT_SCHEMA_SHA256_V1.to_vec(),
    }
}

macro_rules! client_contract {
    ($function:ident, $name:literal) => {
        #[must_use]
        pub fn $function() -> ContractReferenceV1 {
            contract_reference($name)
        }
    };
}

client_contract!(
    projects_client_create_contract_reference_v1,
    "projects_client_create"
);
client_contract!(
    projects_client_update_contract_reference_v1,
    "projects_client_update"
);
client_contract!(
    projects_client_set_state_contract_reference_v1,
    "projects_client_set_state"
);
client_contract!(
    projects_client_add_outcome_contract_reference_v1,
    "projects_client_add_outcome"
);
client_contract!(
    projects_client_update_outcome_contract_reference_v1,
    "projects_client_update_outcome"
);
client_contract!(
    projects_client_set_outcome_state_contract_reference_v1,
    "projects_client_set_outcome_state"
);
client_contract!(
    projects_client_remove_outcome_contract_reference_v1,
    "projects_client_remove_outcome"
);
client_contract!(
    projects_client_add_reference_contract_reference_v1,
    "projects_client_add_reference"
);
client_contract!(
    projects_client_remove_reference_contract_reference_v1,
    "projects_client_remove_reference"
);
client_contract!(
    projects_client_get_contract_reference_v1,
    "projects_client_get"
);
client_contract!(
    projects_client_list_contract_reference_v1,
    "projects_client_list"
);
client_contract!(
    projects_client_list_outcomes_contract_reference_v1,
    "projects_client_list_outcomes"
);
client_contract!(
    projects_client_list_references_contract_reference_v1,
    "projects_client_list_references"
);

#[must_use]
pub fn projects_lifecycle_event_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(PROJECTS_LIFECYCLE_EVENT_CONTRACT_NAME_V1)
}

#[must_use]
pub fn projects_lifecycle_event_publish_request_v1() -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: DurableEnvelopeKindV1::Event as i32,
            contract: Some(projects_lifecycle_event_contract_reference_v1()),
            direction: EventRouteDirectionV1::Publish as i32,
            max_in_flight: 32,
            subscription_requirement: EventSubscriptionRequirementV1::Unspecified as i32,
            max_deliver: 0,
            ack_wait_millis: 0,
        })),
    }
}

#[must_use]
pub fn projects_client_routes_v1() -> [(ContractReferenceV1, &'static str); 13] {
    [
        (
            projects_client_create_contract_reference_v1(),
            PROJECTS_CREATE_CONNECT_PATH_V1,
        ),
        (
            projects_client_update_contract_reference_v1(),
            PROJECTS_UPDATE_CONNECT_PATH_V1,
        ),
        (
            projects_client_set_state_contract_reference_v1(),
            PROJECTS_SET_STATE_CONNECT_PATH_V1,
        ),
        (
            projects_client_add_outcome_contract_reference_v1(),
            PROJECTS_ADD_OUTCOME_CONNECT_PATH_V1,
        ),
        (
            projects_client_update_outcome_contract_reference_v1(),
            PROJECTS_UPDATE_OUTCOME_CONNECT_PATH_V1,
        ),
        (
            projects_client_set_outcome_state_contract_reference_v1(),
            PROJECTS_SET_OUTCOME_STATE_CONNECT_PATH_V1,
        ),
        (
            projects_client_remove_outcome_contract_reference_v1(),
            PROJECTS_REMOVE_OUTCOME_CONNECT_PATH_V1,
        ),
        (
            projects_client_add_reference_contract_reference_v1(),
            PROJECTS_ADD_REFERENCE_CONNECT_PATH_V1,
        ),
        (
            projects_client_remove_reference_contract_reference_v1(),
            PROJECTS_REMOVE_REFERENCE_CONNECT_PATH_V1,
        ),
        (
            projects_client_get_contract_reference_v1(),
            PROJECTS_GET_CONNECT_PATH_V1,
        ),
        (
            projects_client_list_contract_reference_v1(),
            PROJECTS_LIST_CONNECT_PATH_V1,
        ),
        (
            projects_client_list_outcomes_contract_reference_v1(),
            PROJECTS_LIST_OUTCOMES_CONNECT_PATH_V1,
        ),
        (
            projects_client_list_references_contract_reference_v1(),
            PROJECTS_LIST_REFERENCES_CONNECT_PATH_V1,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_client_and_capability_surface_is_provider_neutral() {
        assert_eq!(projects_client_routes_v1().len(), 13);
        assert_eq!(
            [
                PROJECTS_CLIENT_CAPABILITY_ID_V1,
                PROJECTS_LIFECYCLE_EVENT_CAPABILITY_ID_V1,
                PROJECTS_STORAGE_CAPABILITY_ID_V1,
            ],
            [
                "projects.client.v1",
                "projects.lifecycle.event.v1",
                "projects.storage.v1",
            ]
        );
        for forbidden in ["provider", "credential", "raw_payload", "private_locator"] {
            assert!(
                !String::from_utf8_lossy(PROJECTS_CLIENT_DESCRIPTOR_SET_V1).contains(forbidden)
            );
        }
    }
}
