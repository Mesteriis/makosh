//! Exact, still-unadmitted descriptor for the WhatsApp integration runtime.
//!
//! Client ports, the private host bridge and platform dependencies remain
//! separate capability units. This descriptor does not register WhatsApp,
//! approve grants or authorize a managed launch.

use makosh_communications_ingress::admission::communication_observed_publish_request_v1;
use makosh_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ClientRpcRouteV1, ContractReferenceV1, HostCapabilityRequestV1,
    ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
    RuntimeBudgetRequestV1, SettingsSchemaRefV1, StorageNamespaceRequestV1,
    capability_request_v1::Request,
};
pub use makosh_whatsapp_api::client_contract::{WHATSAPP_MODULE_ID, WHATSAPP_OWNER_ID};
use makosh_whatsapp_api::{
    client_contract::{
        WHATSAPP_CLIENT_CONTRACT_MAJOR, WHATSAPP_CLIENT_CONTRACT_REVISION, WhatsAppClientContractV1,
    },
    host_bridge::HOST_BRIDGE_CONTRACT_NAME,
};
use makosh_whatsapp_delivery_intent_contract::{
    WHATSAPP_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1,
    whatsapp_delivery_intent_execute_consume_request_v1,
    whatsapp_delivery_intent_rejected_publish_request_v1,
    whatsapp_delivery_intent_succeeded_publish_request_v1,
};
use sha2::{Digest, Sha256};

use crate::settings::{
    WHATSAPP_SETTINGS_SCHEMA_MAJOR_V1, WHATSAPP_SETTINGS_SCHEMA_REVISION_V1,
    whatsapp_settings_schema_bytes_v1,
};

pub const WHATSAPP_BLOB_CAPABILITY_ID: &str = "whatsapp.blob.v1";
pub const WHATSAPP_EVENTS_CAPABILITY_ID: &str = "whatsapp.events.v1";
pub const WHATSAPP_STORAGE_CAPABILITY_ID: &str = "whatsapp.storage.v1";
pub const WHATSAPP_BLOB_QUOTA_BYTES: u64 = 64 * 1024 * 1024;
pub const WHATSAPP_BLOB_CUSTODY_SCOPE_ID: &str = "whatsapp.content.v1";
pub const WHATSAPP_STORAGE_CONNECTION_BUDGET: u32 = 4;
pub const WHATSAPP_STORAGE_STATEMENT_TIMEOUT_MILLIS: u32 = 5_000;

#[must_use]
pub fn whatsapp_admission_capabilities_v1() -> Vec<CapabilityDescriptorV1> {
    vec![
        whatsapp_blob_capability_v1(),
        whatsapp_client_capability_v1(WhatsAppClientContractV1::Command),
        whatsapp_delivery_intent_capability_v1(),
        whatsapp_events_capability_v1(),
        whatsapp_host_bridge_capability_v1(),
        whatsapp_client_capability_v1(WhatsAppClientContractV1::OperationalQuery),
        whatsapp_client_capability_v1(WhatsAppClientContractV1::OperationalRealtime),
        whatsapp_client_capability_v1(WhatsAppClientContractV1::Query),
        whatsapp_storage_capability_v1(),
    ]
}

fn whatsapp_delivery_intent_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: WHATSAPP_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![
            whatsapp_delivery_intent_execute_consume_request_v1(),
            whatsapp_delivery_intent_succeeded_publish_request_v1(),
            whatsapp_delivery_intent_rejected_publish_request_v1(),
        ],
        ..Default::default()
    }
}

fn whatsapp_client_capability_v1(contract: WhatsAppClientContractV1) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: contract.capability_id().to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Optional as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::ClientRpc as i32,
            contract: Some(whatsapp_client_contract_reference_v1(contract)),
            client_rpc_route: Some(ClientRpcRouteV1 {
                path: contract.connect_path().to_owned(),
            }),
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

fn whatsapp_client_contract_reference_v1(
    contract: WhatsAppClientContractV1,
) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: WHATSAPP_OWNER_ID.to_owned(),
        name: contract.contract_name().to_owned(),
        major: WHATSAPP_CLIENT_CONTRACT_MAJOR,
        revision: WHATSAPP_CLIENT_CONTRACT_REVISION,
        schema_sha256: Sha256::digest(contract.descriptor_set()).to_vec(),
    }
}

fn whatsapp_blob_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: WHATSAPP_BLOB_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: WHATSAPP_BLOB_QUOTA_BYTES,
                custody_scope_id: WHATSAPP_BLOB_CUSTODY_SCOPE_ID.to_owned(),
                allowed_operations: vec![
                    BlobQuotaOperationV1::Write as i32,
                    BlobQuotaOperationV1::ReadRange as i32,
                ],
            })),
        }],
        ..Default::default()
    }
}

fn whatsapp_events_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: WHATSAPP_EVENTS_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![communication_observed_publish_request_v1()],
        ..Default::default()
    }
}

fn whatsapp_host_bridge_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: HOST_BRIDGE_CONTRACT_NAME.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::HostCapability(HostCapabilityRequestV1 {
                capability_id: HOST_BRIDGE_CONTRACT_NAME.to_owned(),
            })),
        }],
        ..Default::default()
    }
}

fn whatsapp_storage_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: WHATSAPP_STORAGE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: WHATSAPP_OWNER_ID.to_owned(),
                connection_budget: WHATSAPP_STORAGE_CONNECTION_BUDGET,
                timeout_millis: WHATSAPP_STORAGE_STATEMENT_TIMEOUT_MILLIS,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn whatsapp_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings_schema = whatsapp_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: WHATSAPP_MODULE_ID.to_owned(),
        owner_id: WHATSAPP_OWNER_ID.to_owned(),
        module_kind: ModuleKindV1::Integration as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: whatsapp_admission_capabilities_v1(),
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: WHATSAPP_SETTINGS_SCHEMA_MAJOR_V1,
            revision: WHATSAPP_SETTINGS_SCHEMA_REVISION_V1,
            artifact_size_bytes: settings_schema.len() as u64,
            sha256: Sha256::digest(&settings_schema).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: WHATSAPP_STORAGE_CONNECTION_BUDGET,
            max_memory_bytes: 256 * 1024 * 1024,
            max_cpu_millis: 1_000,
        }),
        display_name: "WhatsApp".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::{
        v1::{CapabilityCriticalityV1, ModuleKindV1, ProvidedSurfaceKindV1},
        validation::descriptor::validate_descriptor_v1,
    };

    use super::*;

    #[test]
    fn descriptor_is_valid_and_keeps_client_host_and_platform_capabilities_separate() {
        let descriptor = whatsapp_module_descriptor_v1("test");

        assert_eq!(validate_descriptor_v1(&descriptor), Ok(()));
        assert_eq!(descriptor.module_kind, ModuleKindV1::Integration as i32);
        assert_eq!(
            descriptor
                .capabilities
                .iter()
                .map(|capability| capability.capability_id.as_str())
                .collect::<Vec<_>>(),
            [
                WHATSAPP_BLOB_CAPABILITY_ID,
                WhatsAppClientContractV1::Command.capability_id(),
                WHATSAPP_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1,
                WHATSAPP_EVENTS_CAPABILITY_ID,
                HOST_BRIDGE_CONTRACT_NAME,
                WhatsAppClientContractV1::OperationalQuery.capability_id(),
                WhatsAppClientContractV1::OperationalRealtime.capability_id(),
                WhatsAppClientContractV1::Query.capability_id(),
                WHATSAPP_STORAGE_CAPABILITY_ID,
            ]
        );

        for contract in WhatsAppClientContractV1::ALL {
            let capability = descriptor
                .capabilities
                .iter()
                .find(|capability| capability.capability_id == contract.capability_id())
                .expect("WhatsApp client capability");
            assert_eq!(
                capability.criticality,
                CapabilityCriticalityV1::Optional as i32
            );
            assert_eq!(capability.provides.len(), 1);
            assert_eq!(
                capability.provides[0].kind,
                ProvidedSurfaceKindV1::ClientRpc as i32
            );
            assert_eq!(
                capability.provides[0]
                    .client_rpc_route
                    .as_ref()
                    .expect("WhatsApp client route")
                    .path,
                contract.connect_path()
            );
        }

        let host = descriptor
            .capabilities
            .iter()
            .find(|capability| capability.capability_id == HOST_BRIDGE_CONTRACT_NAME)
            .expect("WhatsApp host bridge capability");
        assert!(matches!(
            host.requests[0].request.as_ref(),
            Some(Request::HostCapability(request))
                if request.capability_id == HOST_BRIDGE_CONTRACT_NAME
        ));

        let events = descriptor
            .capabilities
            .iter()
            .find(|capability| capability.capability_id == WHATSAPP_EVENTS_CAPABILITY_ID)
            .expect("WhatsApp events capability");
        assert_eq!(events.provides, []);
        assert!(matches!(
            events.requests[0].request,
            Some(Request::EventRoute(_))
        ));
    }
}
