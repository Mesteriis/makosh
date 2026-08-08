use makosh_ai_contracts::communication_summary_inference_contract_reference_v1;
use makosh_communication_summary_api::{
    COMMUNICATION_SUMMARY_CAPABILITY_ID_V1, COMMUNICATION_SUMMARY_COMMAND_CONNECT_PATH_V1,
    COMMUNICATION_SUMMARY_MODULE_ID_V1, COMMUNICATION_SUMMARY_OWNER_V1,
    COMMUNICATION_SUMMARY_QUERY_CONNECT_PATH_V1,
};
use makosh_communications_ai_source_api::{
    communication_summary_source_prepare_contract_reference_v1,
    communication_summary_source_prepare_publish_request_v1,
    communication_summary_source_prepared_consume_request_v1,
    communication_summary_source_prepared_contract_reference_v1,
    communication_summary_source_rejected_consume_request_v1,
    communication_summary_source_rejected_contract_reference_v1,
};
use makosh_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ClientRpcRouteV1, ContractReferenceV1, ModuleDescriptorV1, ModuleKindV1,
    ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1, RuntimeBudgetRequestV1,
    SettingsSchemaRefV1, SettingsSchemaV1, StorageNamespaceRequestV1,
    capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::contracts::{
    communication_summary_command_contract_v1, communication_summary_query_contract_v1,
    communication_summary_realtime_contract_v1,
};

pub const COMMUNICATION_SUMMARY_STORAGE_CAPABILITY_ID_V1: &str = "communication_summary.storage.v1";
pub const COMMUNICATION_SUMMARY_INFERENCE_CAPABILITY_ID_V1: &str =
    "communication_summary.inference.v1";
pub const COMMUNICATION_SUMMARY_BLOB_CAPABILITY_ID_V1: &str =
    "communication_summary.source.blob.v1";
const STORAGE_CONNECTION_BUDGET_V1: u32 = 4;

#[must_use]
pub fn communication_summary_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn communication_summary_settings_schema_bytes_v1() -> Vec<u8> {
    communication_summary_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn communication_summary_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = communication_summary_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: COMMUNICATION_SUMMARY_MODULE_ID_V1.to_owned(),
        owner_id: COMMUNICATION_SUMMARY_OWNER_V1.to_owned(),
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
            inference_capability(),
            blob_capability(),
            source_prepare_capability(),
            source_prepared_capability(),
            source_rejected_capability(),
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
        display_name: "Communication Summary".to_owned(),
    }
}

fn blob_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_SUMMARY_BLOB_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: 2 * 1024 * 1024,
                custody_scope_id: COMMUNICATION_SUMMARY_OWNER_V1.to_owned(),
                allowed_operations: vec![
                    BlobQuotaOperationV1::Write as i32,
                    BlobQuotaOperationV1::ReadRange as i32,
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
        capability_id: COMMUNICATION_SUMMARY_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            client_surface(
                communication_summary_command_contract_v1(),
                COMMUNICATION_SUMMARY_COMMAND_CONNECT_PATH_V1,
            ),
            client_surface(
                communication_summary_query_contract_v1(),
                COMMUNICATION_SUMMARY_QUERY_CONNECT_PATH_V1,
            ),
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRealtime as i32,
                contract: Some(communication_summary_realtime_contract_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        ..Default::default()
    }
}

fn client_surface(contract: ContractReferenceV1, path: &str) -> ProvidedSurfaceV1 {
    ProvidedSurfaceV1 {
        kind: ProvidedSurfaceKindV1::ClientRpc as i32,
        contract: Some(contract),
        client_rpc_route: Some(ClientRpcRouteV1 {
            path: path.to_owned(),
        }),
        client_blob_route: None,
    }
}

fn inference_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_SUMMARY_INFERENCE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        dependencies: vec![communication_summary_inference_contract_reference_v1()],
        ..Default::default()
    }
}

fn source_prepare_capability() -> CapabilityDescriptorV1 {
    event_capability(
        "communication_summary.source_prepare.v1",
        ProvidedSurfaceKindV1::DurablePublisher,
        communication_summary_source_prepare_contract_reference_v1(),
        communication_summary_source_prepare_publish_request_v1(),
    )
}

fn source_prepared_capability() -> CapabilityDescriptorV1 {
    event_capability(
        "communication_summary.source_prepared.v1",
        ProvidedSurfaceKindV1::DurableConsumer,
        communication_summary_source_prepared_contract_reference_v1(),
        communication_summary_source_prepared_consume_request_v1(),
    )
}

fn source_rejected_capability() -> CapabilityDescriptorV1 {
    event_capability(
        "communication_summary.source_rejected.v1",
        ProvidedSurfaceKindV1::DurableConsumer,
        communication_summary_source_rejected_contract_reference_v1(),
        communication_summary_source_rejected_consume_request_v1(),
    )
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
        capability_id: COMMUNICATION_SUMMARY_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: COMMUNICATION_SUMMARY_OWNER_V1.to_owned(),
                connection_budget: STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: 5_000,
            })),
        }],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::{
        v1::ModuleKindV1,
        validation::descriptor::{validate_descriptor_v1, validate_settings_schema_v1},
    };

    use super::*;

    #[test]
    fn descriptor_is_exact_workflow_with_event_only_source_and_request_rpc_ai() {
        let descriptor = communication_summary_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&communication_summary_settings_schema_v1()).expect("settings");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Workflow as i32);
        assert_eq!(
            descriptor
                .capabilities
                .iter()
                .map(|capability| capability.capability_id.as_str())
                .collect::<Vec<_>>(),
            [
                COMMUNICATION_SUMMARY_CAPABILITY_ID_V1,
                COMMUNICATION_SUMMARY_INFERENCE_CAPABILITY_ID_V1,
                COMMUNICATION_SUMMARY_BLOB_CAPABILITY_ID_V1,
                "communication_summary.source_prepare.v1",
                "communication_summary.source_prepared.v1",
                "communication_summary.source_rejected.v1",
                COMMUNICATION_SUMMARY_STORAGE_CAPABILITY_ID_V1,
            ]
        );
        assert_eq!(
            descriptor.capabilities[1].dependencies,
            vec![communication_summary_inference_contract_reference_v1()]
        );
    }
}
