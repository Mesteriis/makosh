use makosh_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1,
    ProvidedSurfaceV1, RuntimeBudgetRequestV1, SettingsSchemaRefV1, SettingsSchemaV1,
    StorageNamespaceRequestV1, capability_request_v1::Request,
};
pub use makosh_speech_to_text_api::SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1;
use makosh_speech_to_text_api::{
    SPEECH_TO_TEXT_CAPABILITY_ID_V1, SPEECH_TO_TEXT_MAX_AUDIO_BYTES_V1,
    SPEECH_TO_TEXT_MAX_TRANSCRIPT_BYTES_V1, SPEECH_TO_TEXT_MODULE_ID_V1, SPEECH_TO_TEXT_OWNER_V1,
    speech_to_text_contract_reference_v1, speech_to_text_provider_contract_reference_v1,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub const SPEECH_TO_TEXT_PROVIDER_CAPABILITY_ID_V1: &str = "speech_to_text.provider.v1";
pub const SPEECH_TO_TEXT_STORAGE_CAPABILITY_ID_V1: &str = "speech_to_text.storage.v1";
pub const SPEECH_TO_TEXT_STORAGE_CONNECTION_BUDGET_V1: u32 = 4;
pub const SPEECH_TO_TEXT_STORAGE_TIMEOUT_MILLIS_V1: u32 = 5_000;
pub const SPEECH_TO_TEXT_BLOB_CUSTODY_SCOPE_ID_V1: &str = "speech_to_text.artifacts.v1";
pub const SPEECH_TO_TEXT_BLOB_QUOTA_BYTES_V1: u64 =
    SPEECH_TO_TEXT_MAX_AUDIO_BYTES_V1 + SPEECH_TO_TEXT_MAX_TRANSCRIPT_BYTES_V1 as u64;

#[must_use]
pub fn speech_to_text_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn speech_to_text_settings_schema_bytes_v1() -> Vec<u8> {
    speech_to_text_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn speech_to_text_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = speech_to_text_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: SPEECH_TO_TEXT_MODULE_ID_V1.to_owned(),
        owner_id: SPEECH_TO_TEXT_OWNER_V1.to_owned(),
        module_kind: ModuleKindV1::Engine as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: vec![
            blob_capability(),
            provider_dependency_capability(),
            storage_capability(),
            transcribe_capability(),
        ],
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: settings.len() as u64,
            sha256: Sha256::digest(&settings).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: SPEECH_TO_TEXT_STORAGE_CONNECTION_BUDGET_V1,
            max_memory_bytes: 128 * 1024 * 1024,
            max_cpu_millis: 1_000,
        }),
        display_name: "Speech to Text".to_owned(),
    }
}

fn blob_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: SPEECH_TO_TEXT_BLOB_QUOTA_BYTES_V1,
                custody_scope_id: SPEECH_TO_TEXT_BLOB_CUSTODY_SCOPE_ID_V1.to_owned(),
                allowed_operations: vec![BlobQuotaOperationV1::CustodyTransfer as i32],
            })),
        }],
        ..Default::default()
    }
}

fn provider_dependency_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: SPEECH_TO_TEXT_PROVIDER_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        dependencies: vec![speech_to_text_provider_contract_reference_v1()],
        ..Default::default()
    }
}

fn storage_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: SPEECH_TO_TEXT_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: SPEECH_TO_TEXT_OWNER_V1.to_owned(),
                connection_budget: SPEECH_TO_TEXT_STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: SPEECH_TO_TEXT_STORAGE_TIMEOUT_MILLIS_V1,
            })),
        }],
        ..Default::default()
    }
}

fn transcribe_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: SPEECH_TO_TEXT_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::RequestRpc as i32,
            contract: Some(speech_to_text_contract_reference_v1()),
            client_rpc_route: None,
            client_blob_route: None,
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
    fn descriptor_keeps_engine_provider_storage_and_blob_authorities_separate() {
        let descriptor = speech_to_text_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&speech_to_text_settings_schema_v1()).expect("settings");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Engine as i32);
        assert_eq!(descriptor.owner_id, SPEECH_TO_TEXT_OWNER_V1);
        assert_eq!(
            descriptor
                .capabilities
                .iter()
                .map(|value| value.capability_id.as_str())
                .collect::<Vec<_>>(),
            [
                SPEECH_TO_TEXT_BLOB_CAPABILITY_ID_V1,
                SPEECH_TO_TEXT_PROVIDER_CAPABILITY_ID_V1,
                SPEECH_TO_TEXT_STORAGE_CAPABILITY_ID_V1,
                SPEECH_TO_TEXT_CAPABILITY_ID_V1,
            ]
        );
        assert_eq!(
            descriptor.capabilities[1].dependencies,
            vec![speech_to_text_provider_contract_reference_v1()]
        );
    }
}
