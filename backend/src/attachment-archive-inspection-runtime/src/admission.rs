//! Exact managed Engine descriptor for the event-only archive worker slice.

use makosh_attachment_archive_inspection_api::{
    ATTACHMENT_ARCHIVE_INSPECTION_CAPABILITY_ID_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_COMMAND_CONNECT_PATH_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_MODULE_ID_V1, ATTACHMENT_ARCHIVE_INSPECTION_OWNER_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_QUERY_CONNECT_PATH_V1,
};
use makosh_attachment_archive_inspection_ingress::{
    ATTACHMENT_ARCHIVE_INSPECTION_BLOB_TARGET_CAPABILITY_ID_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_INGRESS_MAX_IN_FLIGHT_V1,
    archive_inspection_custody_delegated_contract_reference_v1,
    archive_inspection_custody_delegation_rejected_contract_reference_v1,
    archive_inspection_custody_delegation_requested_contract_reference_v1,
    archive_inspection_custody_delegation_requested_publish_request_v1,
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
    CapabilityRequestV1, ClientRpcRouteV1, ContractReferenceV1, DurableEnvelopeKindV1,
    EventRouteDirectionV1, EventRouteRequestV1, EventSubscriptionRequirementV1, ModuleDescriptorV1,
    ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
    RuntimeBudgetRequestV1, SettingsSchemaRefV1, StorageNamespaceRequestV1,
    capability_request_v1::Request,
};
use sha2::{Digest, Sha256};

use crate::contracts::{
    archive_inspection_command_contract_v1, archive_inspection_query_contract_v1,
    archive_inspection_realtime_contract_v1,
};
use crate::settings::{
    ATTACHMENT_ARCHIVE_INSPECTION_SETTINGS_SCHEMA_MAJOR_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_SETTINGS_SCHEMA_REVISION_V1,
    attachment_archive_inspection_settings_schema_bytes_v1,
};

pub const ATTACHMENT_ARCHIVE_INSPECTION_BLOB_CAPABILITY_ID: &str =
    ATTACHMENT_ARCHIVE_INSPECTION_BLOB_TARGET_CAPABILITY_ID_V1;
pub const ATTACHMENT_ARCHIVE_INSPECTION_CANDIDATE_CAPABILITY_ID: &str =
    "attachment_archive_inspection.candidate.observe.v1";
pub const ATTACHMENT_ARCHIVE_INSPECTION_CUSTODY_REQUEST_CAPABILITY_ID: &str =
    "attachment_archive_inspection.custody-request.publish.v1";
pub const ATTACHMENT_ARCHIVE_INSPECTION_CUSTODY_RESULT_CAPABILITY_ID: &str =
    "attachment_archive_inspection.custody-result.consume.v1";
pub const ATTACHMENT_ARCHIVE_INSPECTION_SAFETY_CAPABILITY_ID: &str =
    "attachment_archive_inspection.safety-state.observe.v1";
pub const ATTACHMENT_ARCHIVE_INSPECTION_STORAGE_CAPABILITY_ID: &str =
    "attachment_archive_inspection.storage.v1";
pub const ATTACHMENT_ARCHIVE_INSPECTION_BLOB_SCOPE_ID: &str =
    "attachment_archive_inspection.zip.source.v1";
pub const ATTACHMENT_ARCHIVE_INSPECTION_STORAGE_CONNECTION_BUDGET: u32 = 4;
pub const ATTACHMENT_ARCHIVE_INSPECTION_EVENT_MAX_DELIVER: u32 = 10;
pub const ATTACHMENT_ARCHIVE_INSPECTION_EVENT_ACK_WAIT_MILLIS: u32 = 30_000;

#[must_use]
pub fn attachment_archive_inspection_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = attachment_archive_inspection_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: ATTACHMENT_ARCHIVE_INSPECTION_MODULE_ID_V1.to_owned(),
        owner_id: ATTACHMENT_ARCHIVE_INSPECTION_OWNER_V1.to_owned(),
        module_kind: ModuleKindV1::Engine as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: capabilities(),
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: ATTACHMENT_ARCHIVE_INSPECTION_SETTINGS_SCHEMA_MAJOR_V1,
            revision: ATTACHMENT_ARCHIVE_INSPECTION_SETTINGS_SCHEMA_REVISION_V1,
            artifact_size_bytes: settings.len() as u64,
            sha256: Sha256::digest(settings).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: ATTACHMENT_ARCHIVE_INSPECTION_STORAGE_CONNECTION_BUDGET,
            max_memory_bytes: 384 * 1024 * 1024,
            max_cpu_millis: 1_000,
        }),
        display_name: "Attachment Archive Inspection".to_owned(),
    }
}

fn capabilities() -> Vec<CapabilityDescriptorV1> {
    vec![
        client(),
        blob(),
        consumer(
            ATTACHMENT_ARCHIVE_INSPECTION_CANDIDATE_CAPABILITY_ID,
            attachment_security_scan_candidate_observed_contract_reference_v1(),
            DurableEnvelopeKindV1::Observation,
            ATTACHMENT_SECURITY_MAX_IN_FLIGHT,
        ),
        custody_request_publisher(),
        custody_result_consumer(),
        consumer(
            ATTACHMENT_ARCHIVE_INSPECTION_SAFETY_CAPABILITY_ID,
            communication_attachment_safety_state_changed_contract_reference_v1(),
            DurableEnvelopeKindV1::Event,
            COMMUNICATION_ATTACHMENT_MAX_IN_FLIGHT,
        ),
        storage(),
    ]
}

fn client() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_ARCHIVE_INSPECTION_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            client_rpc(
                archive_inspection_command_contract_v1(),
                ATTACHMENT_ARCHIVE_INSPECTION_COMMAND_CONNECT_PATH_V1,
            ),
            client_rpc(
                archive_inspection_query_contract_v1(),
                ATTACHMENT_ARCHIVE_INSPECTION_QUERY_CONNECT_PATH_V1,
            ),
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRealtime as i32,
                contract: Some(archive_inspection_realtime_contract_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
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
    kind: DurableEnvelopeKindV1,
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
            kind,
            contract,
            EventRouteDirectionV1::Consume,
            max_in_flight,
        )],
        ..Default::default()
    }
}

fn custody_result_consumer() -> CapabilityDescriptorV1 {
    let contracts = [
        archive_inspection_custody_delegated_contract_reference_v1(),
        archive_inspection_custody_delegation_rejected_contract_reference_v1(),
    ];
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_ARCHIVE_INSPECTION_CUSTODY_RESULT_CAPABILITY_ID.to_owned(),
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
                    ATTACHMENT_ARCHIVE_INSPECTION_INGRESS_MAX_IN_FLIGHT_V1,
                )
            })
            .collect(),
        ..Default::default()
    }
}

fn custody_request_publisher() -> CapabilityDescriptorV1 {
    let contract = archive_inspection_custody_delegation_requested_contract_reference_v1();
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_ARCHIVE_INSPECTION_CUSTODY_REQUEST_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
            contract: Some(contract),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![archive_inspection_custody_delegation_requested_publish_request_v1()],
        ..Default::default()
    }
}

fn event_route(
    kind: DurableEnvelopeKindV1,
    contract: ContractReferenceV1,
    direction: EventRouteDirectionV1,
    max_in_flight: u32,
) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: kind as i32,
            contract: Some(contract),
            direction: direction as i32,
            max_in_flight,
            subscription_requirement: EventSubscriptionRequirementV1::Required as i32,
            max_deliver: ATTACHMENT_ARCHIVE_INSPECTION_EVENT_MAX_DELIVER,
            ack_wait_millis: ATTACHMENT_ARCHIVE_INSPECTION_EVENT_ACK_WAIT_MILLIS,
        })),
    }
}

fn blob() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_ARCHIVE_INSPECTION_BLOB_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: makosh_attachment_archive_inspection_core::DEFAULT_MAX_ARCHIVE_BYTES_V1,
                custody_scope_id: ATTACHMENT_ARCHIVE_INSPECTION_BLOB_SCOPE_ID.to_owned(),
                allowed_operations: vec![
                    BlobQuotaOperationV1::ReadRange as i32,
                    BlobQuotaOperationV1::CustodyTransfer as i32,
                ],
            })),
        }],
        ..Default::default()
    }
}

fn storage() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_ARCHIVE_INSPECTION_STORAGE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: ATTACHMENT_ARCHIVE_INSPECTION_OWNER_V1.to_owned(),
                connection_budget: ATTACHMENT_ARCHIVE_INSPECTION_STORAGE_CONNECTION_BUDGET,
                timeout_millis: 5_000,
            })),
        }],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::validation::descriptor::validate_descriptor_v1;

    use super::*;

    #[test]
    fn descriptor_is_exact_seven_capability_engine() {
        let descriptor = attachment_archive_inspection_module_descriptor_v1("build-1");
        assert_eq!(validate_descriptor_v1(&descriptor), Ok(()));
        assert_eq!(
            descriptor
                .capabilities
                .iter()
                .map(|capability| capability.capability_id.as_str())
                .collect::<Vec<_>>(),
            [
                ATTACHMENT_ARCHIVE_INSPECTION_CAPABILITY_ID_V1,
                ATTACHMENT_ARCHIVE_INSPECTION_BLOB_CAPABILITY_ID,
                ATTACHMENT_ARCHIVE_INSPECTION_CANDIDATE_CAPABILITY_ID,
                ATTACHMENT_ARCHIVE_INSPECTION_CUSTODY_REQUEST_CAPABILITY_ID,
                ATTACHMENT_ARCHIVE_INSPECTION_CUSTODY_RESULT_CAPABILITY_ID,
                ATTACHMENT_ARCHIVE_INSPECTION_SAFETY_CAPABILITY_ID,
                ATTACHMENT_ARCHIVE_INSPECTION_STORAGE_CAPABILITY_ID,
            ]
        );
    }
}
