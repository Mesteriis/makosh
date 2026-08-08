use makosh_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1,
    ProvidedSurfaceV1, RuntimeArtifactRequestV1, RuntimeArtifactUseV1, RuntimeBudgetRequestV1,
    SettingsSchemaRefV1, StorageNamespaceRequestV1, capability_request_v1::Request,
};
use makosh_speech_to_text_api::speech_to_text_provider_contract_reference_v1;
use sha2::{Digest, Sha256};

use crate::settings::whisper_stt_settings_schema_bytes_v1;

pub const WHISPER_STT_OWNER_ID_V1: &str = "whisper_stt";
pub const WHISPER_STT_MODULE_ID_V1: &str = "makosh-whisper-stt-runtime";
pub const WHISPER_STT_PROVIDER_CAPABILITY_ID_V1: &str = "whisper_stt.provider.v1";
pub const WHISPER_STT_BLOB_CAPABILITY_ID_V1: &str = "whisper_stt.blob.v1";
pub const WHISPER_STT_STORAGE_CAPABILITY_ID_V1: &str = "whisper_stt.storage.v1";
pub const WHISPER_STT_NATIVE_CAPABILITY_ID_V1: &str = "whisper_stt.native.v1";
pub const WHISPER_STT_MODEL_ARTIFACT_ID_V1: &str = "whisper_stt.model.v1";
pub const WHISPER_STT_RUNNER_ARTIFACT_ID_V1: &str = "whisper_stt.runner.v1";

#[must_use]
pub fn whisper_stt_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = whisper_stt_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: WHISPER_STT_MODULE_ID_V1.to_owned(),
        owner_id: WHISPER_STT_OWNER_ID_V1.to_owned(),
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
            native_capability(),
            provider_capability(),
            storage_capability(),
        ],
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: settings.len() as u64,
            sha256: Sha256::digest(settings).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 2,
            max_connections: 5,
            max_memory_bytes: 2 * 1024 * 1024 * 1024,
            max_cpu_millis: 8_000,
        }),
        display_name: "Whisper Speech-to-Text".to_owned(),
    }
}

fn provider_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: WHISPER_STT_PROVIDER_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::RequestRpc as i32,
            contract: Some(speech_to_text_provider_contract_reference_v1()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

fn blob_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: WHISPER_STT_BLOB_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: 520 * 1024 * 1024,
                custody_scope_id: "whisper_stt.private_content.v1".to_owned(),
                allowed_operations: vec![
                    BlobQuotaOperationV1::CustodyTransfer as i32,
                    BlobQuotaOperationV1::ReadRange as i32,
                    BlobQuotaOperationV1::Write as i32,
                ],
            })),
        }],
        ..Default::default()
    }
}

fn native_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: WHISPER_STT_NATIVE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![
            artifact_request(
                WHISPER_STT_MODEL_ARTIFACT_ID_V1,
                RuntimeArtifactUseV1::ReadOnlyData,
            ),
            artifact_request(
                WHISPER_STT_RUNNER_ARTIFACT_ID_V1,
                RuntimeArtifactUseV1::NativeExecutable,
            ),
        ],
        ..Default::default()
    }
}

fn artifact_request(artifact_id: &str, use_kind: RuntimeArtifactUseV1) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::RuntimeArtifact(RuntimeArtifactRequestV1 {
            artifact_id: artifact_id.to_owned(),
            r#use: use_kind as i32,
        })),
    }
}

fn storage_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: WHISPER_STT_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: WHISPER_STT_OWNER_ID_V1.to_owned(),
                connection_budget: 4,
                timeout_millis: 5_000,
            })),
        }],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::validation::descriptor::validate_descriptor_v1;

    use super::*;

    #[test]
    fn descriptor_is_an_integration_with_exact_provider_and_native_resources() {
        let descriptor = whisper_stt_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Integration as i32);
        assert_eq!(descriptor.owner_id, WHISPER_STT_OWNER_ID_V1);
        assert_eq!(descriptor.capabilities.len(), 4);
        let blob = descriptor.capabilities[0].requests[0]
            .request
            .as_ref()
            .and_then(|request| match request {
                Request::BlobQuota(request) => Some(request),
                _ => None,
            })
            .expect("Whisper Blob quota");
        assert_eq!(
            blob.allowed_operations,
            vec![
                BlobQuotaOperationV1::CustodyTransfer as i32,
                BlobQuotaOperationV1::ReadRange as i32,
                BlobQuotaOperationV1::Write as i32,
            ]
        );
    }
}
