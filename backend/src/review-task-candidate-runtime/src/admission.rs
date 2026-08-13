use makosh_review_task_candidate_api::{
    REVIEW_TASK_CANDIDATE_BLOB_CAPABILITY_ID_V1, REVIEW_TASK_CANDIDATE_CLIENT_CAPABILITY_ID_V1,
    REVIEW_TASK_CANDIDATE_COMMAND_CONNECT_PATH_V1, REVIEW_TASK_CANDIDATE_LIST_CONNECT_PATH_V1,
    REVIEW_TASK_CANDIDATE_MODULE_ID_V1, REVIEW_TASK_CANDIDATE_OWNER_V1,
    REVIEW_TASK_CANDIDATE_QUERY_CONNECT_PATH_V1, REVIEW_TASK_CANDIDATE_SUBMISSION_CAPABILITY_ID_V1,
    review_task_candidate_approved_contract_reference_v1,
    review_task_candidate_approved_publish_request_v1,
    review_task_candidate_submission_rejected_contract_reference_v1,
    review_task_candidate_submission_rejected_publish_request_v1,
    review_task_candidate_submit_consume_request_v1,
    review_task_candidate_submit_contract_reference_v1,
    review_task_candidate_submitted_contract_reference_v1,
    review_task_candidate_submitted_publish_request_v1,
};
use makosh_review_task_candidate_promotion_api::{
    review_task_candidate_promotion_result_consume_request_v1,
    review_task_candidate_promotion_result_contract_reference_v1,
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

use crate::contracts::{
    command_contract_v1, list_contract_v1, query_contract_v1, realtime_contract_v1,
};

pub const REVIEW_TASK_CANDIDATE_STORAGE_CAPABILITY_ID_V1: &str = "review.task-candidate.storage.v1";
const STORAGE_CONNECTION_BUDGET_V1: u32 = 4;

#[must_use]
pub fn review_task_candidate_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn review_task_candidate_settings_schema_bytes_v1() -> Vec<u8> {
    review_task_candidate_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn review_task_candidate_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = review_task_candidate_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: REVIEW_TASK_CANDIDATE_MODULE_ID_V1.to_owned(),
        owner_id: REVIEW_TASK_CANDIDATE_OWNER_V1.to_owned(),
        module_kind: ModuleKindV1::Domain as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: vec![
            event_capability(
                "review.task-candidate.approved.publisher.v1",
                ProvidedSurfaceKindV1::DurablePublisher,
                review_task_candidate_approved_contract_reference_v1(),
                review_task_candidate_approved_publish_request_v1(),
            ),
            blob_capability(),
            client_capability(),
            event_capability(
                "review.task-candidate.promotion-result.consumer.v1",
                ProvidedSurfaceKindV1::DurableConsumer,
                review_task_candidate_promotion_result_contract_reference_v1(),
                review_task_candidate_promotion_result_consume_request_v1(),
            ),
            event_capability(
                "review.task-candidate.rejected.publisher.v1",
                ProvidedSurfaceKindV1::DurablePublisher,
                review_task_candidate_submission_rejected_contract_reference_v1(),
                review_task_candidate_submission_rejected_publish_request_v1(),
            ),
            storage_capability(),
            event_capability(
                REVIEW_TASK_CANDIDATE_SUBMISSION_CAPABILITY_ID_V1,
                ProvidedSurfaceKindV1::DurableConsumer,
                review_task_candidate_submit_contract_reference_v1(),
                review_task_candidate_submit_consume_request_v1(),
            ),
            event_capability(
                "review.task-candidate.submitted.publisher.v1",
                ProvidedSurfaceKindV1::DurablePublisher,
                review_task_candidate_submitted_contract_reference_v1(),
                review_task_candidate_submitted_publish_request_v1(),
            ),
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
        display_name: "Review Task Candidates".to_owned(),
    }
}

fn client_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: REVIEW_TASK_CANDIDATE_CLIENT_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            client_surface(
                command_contract_v1(),
                REVIEW_TASK_CANDIDATE_COMMAND_CONNECT_PATH_V1,
            ),
            client_surface(
                query_contract_v1(),
                REVIEW_TASK_CANDIDATE_QUERY_CONNECT_PATH_V1,
            ),
            client_surface(
                list_contract_v1(),
                REVIEW_TASK_CANDIDATE_LIST_CONNECT_PATH_V1,
            ),
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRealtime as i32,
                contract: Some(realtime_contract_v1()),
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

fn blob_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: REVIEW_TASK_CANDIDATE_BLOB_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: 2 * 1024 * 1024,
                custody_scope_id: REVIEW_TASK_CANDIDATE_OWNER_V1.to_owned(),
                allowed_operations: vec![
                    BlobQuotaOperationV1::ReadRange as i32,
                    BlobQuotaOperationV1::Write as i32,
                    BlobQuotaOperationV1::CustodyTransfer as i32,
                    BlobQuotaOperationV1::ReleaseCustody as i32,
                ],
            })),
        }],
        ..Default::default()
    }
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
        capability_id: REVIEW_TASK_CANDIDATE_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: REVIEW_TASK_CANDIDATE_OWNER_V1.to_owned(),
                connection_budget: STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: 5_000,
            })),
        }],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_runtime_protocol::validation::descriptor::{
        validate_descriptor_v1, validate_settings_schema_v1,
    };

    #[test]
    fn descriptor_is_a_distinct_review_domain_with_event_only_promotion() {
        let descriptor = review_task_candidate_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&review_task_candidate_settings_schema_v1()).expect("settings");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Domain as i32);
        assert_eq!(descriptor.capabilities.len(), 8);
        assert!(
            descriptor
                .capabilities
                .iter()
                .all(|capability| capability.dependencies.is_empty())
        );
        assert!(
            descriptor
                .capabilities
                .iter()
                .all(|capability| !capability.capability_id.contains("tasks.runtime"))
        );
    }
}
