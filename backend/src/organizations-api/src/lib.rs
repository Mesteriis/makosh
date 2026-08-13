#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    OrganizationsEnvelopeBuildErrorV1, OrganizationsEnvelopeContextV1,
    build_organization_changed_outbox_record_v1,
};

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-organizations-api";
pub const ORGANIZATIONS_OWNER_ID_V1: &str = "organizations";
pub const ORGANIZATIONS_MODULE_ID_V1: &str = "makosh-organizations-runtime";
pub const ORGANIZATIONS_CLIENT_CAPABILITY_ID_V1: &str = "organizations.client.v1";
pub const ORGANIZATIONS_LIFECYCLE_EVENT_CAPABILITY_ID_V1: &str = "organizations.lifecycle.event.v1";
pub const ORGANIZATIONS_STORAGE_CAPABILITY_ID_V1: &str = "organizations.storage.v1";
pub const ORGANIZATIONS_LIFECYCLE_EVENT_CONTRACT_NAME_V1: &str = "organization_changed";
pub const ORGANIZATIONS_CLIENT_CONTRACT_MAJOR_V1: u32 = 1;
pub const ORGANIZATIONS_CLIENT_CONTRACT_REVISION_V1: u32 = 1;

pub const ORGANIZATIONS_CREATE_CONNECT_PATH_V1: &str =
    "/makosh.organizations.client.v1.OrganizationsCommandService/Create";
pub const ORGANIZATIONS_UPDATE_CONNECT_PATH_V1: &str =
    "/makosh.organizations.client.v1.OrganizationsCommandService/Update";
pub const ORGANIZATIONS_SET_STATE_CONNECT_PATH_V1: &str =
    "/makosh.organizations.client.v1.OrganizationsCommandService/SetState";
pub const ORGANIZATIONS_ADD_SOURCE_CONNECT_PATH_V1: &str =
    "/makosh.organizations.client.v1.OrganizationsCommandService/AddSource";
pub const ORGANIZATIONS_REMOVE_SOURCE_CONNECT_PATH_V1: &str =
    "/makosh.organizations.client.v1.OrganizationsCommandService/RemoveSource";
pub const ORGANIZATIONS_GET_CONNECT_PATH_V1: &str =
    "/makosh.organizations.client.v1.OrganizationsQueryService/Get";
pub const ORGANIZATIONS_LIST_CONNECT_PATH_V1: &str =
    "/makosh.organizations.client.v1.OrganizationsQueryService/List";
pub const ORGANIZATIONS_SEARCH_CONNECT_PATH_V1: &str =
    "/makosh.organizations.client.v1.OrganizationsQueryService/Search";
pub const ORGANIZATIONS_LIST_SOURCES_CONNECT_PATH_V1: &str =
    "/makosh.organizations.client.v1.OrganizationsQueryService/ListSources";

pub mod client_wire {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.organizations.client.v1.rs"
    ));
}

include!(concat!(env!("OUT_DIR"), "/organizations_client_schema.rs"));

pub const ORGANIZATIONS_CLIENT_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/organizations-client-v1.bin"));

fn contract_reference(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: ORGANIZATIONS_OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: ORGANIZATIONS_CLIENT_CONTRACT_MAJOR_V1,
        revision: ORGANIZATIONS_CLIENT_CONTRACT_REVISION_V1,
        schema_sha256: ORGANIZATIONS_CLIENT_SCHEMA_SHA256_V1.to_vec(),
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
    organizations_client_create_contract_reference_v1,
    "organizations_client_create"
);
client_contract!(
    organizations_client_update_contract_reference_v1,
    "organizations_client_update"
);
client_contract!(
    organizations_client_set_state_contract_reference_v1,
    "organizations_client_set_state"
);
client_contract!(
    organizations_client_add_source_contract_reference_v1,
    "organizations_client_add_source"
);
client_contract!(
    organizations_client_remove_source_contract_reference_v1,
    "organizations_client_remove_source"
);
client_contract!(
    organizations_client_get_contract_reference_v1,
    "organizations_client_get"
);
client_contract!(
    organizations_client_list_contract_reference_v1,
    "organizations_client_list"
);
client_contract!(
    organizations_client_search_contract_reference_v1,
    "organizations_client_search"
);
client_contract!(
    organizations_client_list_sources_contract_reference_v1,
    "organizations_client_list_sources"
);

#[must_use]
pub fn organizations_lifecycle_event_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(ORGANIZATIONS_LIFECYCLE_EVENT_CONTRACT_NAME_V1)
}

#[must_use]
pub fn organizations_lifecycle_event_publish_request_v1() -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: DurableEnvelopeKindV1::Event as i32,
            contract: Some(organizations_lifecycle_event_contract_reference_v1()),
            direction: EventRouteDirectionV1::Publish as i32,
            max_in_flight: 32,
            subscription_requirement: EventSubscriptionRequirementV1::Unspecified as i32,
            max_deliver: 0,
            ack_wait_millis: 0,
        })),
    }
}

#[must_use]
pub fn organizations_client_routes_v1() -> [(ContractReferenceV1, &'static str); 9] {
    [
        (
            organizations_client_create_contract_reference_v1(),
            ORGANIZATIONS_CREATE_CONNECT_PATH_V1,
        ),
        (
            organizations_client_update_contract_reference_v1(),
            ORGANIZATIONS_UPDATE_CONNECT_PATH_V1,
        ),
        (
            organizations_client_set_state_contract_reference_v1(),
            ORGANIZATIONS_SET_STATE_CONNECT_PATH_V1,
        ),
        (
            organizations_client_add_source_contract_reference_v1(),
            ORGANIZATIONS_ADD_SOURCE_CONNECT_PATH_V1,
        ),
        (
            organizations_client_remove_source_contract_reference_v1(),
            ORGANIZATIONS_REMOVE_SOURCE_CONNECT_PATH_V1,
        ),
        (
            organizations_client_get_contract_reference_v1(),
            ORGANIZATIONS_GET_CONNECT_PATH_V1,
        ),
        (
            organizations_client_list_contract_reference_v1(),
            ORGANIZATIONS_LIST_CONNECT_PATH_V1,
        ),
        (
            organizations_client_search_contract_reference_v1(),
            ORGANIZATIONS_SEARCH_CONNECT_PATH_V1,
        ),
        (
            organizations_client_list_sources_contract_reference_v1(),
            ORGANIZATIONS_LIST_SOURCES_CONNECT_PATH_V1,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_client_and_capability_surface_is_provider_neutral() {
        assert_eq!(organizations_client_routes_v1().len(), 9);
        assert_eq!(
            [
                ORGANIZATIONS_CLIENT_CAPABILITY_ID_V1,
                ORGANIZATIONS_LIFECYCLE_EVENT_CAPABILITY_ID_V1,
                ORGANIZATIONS_STORAGE_CAPABILITY_ID_V1,
            ],
            [
                "organizations.client.v1",
                "organizations.lifecycle.event.v1",
                "organizations.storage.v1",
            ]
        );
        for forbidden in ["provider", "credential", "registration_number", "vat"] {
            assert!(
                !String::from_utf8_lossy(ORGANIZATIONS_CLIENT_DESCRIPTOR_SET_V1)
                    .contains(forbidden)
            );
        }
    }
}
