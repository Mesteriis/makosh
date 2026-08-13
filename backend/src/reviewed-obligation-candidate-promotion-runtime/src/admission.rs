use makosh_obligations_api::{
    create_obligation_from_reviewed_candidate_contract_reference_v1,
    create_obligation_from_reviewed_candidate_publish_request_v1,
    obligation_created_from_reviewed_candidate_consume_request_v1,
    obligation_created_from_reviewed_candidate_contract_reference_v1,
    obligation_creation_from_reviewed_candidate_rejected_consume_request_v1,
    obligation_creation_from_reviewed_candidate_rejected_contract_reference_v1,
};
use makosh_review_obligation_candidate_api::{
    review_obligation_candidate_approved_consume_request_v1,
    review_obligation_candidate_approved_contract_reference_v1,
};
use makosh_review_obligation_candidate_promotion_api::{
    review_obligation_candidate_promotion_result_contract_reference_v1,
    review_obligation_candidate_promotion_result_publish_request_v1,
};
use makosh_reviewed_obligation_candidate_promotion_core::{
    REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_MODULE_ID_V1,
    REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_OWNER_V1,
};
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1, ContractReferenceV1,
    ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1,
    RuntimeBudgetRequestV1, SettingsSchemaRefV1, SettingsSchemaV1, StorageNamespaceRequestV1,
    capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub const REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_STORAGE_CAPABILITY_ID_V1: &str =
    "reviewed_obligation_candidate_promotion.storage.v1";
const STORAGE_CONNECTION_BUDGET_V1: u32 = 4;

#[must_use]
pub fn reviewed_obligation_candidate_promotion_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn reviewed_obligation_candidate_promotion_settings_schema_bytes_v1() -> Vec<u8> {
    reviewed_obligation_candidate_promotion_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn reviewed_obligation_candidate_promotion_module_descriptor_v1(
    build_id: &str,
) -> ModuleDescriptorV1 {
    let settings = reviewed_obligation_candidate_promotion_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_MODULE_ID_V1.to_owned(),
        owner_id: REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_OWNER_V1.to_owned(),
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
                "reviewed_obligation_candidate_promotion.obligations-command.publish.v1",
                ProvidedSurfaceKindV1::DurablePublisher,
                create_obligation_from_reviewed_candidate_contract_reference_v1(),
                create_obligation_from_reviewed_candidate_publish_request_v1(),
            ),
            event_capability(
                "reviewed_obligation_candidate_promotion.obligations-created.consume.v1",
                ProvidedSurfaceKindV1::DurableConsumer,
                obligation_created_from_reviewed_candidate_contract_reference_v1(),
                obligation_created_from_reviewed_candidate_consume_request_v1(),
            ),
            event_capability(
                "reviewed_obligation_candidate_promotion.obligations-rejected.consume.v1",
                ProvidedSurfaceKindV1::DurableConsumer,
                obligation_creation_from_reviewed_candidate_rejected_contract_reference_v1(),
                obligation_creation_from_reviewed_candidate_rejected_consume_request_v1(),
            ),
            event_capability(
                "reviewed_obligation_candidate_promotion.review-approved.consume.v1",
                ProvidedSurfaceKindV1::DurableConsumer,
                review_obligation_candidate_approved_contract_reference_v1(),
                review_obligation_candidate_approved_consume_request_v1(),
            ),
            event_capability(
                "reviewed_obligation_candidate_promotion.review-result.publish.v1",
                ProvidedSurfaceKindV1::DurablePublisher,
                review_obligation_candidate_promotion_result_contract_reference_v1(),
                review_obligation_candidate_promotion_result_publish_request_v1(),
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
        display_name: "Reviewed Obligation Candidate Promotion".to_owned(),
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
        capability_id: REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_OWNER_V1.to_owned(),
                connection_budget: STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: 5_000,
            })),
        }],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::{
        v1::ProvidedSurfaceKindV1,
        validation::descriptor::{validate_descriptor_v1, validate_settings_schema_v1},
    };

    use super::*;

    #[test]
    fn descriptor_is_event_storage_only_workflow() {
        let descriptor = reviewed_obligation_candidate_promotion_module_descriptor_v1("build-1");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&reviewed_obligation_candidate_promotion_settings_schema_v1())
            .expect("settings");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Workflow as i32);
        assert_eq!(
            descriptor.owner_id,
            REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_OWNER_V1
        );
        assert_eq!(descriptor.capabilities.len(), 6);
        assert!(
            descriptor
                .capabilities
                .iter()
                .flat_map(|value| &value.provides)
                .all(|surface| {
                    matches!(
                        ProvidedSurfaceKindV1::try_from(surface.kind),
                        Ok(ProvidedSurfaceKindV1::DurableConsumer
                            | ProvidedSurfaceKindV1::DurablePublisher)
                    )
                })
        );
        assert!(descriptor.capabilities.iter().all(|value| {
            value
                .requests
                .iter()
                .all(|request| !matches!(request.request, Some(Request::BlobQuota(_))))
        }));
    }
}
