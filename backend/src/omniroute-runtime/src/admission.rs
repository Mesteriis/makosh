use makosh_ai_contracts::{
    AI_PROVIDER_EXPLANATION_CAPABILITY_ID_V1, AI_PROVIDER_GENERATION_CAPABILITY_ID_V1,
    AI_PROVIDER_SUMMARY_CAPABILITY_ID_V1, AI_PROVIDER_TRANSLATION_CAPABILITY_ID_V1,
    ai_provider_explanation_contract_reference_v1,
    ai_provider_reply_generation_contract_reference_v1,
    ai_provider_summary_generation_contract_reference_v1,
    ai_provider_translation_contract_reference_v1,
};
use makosh_omniroute_api::{
    OMNIROUTE_CREDENTIAL_PROVISION_CAPABILITY_ID_V1, OMNIROUTE_CREDENTIAL_PURPOSE_ID_V1,
    OMNIROUTE_CREDENTIAL_RESOLVE_CAPABILITY_ID_V1, OMNIROUTE_MODULE_ID_V1, OMNIROUTE_OWNER_ID_V1,
    OMNIROUTE_STORAGE_CAPABILITY_ID_V1, omniroute_settings_schema_bytes_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1, ContractReferenceV1,
    ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
    RuntimeBudgetRequestV1, SettingsSchemaRefV1, StorageNamespaceRequestV1, VaultActionV1,
    VaultPurposeRequestV1, VaultSecretClassV1, VaultTargetScopeV1, capability_request_v1::Request,
};
use sha2::{Digest, Sha256};
#[must_use]
pub fn omniroute_module_descriptor_v1(build: &str) -> ModuleDescriptorV1 {
    let settings = omniroute_settings_schema_bytes_v1();
    let mut capabilities = vec![
        provider(
            AI_PROVIDER_GENERATION_CAPABILITY_ID_V1,
            ai_provider_reply_generation_contract_reference_v1(),
        ),
        provider(
            AI_PROVIDER_SUMMARY_CAPABILITY_ID_V1,
            ai_provider_summary_generation_contract_reference_v1(),
        ),
        provider(
            AI_PROVIDER_TRANSLATION_CAPABILITY_ID_V1,
            ai_provider_translation_contract_reference_v1(),
        ),
        provider(
            AI_PROVIDER_EXPLANATION_CAPABILITY_ID_V1,
            ai_provider_explanation_contract_reference_v1(),
        ),
        vault(
            OMNIROUTE_CREDENTIAL_PROVISION_CAPABILITY_ID_V1,
            vec![
                VaultActionV1::Create as i32,
                VaultActionV1::ReplaceCas as i32,
            ],
        ),
        vault(
            OMNIROUTE_CREDENTIAL_RESOLVE_CAPABILITY_ID_V1,
            vec![VaultActionV1::Resolve as i32],
        ),
        storage(),
    ];
    capabilities.sort_by(|a, b| a.capability_id.cmp(&b.capability_id));
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: OMNIROUTE_MODULE_ID_V1.into(),
        owner_id: OMNIROUTE_OWNER_ID_V1.into(),
        module_kind: ModuleKindV1::Integration as i32,
        module_version: "1".into(),
        build_id: build.into(),
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
            sha256: Sha256::digest(settings).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: 5,
            max_memory_bytes: 128 * 1024 * 1024,
            max_cpu_millis: 1000,
        }),
        display_name: "OmniRoute".into(),
    }
}
fn provider(id: &str, contract: ContractReferenceV1) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: id.into(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::RequestRpc as i32,
            contract: Some(contract),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        ..Default::default()
    }
}
fn vault(id: &str, actions: Vec<i32>) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: id.into(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::VaultPurpose(VaultPurposeRequestV1 {
                purpose_id: OMNIROUTE_CREDENTIAL_PURPOSE_ID_V1.into(),
                requested_lease_ttl_seconds: 60,
                allowed_secret_classes: vec![VaultSecretClassV1::ProviderCredential as i32],
                actions,
                target_scope: VaultTargetScopeV1::ConfigurationInstance as i32,
                key_schema_revision: 0,
            })),
        }],
        ..Default::default()
    }
}
fn storage() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: OMNIROUTE_STORAGE_CAPABILITY_ID_V1.into(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: OMNIROUTE_OWNER_ID_V1.into(),
                connection_budget: 4,
                timeout_millis: 5000,
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
    fn descriptor_has_four_typed_ai_and_three_owner_resource_capabilities() {
        let d = omniroute_module_descriptor_v1("test");
        validate_descriptor_v1(&d).unwrap();
        assert_eq!(d.capabilities.len(), 7);
        assert!(
            d.capabilities
                .iter()
                .all(|c| !c.capability_id.contains("client"))
        );
    }
}
