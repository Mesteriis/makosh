use makosh_identity_resolution_api::identity_resolution_person_match_candidate_contract_reference_v1;
use makosh_review_person_match_candidate_api::{
    REVIEW_PERSON_MATCH_CANDIDATE_MODULE_ID_V1, REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1,
    review_person_match_candidate_approved_contract_reference_v1,
    review_person_match_candidate_approved_publish_request_v1,
    review_person_match_candidate_decision_consume_request_v1,
    review_person_match_candidate_decision_contract_reference_v1,
    review_person_match_candidate_submission_rejected_contract_reference_v1,
    review_person_match_candidate_submission_rejected_publish_request_v1,
    review_person_match_candidate_submitted_contract_reference_v1,
    review_person_match_candidate_submitted_publish_request_v1,
};
use makosh_review_person_match_candidate_promotion_api::{
    review_person_match_candidate_promotion_result_consume_request_v1,
    review_person_match_candidate_promotion_result_contract_reference_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1, ClientRpcRouteV1,
    DurableEnvelopeKindV1, EventRouteDirectionV1, EventRouteRequestV1,
    EventSubscriptionRequirementV1, ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1,
    ProvidedSurfaceKindV1, ProvidedSurfaceV1, RuntimeBudgetRequestV1, SettingsSchemaRefV1,
    SettingsSchemaV1, StorageNamespaceRequestV1, capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

use makosh_review_person_match_candidate_api::{
    REVIEW_PERSON_MATCH_CANDIDATE_CLIENT_CAPABILITY_ID_V1,
    REVIEW_PERSON_MATCH_CANDIDATE_DECIDE_CONNECT_PATH_V1,
    REVIEW_PERSON_MATCH_CANDIDATE_GET_CONNECT_PATH_V1,
    REVIEW_PERSON_MATCH_CANDIDATE_LIST_CONNECT_PATH_V1,
    review_person_match_candidate_client_decide_contract_reference_v1,
    review_person_match_candidate_client_get_contract_reference_v1,
    review_person_match_candidate_client_list_contract_reference_v1,
};

pub const REVIEW_PERSON_MATCH_CANDIDATE_STORAGE_CAPABILITY_ID_V1: &str =
    "review.person-match-candidate.storage.v1";
const STORAGE_CONNECTION_BUDGET_V1: u32 = 4;

#[must_use]
pub fn review_person_match_candidate_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn review_person_match_candidate_settings_schema_bytes_v1() -> Vec<u8> {
    review_person_match_candidate_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn review_person_match_candidate_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = review_person_match_candidate_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 2,
        module_id: REVIEW_PERSON_MATCH_CANDIDATE_MODULE_ID_V1.to_owned(),
        owner_id: REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1.to_owned(),
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
                "review.person-match-candidate.approved.publisher.v1",
                ProvidedSurfaceKindV1::DurablePublisher,
                review_person_match_candidate_approved_contract_reference_v1(),
                review_person_match_candidate_approved_publish_request_v1(),
            ),
            client_capability(),
            event_capability(
                "review.person-match-candidate.decision.consumer.v1",
                ProvidedSurfaceKindV1::DurableConsumer,
                review_person_match_candidate_decision_contract_reference_v1(),
                review_person_match_candidate_decision_consume_request_v1(),
            ),
            event_capability(
                "review.person-match-candidate.identity-resolution-candidate.consumer.v1",
                ProvidedSurfaceKindV1::DurableConsumer,
                identity_resolution_person_match_candidate_contract_reference_v1(),
                identity_resolution_candidate_consume_request(),
            ),
            event_capability(
                "review.person-match-candidate.promotion-result.consumer.v1",
                ProvidedSurfaceKindV1::DurableConsumer,
                review_person_match_candidate_promotion_result_contract_reference_v1(),
                review_person_match_candidate_promotion_result_consume_request_v1(),
            ),
            storage_capability(),
            event_capability(
                "review.person-match-candidate.submission-rejected.publisher.v1",
                ProvidedSurfaceKindV1::DurablePublisher,
                review_person_match_candidate_submission_rejected_contract_reference_v1(),
                review_person_match_candidate_submission_rejected_publish_request_v1(),
            ),
            event_capability(
                "review.person-match-candidate.submitted.publisher.v1",
                ProvidedSurfaceKindV1::DurablePublisher,
                review_person_match_candidate_submitted_contract_reference_v1(),
                review_person_match_candidate_submitted_publish_request_v1(),
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
        display_name: "Review Person Match Candidates".to_owned(),
    }
}

fn client_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: REVIEW_PERSON_MATCH_CANDIDATE_CLIENT_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: [
            (
                review_person_match_candidate_client_decide_contract_reference_v1(),
                REVIEW_PERSON_MATCH_CANDIDATE_DECIDE_CONNECT_PATH_V1,
            ),
            (
                review_person_match_candidate_client_get_contract_reference_v1(),
                REVIEW_PERSON_MATCH_CANDIDATE_GET_CONNECT_PATH_V1,
            ),
            (
                review_person_match_candidate_client_list_contract_reference_v1(),
                REVIEW_PERSON_MATCH_CANDIDATE_LIST_CONNECT_PATH_V1,
            ),
        ]
        .into_iter()
        .map(|(contract, path)| ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::ClientRpc as i32,
            contract: Some(contract),
            client_rpc_route: Some(ClientRpcRouteV1 {
                path: path.to_owned(),
            }),
            client_blob_route: None,
        })
        .collect(),
        ..Default::default()
    }
}

fn identity_resolution_candidate_consume_request() -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: DurableEnvelopeKindV1::Event as i32,
            contract: Some(identity_resolution_person_match_candidate_contract_reference_v1()),
            direction: EventRouteDirectionV1::Consume as i32,
            max_in_flight: 32,
            subscription_requirement: EventSubscriptionRequirementV1::Required as i32,
            max_deliver: 10,
            ack_wait_millis: 30_000,
        })),
    }
}

fn event_capability(
    capability_id: &str,
    kind: ProvidedSurfaceKindV1,
    contract: makosh_runtime_protocol::v1::ContractReferenceV1,
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
        capability_id: REVIEW_PERSON_MATCH_CANDIDATE_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1.to_owned(),
                connection_budget: STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: 5_000,
            })),
        }],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::validation::descriptor::{
        validate_descriptor_v1, validate_settings_schema_v1,
    };

    use super::*;

    #[test]
    fn descriptor_is_review_owned_with_exact_client_routes() {
        let descriptor = review_person_match_candidate_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&review_person_match_candidate_settings_schema_v1())
            .expect("settings");
        assert_eq!(descriptor.owner_id, "review");
        assert_eq!(descriptor.capabilities.len(), 8);
        let client = descriptor
            .capabilities
            .iter()
            .find(|capability| {
                capability.capability_id == REVIEW_PERSON_MATCH_CANDIDATE_CLIENT_CAPABILITY_ID_V1
            })
            .expect("client");
        assert_eq!(client.provides.len(), 3);
    }
}
