//! Exact workflow descriptor and capability admission.

use makosh_attachment_preview_api::{
    ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1, ATTACHMENT_PREVIEW_MAX_VIDEO_BYTES_V1,
    ATTACHMENT_PREVIEW_MODULE_ID_V1, ATTACHMENT_PREVIEW_OWNER_V1,
    ATTACHMENT_PREVIEW_QUERY_CONNECT_PATH_V1, ATTACHMENT_PREVIEW_READ_BLOB_PATH_V1,
    ATTACHMENT_PREVIEW_TICKET_CONNECT_PATH_V1,
};
use makosh_attachment_preview_ingress::{
    ATTACHMENT_PREVIEW_BLOB_TARGET_CAPABILITY_ID_V1, ATTACHMENT_PREVIEW_INGRESS_MAX_IN_FLIGHT_V1,
    attachment_preview_custody_delegated_contract_reference_v1,
    attachment_preview_custody_delegation_rejected_contract_reference_v1,
    attachment_preview_custody_delegation_requested_contract_reference_v1,
    attachment_preview_custody_delegation_requested_publish_request_v1,
};
use makosh_attachment_security_contract::admission::{
    ATTACHMENT_SECURITY_MAX_IN_FLIGHT,
    attachment_security_scan_candidate_observed_contract_reference_v1,
};
use makosh_communications_attachment_contract::admission::{
    COMMUNICATION_ATTACHMENT_MAX_IN_FLIGHT,
    communication_attachment_safety_state_changed_contract_reference_v1,
};
use makosh_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ClientBlobRouteV1, ClientRpcRouteV1, ContractReferenceV1,
    DurableEnvelopeKindV1, EventRouteDirectionV1, EventRouteRequestV1,
    EventSubscriptionRequirementV1, ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1,
    ProvidedSurfaceKindV1, ProvidedSurfaceV1, RuntimeBudgetRequestV1, SettingsSchemaRefV1,
    SettingsSchemaV1, StorageNamespaceRequestV1, capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::contracts::{
    command_contract_v1, query_contract_v1, read_contract_v1, realtime_contract_v1,
    ticket_contract_v1,
};

pub const ATTACHMENT_PREVIEW_CLIENT_CAPABILITY_ID_V1: &str = "attachment_preview.client.v1";
pub const ATTACHMENT_PREVIEW_BLOB_CAPABILITY_ID_V1: &str =
    ATTACHMENT_PREVIEW_BLOB_TARGET_CAPABILITY_ID_V1;
pub const ATTACHMENT_PREVIEW_CANDIDATE_CAPABILITY_ID_V1: &str =
    "attachment_preview.candidate.observe.v1";
pub const ATTACHMENT_PREVIEW_SAFETY_CAPABILITY_ID_V1: &str = "attachment_preview.safety.observe.v1";
pub const ATTACHMENT_PREVIEW_CUSTODY_REQUEST_CAPABILITY_ID_V1: &str =
    "attachment_preview.custody-request.publish.v1";
pub const ATTACHMENT_PREVIEW_CUSTODY_RESULT_CAPABILITY_ID_V1: &str =
    "attachment_preview.custody-result.consume.v1";
pub const ATTACHMENT_PREVIEW_STORAGE_CAPABILITY_ID_V1: &str = "attachment_preview.storage.v1";
pub const ATTACHMENT_PREVIEW_BLOB_SCOPE_ID_V1: &str = "attachment_preview.artifact.v1";
pub const ATTACHMENT_PREVIEW_BLOB_QUOTA_BYTES_V1: u64 = 256 * 1024 * 1024;
pub const ATTACHMENT_PREVIEW_STORAGE_CONNECTION_BUDGET_V1: u32 = 4;

#[must_use]
pub fn attachment_preview_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = attachment_preview_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: ATTACHMENT_PREVIEW_MODULE_ID_V1.to_owned(),
        owner_id: ATTACHMENT_PREVIEW_OWNER_V1.to_owned(),
        module_kind: ModuleKindV1::Workflow as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: vec![
            blob_capability(),
            consumer(
                ATTACHMENT_PREVIEW_CANDIDATE_CAPABILITY_ID_V1,
                attachment_security_scan_candidate_observed_contract_reference_v1(),
                DurableEnvelopeKindV1::Observation,
                ATTACHMENT_SECURITY_MAX_IN_FLIGHT,
            ),
            client_capability(),
            custody_request_publisher(),
            custody_result_consumer(),
            consumer(
                ATTACHMENT_PREVIEW_SAFETY_CAPABILITY_ID_V1,
                communication_attachment_safety_state_changed_contract_reference_v1(),
                DurableEnvelopeKindV1::Event,
                COMMUNICATION_ATTACHMENT_MAX_IN_FLIGHT,
            ),
            storage_capability(),
        ],
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: settings.len() as u64,
            sha256: Sha256::digest(settings).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: ATTACHMENT_PREVIEW_STORAGE_CONNECTION_BUDGET_V1,
            max_memory_bytes: 768 * 1024 * 1024,
            max_cpu_millis: 2_000,
        }),
        display_name: "Attachment Preview".to_owned(),
    }
}

#[must_use]
pub fn attachment_preview_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn attachment_preview_settings_schema_bytes_v1() -> Vec<u8> {
    attachment_preview_settings_schema_v1().encode_to_vec()
}

fn client_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_PREVIEW_CLIENT_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![blob_quota(vec![BlobQuotaOperationV1::ReadRange as i32])],
        provides: vec![
            client_rpc(
                command_contract_v1(),
                ATTACHMENT_PREVIEW_COMMAND_CONNECT_PATH_V1,
            ),
            client_rpc(
                query_contract_v1(),
                ATTACHMENT_PREVIEW_QUERY_CONNECT_PATH_V1,
            ),
            client_rpc(
                ticket_contract_v1(),
                ATTACHMENT_PREVIEW_TICKET_CONNECT_PATH_V1,
            ),
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRealtime as i32,
                contract: Some(realtime_contract_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientBlob as i32,
                contract: Some(read_contract_v1()),
                client_rpc_route: None,
                client_blob_route: Some(ClientBlobRouteV1 {
                    path: ATTACHMENT_PREVIEW_READ_BLOB_PATH_V1.to_owned(),
                    max_response_bytes: ATTACHMENT_PREVIEW_MAX_VIDEO_BYTES_V1,
                }),
            },
        ],
        ..Default::default()
    }
}

fn blob_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_PREVIEW_BLOB_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![blob_quota(vec![
            BlobQuotaOperationV1::CustodyTransfer as i32,
            BlobQuotaOperationV1::ReadRange as i32,
            BlobQuotaOperationV1::Write as i32,
        ])],
        ..Default::default()
    }
}

fn blob_quota(allowed_operations: Vec<i32>) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
            max_bytes: ATTACHMENT_PREVIEW_BLOB_QUOTA_BYTES_V1,
            custody_scope_id: ATTACHMENT_PREVIEW_BLOB_SCOPE_ID_V1.to_owned(),
            allowed_operations,
        })),
    }
}

fn storage_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_PREVIEW_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: ATTACHMENT_PREVIEW_OWNER_V1.to_owned(),
                connection_budget: ATTACHMENT_PREVIEW_STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: 5_000,
            })),
        }],
        ..Default::default()
    }
}

fn client_rpc(contract: ContractReferenceV1, path: &str) -> ProvidedSurfaceV1 {
    ProvidedSurfaceV1 {
        kind: ProvidedSurfaceKindV1::ClientRpc as i32,
        contract: Some(contract),
        client_rpc_route: Some(ClientRpcRouteV1 {
            path: path.to_owned(),
        }),
        client_blob_route: None,
    }
}

fn consumer(
    capability_id: &str,
    contract: ContractReferenceV1,
    envelope_kind: DurableEnvelopeKindV1,
    max_in_flight: u32,
) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: capability_id.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::DurableConsumer as i32,
            contract: Some(contract.clone()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![event_route(
            envelope_kind,
            contract,
            EventRouteDirectionV1::Consume,
            max_in_flight,
        )],
        ..Default::default()
    }
}

fn custody_request_publisher() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_PREVIEW_CUSTODY_REQUEST_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
            contract: Some(attachment_preview_custody_delegation_requested_contract_reference_v1()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![attachment_preview_custody_delegation_requested_publish_request_v1()],
        ..Default::default()
    }
}

fn custody_result_consumer() -> CapabilityDescriptorV1 {
    let contracts = [
        attachment_preview_custody_delegated_contract_reference_v1(),
        attachment_preview_custody_delegation_rejected_contract_reference_v1(),
    ];
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_PREVIEW_CUSTODY_RESULT_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: contracts
            .iter()
            .cloned()
            .map(|contract| ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurableConsumer as i32,
                contract: Some(contract),
                client_rpc_route: None,
                client_blob_route: None,
            })
            .collect(),
        requests: contracts
            .into_iter()
            .map(|contract| {
                event_route(
                    DurableEnvelopeKindV1::Result,
                    contract,
                    EventRouteDirectionV1::Consume,
                    ATTACHMENT_PREVIEW_INGRESS_MAX_IN_FLIGHT_V1,
                )
            })
            .collect(),
        ..Default::default()
    }
}

fn event_route(
    envelope_kind: DurableEnvelopeKindV1,
    contract: ContractReferenceV1,
    direction: EventRouteDirectionV1,
    max_in_flight: u32,
) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: envelope_kind as i32,
            contract: Some(contract),
            direction: direction as i32,
            max_in_flight,
            subscription_requirement: if direction == EventRouteDirectionV1::Consume {
                EventSubscriptionRequirementV1::Required as i32
            } else {
                EventSubscriptionRequirementV1::Unspecified as i32
            },
            max_deliver: u32::from(direction == EventRouteDirectionV1::Consume) * 10,
            ack_wait_millis: u32::from(direction == EventRouteDirectionV1::Consume) * 30_000,
        })),
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::validation::descriptor::{
        validate_descriptor_v1, validate_settings_schema_v1,
    };

    use super::*;

    #[test]
    fn descriptor_is_an_exact_workflow_with_private_blob_delivery() {
        let descriptor = attachment_preview_module_descriptor_v1("test-build");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&attachment_preview_settings_schema_v1()).expect("settings");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Workflow as i32);
        assert_eq!(descriptor.capabilities.len(), 7);
        let client = descriptor
            .capabilities
            .iter()
            .find(|capability| {
                capability.capability_id == ATTACHMENT_PREVIEW_CLIENT_CAPABILITY_ID_V1
            })
            .expect("client capability");
        assert_eq!(client.provides.len(), 5);
        assert!(client.provides.iter().any(|surface| {
            surface.kind == ProvidedSurfaceKindV1::ClientBlob as i32
                && surface.client_blob_route.as_ref().is_some_and(|route| {
                    route.path == ATTACHMENT_PREVIEW_READ_BLOB_PATH_V1
                        && route.max_response_bytes == ATTACHMENT_PREVIEW_MAX_VIDEO_BYTES_V1
                })
        }));
        assert!(descriptor.capabilities.iter().all(|capability| {
            capability
                .requests
                .iter()
                .all(|request| !matches!(request.request, Some(Request::TelemetrySignal(_))))
        }));
    }
}
