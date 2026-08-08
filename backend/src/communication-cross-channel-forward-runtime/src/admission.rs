use makosh_communication_cross_channel_forward_api::{
    COMMUNICATION_CROSS_CHANNEL_FORWARD_CAPABILITY_ID_V1,
    COMMUNICATION_CROSS_CHANNEL_FORWARD_COMMAND_CONNECT_PATH_V1,
    COMMUNICATION_CROSS_CHANNEL_FORWARD_MODULE_ID_V1, COMMUNICATION_CROSS_CHANNEL_FORWARD_OWNER_V1,
    COMMUNICATION_CROSS_CHANNEL_FORWARD_QUERY_CONNECT_PATH_V1,
};
use makosh_communication_delivery_intent_ingress_api::{
    communication_delivery_intent_rejected_consume_request_v1,
    communication_delivery_intent_rejected_contract_reference_v1,
    communication_delivery_intent_submit_contract_reference_v1,
    communication_delivery_intent_submit_publish_request_v1,
    communication_delivery_intent_submitted_consume_request_v1,
    communication_delivery_intent_submitted_contract_reference_v1,
};
use makosh_communications_cross_channel_forward_source_api::{
    cross_channel_forward_source_prepare_contract_reference_v1,
    cross_channel_forward_source_prepare_publish_request_v1,
    cross_channel_forward_source_prepared_consume_request_v1,
    cross_channel_forward_source_prepared_contract_reference_v1,
    cross_channel_forward_source_rejected_consume_request_v1,
    cross_channel_forward_source_rejected_contract_reference_v1,
};
use makosh_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ClientRpcRouteV1, ContractReferenceV1, ModuleDescriptorV1, ModuleKindV1,
    ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1, RuntimeBudgetRequestV1,
    SettingsSchemaRefV1, SettingsSchemaV1, StorageNamespaceRequestV1,
    capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::{
    COMMUNICATION_CROSS_CHANNEL_FORWARD_BLOB_CAPABILITY_ID_V1,
    contracts::{
        cross_channel_forward_command_contract_v1, cross_channel_forward_query_contract_v1,
        cross_channel_forward_realtime_contract_v1,
    },
};

pub const COMMUNICATION_CROSS_CHANNEL_FORWARD_STORAGE_CAPABILITY_ID_V1: &str =
    "communication_cross_channel_forward.storage.v1";
const STORAGE_CONNECTION_BUDGET_V1: u32 = 4;
const BLOB_QUOTA_BYTES_V1: u64 = 128 * 1024 * 1024;

#[must_use]
pub fn communication_cross_channel_forward_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn communication_cross_channel_forward_settings_schema_bytes_v1() -> Vec<u8> {
    communication_cross_channel_forward_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn communication_cross_channel_forward_module_descriptor_v1(
    build_id: &str,
) -> ModuleDescriptorV1 {
    let settings = communication_cross_channel_forward_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 3,
        module_id: COMMUNICATION_CROSS_CHANNEL_FORWARD_MODULE_ID_V1.to_owned(),
        owner_id: COMMUNICATION_CROSS_CHANNEL_FORWARD_OWNER_V1.to_owned(),
        module_kind: ModuleKindV1::Workflow as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: vec![
            client_capability(),
            blob_capability(),
            delivery_rejected_capability(),
            delivery_submit_capability(),
            delivery_submitted_capability(),
            source_prepare_capability(),
            source_prepared_capability(),
            source_rejected_capability(),
            storage_capability(),
        ],
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: settings.len() as u64,
            sha256: Sha256::digest(&settings).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: STORAGE_CONNECTION_BUDGET_V1,
            max_memory_bytes: 64 * 1024 * 1024,
            max_cpu_millis: 500,
        }),
        display_name: "Communication Cross-channel Forward".to_owned(),
    }
}

fn client_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_CROSS_CHANNEL_FORWARD_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            client_surface(
                cross_channel_forward_command_contract_v1(),
                COMMUNICATION_CROSS_CHANNEL_FORWARD_COMMAND_CONNECT_PATH_V1,
            ),
            client_surface(
                cross_channel_forward_query_contract_v1(),
                COMMUNICATION_CROSS_CHANNEL_FORWARD_QUERY_CONNECT_PATH_V1,
            ),
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRealtime as i32,
                contract: Some(cross_channel_forward_realtime_contract_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        ..Default::default()
    }
}

fn client_surface(contract: ContractReferenceV1, path: &str) -> ProvidedSurfaceV1 {
    ProvidedSurfaceV1 {
        kind: ProvidedSurfaceKindV1::ClientRpc as i32,
        contract: Some(contract),
        client_rpc_route: Some(ClientRpcRouteV1 {
            path: path.to_owned(),
        }),
        client_blob_route: None,
    }
}

fn source_prepare_capability() -> CapabilityDescriptorV1 {
    event_capability(
        "communication_cross_channel_forward.source_prepare.v1",
        ProvidedSurfaceKindV1::DurablePublisher,
        cross_channel_forward_source_prepare_contract_reference_v1(),
        cross_channel_forward_source_prepare_publish_request_v1(),
    )
}

fn source_prepared_capability() -> CapabilityDescriptorV1 {
    event_capability(
        "communication_cross_channel_forward.source_prepared.v1",
        ProvidedSurfaceKindV1::DurableConsumer,
        cross_channel_forward_source_prepared_contract_reference_v1(),
        cross_channel_forward_source_prepared_consume_request_v1(),
    )
}

fn source_rejected_capability() -> CapabilityDescriptorV1 {
    event_capability(
        "communication_cross_channel_forward.source_rejected.v1",
        ProvidedSurfaceKindV1::DurableConsumer,
        cross_channel_forward_source_rejected_contract_reference_v1(),
        cross_channel_forward_source_rejected_consume_request_v1(),
    )
}

fn delivery_submit_capability() -> CapabilityDescriptorV1 {
    event_capability(
        "communication_cross_channel_forward.delivery_submit.v1",
        ProvidedSurfaceKindV1::DurablePublisher,
        communication_delivery_intent_submit_contract_reference_v1(),
        communication_delivery_intent_submit_publish_request_v1(),
    )
}

fn delivery_submitted_capability() -> CapabilityDescriptorV1 {
    event_capability(
        "communication_cross_channel_forward.delivery_submitted.v1",
        ProvidedSurfaceKindV1::DurableConsumer,
        communication_delivery_intent_submitted_contract_reference_v1(),
        communication_delivery_intent_submitted_consume_request_v1(),
    )
}

fn delivery_rejected_capability() -> CapabilityDescriptorV1 {
    event_capability(
        "communication_cross_channel_forward.delivery_rejected.v1",
        ProvidedSurfaceKindV1::DurableConsumer,
        communication_delivery_intent_rejected_contract_reference_v1(),
        communication_delivery_intent_rejected_consume_request_v1(),
    )
}

fn event_capability(
    capability_id: &str,
    kind: ProvidedSurfaceKindV1,
    contract: ContractReferenceV1,
    request: CapabilityRequestV1,
) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: capability_id.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: kind as i32,
            contract: Some(contract),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![request],
        ..Default::default()
    }
}

fn storage_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_CROSS_CHANNEL_FORWARD_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: COMMUNICATION_CROSS_CHANNEL_FORWARD_OWNER_V1.to_owned(),
                connection_budget: STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: 5_000,
            })),
        }],
        ..Default::default()
    }
}

fn blob_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATION_CROSS_CHANNEL_FORWARD_BLOB_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: BLOB_QUOTA_BYTES_V1,
                custody_scope_id: COMMUNICATION_CROSS_CHANNEL_FORWARD_OWNER_V1.to_owned(),
                allowed_operations: vec![
                    BlobQuotaOperationV1::Write as i32,
                    BlobQuotaOperationV1::ReadRange as i32,
                    BlobQuotaOperationV1::CustodyTransfer as i32,
                    BlobQuotaOperationV1::ReleaseCustody as i32,
                ],
            })),
        }],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::communication_cross_channel_forward_module_descriptor_v1;
    use makosh_runtime_protocol::v1::capability_request_v1::Request;

    #[test]
    fn descriptor_requests_only_exact_event_storage_and_blob_capabilities() {
        let descriptor = communication_cross_channel_forward_module_descriptor_v1("build-1");
        assert_eq!(descriptor.capabilities.len(), 9);
        assert_eq!(descriptor.capabilities[0].provides.len(), 3);
        assert_eq!(
            descriptor
                .runtime_budget_request
                .expect("budget")
                .max_processes,
            1
        );
        assert!(
            descriptor
                .capabilities
                .iter()
                .flat_map(|capability| &capability.requests)
                .all(|request| matches!(
                    request.request,
                    Some(Request::EventRoute(_))
                        | Some(Request::StorageNamespace(_))
                        | Some(Request::BlobQuota(_))
                ))
        );
    }
}
