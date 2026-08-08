use makosh_review_attention_api::{
    REVIEW_ATTENTION_COMMAND_CAPABILITY_ID_V1, REVIEW_ATTENTION_COMMAND_CONNECT_PATH_V1,
    REVIEW_ATTENTION_MODULE_ID_V1, REVIEW_ATTENTION_OWNER_V1,
    REVIEW_ATTENTION_QUERY_CAPABILITY_ID_V1, REVIEW_ATTENTION_QUERY_CONNECT_PATH_V1,
    REVIEW_ATTENTION_REALTIME_CAPABILITY_ID_V1,
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
    review_attention_command_contract_v1, review_attention_query_contract_v1,
    review_attention_realtime_contract_v1,
};

pub const REVIEW_ATTENTION_STORAGE_CAPABILITY_ID_V1: &str =
    "review.communication-attention.storage.v1";
pub const REVIEW_ATTENTION_STORAGE_CONNECTION_BUDGET_V1: u32 = 4;

#[must_use]
pub fn review_attention_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn review_attention_settings_schema_bytes_v1() -> Vec<u8> {
    review_attention_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn review_attention_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = review_attention_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: REVIEW_ATTENTION_MODULE_ID_V1.to_owned(),
        owner_id: REVIEW_ATTENTION_OWNER_V1.to_owned(),
        module_kind: ModuleKindV1::Domain as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: vec![
            command_capability(),
            query_capability(),
            realtime_capability(),
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
            max_connections: REVIEW_ATTENTION_STORAGE_CONNECTION_BUDGET_V1,
            max_memory_bytes: 64 * 1024 * 1024,
            max_cpu_millis: 500,
        }),
        display_name: "Review Communications Attention".to_owned(),
    }
}

fn command_capability() -> CapabilityDescriptorV1 {
    client_capability(
        REVIEW_ATTENTION_COMMAND_CAPABILITY_ID_V1,
        review_attention_command_contract_v1(),
        Some(REVIEW_ATTENTION_COMMAND_CONNECT_PATH_V1),
        ProvidedSurfaceKindV1::RequestRpc,
    )
}

fn query_capability() -> CapabilityDescriptorV1 {
    client_capability(
        REVIEW_ATTENTION_QUERY_CAPABILITY_ID_V1,
        review_attention_query_contract_v1(),
        Some(REVIEW_ATTENTION_QUERY_CONNECT_PATH_V1),
        ProvidedSurfaceKindV1::QueryRpc,
    )
}

fn realtime_capability() -> CapabilityDescriptorV1 {
    client_capability(
        REVIEW_ATTENTION_REALTIME_CAPABILITY_ID_V1,
        review_attention_realtime_contract_v1(),
        None,
        ProvidedSurfaceKindV1::ClientRealtime,
    )
}

fn client_capability(
    capability_id: &str,
    contract: makosh_runtime_protocol::v1::ContractReferenceV1,
    client_path: Option<&str>,
    service_kind: ProvidedSurfaceKindV1,
) -> CapabilityDescriptorV1 {
    let mut provides = vec![ProvidedSurfaceV1 {
        kind: service_kind as i32,
        contract: Some(contract.clone()),
        client_rpc_route: None,
        client_blob_route: None,
    }];
    if let Some(path) = client_path {
        provides.push(ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::ClientRpc as i32,
            contract: Some(contract),
            client_rpc_route: Some(ClientRpcRouteV1 {
                path: path.to_owned(),
            }),
            client_blob_route: None,
        });
    }
    CapabilityDescriptorV1 {
        capability_id: capability_id.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides,
        ..Default::default()
    }
}

fn storage_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: REVIEW_ATTENTION_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: REVIEW_ATTENTION_OWNER_V1.to_owned(),
                connection_budget: REVIEW_ATTENTION_STORAGE_CONNECTION_BUDGET_V1,
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
    fn descriptor_admits_four_exact_review_capabilities() {
        let descriptor = review_attention_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&review_attention_settings_schema_v1()).expect("settings");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Domain as i32);
        assert_eq!(descriptor.capabilities.len(), 4);
        assert_eq!(
            descriptor.capabilities[0].capability_id,
            REVIEW_ATTENTION_COMMAND_CAPABILITY_ID_V1
        );
        assert_eq!(
            descriptor.capabilities[1].capability_id,
            REVIEW_ATTENTION_QUERY_CAPABILITY_ID_V1
        );
        assert_eq!(
            descriptor.capabilities[2].capability_id,
            REVIEW_ATTENTION_REALTIME_CAPABILITY_ID_V1
        );
        assert_eq!(
            descriptor.capabilities[3].capability_id,
            REVIEW_ATTENTION_STORAGE_CAPABILITY_ID_V1
        );
    }
}
