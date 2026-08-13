use makosh_communications_call_evidence_ingress::call_evidence_observed_publish_request_v1;
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1, ClientRpcRouteV1,
    ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
    RuntimeBudgetRequestV1, SettingsSchemaRefV1, StorageNamespaceRequestV1, VaultActionV1,
    VaultPurposeRequestV1, VaultSecretClassV1, VaultTargetScopeV1, capability_request_v1::Request,
};
use makosh_zoom_api::{
    ZOOM_ACCOUNT_CLIENT_CAPABILITY_ID_V1, ZOOM_CALL_EVIDENCE_CAPABILITY_ID_V1,
    ZOOM_CREDENTIAL_PROVISION_CAPABILITY_ID_V1, ZOOM_CREDENTIAL_PURPOSE_ID_V1,
    ZOOM_CREDENTIAL_RESOLVE_CAPABILITY_ID_V1, ZOOM_MODULE_ID_V1, ZOOM_OWNER_ID_V1,
    ZOOM_PROVIDER_CAPABILITY_ID_V1, ZOOM_STORAGE_CAPABILITY_ID_V1, zoom_client_routes_v1,
    zoom_settings_schema_bytes_v1,
};
use sha2::{Digest, Sha256};
#[must_use]
pub fn zoom_module_descriptor_v1(build: &str) -> ModuleDescriptorV1 {
    let settings = zoom_settings_schema_bytes_v1();
    let mut capabilities = vec![
        client(),
        events(),
        provider(),
        vault(
            ZOOM_CREDENTIAL_PROVISION_CAPABILITY_ID_V1,
            vec![
                VaultActionV1::Create as i32,
                VaultActionV1::ReplaceCas as i32,
            ],
        ),
        vault(
            ZOOM_CREDENTIAL_RESOLVE_CAPABILITY_ID_V1,
            vec![VaultActionV1::Resolve as i32],
        ),
        storage(),
    ];
    capabilities.sort_by(|a, b| a.capability_id.cmp(&b.capability_id));
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: ZOOM_MODULE_ID_V1.into(),
        owner_id: ZOOM_OWNER_ID_V1.into(),
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
        display_name: "Zoom".into(),
    }
}
fn client() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ZOOM_ACCOUNT_CLIENT_CAPABILITY_ID_V1.into(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: zoom_client_routes_v1()
            .into_iter()
            .map(|(contract, path)| ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRpc as i32,
                contract: Some(contract),
                client_rpc_route: Some(ClientRpcRouteV1 { path: path.into() }),
                client_blob_route: None,
            })
            .collect(),
        ..Default::default()
    }
}
fn events() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ZOOM_CALL_EVIDENCE_CAPABILITY_ID_V1.into(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![call_evidence_observed_publish_request_v1()],
        ..Default::default()
    }
}
fn provider() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ZOOM_PROVIDER_CAPABILITY_ID_V1.into(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
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
                purpose_id: ZOOM_CREDENTIAL_PURPOSE_ID_V1.into(),
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
        capability_id: ZOOM_STORAGE_CAPABILITY_ID_V1.into(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: ZOOM_OWNER_ID_V1.into(),
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
    fn descriptor_has_exact_six_separate_capabilities() {
        let d = zoom_module_descriptor_v1("test");
        validate_descriptor_v1(&d).unwrap();
        assert_eq!(d.capabilities.len(), 6);
        assert_eq!(d.module_kind, ModuleKindV1::Integration as i32);
    }
}
