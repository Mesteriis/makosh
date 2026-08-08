//! Exact technical admission for the independently managed workflow runtime.

use makosh_communication_delivery_intent_api::{
    COMMUNICATION_DELIVERY_INTENT_CAPABILITY_ID_V1,
    COMMUNICATION_DELIVERY_INTENT_COMMAND_CONNECT_PATH_V1,
    COMMUNICATION_DELIVERY_INTENT_COMMAND_CONTRACT_NAME_V1,
    COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
    COMMUNICATION_DELIVERY_INTENT_CONTRACT_REVISION_V1, COMMUNICATION_DELIVERY_INTENT_MODULE_ID_V1,
    COMMUNICATION_DELIVERY_INTENT_OWNER_V1, COMMUNICATION_DELIVERY_INTENT_QUERY_CONNECT_PATH_V1,
    COMMUNICATION_DELIVERY_INTENT_QUERY_CONTRACT_NAME_V1,
    COMMUNICATION_DELIVERY_INTENT_REALTIME_CONTRACT_NAME_V1,
    COMMUNICATION_DELIVERY_INTENT_SCHEMA_SHA256,
};
use makosh_communication_delivery_intent_ingress_api::{
    communication_delivery_intent_rejected_contract_reference_v1,
    communication_delivery_intent_rejected_publish_request_v1,
    communication_delivery_intent_submit_consume_request_v1,
    communication_delivery_intent_submit_contract_reference_v1,
    communication_delivery_intent_submitted_contract_reference_v1,
    communication_delivery_intent_submitted_publish_request_v1,
};
use makosh_communications_api::COMMUNICATIONS_QUERY_SCHEMA_SHA256;
use makosh_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ClientRpcRouteV1, ContractReferenceV1, ModuleDescriptorV1, ModuleKindV1,
    ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1, RuntimeBudgetRequestV1,
    SettingsSchemaRefV1, SettingsSchemaV1, StorageNamespaceRequestV1,
    capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::provider_event_admission::{
    delivery_intent_mail_events_capability_v1, delivery_intent_telegram_events_capability_v1,
    delivery_intent_whatsapp_events_capability_v1, delivery_intent_zulip_events_capability_v1,
};

pub const COMMUNICATION_DELIVERY_INTENT_STORAGE_CAPABILITY_ID_V1: &str =
    "communication_delivery_intent.storage.v1";
pub const COMMUNICATION_DELIVERY_INTENT_BLOB_CAPABILITY_ID_V1: &str =
    "communication_delivery_intent.blob.v1";
pub const COMMUNICATION_DELIVERY_INTENT_COMMUNICATIONS_QUERY_CAPABILITY_ID_V1: &str =
    "communication_delivery_intent.communications_query.v1";
pub const COMMUNICATION_DELIVERY_INTENT_BLOB_CUSTODY_SCOPE_ID_V1: &str =
    "communication_delivery_intent.body.v1";
pub const COMMUNICATION_DELIVERY_INTENT_BLOB_QUOTA_BYTES_V1: u64 = 16 * 1024 * 1024;
pub const COMMUNICATION_DELIVERY_INTENT_STORAGE_CONNECTION_BUDGET_V1: u32 = 4;
pub const COMMUNICATION_DELIVERY_INTENT_STORAGE_TIMEOUT_MILLIS_V1: u32 = 5_000;

#[must_use]
pub fn communication_delivery_intent_client_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_DELIVERY_INTENT_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            delivery_intent_client_surface(
                COMMUNICATION_DELIVERY_INTENT_COMMAND_CONTRACT_NAME_V1,
                COMMUNICATION_DELIVERY_INTENT_COMMAND_CONNECT_PATH_V1,
            ),
            delivery_intent_client_surface(
                COMMUNICATION_DELIVERY_INTENT_QUERY_CONTRACT_NAME_V1,
                COMMUNICATION_DELIVERY_INTENT_QUERY_CONNECT_PATH_V1,
            ),
            delivery_intent_module_request_surface(),
            delivery_intent_realtime_surface(),
        ],
        ..Default::default()
    }
}

fn delivery_intent_module_request_surface() -> ProvidedSurfaceV1 {
    ProvidedSurfaceV1 {
        kind: ProvidedSurfaceKindV1::RequestRpc as i32,
        contract: Some(ContractReferenceV1 {
            owner: COMMUNICATION_DELIVERY_INTENT_OWNER_V1.to_owned(),
            name: COMMUNICATION_DELIVERY_INTENT_COMMAND_CONTRACT_NAME_V1.to_owned(),
            major: COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
            revision: COMMUNICATION_DELIVERY_INTENT_CONTRACT_REVISION_V1,
            schema_sha256: COMMUNICATION_DELIVERY_INTENT_SCHEMA_SHA256.to_vec(),
        }),
        client_rpc_route: None,
        client_blob_route: None,
    }
}

fn delivery_intent_realtime_surface() -> ProvidedSurfaceV1 {
    ProvidedSurfaceV1 {
        kind: ProvidedSurfaceKindV1::ClientRealtime as i32,
        contract: Some(ContractReferenceV1 {
            owner: COMMUNICATION_DELIVERY_INTENT_OWNER_V1.to_owned(),
            name: COMMUNICATION_DELIVERY_INTENT_REALTIME_CONTRACT_NAME_V1.to_owned(),
            major: COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
            revision: COMMUNICATION_DELIVERY_INTENT_CONTRACT_REVISION_V1,
            schema_sha256: COMMUNICATION_DELIVERY_INTENT_SCHEMA_SHA256.to_vec(),
        }),
        client_rpc_route: None,
        client_blob_route: None,
    }
}

fn delivery_intent_client_surface(contract_name: &str, path: &str) -> ProvidedSurfaceV1 {
    ProvidedSurfaceV1 {
        kind: ProvidedSurfaceKindV1::ClientRpc as i32,
        contract: Some(ContractReferenceV1 {
            owner: COMMUNICATION_DELIVERY_INTENT_OWNER_V1.to_owned(),
            name: contract_name.to_owned(),
            major: COMMUNICATION_DELIVERY_INTENT_CONTRACT_MAJOR_V1,
            revision: COMMUNICATION_DELIVERY_INTENT_CONTRACT_REVISION_V1,
            schema_sha256: COMMUNICATION_DELIVERY_INTENT_SCHEMA_SHA256.to_vec(),
        }),
        client_rpc_route: Some(ClientRpcRouteV1 {
            path: path.to_owned(),
        }),
        client_blob_route: None,
    }
}

#[must_use]
pub fn communication_delivery_intent_storage_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_DELIVERY_INTENT_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: COMMUNICATION_DELIVERY_INTENT_OWNER_V1.to_owned(),
                connection_budget: COMMUNICATION_DELIVERY_INTENT_STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: COMMUNICATION_DELIVERY_INTENT_STORAGE_TIMEOUT_MILLIS_V1,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communication_delivery_intent_blob_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_DELIVERY_INTENT_BLOB_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: COMMUNICATION_DELIVERY_INTENT_BLOB_QUOTA_BYTES_V1,
                custody_scope_id: COMMUNICATION_DELIVERY_INTENT_BLOB_CUSTODY_SCOPE_ID_V1.to_owned(),
                allowed_operations: vec![
                    BlobQuotaOperationV1::ReadRange as i32,
                    BlobQuotaOperationV1::CustodyTransfer as i32,
                    BlobQuotaOperationV1::ReleaseCustody as i32,
                    BlobQuotaOperationV1::Write as i32,
                ],
            })),
        }],
        ..Default::default()
    }
}

fn delivery_intent_ingress_submit_capability_v1() -> CapabilityDescriptorV1 {
    event_capability(
        "communication_delivery_intent.ingress_submit.v1",
        ProvidedSurfaceKindV1::DurableConsumer,
        communication_delivery_intent_submit_contract_reference_v1(),
        communication_delivery_intent_submit_consume_request_v1(),
    )
}

fn delivery_intent_ingress_submitted_capability_v1() -> CapabilityDescriptorV1 {
    event_capability(
        "communication_delivery_intent.ingress_submitted.v1",
        ProvidedSurfaceKindV1::DurablePublisher,
        communication_delivery_intent_submitted_contract_reference_v1(),
        communication_delivery_intent_submitted_publish_request_v1(),
    )
}

fn delivery_intent_ingress_rejected_capability_v1() -> CapabilityDescriptorV1 {
    event_capability(
        "communication_delivery_intent.ingress_rejected.v1",
        ProvidedSurfaceKindV1::DurablePublisher,
        communication_delivery_intent_rejected_contract_reference_v1(),
        communication_delivery_intent_rejected_publish_request_v1(),
    )
}

fn event_capability(
    capability_id: &str,
    surface_kind: ProvidedSurfaceKindV1,
    contract: ContractReferenceV1,
    request: CapabilityRequestV1,
) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: capability_id.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: surface_kind as i32,
            contract: Some(contract),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![request],
        ..Default::default()
    }
}

#[must_use]
pub fn communication_delivery_intent_communications_query_capability_v1() -> CapabilityDescriptorV1
{
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_DELIVERY_INTENT_COMMUNICATIONS_QUERY_CAPABILITY_ID_V1
            .to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        dependencies: vec![ContractReferenceV1 {
            owner: "communications".to_owned(),
            name: "communications.query".to_owned(),
            major: 1,
            revision: 1,
            schema_sha256: COMMUNICATIONS_QUERY_SCHEMA_SHA256.to_vec(),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communication_delivery_intent_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn communication_delivery_intent_settings_schema_bytes_v1() -> Vec<u8> {
    communication_delivery_intent_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn communication_delivery_intent_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings_schema = communication_delivery_intent_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 7,
        module_id: COMMUNICATION_DELIVERY_INTENT_MODULE_ID_V1.to_owned(),
        owner_id: COMMUNICATION_DELIVERY_INTENT_OWNER_V1.to_owned(),
        module_kind: ModuleKindV1::Workflow as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: vec![
            communication_delivery_intent_client_capability_v1(),
            communication_delivery_intent_blob_capability_v1(),
            communication_delivery_intent_communications_query_capability_v1(),
            delivery_intent_ingress_rejected_capability_v1(),
            delivery_intent_ingress_submit_capability_v1(),
            delivery_intent_ingress_submitted_capability_v1(),
            delivery_intent_mail_events_capability_v1(),
            communication_delivery_intent_storage_capability_v1(),
            delivery_intent_telegram_events_capability_v1(),
            delivery_intent_whatsapp_events_capability_v1(),
            delivery_intent_zulip_events_capability_v1(),
        ],
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: settings_schema.len() as u64,
            sha256: Sha256::digest(&settings_schema).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: COMMUNICATION_DELIVERY_INTENT_STORAGE_CONNECTION_BUDGET_V1,
            max_memory_bytes: 64 * 1024 * 1024,
            max_cpu_millis: 500,
        }),
        display_name: "Communication Delivery Intent".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::validation::descriptor::{
        validate_descriptor_v1, validate_settings_schema_v1,
    };

    use super::*;

    #[test]
    fn descriptor_admits_client_realtime_query_blob_storage_and_provider_event_units() {
        let descriptor = communication_delivery_intent_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&communication_delivery_intent_settings_schema_v1())
            .expect("settings");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Workflow as i32);
        assert_eq!(descriptor.capabilities.len(), 11);
        assert_eq!(
            descriptor.capabilities[0].capability_id,
            COMMUNICATION_DELIVERY_INTENT_CAPABILITY_ID_V1
        );
        assert_eq!(
            descriptor.capabilities[1].capability_id,
            COMMUNICATION_DELIVERY_INTENT_BLOB_CAPABILITY_ID_V1
        );
        assert_eq!(
            descriptor.capabilities[2].capability_id,
            COMMUNICATION_DELIVERY_INTENT_COMMUNICATIONS_QUERY_CAPABILITY_ID_V1
        );
        assert_eq!(
            descriptor.capabilities[7].capability_id,
            COMMUNICATION_DELIVERY_INTENT_STORAGE_CAPABILITY_ID_V1
        );
        assert_eq!(
            descriptor.capabilities[3..=5]
                .iter()
                .map(|capability| capability.capability_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "communication_delivery_intent.ingress_rejected.v1",
                "communication_delivery_intent.ingress_submit.v1",
                "communication_delivery_intent.ingress_submitted.v1",
            ]
        );
        let blob_operations = descriptor.capabilities[1].requests[0]
            .request
            .as_ref()
            .and_then(|request| match request {
                Request::BlobQuota(quota) => Some(quota.allowed_operations.as_slice()),
                _ => None,
            });
        assert_eq!(
            blob_operations,
            Some(
                [
                    BlobQuotaOperationV1::ReadRange as i32,
                    BlobQuotaOperationV1::CustodyTransfer as i32,
                    BlobQuotaOperationV1::ReleaseCustody as i32,
                    BlobQuotaOperationV1::Write as i32,
                ]
                .as_slice()
            )
        );
        assert_eq!(descriptor.capabilities[0].provides.len(), 4);
        assert_eq!(
            descriptor.capabilities[0].provides[0]
                .client_rpc_route
                .as_ref()
                .map(|route| route.path.as_str()),
            Some(COMMUNICATION_DELIVERY_INTENT_COMMAND_CONNECT_PATH_V1)
        );
        assert_eq!(
            descriptor.capabilities[0].provides[2].kind,
            ProvidedSurfaceKindV1::RequestRpc as i32
        );
        assert_eq!(
            descriptor.capabilities[0].provides[2]
                .contract
                .as_ref()
                .map(|contract| contract.name.as_str()),
            Some(COMMUNICATION_DELIVERY_INTENT_COMMAND_CONTRACT_NAME_V1)
        );
        assert_eq!(
            descriptor.capabilities[0].provides[3].kind,
            ProvidedSurfaceKindV1::ClientRealtime as i32
        );
        assert_eq!(
            descriptor.capabilities[0].provides[3]
                .contract
                .as_ref()
                .map(|contract| contract.name.as_str()),
            Some(COMMUNICATION_DELIVERY_INTENT_REALTIME_CONTRACT_NAME_V1)
        );
        assert_eq!(
            descriptor.capabilities[2].dependencies,
            vec![ContractReferenceV1 {
                owner: "communications".to_owned(),
                name: "communications.query".to_owned(),
                major: 1,
                revision: 1,
                schema_sha256: COMMUNICATIONS_QUERY_SCHEMA_SHA256.to_vec(),
            }]
        );
    }
}
