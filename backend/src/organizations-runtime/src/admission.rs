use makosh_organizations_api::{
    ORGANIZATIONS_CLIENT_CAPABILITY_ID_V1, ORGANIZATIONS_LIFECYCLE_EVENT_CAPABILITY_ID_V1,
    ORGANIZATIONS_MODULE_ID_V1, ORGANIZATIONS_OWNER_ID_V1, ORGANIZATIONS_STORAGE_CAPABILITY_ID_V1,
    organizations_client_routes_v1, organizations_lifecycle_event_contract_reference_v1,
    organizations_lifecycle_event_publish_request_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1, ClientRpcRouteV1,
    ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
    RuntimeBudgetRequestV1, SettingsSchemaRefV1, SettingsSchemaV1, StorageNamespaceRequestV1,
    capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

const STORAGE_CONNECTION_BUDGET_V1: u32 = 4;
pub(crate) const ORGANIZATIONS_RUNTIME_CAPABILITY_IDS_V1: [&str; 3] = [
    "organizations.client.v1",
    "organizations.lifecycle.event.v1",
    "organizations.storage.v1",
];

#[must_use]
pub fn organizations_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn organizations_settings_schema_bytes_v1() -> Vec<u8> {
    organizations_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn organizations_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = organizations_settings_schema_bytes_v1();
    let mut capabilities = vec![
        client_capability(),
        lifecycle_capability(),
        storage_capability(),
    ];
    capabilities.sort_by(|left, right| left.capability_id.cmp(&right.capability_id));
    debug_assert_eq!(
        capabilities
            .iter()
            .map(|value| value.capability_id.as_str())
            .collect::<Vec<_>>(),
        ORGANIZATIONS_RUNTIME_CAPABILITY_IDS_V1,
    );
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: ORGANIZATIONS_MODULE_ID_V1.to_owned(),
        owner_id: ORGANIZATIONS_OWNER_ID_V1.to_owned(),
        module_kind: ModuleKindV1::Domain as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities,
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
        display_name: "Organizations".to_owned(),
    }
}

fn client_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ORGANIZATIONS_CLIENT_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: organizations_client_routes_v1()
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

fn lifecycle_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ORGANIZATIONS_LIFECYCLE_EVENT_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
            contract: Some(organizations_lifecycle_event_contract_reference_v1()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![organizations_lifecycle_event_publish_request_v1()],
        ..Default::default()
    }
}

fn storage_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ORGANIZATIONS_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: ORGANIZATIONS_OWNER_ID_V1.to_owned(),
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
    fn descriptor_is_exact_provider_neutral_domain() {
        let descriptor = organizations_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&organizations_settings_schema_v1()).expect("settings");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Domain as i32);
        assert_eq!(descriptor.capabilities.len(), 3);
        assert_eq!(descriptor.capabilities[0].provides.len(), 9);
        assert!(descriptor.capabilities.iter().all(|value| {
            !value.capability_id.contains("provider") && !value.capability_id.contains("credential")
        }));
    }
}
