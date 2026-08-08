//! Exact workflow descriptor and capability admission.

use makosh_communications_evidence_export_source_api::{
    evidence_export_prepare_contract_reference_v1, evidence_export_prepare_publish_request_v1,
    evidence_export_prepared_consume_request_v1, evidence_export_rejected_consume_request_v1,
};
use makosh_communications_export_api::{
    COMMUNICATIONS_EXPORT_CAPABILITY_ID_V1, COMMUNICATIONS_EXPORT_COMMAND_CONNECT_PATH_V1,
    COMMUNICATIONS_EXPORT_COMMAND_CONTRACT_NAME_V1, COMMUNICATIONS_EXPORT_CONTRACT_MAJOR_V1,
    COMMUNICATIONS_EXPORT_CONTRACT_REVISION_V1, COMMUNICATIONS_EXPORT_MAX_ARTIFACT_BYTES_V1,
    COMMUNICATIONS_EXPORT_MODULE_ID_V1, COMMUNICATIONS_EXPORT_OWNER_V1,
    COMMUNICATIONS_EXPORT_QUERY_CONNECT_PATH_V1, COMMUNICATIONS_EXPORT_QUERY_CONTRACT_NAME_V1,
    COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1, COMMUNICATIONS_EXPORT_READ_CONTRACT_NAME_V1,
    COMMUNICATIONS_EXPORT_REALTIME_CONTRACT_NAME_V1, COMMUNICATIONS_EXPORT_SCHEMA_SHA256,
    COMMUNICATIONS_EXPORT_TICKET_CONNECT_PATH_V1, COMMUNICATIONS_EXPORT_TICKET_CONTRACT_NAME_V1,
};
use makosh_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ClientBlobRouteV1, ClientRpcRouteV1, ContractReferenceV1,
    ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
    RuntimeBudgetRequestV1, SettingsSchemaRefV1, SettingsSchemaV1, StorageNamespaceRequestV1,
    capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub const COMMUNICATIONS_EXPORT_EVENTS_CAPABILITY_ID_V1: &str = "communications_export.events.v1";
pub const COMMUNICATIONS_EXPORT_BLOB_CAPABILITY_ID_V1: &str = "communications_export.blob.v1";
pub const COMMUNICATIONS_EXPORT_STORAGE_CAPABILITY_ID_V1: &str = "communications_export.storage.v1";
pub const COMMUNICATIONS_EXPORT_BLOB_CUSTODY_SCOPE_ID_V1: &str =
    "communications_export.artifact.v1";
pub const COMMUNICATIONS_EXPORT_BLOB_QUOTA_BYTES_V1: u64 = 64 * 1024 * 1024;
pub const COMMUNICATIONS_EXPORT_STORAGE_CONNECTION_BUDGET_V1: u32 = 4;
pub const COMMUNICATIONS_EXPORT_STORAGE_TIMEOUT_MILLIS_V1: u32 = 5_000;

#[must_use]
pub fn communications_export_capabilities_v1() -> Vec<CapabilityDescriptorV1> {
    vec![
        communications_export_client_capability_v1(),
        communications_export_blob_capability_v1(),
        communications_export_events_capability_v1(),
        communications_export_storage_capability_v1(),
    ]
}

#[must_use]
pub fn communications_export_blob_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_EXPORT_BLOB_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: COMMUNICATIONS_EXPORT_BLOB_QUOTA_BYTES_V1,
                custody_scope_id: COMMUNICATIONS_EXPORT_BLOB_CUSTODY_SCOPE_ID_V1.to_owned(),
                allowed_operations: vec![
                    BlobQuotaOperationV1::Write as i32,
                    BlobQuotaOperationV1::ReadRange as i32,
                    BlobQuotaOperationV1::CustodyTransfer as i32,
                ],
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_export_client_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_EXPORT_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: COMMUNICATIONS_EXPORT_BLOB_QUOTA_BYTES_V1,
                custody_scope_id: COMMUNICATIONS_EXPORT_BLOB_CUSTODY_SCOPE_ID_V1.to_owned(),
                allowed_operations: vec![BlobQuotaOperationV1::ReadRange as i32],
            })),
        }],
        provides: vec![
            client_rpc_surface(
                communications_export_command_contract_reference_v1(),
                COMMUNICATIONS_EXPORT_COMMAND_CONNECT_PATH_V1,
            ),
            client_rpc_surface(
                communications_export_query_contract_reference_v1(),
                COMMUNICATIONS_EXPORT_QUERY_CONNECT_PATH_V1,
            ),
            client_rpc_surface(
                communications_export_ticket_contract_reference_v1(),
                COMMUNICATIONS_EXPORT_TICKET_CONNECT_PATH_V1,
            ),
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientBlob as i32,
                contract: Some(communications_export_read_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: Some(ClientBlobRouteV1 {
                    path: COMMUNICATIONS_EXPORT_READ_BLOB_PATH_V1.to_owned(),
                    max_response_bytes: COMMUNICATIONS_EXPORT_MAX_ARTIFACT_BYTES_V1,
                }),
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRealtime as i32,
                contract: Some(communications_export_realtime_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_export_events_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_EXPORT_EVENTS_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
            contract: Some(evidence_export_prepare_contract_reference_v1()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![
            evidence_export_prepare_publish_request_v1(),
            evidence_export_prepared_consume_request_v1(),
            evidence_export_rejected_consume_request_v1(),
        ],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_export_storage_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_EXPORT_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: COMMUNICATIONS_EXPORT_OWNER_V1.to_owned(),
                connection_budget: COMMUNICATIONS_EXPORT_STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: COMMUNICATIONS_EXPORT_STORAGE_TIMEOUT_MILLIS_V1,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_export_command_contract_reference_v1() -> ContractReferenceV1 {
    client_contract(COMMUNICATIONS_EXPORT_COMMAND_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communications_export_query_contract_reference_v1() -> ContractReferenceV1 {
    client_contract(COMMUNICATIONS_EXPORT_QUERY_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communications_export_ticket_contract_reference_v1() -> ContractReferenceV1 {
    client_contract(COMMUNICATIONS_EXPORT_TICKET_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communications_export_read_contract_reference_v1() -> ContractReferenceV1 {
    client_contract(COMMUNICATIONS_EXPORT_READ_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communications_export_realtime_contract_reference_v1() -> ContractReferenceV1 {
    client_contract(COMMUNICATIONS_EXPORT_REALTIME_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communications_export_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn communications_export_settings_schema_bytes_v1() -> Vec<u8> {
    communications_export_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn communications_export_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings_schema = communications_export_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: COMMUNICATIONS_EXPORT_MODULE_ID_V1.to_owned(),
        owner_id: COMMUNICATIONS_EXPORT_OWNER_V1.to_owned(),
        module_kind: ModuleKindV1::Workflow as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: communications_export_capabilities_v1(),
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: settings_schema.len() as u64,
            sha256: Sha256::digest(&settings_schema).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: COMMUNICATIONS_EXPORT_STORAGE_CONNECTION_BUDGET_V1,
            max_memory_bytes: 128 * 1024 * 1024,
            max_cpu_millis: 1_000,
        }),
        display_name: "Communications Export".to_owned(),
    }
}

fn client_contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATIONS_EXPORT_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: COMMUNICATIONS_EXPORT_CONTRACT_MAJOR_V1,
        revision: COMMUNICATIONS_EXPORT_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATIONS_EXPORT_SCHEMA_SHA256.to_vec(),
    }
}

fn client_rpc_surface(contract: ContractReferenceV1, path: &str) -> ProvidedSurfaceV1 {
    ProvidedSurfaceV1 {
        kind: ProvidedSurfaceKindV1::ClientRpc as i32,
        contract: Some(contract),
        client_rpc_route: Some(ClientRpcRouteV1 {
            path: path.to_owned(),
        }),
        client_blob_route: None,
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::validation::descriptor::{
        validate_descriptor_v1, validate_settings_schema_v1,
    };

    use super::*;

    #[test]
    fn workflow_descriptor_is_exact_and_valid() {
        let descriptor = communications_export_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&communications_export_settings_schema_v1()).expect("settings");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Workflow as i32);
        assert_eq!(descriptor.capabilities.len(), 4);
        let client = descriptor
            .capabilities
            .iter()
            .find(|capability| capability.capability_id == COMMUNICATIONS_EXPORT_CAPABILITY_ID_V1)
            .expect("client capability");
        let Some(Request::BlobQuota(client_blob_quota)) = client
            .requests
            .first()
            .and_then(|request| request.request.as_ref())
        else {
            panic!("client Blob route must carry its own read-range quota");
        };
        assert_eq!(
            client_blob_quota.max_bytes,
            COMMUNICATIONS_EXPORT_BLOB_QUOTA_BYTES_V1
        );
        assert_eq!(
            client_blob_quota.custody_scope_id,
            COMMUNICATIONS_EXPORT_BLOB_CUSTODY_SCOPE_ID_V1
        );
        assert_eq!(
            client_blob_quota.allowed_operations,
            vec![BlobQuotaOperationV1::ReadRange as i32]
        );
    }
}
