use makosh_ai_contracts::{
    ai_provider_explanation_contract_reference_v1,
    ai_provider_reply_generation_contract_reference_v1,
    ai_provider_summary_generation_contract_reference_v1,
    ai_provider_translation_contract_reference_v1,
};
use makosh_ollama_ai_api::{
    OLLAMA_AI_EXPLANATION_CAPABILITY_ID_V1, OLLAMA_AI_MODULE_ID_V1,
    OLLAMA_AI_PROVIDER_CAPABILITY_ID_V1, OLLAMA_AI_STORAGE_CAPABILITY_ID_V1,
    OLLAMA_AI_SUMMARY_CAPABILITY_ID_V1, OLLAMA_AI_TRANSLATION_CAPABILITY_ID_V1, OLLAMA_OWNER_ID_V1,
    ollama_ai_settings_schema_bytes_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1, ModuleDescriptorV1,
    ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
    RuntimeBudgetRequestV1, SettingsSchemaRefV1, StorageNamespaceRequestV1,
    capability_request_v1::Request,
};
use sha2::{Digest, Sha256};

pub const OLLAMA_AI_STORAGE_CONNECTION_BUDGET_V1: u32 = 4;
pub const OLLAMA_AI_STORAGE_TIMEOUT_MILLIS_V1: u32 = 5_000;

#[must_use]
pub fn ollama_ai_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = ollama_ai_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: OLLAMA_AI_MODULE_ID_V1.to_owned(),
        owner_id: OLLAMA_OWNER_ID_V1.to_owned(),
        module_kind: ModuleKindV1::Integration as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: vec![
            explanation_capability_v1(),
            provider_capability_v1(),
            summary_capability_v1(),
            translation_capability_v1(),
            storage_capability_v1(),
        ],
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: settings.len() as u64,
            sha256: Sha256::digest(&settings).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: OLLAMA_AI_STORAGE_CONNECTION_BUDGET_V1 + 1,
            max_memory_bytes: 128 * 1024 * 1024,
            max_cpu_millis: 1_000,
        }),
        display_name: "Ollama AI".to_owned(),
    }
}

fn explanation_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: OLLAMA_AI_EXPLANATION_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::RequestRpc as i32,
            contract: Some(ai_provider_explanation_contract_reference_v1()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

fn provider_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: OLLAMA_AI_PROVIDER_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::RequestRpc as i32,
            contract: Some(ai_provider_reply_generation_contract_reference_v1()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

fn storage_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: OLLAMA_AI_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: OLLAMA_OWNER_ID_V1.to_owned(),
                connection_budget: OLLAMA_AI_STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: OLLAMA_AI_STORAGE_TIMEOUT_MILLIS_V1,
            })),
        }],
        ..Default::default()
    }
}

fn summary_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: OLLAMA_AI_SUMMARY_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::RequestRpc as i32,
            contract: Some(ai_provider_summary_generation_contract_reference_v1()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

fn translation_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: OLLAMA_AI_TRANSLATION_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::RequestRpc as i32,
            contract: Some(ai_provider_translation_contract_reference_v1()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::validation::descriptor::validate_descriptor_v1;

    use super::*;

    #[test]
    fn descriptor_has_distinct_explanation_reply_summary_and_translation_provider_capabilities() {
        let descriptor = ollama_ai_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Integration as i32);
        assert_eq!(descriptor.owner_id, OLLAMA_OWNER_ID_V1);
        assert_eq!(
            descriptor
                .capabilities
                .iter()
                .map(|capability| capability.capability_id.as_str())
                .collect::<Vec<_>>(),
            [
                OLLAMA_AI_EXPLANATION_CAPABILITY_ID_V1,
                OLLAMA_AI_PROVIDER_CAPABILITY_ID_V1,
                OLLAMA_AI_SUMMARY_CAPABILITY_ID_V1,
                OLLAMA_AI_TRANSLATION_CAPABILITY_ID_V1,
                OLLAMA_AI_STORAGE_CAPABILITY_ID_V1,
            ]
        );
    }
}
