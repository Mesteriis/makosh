use makosh_documents_api::{
    DOCUMENTS_BLOB_CAPABILITY_ID_V1, DOCUMENTS_CLIENT_CAPABILITY_ID_V1,
    DOCUMENTS_LIFECYCLE_EVENT_CAPABILITY_ID_V1, DOCUMENTS_MODULE_ID_V1, DOCUMENTS_OWNER_ID_V1,
    DOCUMENTS_STORAGE_CAPABILITY_ID_V1, documents_client_routes_v1,
    documents_lifecycle_event_contract_reference_v1, documents_lifecycle_event_publish_request_v1,
};
use makosh_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ClientRpcRouteV1, ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1,
    ProvidedSurfaceKindV1, ProvidedSurfaceV1, RuntimeBudgetRequestV1, SettingsSchemaRefV1,
    SettingsSchemaV1, StorageNamespaceRequestV1, capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

const STORAGE_CONNECTION_BUDGET_V1: u32 = 4;
pub(crate) const DOCUMENTS_RUNTIME_CAPABILITY_IDS_V1: [&str; 4] = [
    "documents.blob.v1",
    "documents.client.v1",
    "documents.lifecycle.event.v1",
    "documents.storage.v1",
];

#[must_use]
pub fn documents_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn documents_settings_schema_bytes_v1() -> Vec<u8> {
    documents_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn documents_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = documents_settings_schema_bytes_v1();
    let mut capabilities = vec![
        blob_capability(),
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
        DOCUMENTS_RUNTIME_CAPABILITY_IDS_V1,
    );
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: DOCUMENTS_MODULE_ID_V1.to_owned(),
        owner_id: DOCUMENTS_OWNER_ID_V1.to_owned(),
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
        display_name: "Documents".to_owned(),
    }
}

fn blob_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: DOCUMENTS_BLOB_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: makosh_documents_core::MAX_DOCUMENT_BYTES_V1,
                custody_scope_id: DOCUMENTS_OWNER_ID_V1.to_owned(),
                allowed_operations: vec![
                    BlobQuotaOperationV1::CustodyTransfer as i32,
                    BlobQuotaOperationV1::ReleaseCustody as i32,
                ],
            })),
        }],
        ..Default::default()
    }
}

fn client_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: DOCUMENTS_CLIENT_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: documents_client_routes_v1()
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
        capability_id: DOCUMENTS_LIFECYCLE_EVENT_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
            contract: Some(documents_lifecycle_event_contract_reference_v1()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![documents_lifecycle_event_publish_request_v1()],
        ..Default::default()
    }
}

fn storage_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: DOCUMENTS_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: DOCUMENTS_OWNER_ID_V1.to_owned(),
                connection_budget: STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: 5_000,
            })),
        }],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_runtime_protocol::validation::descriptor::{
        validate_descriptor_v1, validate_settings_schema_v1,
    };

    #[test]
    fn descriptor_is_exact_documents_domain_with_blob_custody_only() {
        let descriptor = documents_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&documents_settings_schema_v1()).expect("settings");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Domain as i32);
        assert_eq!(descriptor.capabilities.len(), 4);
        assert_eq!(descriptor.capabilities[1].provides.len(), 11);
    }
}
