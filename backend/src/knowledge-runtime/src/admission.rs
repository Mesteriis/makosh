use makosh_knowledge_command_api::{
    KNOWLEDGE_MODULE_ID_V1, KNOWLEDGE_OWNER_ID_V1,
    KNOWLEDGE_REVIEWED_CANDIDATE_BLOB_CAPABILITY_ID_V1,
    KNOWLEDGE_REVIEWED_CANDIDATE_COMMAND_CAPABILITY_ID_V1,
    create_knowledge_note_from_reviewed_candidate_consume_request_v1,
    create_knowledge_note_from_reviewed_candidate_contract_reference_v1,
    knowledge_note_created_from_reviewed_candidate_contract_reference_v1,
    knowledge_note_created_from_reviewed_candidate_publish_request_v1,
    knowledge_note_creation_from_reviewed_candidate_rejected_contract_reference_v1,
    knowledge_note_creation_from_reviewed_candidate_rejected_publish_request_v1,
};
use makosh_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ContractReferenceV1, ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1,
    ProvidedSurfaceKindV1, ProvidedSurfaceV1, RuntimeBudgetRequestV1, SettingsSchemaRefV1,
    SettingsSchemaV1, StorageNamespaceRequestV1, capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub const KNOWLEDGE_STORAGE_CAPABILITY_ID_V1: &str = "knowledge.storage.v1";
const STORAGE_CONNECTION_BUDGET_V1: u32 = 4;

#[must_use]
pub fn knowledge_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn knowledge_settings_schema_bytes_v1() -> Vec<u8> {
    knowledge_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn knowledge_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = knowledge_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: KNOWLEDGE_MODULE_ID_V1.to_owned(),
        owner_id: KNOWLEDGE_OWNER_ID_V1.to_owned(),
        module_kind: ModuleKindV1::Domain as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: vec![
            blob_capability(),
            event_capability(
                KNOWLEDGE_REVIEWED_CANDIDATE_COMMAND_CAPABILITY_ID_V1,
                ProvidedSurfaceKindV1::DurableConsumer,
                create_knowledge_note_from_reviewed_candidate_contract_reference_v1(),
                create_knowledge_note_from_reviewed_candidate_consume_request_v1(),
            ),
            event_capability(
                "knowledge.reviewed-candidate.created.publisher.v1",
                ProvidedSurfaceKindV1::DurablePublisher,
                knowledge_note_created_from_reviewed_candidate_contract_reference_v1(),
                knowledge_note_created_from_reviewed_candidate_publish_request_v1(),
            ),
            event_capability(
                "knowledge.reviewed-candidate.rejected.publisher.v1",
                ProvidedSurfaceKindV1::DurablePublisher,
                knowledge_note_creation_from_reviewed_candidate_rejected_contract_reference_v1(),
                knowledge_note_creation_from_reviewed_candidate_rejected_publish_request_v1(),
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
        display_name: "Knowledge".to_owned(),
    }
}

fn blob_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: KNOWLEDGE_REVIEWED_CANDIDATE_BLOB_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes:
                    makosh_knowledge_command_api::KNOWLEDGE_REVIEWED_CANDIDATE_MAX_BLOB_BYTES_V1,
                custody_scope_id: KNOWLEDGE_OWNER_ID_V1.to_owned(),
                allowed_operations: vec![
                    BlobQuotaOperationV1::ReadRange as i32,
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
        capability_id: KNOWLEDGE_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: KNOWLEDGE_OWNER_ID_V1.to_owned(),
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
    fn descriptor_is_a_knowledge_domain_without_foreign_owner_capabilities() {
        let descriptor = knowledge_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&knowledge_settings_schema_v1()).expect("settings");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Domain as i32);
        assert_eq!(descriptor.owner_id, "knowledge");
        assert_eq!(descriptor.capabilities.len(), 5);
        assert!(descriptor.capabilities.iter().all(|capability| {
            !capability.capability_id.contains("review")
                || capability
                    .capability_id
                    .starts_with("knowledge.reviewed-candidate")
        }));
    }
}
