use makosh_consistency_api::{
    CONSISTENCY_CLAIM_CAPABILITY_ID_V1, CONSISTENCY_CLIENT_CAPABILITY_ID_V1,
    CONSISTENCY_MODULE_ID_V1, CONSISTENCY_OWNER_ID_V1, CONSISTENCY_STORAGE_CAPABILITY_ID_V1,
    consistency_client_routes_v1,
};
use makosh_persons_api::{
    persons_owner_event_contract_reference_v1, persons_owner_event_publish_request_v1,
};
use makosh_relationships_api::{
    relationships_lifecycle_event_contract_reference_v1,
    relationships_lifecycle_event_publish_request_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1, ClientRpcRouteV1,
    EventRouteDirectionV1, EventSubscriptionRequirementV1, ModuleDescriptorV1, ModuleKindV1,
    ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1, RuntimeBudgetRequestV1,
    SettingsSchemaRefV1, SettingsSchemaV1, StorageNamespaceRequestV1,
    capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};
#[must_use]
pub fn consistency_settings_schema_bytes_v1() -> Vec<u8> {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
    .encode_to_vec()
}
#[must_use]
pub fn consistency_module_descriptor_v1(build: &str) -> ModuleDescriptorV1 {
    let settings = consistency_settings_schema_bytes_v1();
    let mut capabilities = vec![client(), projection(), storage()];
    capabilities.sort_by(|a, b| a.capability_id.cmp(&b.capability_id));
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: CONSISTENCY_MODULE_ID_V1.into(),
        owner_id: CONSISTENCY_OWNER_ID_V1.into(),
        module_kind: ModuleKindV1::Engine as i32,
        module_version: "1".into(),
        build_id: build.into(),
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
            max_connections: 4,
            max_memory_bytes: 64 * 1024 * 1024,
            max_cpu_millis: 500,
        }),
        display_name: "Consistency".into(),
    }
}
fn client() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: CONSISTENCY_CLIENT_CAPABILITY_ID_V1.into(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: consistency_client_routes_v1()
            .into_iter()
            .map(|(contract, path)| ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRpc as i32,
                contract: Some(contract),
                client_rpc_route: Some(ClientRpcRouteV1 { path: path.into() }),
                client_blob_route: None,
            })
            .collect(),
        ..Default::default()
    }
}
fn projection() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: CONSISTENCY_CLAIM_CAPABILITY_ID_V1.into(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            persons_owner_event_contract_reference_v1(),
            relationships_lifecycle_event_contract_reference_v1(),
        ]
        .into_iter()
        .map(|contract| ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::DurableConsumer as i32,
            contract: Some(contract),
            client_rpc_route: None,
            client_blob_route: None,
        })
        .collect(),
        requests: vec![
            consume(persons_owner_event_publish_request_v1()),
            consume(relationships_lifecycle_event_publish_request_v1()),
        ],
        ..Default::default()
    }
}
fn consume(mut request: CapabilityRequestV1) -> CapabilityRequestV1 {
    if let Some(Request::EventRoute(route)) = &mut request.request {
        route.direction = EventRouteDirectionV1::Consume as i32;
        route.subscription_requirement = EventSubscriptionRequirementV1::Required as i32;
        route.max_deliver = 10;
        route.ack_wait_millis = 30_000
    }
    request
}
fn storage() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: CONSISTENCY_STORAGE_CAPABILITY_ID_V1.into(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: CONSISTENCY_OWNER_ID_V1.into(),
                connection_budget: 4,
                timeout_millis: 5000,
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
    fn descriptor_is_three_capability_read_only_projection() {
        let value = consistency_module_descriptor_v1("test");
        validate_descriptor_v1(&value).unwrap();
        assert_eq!(value.capabilities.len(), 3);
        assert!(
            value
                .capabilities
                .iter()
                .flat_map(|c| &c.provides)
                .all(|s| s.kind != ProvidedSurfaceKindV1::DurablePublisher as i32)
        );
    }
}
