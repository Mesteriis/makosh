//! Exact descriptor and capability admission for the Attachment Security engine.

use makosh_attachment_archive_inspection_ingress::{
    ATTACHMENT_SECURITY_ARCHIVE_DELEGATION_CAPABILITY_ID_V1,
    archive_inspection_custody_delegated_contract_reference_v1,
    archive_inspection_custody_delegated_publish_request_v1,
    archive_inspection_custody_delegation_rejected_contract_reference_v1,
    archive_inspection_custody_delegation_rejected_publish_request_v1,
    archive_inspection_custody_delegation_requested_contract_reference_v1,
};
use makosh_attachment_preview_ingress::{
    ATTACHMENT_SECURITY_PREVIEW_DELEGATION_CAPABILITY_ID_V1,
    attachment_preview_custody_delegated_contract_reference_v1,
    attachment_preview_custody_delegated_publish_request_v1,
    attachment_preview_custody_delegation_rejected_contract_reference_v1,
    attachment_preview_custody_delegation_rejected_publish_request_v1,
    attachment_preview_custody_delegation_requested_contract_reference_v1,
};
use makosh_attachment_security_contract::admission::{
    ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_CAPABILITY_ID,
    ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_MODULE_ID,
    ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_OWNER_ID, ATTACHMENT_SECURITY_MAX_IN_FLIGHT,
    attachment_security_scan_candidate_observed_contract_reference_v1,
};
use makosh_attachment_text_extraction_ingress::{
    ATTACHMENT_SECURITY_TEXT_EXTRACTION_DELEGATION_CAPABILITY_ID_V1,
    attachment_text_custody_delegated_contract_reference_v1,
    attachment_text_custody_delegated_publish_request_v1,
    attachment_text_custody_delegation_rejected_contract_reference_v1,
    attachment_text_custody_delegation_rejected_publish_request_v1,
    attachment_text_custody_delegation_requested_contract_reference_v1,
};
use makosh_communications_attachment_contract::admission::{
    COMMUNICATION_ATTACHMENT_MAX_IN_FLIGHT,
    communication_attachment_safety_state_changed_contract_reference_v1,
    communication_attachment_safety_verdict_observed_contract_reference_v1,
    communication_attachment_safety_verdict_observed_publish_request_v1,
};
use makosh_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, ModuleDescriptorV1, ModuleKindV1,
    ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1, RuntimeBudgetRequestV1,
    SettingsSchemaRefV1, StorageNamespaceRequestV1, capability_request_v1::Request,
};
use sha2::{Digest, Sha256};

use crate::settings::{
    ATTACHMENT_SECURITY_SETTINGS_SCHEMA_MAJOR_V1, ATTACHMENT_SECURITY_SETTINGS_SCHEMA_REVISION_V1,
    attachment_security_settings_schema_bytes_v1,
};

pub const ATTACHMENT_SECURITY_CANDIDATE_OBSERVE_CAPABILITY_ID: &str =
    "attachment_security.candidate.observe.v1";
pub const ATTACHMENT_SECURITY_COMMUNICATIONS_STATE_OBSERVE_CAPABILITY_ID: &str =
    "attachment_security.communications-state.observe.v1";
pub const ATTACHMENT_SECURITY_VERDICT_PUBLISH_CAPABILITY_ID: &str =
    "attachment_security.verdict.publish.v1";
pub const ATTACHMENT_SECURITY_ARCHIVE_DELEGATION_CONSUME_CAPABILITY_ID: &str =
    ATTACHMENT_SECURITY_ARCHIVE_DELEGATION_CAPABILITY_ID_V1;
pub const ATTACHMENT_SECURITY_ARCHIVE_DELEGATION_RESULT_PUBLISH_CAPABILITY_ID: &str =
    "attachment_security.archive-delegation-result.publish.v1";
pub const ATTACHMENT_SECURITY_TEXT_DELEGATION_CONSUME_CAPABILITY_ID: &str =
    ATTACHMENT_SECURITY_TEXT_EXTRACTION_DELEGATION_CAPABILITY_ID_V1;
pub const ATTACHMENT_SECURITY_TEXT_DELEGATION_RESULT_PUBLISH_CAPABILITY_ID: &str =
    "attachment_security.text-extraction-delegation-result.publish.v1";
pub const ATTACHMENT_SECURITY_PREVIEW_DELEGATION_CONSUME_CAPABILITY_ID: &str =
    ATTACHMENT_SECURITY_PREVIEW_DELEGATION_CAPABILITY_ID_V1;
pub const ATTACHMENT_SECURITY_PREVIEW_DELEGATION_RESULT_PUBLISH_CAPABILITY_ID: &str =
    "attachment_security.preview-delegation-result.publish.v1";
pub const ATTACHMENT_SECURITY_BLOB_CAPABILITY_ID: &str =
    ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_CAPABILITY_ID;
pub const ATTACHMENT_SECURITY_STORAGE_CAPABILITY_ID: &str = "attachment_security.storage.v1";
pub const ATTACHMENT_SECURITY_MODULE_ID: &str = ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_MODULE_ID;
pub const ATTACHMENT_SECURITY_OWNER_ID: &str = ATTACHMENT_SECURITY_BLOB_CUSTODY_TARGET_OWNER_ID;
pub const ATTACHMENT_SECURITY_BLOB_QUOTA_BYTES: u64 = 64 * 1024 * 1024;
pub const ATTACHMENT_SECURITY_BLOB_CUSTODY_SCOPE_ID: &str = "attachment_security.scan.content.v1";
pub const ATTACHMENT_SECURITY_STORAGE_CONNECTION_BUDGET: u32 = 4;
pub const ATTACHMENT_SECURITY_STORAGE_STATEMENT_TIMEOUT_MILLIS: u32 = 5_000;
pub const ATTACHMENT_SECURITY_EVENT_MAX_DELIVER: u32 = 8;
pub const ATTACHMENT_SECURITY_EVENT_ACK_WAIT_MILLIS: u32 = 30_000;

#[must_use]
pub fn attachment_security_admission_capabilities_v1() -> Vec<CapabilityDescriptorV1> {
    vec![
        archive_delegation_result_publisher(),
        durable_consumer(
            ATTACHMENT_SECURITY_ARCHIVE_DELEGATION_CONSUME_CAPABILITY_ID,
            archive_inspection_custody_delegation_requested_contract_reference_v1(),
            DurableEnvelopeKindV1::Command,
            ATTACHMENT_SECURITY_MAX_IN_FLIGHT,
        ),
        blob_custody(),
        durable_consumer(
            ATTACHMENT_SECURITY_CANDIDATE_OBSERVE_CAPABILITY_ID,
            attachment_security_scan_candidate_observed_contract_reference_v1(),
            DurableEnvelopeKindV1::Observation,
            ATTACHMENT_SECURITY_MAX_IN_FLIGHT,
        ),
        durable_consumer(
            ATTACHMENT_SECURITY_COMMUNICATIONS_STATE_OBSERVE_CAPABILITY_ID,
            communication_attachment_safety_state_changed_contract_reference_v1(),
            DurableEnvelopeKindV1::Event,
            COMMUNICATION_ATTACHMENT_MAX_IN_FLIGHT,
        ),
        preview_delegation_result_publisher(),
        durable_consumer(
            ATTACHMENT_SECURITY_PREVIEW_DELEGATION_CONSUME_CAPABILITY_ID,
            attachment_preview_custody_delegation_requested_contract_reference_v1(),
            DurableEnvelopeKindV1::Command,
            ATTACHMENT_SECURITY_MAX_IN_FLIGHT,
        ),
        storage(),
        text_delegation_result_publisher(),
        durable_consumer(
            ATTACHMENT_SECURITY_TEXT_DELEGATION_CONSUME_CAPABILITY_ID,
            attachment_text_custody_delegation_requested_contract_reference_v1(),
            DurableEnvelopeKindV1::Command,
            ATTACHMENT_SECURITY_MAX_IN_FLIGHT,
        ),
        verdict_publisher(),
    ]
}

#[must_use]
pub fn attachment_security_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings_schema = attachment_security_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: ATTACHMENT_SECURITY_MODULE_ID.to_owned(),
        owner_id: ATTACHMENT_SECURITY_OWNER_ID.to_owned(),
        module_kind: ModuleKindV1::Engine as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: attachment_security_admission_capabilities_v1(),
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: ATTACHMENT_SECURITY_SETTINGS_SCHEMA_MAJOR_V1,
            revision: ATTACHMENT_SECURITY_SETTINGS_SCHEMA_REVISION_V1,
            artifact_size_bytes: settings_schema.len() as u64,
            sha256: Sha256::digest(&settings_schema).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: ATTACHMENT_SECURITY_STORAGE_CONNECTION_BUDGET,
            max_memory_bytes: 256 * 1024 * 1024,
            max_cpu_millis: 1_000,
        }),
        display_name: "Attachment Security".to_owned(),
    }
}

fn durable_consumer(
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
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::EventRoute(EventRouteRequestV1 {
                envelope_kind: envelope_kind as i32,
                contract: Some(contract),
                direction: EventRouteDirectionV1::Consume as i32,
                max_in_flight,
                subscription_requirement: EventSubscriptionRequirementV1::Required as i32,
                max_deliver: ATTACHMENT_SECURITY_EVENT_MAX_DELIVER,
                ack_wait_millis: ATTACHMENT_SECURITY_EVENT_ACK_WAIT_MILLIS,
            })),
        }],
        ..Default::default()
    }
}

fn verdict_publisher() -> CapabilityDescriptorV1 {
    let contract = communication_attachment_safety_verdict_observed_contract_reference_v1();
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_SECURITY_VERDICT_PUBLISH_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
            contract: Some(contract),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![communication_attachment_safety_verdict_observed_publish_request_v1()],
        ..Default::default()
    }
}

fn archive_delegation_result_publisher() -> CapabilityDescriptorV1 {
    let delegated = archive_inspection_custody_delegated_contract_reference_v1();
    let rejected = archive_inspection_custody_delegation_rejected_contract_reference_v1();
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_SECURITY_ARCHIVE_DELEGATION_RESULT_PUBLISH_CAPABILITY_ID
            .to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(delegated),
                client_rpc_route: None,
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(rejected),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        requests: vec![
            archive_inspection_custody_delegated_publish_request_v1(),
            archive_inspection_custody_delegation_rejected_publish_request_v1(),
        ],
        ..Default::default()
    }
}

fn text_delegation_result_publisher() -> CapabilityDescriptorV1 {
    let delegated = attachment_text_custody_delegated_contract_reference_v1();
    let rejected = attachment_text_custody_delegation_rejected_contract_reference_v1();
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_SECURITY_TEXT_DELEGATION_RESULT_PUBLISH_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(delegated),
                client_rpc_route: None,
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(rejected),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        requests: vec![
            attachment_text_custody_delegated_publish_request_v1(),
            attachment_text_custody_delegation_rejected_publish_request_v1(),
        ],
        ..Default::default()
    }
}

fn preview_delegation_result_publisher() -> CapabilityDescriptorV1 {
    let delegated = attachment_preview_custody_delegated_contract_reference_v1();
    let rejected = attachment_preview_custody_delegation_rejected_contract_reference_v1();
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_SECURITY_PREVIEW_DELEGATION_RESULT_PUBLISH_CAPABILITY_ID
            .to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(delegated),
                client_rpc_route: None,
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(rejected),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        requests: vec![
            attachment_preview_custody_delegated_publish_request_v1(),
            attachment_preview_custody_delegation_rejected_publish_request_v1(),
        ],
        ..Default::default()
    }
}

fn blob_custody() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: ATTACHMENT_SECURITY_BLOB_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: ATTACHMENT_SECURITY_BLOB_QUOTA_BYTES,
                custody_scope_id: ATTACHMENT_SECURITY_BLOB_CUSTODY_SCOPE_ID.to_owned(),
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
        capability_id: ATTACHMENT_SECURITY_STORAGE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: ATTACHMENT_SECURITY_OWNER_ID.to_owned(),
                connection_budget: ATTACHMENT_SECURITY_STORAGE_CONNECTION_BUDGET,
                timeout_millis: ATTACHMENT_SECURITY_STORAGE_STATEMENT_TIMEOUT_MILLIS,
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
    fn descriptor_has_exact_owner_boundaries() {
        let descriptor = attachment_security_module_descriptor_v1("test");
        assert_eq!(validate_descriptor_v1(&descriptor), Ok(()));
        assert_eq!(descriptor.module_kind, ModuleKindV1::Engine as i32);
        assert_eq!(descriptor.owner_id, ATTACHMENT_SECURITY_OWNER_ID);
        assert_eq!(
            descriptor
                .capabilities
                .iter()
                .map(|capability| capability.capability_id.as_str())
                .collect::<Vec<_>>(),
            [
                ATTACHMENT_SECURITY_ARCHIVE_DELEGATION_RESULT_PUBLISH_CAPABILITY_ID,
                ATTACHMENT_SECURITY_ARCHIVE_DELEGATION_CONSUME_CAPABILITY_ID,
                ATTACHMENT_SECURITY_BLOB_CAPABILITY_ID,
                ATTACHMENT_SECURITY_CANDIDATE_OBSERVE_CAPABILITY_ID,
                ATTACHMENT_SECURITY_COMMUNICATIONS_STATE_OBSERVE_CAPABILITY_ID,
                ATTACHMENT_SECURITY_PREVIEW_DELEGATION_RESULT_PUBLISH_CAPABILITY_ID,
                ATTACHMENT_SECURITY_PREVIEW_DELEGATION_CONSUME_CAPABILITY_ID,
                ATTACHMENT_SECURITY_STORAGE_CAPABILITY_ID,
                ATTACHMENT_SECURITY_TEXT_DELEGATION_RESULT_PUBLISH_CAPABILITY_ID,
                ATTACHMENT_SECURITY_TEXT_DELEGATION_CONSUME_CAPABILITY_ID,
                ATTACHMENT_SECURITY_VERDICT_PUBLISH_CAPABILITY_ID,
            ]
        );
    }
}
