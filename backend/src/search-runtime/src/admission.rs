use makosh_calendar_api::{
    calendar_lifecycle_event_contract_reference_v1, calendar_lifecycle_event_publish_request_v1,
};
use makosh_decisions_api::{
    decisions_lifecycle_event_contract_reference_v1, decisions_lifecycle_event_publish_request_v1,
};
use makosh_documents_api::{
    documents_lifecycle_event_contract_reference_v1, documents_lifecycle_event_publish_request_v1,
};
use makosh_knowledge_command_api::{
    knowledge_lifecycle_event_contract_reference_v1, knowledge_lifecycle_event_publish_request_v1,
};
use makosh_obligations_api::{
    obligations_lifecycle_event_contract_reference_v1,
    obligations_lifecycle_event_publish_request_v1,
};
use makosh_organizations_api::{
    organizations_lifecycle_event_contract_reference_v1,
    organizations_lifecycle_event_publish_request_v1,
};
use makosh_persons_api::{
    persons_owner_event_contract_reference_v1, persons_owner_event_publish_request_v1,
};
use makosh_projects_api::{
    projects_lifecycle_event_contract_reference_v1, projects_lifecycle_event_publish_request_v1,
};
use makosh_relationships_api::{
    relationships_lifecycle_event_contract_reference_v1,
    relationships_lifecycle_event_publish_request_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1, ClientRpcRouteV1,
    EventRouteDirectionV1, EventSubscriptionRequirementV1, ModuleDescriptorV1, ModuleKindV1,
    ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1, RuntimeBudgetRequestV1,
    SettingsSchemaRefV1, SettingsSchemaV1, StorageNamespaceRequestV1, VaultActionV1,
    VaultPurposeRequestV1, VaultSecretClassV1, VaultTargetScopeV1, capability_request_v1::Request,
};
use makosh_search_api::{
    SEARCH_CLIENT_CAPABILITY_ID_V1, SEARCH_MODULE_ID_V1, SEARCH_OWNER_ID_V1,
    SEARCH_PROJECTION_CAPABILITY_ID_V1, SEARCH_STORAGE_CAPABILITY_ID_V1, search_client_routes_v1,
};
use makosh_tasks_command_api::{
    tasks_lifecycle_event_contract_reference_v1, tasks_lifecycle_event_publish_request_v1,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub const SEARCH_OWNER_KEY_PURPOSE_ID_V1: &str = "search.global.index";
pub const SEARCH_OWNER_KEY_SCHEMA_REVISION_V1: u32 = 1;
pub const SEARCH_OWNER_KEY_TTL_SECONDS_V1: u32 = 300;
const STORAGE_CONNECTIONS: u32 = 4;

#[must_use]
pub fn search_settings_schema_bytes_v1() -> Vec<u8> {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
    .encode_to_vec()
}

#[must_use]
pub fn search_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = search_settings_schema_bytes_v1();
    let mut capabilities = vec![
        client_capability(),
        projection_capability(),
        storage_capability(),
    ];
    capabilities.sort_by(|left, right| left.capability_id.cmp(&right.capability_id));
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: SEARCH_MODULE_ID_V1.to_owned(),
        owner_id: SEARCH_OWNER_ID_V1.to_owned(),
        module_kind: ModuleKindV1::Engine as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities,
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: settings.len() as u64,
            sha256: Sha256::digest(&settings).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: STORAGE_CONNECTIONS,
            max_memory_bytes: 64 * 1024 * 1024,
            max_cpu_millis: 500,
        }),
        display_name: "Search".to_owned(),
    }
}

fn client_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: SEARCH_CLIENT_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: search_client_routes_v1()
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

fn projection_capability() -> CapabilityDescriptorV1 {
    let contracts = vec![
        persons_owner_event_contract_reference_v1(),
        organizations_lifecycle_event_contract_reference_v1(),
        relationships_lifecycle_event_contract_reference_v1(),
        projects_lifecycle_event_contract_reference_v1(),
        tasks_lifecycle_event_contract_reference_v1(),
        obligations_lifecycle_event_contract_reference_v1(),
        decisions_lifecycle_event_contract_reference_v1(),
        calendar_lifecycle_event_contract_reference_v1(),
        documents_lifecycle_event_contract_reference_v1(),
        knowledge_lifecycle_event_contract_reference_v1(),
    ];
    let mut requests = vec![
        consume(persons_owner_event_publish_request_v1()),
        consume(organizations_lifecycle_event_publish_request_v1()),
        consume(relationships_lifecycle_event_publish_request_v1()),
        consume(projects_lifecycle_event_publish_request_v1()),
        consume(tasks_lifecycle_event_publish_request_v1()),
        consume(obligations_lifecycle_event_publish_request_v1()),
        consume(decisions_lifecycle_event_publish_request_v1()),
        consume(calendar_lifecycle_event_publish_request_v1()),
        consume(documents_lifecycle_event_publish_request_v1()),
        consume(knowledge_lifecycle_event_publish_request_v1()),
        CapabilityRequestV1 {
            request: Some(Request::VaultPurpose(VaultPurposeRequestV1 {
                purpose_id: SEARCH_OWNER_KEY_PURPOSE_ID_V1.to_owned(),
                requested_lease_ttl_seconds: SEARCH_OWNER_KEY_TTL_SECONDS_V1,
                allowed_secret_classes: vec![VaultSecretClassV1::OwnerDerivedKey as i32],
                actions: vec![VaultActionV1::IssueOwnerDerivedKey as i32],
                target_scope: VaultTargetScopeV1::OwnerDerivedProjectionKey as i32,
                key_schema_revision: SEARCH_OWNER_KEY_SCHEMA_REVISION_V1,
            })),
        },
    ];
    requests.sort_by(|left, right| format!("{left:?}").cmp(&format!("{right:?}")));
    CapabilityDescriptorV1 {
        capability_id: SEARCH_PROJECTION_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: contracts
            .into_iter()
            .map(|contract| ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurableConsumer as i32,
                contract: Some(contract),
                client_rpc_route: None,
                client_blob_route: None,
            })
            .collect(),
        requests,
        ..Default::default()
    }
}

fn consume(mut request: CapabilityRequestV1) -> CapabilityRequestV1 {
    if let Some(Request::EventRoute(route)) = &mut request.request {
        route.direction = EventRouteDirectionV1::Consume as i32;
        route.subscription_requirement = EventSubscriptionRequirementV1::Required as i32;
        route.max_deliver = 10;
        route.ack_wait_millis = 30_000;
    }
    request
}

fn storage_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: SEARCH_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: SEARCH_OWNER_ID_V1.to_owned(),
                connection_budget: STORAGE_CONNECTIONS,
                timeout_millis: 5_000,
            })),
        }],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_runtime_protocol::validation::descriptor::validate_descriptor_v1;

    #[test]
    fn descriptor_is_read_only_projection_engine_with_exact_three_capabilities() {
        let descriptor = search_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Engine as i32);
        assert_eq!(descriptor.capabilities.len(), 3);
        assert!(
            descriptor
                .capabilities
                .iter()
                .flat_map(|value| &value.provides)
                .all(|surface| { surface.kind != ProvidedSurfaceKindV1::DurablePublisher as i32 })
        );
    }
}
