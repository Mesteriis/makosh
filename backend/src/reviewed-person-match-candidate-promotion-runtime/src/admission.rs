use makosh_persons_api::{
    persons_command_contract_reference_v1, persons_command_rejected_contract_reference_v1,
    persons_command_succeeded_contract_reference_v1,
};
use makosh_review_person_match_candidate_api::{
    review_person_match_candidate_approved_consume_request_v1,
    review_person_match_candidate_approved_contract_reference_v1,
};
use makosh_review_person_match_candidate_promotion_api::{
    review_person_match_candidate_promotion_result_contract_reference_v1,
    review_person_match_candidate_promotion_result_publish_request_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1, DurableEnvelopeKindV1,
    EventRouteDirectionV1, EventRouteRequestV1, EventSubscriptionRequirementV1, ModuleDescriptorV1,
    ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
    RuntimeBudgetRequestV1, SettingsSchemaRefV1, SettingsSchemaV1, StorageNamespaceRequestV1,
    capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub const REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_MODULE_ID_V1: &str =
    "makosh-reviewed-person-match-candidate-promotion-runtime";
pub const REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1: &str =
    "reviewed_person_match_candidate_promotion";
pub const REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_STORAGE_CAPABILITY_ID_V1: &str =
    "reviewed-person-match-candidate-promotion.storage.v1";
const STORAGE_CONNECTION_BUDGET_V1: u32 = 4;

#[must_use]
pub fn reviewed_person_match_candidate_promotion_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn reviewed_person_match_candidate_promotion_settings_schema_bytes_v1() -> Vec<u8> {
    reviewed_person_match_candidate_promotion_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn reviewed_person_match_candidate_promotion_module_descriptor_v1(
    build_id: &str,
) -> ModuleDescriptorV1 {
    let settings = reviewed_person_match_candidate_promotion_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_MODULE_ID_V1.to_owned(),
        owner_id: REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1.to_owned(),
        module_kind: ModuleKindV1::Workflow as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: vec![
            event_capability(
                "reviewed-person-match-candidate-promotion.approval.consumer.v1",
                ProvidedSurfaceKindV1::DurableConsumer,
                review_person_match_candidate_approved_contract_reference_v1(),
                review_person_match_candidate_approved_consume_request_v1(),
            ),
            event_capability(
                "reviewed-person-match-candidate-promotion.persons-command.publisher.v1",
                ProvidedSurfaceKindV1::DurablePublisher,
                persons_command_contract_reference_v1(),
                event_request(
                    DurableEnvelopeKindV1::Command,
                    persons_command_contract_reference_v1(),
                    EventRouteDirectionV1::Publish,
                ),
            ),
            event_capability(
                "reviewed-person-match-candidate-promotion.persons-rejected.consumer.v1",
                ProvidedSurfaceKindV1::DurableConsumer,
                persons_command_rejected_contract_reference_v1(),
                event_request(
                    DurableEnvelopeKindV1::Result,
                    persons_command_rejected_contract_reference_v1(),
                    EventRouteDirectionV1::Consume,
                ),
            ),
            event_capability(
                "reviewed-person-match-candidate-promotion.persons-succeeded.consumer.v1",
                ProvidedSurfaceKindV1::DurableConsumer,
                persons_command_succeeded_contract_reference_v1(),
                event_request(
                    DurableEnvelopeKindV1::Result,
                    persons_command_succeeded_contract_reference_v1(),
                    EventRouteDirectionV1::Consume,
                ),
            ),
            event_capability(
                "reviewed-person-match-candidate-promotion.result.publisher.v1",
                ProvidedSurfaceKindV1::DurablePublisher,
                review_person_match_candidate_promotion_result_contract_reference_v1(),
                review_person_match_candidate_promotion_result_publish_request_v1(),
            ),
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
        display_name: "Reviewed Person Match Candidate Promotion".to_owned(),
    }
}

fn event_request(
    kind: DurableEnvelopeKindV1,
    contract: makosh_runtime_protocol::v1::ContractReferenceV1,
    direction: EventRouteDirectionV1,
) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: kind as i32,
            contract: Some(contract),
            direction: direction as i32,
            max_in_flight: 32,
            subscription_requirement: if direction == EventRouteDirectionV1::Consume {
                EventSubscriptionRequirementV1::Required as i32
            } else {
                EventSubscriptionRequirementV1::Unspecified as i32
            },
            max_deliver: if direction == EventRouteDirectionV1::Consume {
                10
            } else {
                0
            },
            ack_wait_millis: if direction == EventRouteDirectionV1::Consume {
                30_000
            } else {
                0
            },
        })),
    }
}

fn event_capability(
    id: &str,
    kind: ProvidedSurfaceKindV1,
    contract: makosh_runtime_protocol::v1::ContractReferenceV1,
    request: CapabilityRequestV1,
) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: id.to_owned(),
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
        capability_id: REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_STORAGE_CAPABILITY_ID_V1
            .to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1.to_owned(),
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
    fn descriptor_is_unsigned_workflow_event_storage_only() {
        let descriptor = reviewed_person_match_candidate_promotion_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(
            &reviewed_person_match_candidate_promotion_settings_schema_v1(),
        )
        .expect("settings");
        assert_eq!(
            descriptor.owner_id,
            REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1
        );
        assert_eq!(descriptor.capabilities.len(), 6);
    }
}
