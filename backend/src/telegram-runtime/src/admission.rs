//! Exact, still-unadmitted descriptor for the Telegram integration runtime.
//!
//! Client ports and platform dependencies stay separate capability units. This
//! descriptor does not register Telegram in the production inventory or grant
//! any capability.

use makosh_communications_call_evidence_ingress::call_evidence_observed_publish_request_v1;
use makosh_communications_ingress::admission::communication_observed_publish_request_v1;
use makosh_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ClientRpcRouteV1, ContractReferenceV1, IntegrationStateRequestV1,
    ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
    RuntimeArtifactRequestV1, RuntimeArtifactUseV1, RuntimeBudgetRequestV1, SettingsSchemaRefV1,
    StorageNamespaceRequestV1, VaultActionV1, VaultPurposeRequestV1, VaultSecretClassV1,
    VaultTargetScopeV1, capability_request_v1::Request,
};
use makosh_telegram_api::client_contract::{
    TELEGRAM_AUTHORIZATION_REALTIME_CAPABILITY_ID_V1,
    TELEGRAM_AUTHORIZATION_STATUS_CHANGED_CONTRACT_NAME_V1, TELEGRAM_CLIENT_CONTRACT_MAJOR,
    TELEGRAM_CLIENT_CONTRACT_REVISION, TELEGRAM_CLIENT_DESCRIPTOR_SET_V1, TELEGRAM_MODULE_ID,
    TELEGRAM_OWNER_ID, TelegramClientContractV1,
};
use makosh_telegram_automation_api::contract::{
    TELEGRAM_AUTOMATION_CONTRACT_MAJOR, TELEGRAM_AUTOMATION_CONTRACT_REVISION,
    TELEGRAM_AUTOMATION_DESCRIPTOR_SET_V1, TelegramAutomationContractV1,
};
use makosh_telegram_calls_api::contract::{
    TELEGRAM_CALLS_CONTRACT_MAJOR, TELEGRAM_CALLS_CONTRACT_REVISION,
    TELEGRAM_CALLS_DESCRIPTOR_SET_V1, TelegramCallsContractV1,
};
use makosh_telegram_core::{TELEGRAM_API_HASH_PURPOSE_ID, TELEGRAM_SESSION_STORE_KEY_PURPOSE_ID};
use makosh_telegram_delivery_intent_contract::{
    TELEGRAM_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1,
    telegram_delivery_intent_execute_consume_request_v1,
    telegram_delivery_intent_rejected_publish_request_v1,
    telegram_delivery_intent_succeeded_publish_request_v1,
};
use sha2::{Digest, Sha256};

use crate::settings::{
    TELEGRAM_SETTINGS_SCHEMA_MAJOR_V1, TELEGRAM_SETTINGS_SCHEMA_REVISION_V1,
    telegram_settings_schema_bytes_v1,
};

pub const TELEGRAM_BLOB_CAPABILITY_ID: &str = "telegram.blob.v1";
pub const TELEGRAM_API_HASH_PROVISIONING_CAPABILITY_ID: &str =
    "telegram.api-hash.credential-provisioning.v1";
pub const TELEGRAM_CREDENTIALS_CAPABILITY_ID: &str = "telegram.credentials.v1";
pub const TELEGRAM_EVENTS_CAPABILITY_ID: &str = "telegram.events.v1";
pub const TELEGRAM_CALL_EVIDENCE_PUBLISH_CAPABILITY_ID: &str = "telegram.call-evidence.publish.v1";
pub const TELEGRAM_RUNTIME_CAPABILITY_ID: &str = "telegram.runtime.v1";
pub const TELEGRAM_STORAGE_CAPABILITY_ID: &str = "telegram.storage.v1";
pub const TELEGRAM_SESSION_STORE_KEY_PROVISIONING_CAPABILITY_ID: &str =
    "telegram.session-store-key.credential-provisioning.v1";
pub const TELEGRAM_TDJSON_ARTIFACT_ID: &str = "telegram.tdjson.v1";
pub const TELEGRAM_TGCALLS_ARTIFACT_ID: &str = "telegram.tgcalls.v1";
pub const TELEGRAM_STATE_LAYOUT_REVISION_V1: u32 = 1;
pub const TELEGRAM_BLOB_QUOTA_BYTES: u64 = 64 * 1024 * 1024;
pub const TELEGRAM_BLOB_CUSTODY_SCOPE_ID: &str = "telegram.content.v1";
pub const TELEGRAM_STORAGE_CONNECTION_BUDGET: u32 = 4;
pub const TELEGRAM_STORAGE_STATEMENT_TIMEOUT_MILLIS: u32 = 5_000;
pub const TELEGRAM_CREDENTIAL_LEASE_TTL_SECONDS: u32 = 60;

#[must_use]
pub fn telegram_admission_capabilities_v1() -> Vec<CapabilityDescriptorV1> {
    vec![
        telegram_credential_provisioning_capability_v1(
            TELEGRAM_API_HASH_PROVISIONING_CAPABILITY_ID,
            TELEGRAM_API_HASH_PURPOSE_ID,
            VaultSecretClassV1::ProviderCredential,
        ),
        telegram_authorization_realtime_capability_v1(),
        telegram_client_capability_v1(TelegramClientContractV1::Authorization),
        telegram_automation_client_capability_v1(TelegramAutomationContractV1::Command),
        telegram_automation_client_capability_v1(TelegramAutomationContractV1::Query),
        telegram_blob_capability_v1(),
        telegram_call_evidence_publish_capability_v1(),
        telegram_calls_client_capability_v1(TelegramCallsContractV1::Command),
        telegram_calls_client_capability_v1(TelegramCallsContractV1::Query),
        telegram_calls_client_capability_v1(TelegramCallsContractV1::Realtime),
        telegram_client_capability_v1(TelegramClientContractV1::Command),
        telegram_credentials_capability_v1(),
        telegram_delivery_intent_capability_v1(),
        telegram_events_capability_v1(),
        telegram_client_capability_v1(TelegramClientContractV1::Lifecycle),
        telegram_client_capability_v1(TelegramClientContractV1::Query),
        telegram_client_capability_v1(TelegramClientContractV1::Realtime),
        telegram_client_capability_v1(TelegramClientContractV1::Reconfiguration),
        telegram_runtime_capability_v1(),
        telegram_credential_provisioning_capability_v1(
            TELEGRAM_SESSION_STORE_KEY_PROVISIONING_CAPABILITY_ID,
            TELEGRAM_SESSION_STORE_KEY_PURPOSE_ID,
            VaultSecretClassV1::SessionStoreKey,
        ),
        telegram_storage_capability_v1(),
    ]
}

fn telegram_authorization_realtime_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: TELEGRAM_AUTHORIZATION_REALTIME_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::ClientRealtime as i32,
            contract: Some(ContractReferenceV1 {
                owner: TELEGRAM_OWNER_ID.to_owned(),
                name: TELEGRAM_AUTHORIZATION_STATUS_CHANGED_CONTRACT_NAME_V1.to_owned(),
                major: TELEGRAM_CLIENT_CONTRACT_MAJOR,
                revision: TELEGRAM_CLIENT_CONTRACT_REVISION,
                schema_sha256: Sha256::digest(TELEGRAM_CLIENT_DESCRIPTOR_SET_V1).to_vec(),
            }),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

fn telegram_call_evidence_publish_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: TELEGRAM_CALL_EVIDENCE_PUBLISH_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![call_evidence_observed_publish_request_v1()],
        ..Default::default()
    }
}

fn telegram_calls_client_capability_v1(
    contract: TelegramCallsContractV1,
) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: contract.capability_id().to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::ClientRpc as i32,
            contract: Some(ContractReferenceV1 {
                owner: TELEGRAM_OWNER_ID.to_owned(),
                name: contract.contract_name().to_owned(),
                major: TELEGRAM_CALLS_CONTRACT_MAJOR,
                revision: TELEGRAM_CALLS_CONTRACT_REVISION,
                schema_sha256: Sha256::digest(TELEGRAM_CALLS_DESCRIPTOR_SET_V1).to_vec(),
            }),
            client_rpc_route: Some(ClientRpcRouteV1 {
                path: contract.connect_path().to_owned(),
            }),
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

fn telegram_automation_client_capability_v1(
    contract: TelegramAutomationContractV1,
) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: contract.capability_id().to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::ClientRpc as i32,
            contract: Some(ContractReferenceV1 {
                owner: TELEGRAM_OWNER_ID.to_owned(),
                name: contract.contract_name().to_owned(),
                major: TELEGRAM_AUTOMATION_CONTRACT_MAJOR,
                revision: TELEGRAM_AUTOMATION_CONTRACT_REVISION,
                schema_sha256: Sha256::digest(TELEGRAM_AUTOMATION_DESCRIPTOR_SET_V1).to_vec(),
            }),
            client_rpc_route: Some(ClientRpcRouteV1 {
                path: contract.connect_path().to_owned(),
            }),
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

fn telegram_client_capability_v1(contract: TelegramClientContractV1) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: contract.capability_id().to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::ClientRpc as i32,
            contract: Some(telegram_client_contract_reference_v1(contract)),
            client_rpc_route: Some(ClientRpcRouteV1 {
                path: contract.connect_path().to_owned(),
            }),
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

fn telegram_client_contract_reference_v1(
    contract: TelegramClientContractV1,
) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: TELEGRAM_OWNER_ID.to_owned(),
        name: contract.contract_name().to_owned(),
        major: TELEGRAM_CLIENT_CONTRACT_MAJOR,
        revision: TELEGRAM_CLIENT_CONTRACT_REVISION,
        schema_sha256: Sha256::digest(TELEGRAM_CLIENT_DESCRIPTOR_SET_V1).to_vec(),
    }
}

fn telegram_blob_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: TELEGRAM_BLOB_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: TELEGRAM_BLOB_QUOTA_BYTES,
                custody_scope_id: TELEGRAM_BLOB_CUSTODY_SCOPE_ID.to_owned(),
                allowed_operations: vec![
                    BlobQuotaOperationV1::Write as i32,
                    BlobQuotaOperationV1::ReadRange as i32,
                ],
            })),
        }],
        ..Default::default()
    }
}

fn telegram_credentials_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: TELEGRAM_CREDENTIALS_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![
            vault_purpose_request_v1(
                TELEGRAM_API_HASH_PURPOSE_ID,
                VaultSecretClassV1::ProviderCredential,
            ),
            vault_purpose_request_v1(
                TELEGRAM_SESSION_STORE_KEY_PURPOSE_ID,
                VaultSecretClassV1::SessionStoreKey,
            ),
        ],
        ..Default::default()
    }
}

fn telegram_credential_provisioning_capability_v1(
    capability_id: &str,
    purpose_id: &str,
    secret_class: VaultSecretClassV1,
) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: capability_id.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Optional as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::VaultPurpose(VaultPurposeRequestV1 {
                purpose_id: purpose_id.to_owned(),
                requested_lease_ttl_seconds: TELEGRAM_CREDENTIAL_LEASE_TTL_SECONDS,
                allowed_secret_classes: vec![secret_class as i32],
                actions: vec![
                    VaultActionV1::Create as i32,
                    VaultActionV1::ReplaceCas as i32,
                ],
                target_scope: VaultTargetScopeV1::ConfigurationInstance as i32,
                key_schema_revision: 0,
            })),
        }],
        ..Default::default()
    }
}

fn vault_purpose_request_v1(
    purpose_id: &str,
    secret_class: VaultSecretClassV1,
) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::VaultPurpose(VaultPurposeRequestV1 {
            purpose_id: purpose_id.to_owned(),
            requested_lease_ttl_seconds: TELEGRAM_CREDENTIAL_LEASE_TTL_SECONDS,
            allowed_secret_classes: vec![secret_class as i32],
            actions: vec![VaultActionV1::Resolve as i32],
            target_scope: VaultTargetScopeV1::ConfigurationInstance as i32,
            key_schema_revision: 0,
        })),
    }
}

fn telegram_events_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: TELEGRAM_EVENTS_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![communication_observed_publish_request_v1()],
        ..Default::default()
    }
}

fn telegram_delivery_intent_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: TELEGRAM_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![
            telegram_delivery_intent_execute_consume_request_v1(),
            telegram_delivery_intent_succeeded_publish_request_v1(),
            telegram_delivery_intent_rejected_publish_request_v1(),
        ],
        ..Default::default()
    }
}

fn telegram_runtime_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: TELEGRAM_RUNTIME_CAPABILITY_ID.to_owned(),
        capability_revision: 2,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![
            CapabilityRequestV1 {
                request: Some(Request::RuntimeArtifact(RuntimeArtifactRequestV1 {
                    artifact_id: TELEGRAM_TDJSON_ARTIFACT_ID.to_owned(),
                    r#use: RuntimeArtifactUseV1::NativeDynamicLibrary as i32,
                })),
            },
            CapabilityRequestV1 {
                request: Some(Request::RuntimeArtifact(RuntimeArtifactRequestV1 {
                    artifact_id: TELEGRAM_TGCALLS_ARTIFACT_ID.to_owned(),
                    r#use: RuntimeArtifactUseV1::NativeDynamicLibrary as i32,
                })),
            },
            CapabilityRequestV1 {
                request: Some(Request::IntegrationState(IntegrationStateRequestV1 {
                    state_layout_revision: TELEGRAM_STATE_LAYOUT_REVISION_V1,
                })),
            },
        ],
        ..Default::default()
    }
}

fn telegram_storage_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: TELEGRAM_STORAGE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: TELEGRAM_OWNER_ID.to_owned(),
                connection_budget: TELEGRAM_STORAGE_CONNECTION_BUDGET,
                timeout_millis: TELEGRAM_STORAGE_STATEMENT_TIMEOUT_MILLIS,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn telegram_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings_schema = telegram_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 6,
        module_id: TELEGRAM_MODULE_ID.to_owned(),
        owner_id: TELEGRAM_OWNER_ID.to_owned(),
        module_kind: ModuleKindV1::Integration as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: telegram_admission_capabilities_v1(),
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: TELEGRAM_SETTINGS_SCHEMA_MAJOR_V1,
            revision: TELEGRAM_SETTINGS_SCHEMA_REVISION_V1,
            artifact_size_bytes: settings_schema.len() as u64,
            sha256: Sha256::digest(&settings_schema).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: TELEGRAM_STORAGE_CONNECTION_BUDGET,
            max_memory_bytes: 512 * 1024 * 1024,
            max_cpu_millis: 1_000,
        }),
        display_name: "Telegram".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::{
        v1::{
            ModuleKindV1, ProvidedSurfaceKindV1, RuntimeArtifactUseV1,
            capability_request_v1::Request,
        },
        validation::descriptor::validate_descriptor_v1,
    };
    use makosh_telegram_api::client_contract::TelegramClientContractV1;
    use makosh_telegram_automation_api::contract::TelegramAutomationContractV1;
    use makosh_telegram_calls_api::contract::TelegramCallsContractV1;

    use super::{
        TELEGRAM_API_HASH_PROVISIONING_CAPABILITY_ID, TELEGRAM_BLOB_CAPABILITY_ID,
        TELEGRAM_CALL_EVIDENCE_PUBLISH_CAPABILITY_ID, TELEGRAM_CREDENTIALS_CAPABILITY_ID,
        TELEGRAM_EVENTS_CAPABILITY_ID, TELEGRAM_RUNTIME_CAPABILITY_ID,
        TELEGRAM_SESSION_STORE_KEY_PROVISIONING_CAPABILITY_ID, TELEGRAM_STORAGE_CAPABILITY_ID,
        telegram_module_descriptor_v1,
    };
    use makosh_telegram_delivery_intent_contract::TELEGRAM_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1;

    #[test]
    fn descriptor_is_valid_and_keeps_client_and_platform_capabilities_separate() {
        let descriptor = telegram_module_descriptor_v1("test");

        assert_eq!(validate_descriptor_v1(&descriptor), Ok(()));
        assert_eq!(descriptor.module_kind, ModuleKindV1::Integration as i32);
        assert_eq!(
            descriptor
                .capabilities
                .iter()
                .map(|capability| capability.capability_id.as_str())
                .collect::<Vec<_>>(),
            [
                TELEGRAM_API_HASH_PROVISIONING_CAPABILITY_ID,
                "telegram.authorization.realtime.v1",
                "telegram.authorization.v1",
                "telegram.automation.command.v1",
                "telegram.automation.query.v1",
                TELEGRAM_BLOB_CAPABILITY_ID,
                TELEGRAM_CALL_EVIDENCE_PUBLISH_CAPABILITY_ID,
                "telegram.calls.command.v1",
                "telegram.calls.query.v1",
                "telegram.calls.realtime.v1",
                "telegram.command.v1",
                TELEGRAM_CREDENTIALS_CAPABILITY_ID,
                TELEGRAM_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1,
                TELEGRAM_EVENTS_CAPABILITY_ID,
                "telegram.lifecycle.v1",
                "telegram.query.v1",
                "telegram.realtime.v1",
                "telegram.reconfiguration.v1",
                TELEGRAM_RUNTIME_CAPABILITY_ID,
                TELEGRAM_SESSION_STORE_KEY_PROVISIONING_CAPABILITY_ID,
                TELEGRAM_STORAGE_CAPABILITY_ID,
            ]
        );

        let client_surfaces = descriptor
            .capabilities
            .iter()
            .flat_map(|capability| &capability.provides)
            .collect::<Vec<_>>();
        assert_eq!(
            client_surfaces.len(),
            TelegramClientContractV1::ALL.len()
                + TelegramAutomationContractV1::ALL.len()
                + TelegramCallsContractV1::ALL.len()
                + 1
        );
        assert!(descriptor.capabilities.iter().any(|capability| {
            capability.capability_id == TelegramCallsContractV1::Command.capability_id()
        }));
        assert_eq!(
            client_surfaces
                .iter()
                .filter(|surface| surface.kind == ProvidedSurfaceKindV1::ClientRealtime as i32)
                .count(),
            1
        );
        assert!(client_surfaces.iter().all(|surface| {
            surface.contract.is_some()
                && ((surface.kind == ProvidedSurfaceKindV1::ClientRpc as i32
                    && surface.client_rpc_route.is_some())
                    || (surface.kind == ProvidedSurfaceKindV1::ClientRealtime as i32
                        && surface.client_rpc_route.is_none()))
        }));

        let runtime = descriptor
            .capabilities
            .iter()
            .find(|capability| capability.capability_id == TELEGRAM_RUNTIME_CAPABILITY_ID)
            .expect("Telegram runtime capability");
        assert!(runtime.requests.iter().any(|request| matches!(
            request.request,
            Some(Request::RuntimeArtifact(ref artifact))
                if artifact.artifact_id == "telegram.tdjson.v1"
                    && artifact.r#use == RuntimeArtifactUseV1::NativeDynamicLibrary as i32
        )));
        assert!(runtime.requests.iter().any(|request| matches!(
            request.request,
            Some(Request::RuntimeArtifact(ref artifact))
                if artifact.artifact_id == "telegram.tgcalls.v1"
                    && artifact.r#use == RuntimeArtifactUseV1::NativeDynamicLibrary as i32
        )));
        assert!(runtime.requests.iter().any(|request| matches!(
            request.request,
            Some(Request::IntegrationState(ref state)) if state.state_layout_revision == 1
        )));
    }
}
