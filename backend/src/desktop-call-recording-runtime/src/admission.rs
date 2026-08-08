use makosh_call_transcription_ingress::{
    recording_ready_publish_request_v1, recording_rejected_publish_request_v1,
};
use makosh_desktop_call_recording_api::{
    GET_CONTRACT_NAME_V1, HOST_CONTRACT_NAME_V1, MODULE_ID_V1, OWNER_ID_V1,
    REALTIME_CONTRACT_NAME_V1, START_CONTRACT_NAME_V1, STOP_CONTRACT_NAME_V1,
    contract_reference_v1,
};
use makosh_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ClientRpcRouteV1, HostCapabilityRequestV1, ModuleDescriptorV1,
    ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
    RuntimeBudgetRequestV1, SettingsSchemaRefV1, StorageNamespaceRequestV1,
    capability_request_v1::Request,
};
use sha2::{Digest, Sha256};

use crate::settings::{
    SETTINGS_SCHEMA_MAJOR_V1, SETTINGS_SCHEMA_REVISION_V1, settings_schema_bytes_v1,
};

pub const CLIENT_CAPABILITY_ID_V1: &str = "desktop_call_recording.client.v1";
pub const HOST_CAPABILITY_ID_V1: &str = HOST_CONTRACT_NAME_V1;
pub const EVENTS_CAPABILITY_ID_V1: &str = "desktop_call_recording.events.v1";
pub const BLOB_CAPABILITY_ID_V1: &str = "desktop_call_recording.blob.v1";
pub const STORAGE_CAPABILITY_ID_V1: &str = "desktop_call_recording.storage.v1";
pub const BLOB_CUSTODY_SCOPE_V1: &str = "desktop_call_recording.private_audio.v1";
pub const START_PATH_V1: &str =
    "/makosh.desktop_call_recording.v1.DesktopCallRecordingService/Start";
pub const STOP_PATH_V1: &str = "/makosh.desktop_call_recording.v1.DesktopCallRecordingService/Stop";
pub const GET_PATH_V1: &str = "/makosh.desktop_call_recording.v1.DesktopCallRecordingService/Get";

#[must_use]
pub fn module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: MODULE_ID_V1.to_owned(),
        owner_id: OWNER_ID_V1.to_owned(),
        module_kind: ModuleKindV1::Integration as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: vec![
            blob_capability(),
            client_capability(),
            events_capability(),
            host_capability(),
            storage_capability(),
        ],
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: SETTINGS_SCHEMA_MAJOR_V1,
            revision: SETTINGS_SCHEMA_REVISION_V1,
            artifact_size_bytes: settings.len() as u64,
            sha256: Sha256::digest(settings).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: 4,
            max_memory_bytes: 192 * 1024 * 1024,
            max_cpu_millis: 2_000,
        }),
        display_name: "Desktop Call Recording".to_owned(),
    }
}

fn blob_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: BLOB_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: 64 * 1024 * 1024,
                custody_scope_id: BLOB_CUSTODY_SCOPE_V1.to_owned(),
                allowed_operations: vec![
                    BlobQuotaOperationV1::Write as i32,
                    BlobQuotaOperationV1::CustodyTransfer as i32,
                ],
            })),
        }],
        ..Default::default()
    }
}

fn client_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: CLIENT_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            client_rpc(START_PATH_V1, START_CONTRACT_NAME_V1),
            client_rpc(STOP_PATH_V1, STOP_CONTRACT_NAME_V1),
            client_rpc(GET_PATH_V1, GET_CONTRACT_NAME_V1),
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRealtime as i32,
                contract: Some(contract_reference_v1(REALTIME_CONTRACT_NAME_V1)),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        ..Default::default()
    }
}

fn client_rpc(path: &str, contract_name: &str) -> ProvidedSurfaceV1 {
    ProvidedSurfaceV1 {
        kind: ProvidedSurfaceKindV1::ClientRpc as i32,
        contract: Some(contract_reference_v1(contract_name)),
        client_rpc_route: Some(ClientRpcRouteV1 {
            path: path.to_owned(),
        }),
        client_blob_route: None,
    }
}

fn events_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: EVENTS_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![
            recording_ready_publish_request_v1(),
            recording_rejected_publish_request_v1(),
        ],
        ..Default::default()
    }
}
fn host_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: HOST_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::HostCapability(HostCapabilityRequestV1 {
                capability_id: HOST_CONTRACT_NAME_V1.to_owned(),
            })),
        }],
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
                connection_budget: 4,
                timeout_millis: 5_000,
            })),
        }],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_runtime_protocol::validation::descriptor::validate_descriptor_v1;
    #[test]
    fn descriptor_is_an_integration_with_separate_capabilities() {
        let descriptor = module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Integration as i32);
        assert_eq!(descriptor.capabilities.len(), 5);
        assert!(
            descriptor
                .capabilities
                .iter()
                .any(|value| value.capability_id == HOST_CAPABILITY_ID_V1)
        );
    }
}
