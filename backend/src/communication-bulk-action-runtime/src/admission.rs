use makosh_communication_bulk_action_api::{
    COMMUNICATION_BULK_ACTION_CAPABILITY_ID_V1, COMMUNICATION_BULK_ACTION_COMMAND_CONNECT_PATH_V1,
    COMMUNICATION_BULK_ACTION_MODULE_ID_V1, COMMUNICATION_BULK_ACTION_OWNER_V1,
    COMMUNICATION_BULK_ACTION_QUERY_CONNECT_PATH_V1,
};
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1, ClientRpcRouteV1,
    ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
    RuntimeBudgetRequestV1, SettingsSchemaRefV1, SettingsSchemaV1, StorageNamespaceRequestV1,
    capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::contracts::{
    bulk_command_contract_v1, bulk_query_contract_v1, bulk_realtime_contract_v1,
    delivery_intent_command_contract_v1,
};

pub const COMMUNICATION_BULK_ACTION_STORAGE_CAPABILITY_ID_V1: &str =
    "communication_bulk_action.storage.v1";
pub const COMMUNICATION_BULK_ACTION_DELIVERY_DEPENDENCY_CAPABILITY_ID_V1: &str =
    "communication_bulk_action.delivery_intent.v1";
pub const COMMUNICATION_BULK_ACTION_STORAGE_CONNECTION_BUDGET_V1: u32 = 4;

#[must_use]
pub fn communication_bulk_action_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn communication_bulk_action_settings_schema_bytes_v1() -> Vec<u8> {
    communication_bulk_action_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn communication_bulk_action_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = communication_bulk_action_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: COMMUNICATION_BULK_ACTION_MODULE_ID_V1.to_owned(),
        owner_id: COMMUNICATION_BULK_ACTION_OWNER_V1.to_owned(),
        module_kind: ModuleKindV1::Workflow as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: vec![
            client_capability(),
            delivery_dependency_capability(),
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
            max_connections: COMMUNICATION_BULK_ACTION_STORAGE_CONNECTION_BUDGET_V1,
            max_memory_bytes: 64 * 1024 * 1024,
            max_cpu_millis: 500,
        }),
        display_name: "Communication Bulk Delivery".to_owned(),
    }
}

fn client_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_BULK_ACTION_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            client_surface(
                bulk_command_contract_v1(),
                COMMUNICATION_BULK_ACTION_COMMAND_CONNECT_PATH_V1,
            ),
            client_surface(
                bulk_query_contract_v1(),
                COMMUNICATION_BULK_ACTION_QUERY_CONNECT_PATH_V1,
            ),
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRealtime as i32,
                contract: Some(bulk_realtime_contract_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        ..Default::default()
    }
}

fn client_surface(
    contract: makosh_runtime_protocol::v1::ContractReferenceV1,
    path: &str,
) -> ProvidedSurfaceV1 {
    ProvidedSurfaceV1 {
        kind: ProvidedSurfaceKindV1::ClientRpc as i32,
        contract: Some(contract),
        client_rpc_route: Some(ClientRpcRouteV1 {
            path: path.to_owned(),
        }),
        client_blob_route: None,
    }
}

fn storage_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_BULK_ACTION_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: COMMUNICATION_BULK_ACTION_OWNER_V1.to_owned(),
                connection_budget: COMMUNICATION_BULK_ACTION_STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: 5_000,
            })),
        }],
        ..Default::default()
    }
}

fn delivery_dependency_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_BULK_ACTION_DELIVERY_DEPENDENCY_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        dependencies: vec![delivery_intent_command_contract_v1()],
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
    fn descriptor_admits_only_client_storage_and_exact_delivery_dependency() {
        let descriptor = communication_bulk_action_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&communication_bulk_action_settings_schema_v1())
            .expect("settings");
        assert_eq!(descriptor.capabilities.len(), 3);
        assert_eq!(descriptor.capabilities[0].provides.len(), 3);
        assert_eq!(
            descriptor.capabilities[1].dependencies,
            vec![delivery_intent_command_contract_v1()]
        );
    }
}
