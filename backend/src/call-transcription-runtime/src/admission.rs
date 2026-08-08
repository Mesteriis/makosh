use makosh_call_transcription_api::{
    CAPABILITY_ID_V1, GET_CONNECT_PATH_V1, GET_CONTRACT_NAME_V1, MAX_TRANSCRIPT_BYTES_V1,
    MODULE_ID_V1, OWNER_ID_V1, READ_CONTRACT_NAME_V1, REALTIME_CONTRACT_NAME_V1,
    START_CONNECT_PATH_V1, START_CONTRACT_NAME_V1, TICKET_CONNECT_PATH_V1, TICKET_CONTRACT_NAME_V1,
    TRANSCRIPT_BLOB_PATH_V1, contract_reference_v1,
};
use makosh_call_transcription_ingress::{
    RECORDING_READY_CONTRACT_NAME_V1, RECORDING_REJECTED_CONTRACT_NAME_V1,
    TARGET_BLOB_CAPABILITY_ID_V1, contract_reference_v1 as ingress_contract_reference_v1,
    recording_ready_consume_request_v1, recording_rejected_consume_request_v1,
};
use makosh_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ClientBlobRouteV1, ClientRpcRouteV1, ContractReferenceV1,
    ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
    RuntimeBudgetRequestV1, SettingsSchemaRefV1, SettingsSchemaV1, StorageNamespaceRequestV1,
    capability_request_v1::Request,
};
use makosh_speech_to_text_api::speech_to_text_contract_reference_v1;
use prost::Message;
use sha2::{Digest, Sha256};

pub const STORAGE_CAPABILITY_ID_V1: &str = "call_transcription.storage.v1";
pub const STT_DEPENDENCY_CAPABILITY_ID_V1: &str = "call_transcription.stt.v1";
pub const RECORDING_READY_CAPABILITY_ID_V1: &str = "call_transcription.recording_ready.v1";
pub const RECORDING_REJECTED_CAPABILITY_ID_V1: &str = "call_transcription.recording_rejected.v1";
pub const BLOB_CAPABILITY_ID_V1: &str = TARGET_BLOB_CAPABILITY_ID_V1;
pub const STORAGE_CONNECTION_BUDGET_V1: u32 = 4;
pub const STORAGE_TIMEOUT_MILLIS_V1: u32 = 5_000;
pub const BLOB_QUOTA_BYTES_V1: u64 = 64 * 1024 * 1024 + MAX_TRANSCRIPT_BYTES_V1;

#[must_use]
pub fn settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn settings_schema_bytes_v1() -> Vec<u8> {
    settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: MODULE_ID_V1.to_owned(),
        owner_id: OWNER_ID_V1.to_owned(),
        module_kind: ModuleKindV1::Workflow as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: vec![
            event_consumer_capability(
                RECORDING_READY_CAPABILITY_ID_V1,
                ingress_contract_reference_v1(RECORDING_READY_CONTRACT_NAME_V1),
                recording_ready_consume_request_v1(),
            ),
            event_consumer_capability(
                RECORDING_REJECTED_CAPABILITY_ID_V1,
                ingress_contract_reference_v1(RECORDING_REJECTED_CONTRACT_NAME_V1),
                recording_rejected_consume_request_v1(),
            ),
            blob_capability(),
            storage_capability(),
            stt_dependency_capability(),
            client_capability(),
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
            max_memory_bytes: 128 * 1024 * 1024,
            max_cpu_millis: 1_000,
        }),
        display_name: "Call Transcription".to_owned(),
    }
}

fn client_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![blob_quota(vec![BlobQuotaOperationV1::ReadRange as i32])],
        provides: vec![
            client_rpc(
                contract_reference_v1(START_CONTRACT_NAME_V1),
                START_CONNECT_PATH_V1,
            ),
            client_rpc(
                contract_reference_v1(GET_CONTRACT_NAME_V1),
                GET_CONNECT_PATH_V1,
            ),
            client_rpc(
                contract_reference_v1(TICKET_CONTRACT_NAME_V1),
                TICKET_CONNECT_PATH_V1,
            ),
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRealtime as i32,
                contract: Some(contract_reference_v1(REALTIME_CONTRACT_NAME_V1)),
                client_rpc_route: None,
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientBlob as i32,
                contract: Some(contract_reference_v1(READ_CONTRACT_NAME_V1)),
                client_rpc_route: None,
                client_blob_route: Some(ClientBlobRouteV1 {
                    path: TRANSCRIPT_BLOB_PATH_V1.to_owned(),
                    max_response_bytes: MAX_TRANSCRIPT_BYTES_V1,
                }),
            },
        ],
        ..Default::default()
    }
}

fn client_rpc(contract: ContractReferenceV1, path: &str) -> ProvidedSurfaceV1 {
    ProvidedSurfaceV1 {
        kind: ProvidedSurfaceKindV1::ClientRpc as i32,
        contract: Some(contract),
        client_rpc_route: Some(ClientRpcRouteV1 {
            path: path.to_owned(),
        }),
        client_blob_route: None,
    }
}

fn blob_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: BLOB_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![blob_quota(vec![
            BlobQuotaOperationV1::Write as i32,
            BlobQuotaOperationV1::ReadRange as i32,
            BlobQuotaOperationV1::CustodyTransfer as i32,
            BlobQuotaOperationV1::ReleaseCustody as i32,
        ])],
        ..Default::default()
    }
}

fn blob_quota(allowed_operations: Vec<i32>) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
            max_bytes: BLOB_QUOTA_BYTES_V1,
            custody_scope_id: "call_transcription.artifacts.v1".to_owned(),
            allowed_operations,
        })),
    }
}

fn stt_dependency_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: STT_DEPENDENCY_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        dependencies: vec![speech_to_text_contract_reference_v1()],
        ..Default::default()
    }
}

fn event_consumer_capability(
    capability_id: &str,
    contract: ContractReferenceV1,
    request: CapabilityRequestV1,
) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: capability_id.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::DurableConsumer as i32,
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
        capability_id: STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: OWNER_ID_V1.to_owned(),
                connection_budget: STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: STORAGE_TIMEOUT_MILLIS_V1,
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
    fn descriptor_is_a_workflow_with_only_public_contract_dependencies() {
        let descriptor = module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&settings_schema_v1()).expect("settings");
        assert_eq!(descriptor.owner_id, OWNER_ID_V1);
        assert_eq!(descriptor.module_kind, ModuleKindV1::Workflow as i32);
        assert_eq!(
            descriptor
                .capabilities
                .iter()
                .map(|value| value.capability_id.as_str())
                .collect::<Vec<_>>(),
            [
                RECORDING_READY_CAPABILITY_ID_V1,
                RECORDING_REJECTED_CAPABILITY_ID_V1,
                BLOB_CAPABILITY_ID_V1,
                STORAGE_CAPABILITY_ID_V1,
                STT_DEPENDENCY_CAPABILITY_ID_V1,
                CAPABILITY_ID_V1,
            ]
        );
        assert_eq!(
            descriptor.capabilities[4].dependencies,
            vec![speech_to_text_contract_reference_v1()]
        );
    }
}
