use makosh_persons_api::{
    PERSONS_COMMAND_CAPABILITY_ID_V1, PERSONS_COMMAND_REJECTED_CAPABILITY_ID_V1,
    PERSONS_COMMAND_SUCCEEDED_CAPABILITY_ID_V1, PERSONS_MODULE_ID_V1,
    PERSONS_OWNER_EVENT_CAPABILITY_ID_V1, PERSONS_OWNER_ID_V1,
    PERSONS_REVIEW_CANDIDATE_CAPABILITY_ID_V1, persons_command_consume_request_v1,
    persons_command_contract_reference_v1, persons_command_rejected_contract_reference_v1,
    persons_command_rejected_publish_request_v1, persons_command_succeeded_contract_reference_v1,
    persons_command_succeeded_publish_request_v1, persons_owner_event_contract_reference_v1,
    persons_owner_event_publish_request_v1, persons_review_candidate_contract_reference_v1,
    persons_review_candidate_publish_request_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1, ClientRpcRouteV1,
    ContractReferenceV1, ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1,
    ProvidedSurfaceV1, RuntimeBudgetRequestV1, SettingsSchemaRefV1, SettingsSchemaV1,
    StorageNamespaceRequestV1, capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

use makosh_persons_api::{
    PERSONS_CLIENT_CAPABILITY_ID_V1, PERSONS_CREATE_CONNECT_PATH_V1,
    PERSONS_GET_PROFILE_CONNECT_PATH_V1, PERSONS_LIST_DIRECTORY_CONNECT_PATH_V1,
    PERSONS_LIST_SOURCE_LINKS_CONNECT_PATH_V1, PERSONS_UPDATE_OWNER_PROFILE_CONNECT_PATH_V1,
    persons_client_create_contract_reference_v1, persons_client_get_profile_contract_reference_v1,
    persons_client_list_directory_contract_reference_v1,
    persons_client_list_source_links_contract_reference_v1,
    persons_client_update_profile_contract_reference_v1,
};

pub const PERSONS_STORAGE_CAPABILITY_ID_V1: &str = "persons.storage.v1";
const STORAGE_CONNECTION_BUDGET_V1: u32 = 4;

#[must_use]
pub fn persons_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn persons_settings_schema_bytes_v1() -> Vec<u8> {
    persons_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn persons_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = persons_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 2,
        module_id: PERSONS_MODULE_ID_V1.to_owned(),
        owner_id: PERSONS_OWNER_ID_V1.to_owned(),
        module_kind: ModuleKindV1::Domain as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: vec![
            client_capability(),
            event_capability(
                PERSONS_COMMAND_REJECTED_CAPABILITY_ID_V1,
                ProvidedSurfaceKindV1::DurablePublisher,
                persons_command_rejected_contract_reference_v1(),
                persons_command_rejected_publish_request_v1(),
            ),
            event_capability(
                PERSONS_COMMAND_SUCCEEDED_CAPABILITY_ID_V1,
                ProvidedSurfaceKindV1::DurablePublisher,
                persons_command_succeeded_contract_reference_v1(),
                persons_command_succeeded_publish_request_v1(),
            ),
            event_capability(
                PERSONS_COMMAND_CAPABILITY_ID_V1,
                ProvidedSurfaceKindV1::DurableConsumer,
                persons_command_contract_reference_v1(),
                persons_command_consume_request_v1(),
            ),
            event_capability(
                PERSONS_OWNER_EVENT_CAPABILITY_ID_V1,
                ProvidedSurfaceKindV1::DurablePublisher,
                persons_owner_event_contract_reference_v1(),
                persons_owner_event_publish_request_v1(),
            ),
            event_capability(
                PERSONS_REVIEW_CANDIDATE_CAPABILITY_ID_V1,
                ProvidedSurfaceKindV1::DurablePublisher,
                persons_review_candidate_contract_reference_v1(),
                persons_review_candidate_publish_request_v1(),
            ),
            storage_capability(),
        ],
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: settings.len() as u64,
            sha256: Sha256::digest(&settings).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: STORAGE_CONNECTION_BUDGET_V1,
            max_memory_bytes: 64 * 1024 * 1024,
            max_cpu_millis: 500,
        }),
        display_name: "Persons".to_owned(),
    }
}

fn client_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: PERSONS_CLIENT_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: [
            (
                persons_client_create_contract_reference_v1(),
                PERSONS_CREATE_CONNECT_PATH_V1,
            ),
            (
                persons_client_update_profile_contract_reference_v1(),
                PERSONS_UPDATE_OWNER_PROFILE_CONNECT_PATH_V1,
            ),
            (
                persons_client_list_directory_contract_reference_v1(),
                PERSONS_LIST_DIRECTORY_CONNECT_PATH_V1,
            ),
            (
                persons_client_get_profile_contract_reference_v1(),
                PERSONS_GET_PROFILE_CONNECT_PATH_V1,
            ),
            (
                persons_client_list_source_links_contract_reference_v1(),
                PERSONS_LIST_SOURCE_LINKS_CONNECT_PATH_V1,
            ),
        ]
        .into_iter()
        .map(|(contract, path)| ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::ClientRpc as i32,
            contract: Some(contract),
            client_rpc_route: Some(ClientRpcRouteV1 {
                path: path.to_owned(),
            }),
            client_blob_route: None,
        })
        .collect(),
        ..Default::default()
    }
}

fn event_capability(
    capability_id: &str,
    kind: ProvidedSurfaceKindV1,
    contract: ContractReferenceV1,
    request: CapabilityRequestV1,
) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: capability_id.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: kind as i32,
            contract: Some(contract),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![request],
        ..Default::default()
    }
}

fn storage_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: PERSONS_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: PERSONS_OWNER_ID_V1.to_owned(),
                connection_budget: STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: 5_000,
            })),
        }],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::validation::descriptor::{
        validate_descriptor_v1, validate_settings_schema_v1,
    };

    use super::*;

    #[test]
    fn descriptor_has_five_event_routes_owner_storage_and_exact_client_rpc() {
        let descriptor = persons_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&persons_settings_schema_v1()).expect("settings");
        assert_eq!(descriptor.capabilities.len(), 7);
        assert_eq!(descriptor.owner_id, PERSONS_OWNER_ID_V1);
        assert!(
            descriptor
                .capabilities
                .iter()
                .all(|capability| capability.capability_id.starts_with("persons."))
        );
        let client = descriptor
            .capabilities
            .iter()
            .find(|capability| capability.capability_id == PERSONS_CLIENT_CAPABILITY_ID_V1)
            .expect("client");
        assert_eq!(client.provides.len(), 5);
    }
}
