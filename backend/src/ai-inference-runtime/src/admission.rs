use makosh_ai_contracts::{
    AI_ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1,
    AI_ATTACHMENT_TRANSLATION_REQUEST_CAPABILITY_ID_V1, AI_EXPLANATION_REQUEST_CAPABILITY_ID_V1,
    AI_INFERENCE_BLOB_CAPABILITY_ID_V1, AI_INFERENCE_MODULE_ID_V1,
    AI_INFERENCE_REQUEST_CAPABILITY_ID_V1, AI_OWNER_V1, AI_PROVIDER_EXPLANATION_CAPABILITY_ID_V1,
    AI_PROVIDER_GENERATION_CAPABILITY_ID_V1, AI_PROVIDER_SUMMARY_CAPABILITY_ID_V1,
    AI_PROVIDER_TRANSLATION_CAPABILITY_ID_V1, AI_SUMMARY_REQUEST_CAPABILITY_ID_V1,
    AI_TRANSLATION_REQUEST_CAPABILITY_ID_V1, ai_provider_explanation_contract_reference_v1,
    ai_provider_reply_generation_contract_reference_v1,
    ai_provider_summary_generation_contract_reference_v1,
    ai_provider_translation_contract_reference_v1,
    attachment_translation_inference_contract_reference_v1,
    communication_explanation_inference_contract_reference_v1,
    communication_reply_inference_contract_reference_v1,
    communication_summary_inference_contract_reference_v1,
    communication_translation_inference_contract_reference_v1,
};
use makosh_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1,
    ProvidedSurfaceV1, RuntimeBudgetRequestV1, SettingsSchemaRefV1, SettingsSchemaV1,
    StorageNamespaceRequestV1, capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub const AI_INFERENCE_STORAGE_CAPABILITY_ID_V1: &str = "ai.inference.storage.v1";
pub const AI_INFERENCE_STORAGE_CONNECTION_BUDGET_V1: u32 = 4;
pub const AI_INFERENCE_STORAGE_TIMEOUT_MILLIS_V1: u32 = 5_000;
pub const AI_INFERENCE_BLOB_CUSTODY_SCOPE_ID_V1: &str = "ai.inference.source.v1";
pub const AI_INFERENCE_BLOB_QUOTA_BYTES_V1: u64 = 4 * AI_ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1;

#[must_use]
pub fn ai_inference_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn ai_inference_settings_schema_bytes_v1() -> Vec<u8> {
    ai_inference_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn ai_inference_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = ai_inference_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: AI_INFERENCE_MODULE_ID_V1.to_owned(),
        owner_id: AI_OWNER_V1.to_owned(),
        module_kind: ModuleKindV1::Engine as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: vec![
            attachment_translation_request_capability(),
            explanation_request_capability(),
            blob_capability(),
            inference_request_capability(),
            storage_capability(),
            explanation_provider_dependency_capability(),
            provider_dependency_capability(),
            summary_provider_dependency_capability(),
            translation_provider_dependency_capability(),
            summary_request_capability(),
            translation_request_capability(),
        ],
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: settings.len() as u64,
            sha256: Sha256::digest(&settings).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: AI_INFERENCE_STORAGE_CONNECTION_BUDGET_V1,
            max_memory_bytes: 128 * 1024 * 1024,
            max_cpu_millis: 1_000,
        }),
        display_name: "AI Inference".to_owned(),
    }
}

fn attachment_translation_request_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: AI_ATTACHMENT_TRANSLATION_REQUEST_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::RequestRpc as i32,
            contract: Some(attachment_translation_inference_contract_reference_v1()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

fn inference_request_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: AI_INFERENCE_REQUEST_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::RequestRpc as i32,
            contract: Some(communication_reply_inference_contract_reference_v1()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

fn provider_dependency_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: AI_PROVIDER_GENERATION_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        dependencies: vec![ai_provider_reply_generation_contract_reference_v1()],
        ..Default::default()
    }
}

fn summary_request_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: AI_SUMMARY_REQUEST_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::RequestRpc as i32,
            contract: Some(communication_summary_inference_contract_reference_v1()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

fn summary_provider_dependency_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: AI_PROVIDER_SUMMARY_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        dependencies: vec![ai_provider_summary_generation_contract_reference_v1()],
        ..Default::default()
    }
}

fn translation_request_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: AI_TRANSLATION_REQUEST_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::RequestRpc as i32,
            contract: Some(communication_translation_inference_contract_reference_v1()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

fn translation_provider_dependency_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: AI_PROVIDER_TRANSLATION_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        dependencies: vec![ai_provider_translation_contract_reference_v1()],
        ..Default::default()
    }
}

fn explanation_request_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: AI_EXPLANATION_REQUEST_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::RequestRpc as i32,
            contract: Some(communication_explanation_inference_contract_reference_v1()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

fn explanation_provider_dependency_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: AI_PROVIDER_EXPLANATION_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        dependencies: vec![ai_provider_explanation_contract_reference_v1()],
        ..Default::default()
    }
}

fn blob_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: AI_INFERENCE_BLOB_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: AI_INFERENCE_BLOB_QUOTA_BYTES_V1,
                custody_scope_id: AI_INFERENCE_BLOB_CUSTODY_SCOPE_ID_V1.to_owned(),
                allowed_operations: vec![
                    BlobQuotaOperationV1::ReadRange as i32,
                    BlobQuotaOperationV1::CustodyTransfer as i32,
                ],
            })),
        }],
        ..Default::default()
    }
}

fn storage_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: AI_INFERENCE_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: AI_OWNER_V1.to_owned(),
                connection_budget: AI_INFERENCE_STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: AI_INFERENCE_STORAGE_TIMEOUT_MILLIS_V1,
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
    fn descriptor_has_separate_reply_summary_translation_and_explanation_capabilities() {
        let descriptor = ai_inference_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&ai_inference_settings_schema_v1()).expect("settings");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Engine as i32);
        assert_eq!(descriptor.owner_id, AI_OWNER_V1);
        assert_eq!(
            descriptor
                .capabilities
                .iter()
                .map(|capability| capability.capability_id.as_str())
                .collect::<Vec<_>>(),
            [
                AI_ATTACHMENT_TRANSLATION_REQUEST_CAPABILITY_ID_V1,
                AI_EXPLANATION_REQUEST_CAPABILITY_ID_V1,
                AI_INFERENCE_BLOB_CAPABILITY_ID_V1,
                AI_INFERENCE_REQUEST_CAPABILITY_ID_V1,
                AI_INFERENCE_STORAGE_CAPABILITY_ID_V1,
                AI_PROVIDER_EXPLANATION_CAPABILITY_ID_V1,
                AI_PROVIDER_GENERATION_CAPABILITY_ID_V1,
                AI_PROVIDER_SUMMARY_CAPABILITY_ID_V1,
                AI_PROVIDER_TRANSLATION_CAPABILITY_ID_V1,
                AI_SUMMARY_REQUEST_CAPABILITY_ID_V1,
                AI_TRANSLATION_REQUEST_CAPABILITY_ID_V1,
            ]
        );
        assert_eq!(
            descriptor.capabilities[6].dependencies,
            vec![ai_provider_reply_generation_contract_reference_v1()]
        );
        assert_eq!(
            descriptor.capabilities[7].dependencies,
            vec![ai_provider_summary_generation_contract_reference_v1()]
        );
        assert_eq!(
            descriptor.capabilities[8].dependencies,
            vec![ai_provider_translation_contract_reference_v1()]
        );
        assert_eq!(
            descriptor.capabilities[5].dependencies,
            vec![ai_provider_explanation_contract_reference_v1()]
        );
    }
}
