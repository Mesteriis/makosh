use makosh_calendar_api::{
    CALENDAR_CLIENT_CAPABILITY_ID_V1, CALENDAR_LIFECYCLE_EVENT_CAPABILITY_ID_V1,
    CALENDAR_MODULE_ID_V1, CALENDAR_OWNER_ID_V1, CALENDAR_SCHEDULER_DUE_CAPABILITY_ID_V1,
    CALENDAR_SCHEDULER_RECEIPT_CAPABILITY_ID_V1,
    CALENDAR_SCHEDULER_SCHEDULE_COMMAND_CAPABILITY_ID_V1,
    CALENDAR_SCHEDULER_SCHEDULE_RESULT_CAPABILITY_ID_V1, CALENDAR_STORAGE_CAPABILITY_ID_V1,
    calendar_client_routes_v1, calendar_lifecycle_event_contract_reference_v1,
    calendar_lifecycle_event_publish_request_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1, ClientRpcRouteV1,
    ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1, EventRouteRequestV1,
    EventSubscriptionRequirementV1, ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1,
    ProvidedSurfaceKindV1, ProvidedSurfaceV1, RuntimeBudgetRequestV1, SchedulerJobRequestV1,
    SettingsSchemaRefV1, SettingsSchemaV1, StorageNamespaceRequestV1,
    capability_request_v1::Request,
};
use makosh_scheduler_protocol::SCHEDULER_JOB_DESCRIPTOR_SET_V1;
use prost::Message;
use sha2::{Digest, Sha256};

const STORAGE_CONNECTION_BUDGET_V1: u32 = 4;
pub(crate) const CALENDAR_RUNTIME_CAPABILITY_IDS_V1: [&str; 7] = [
    "calendar.client.v1",
    "calendar.lifecycle.event.v1",
    "calendar.scheduler.due.v1",
    "calendar.scheduler.receipt.v1",
    "calendar.scheduler.schedule-command.v1",
    "calendar.scheduler.schedule-result.v1",
    "calendar.storage.v1",
];

#[must_use]
pub fn calendar_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn calendar_settings_schema_bytes_v1() -> Vec<u8> {
    calendar_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn calendar_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = calendar_settings_schema_bytes_v1();
    let mut capabilities = vec![
        client_capability(),
        event_capability(
            CALENDAR_LIFECYCLE_EVENT_CAPABILITY_ID_V1,
            ProvidedSurfaceKindV1::DurablePublisher,
            calendar_lifecycle_event_contract_reference_v1(),
            vec![calendar_lifecycle_event_publish_request_v1()],
        ),
        scheduler_due_capability(),
        scheduler_receipt_capability(),
        scheduler_schedule_command_capability(),
        scheduler_schedule_result_capability(),
        storage_capability(),
    ];
    capabilities.sort_by(|left, right| left.capability_id.cmp(&right.capability_id));
    debug_assert_eq!(
        capabilities
            .iter()
            .map(|value| value.capability_id.as_str())
            .collect::<Vec<_>>(),
        CALENDAR_RUNTIME_CAPABILITY_IDS_V1,
    );
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: CALENDAR_MODULE_ID_V1.to_owned(),
        owner_id: CALENDAR_OWNER_ID_V1.to_owned(),
        module_kind: ModuleKindV1::Domain as i32,
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
            max_connections: STORAGE_CONNECTION_BUDGET_V1,
            max_memory_bytes: 64 * 1024 * 1024,
            max_cpu_millis: 500,
        }),
        display_name: "Calendar".to_owned(),
    }
}

#[must_use]
pub fn scheduler_job_contract_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: CALENDAR_OWNER_ID_V1.to_owned(),
        name: "reminder_due".to_owned(),
        major: 1,
        revision: 1,
        schema_sha256: Sha256::digest(SCHEDULER_JOB_DESCRIPTOR_SET_V1).to_vec(),
    }
}

#[must_use]
pub fn scheduler_receipt_contract_v1() -> ContractReferenceV1 {
    scheduler_contract("job_receipt")
}

#[must_use]
pub fn scheduler_schedule_control_contract_v1() -> ContractReferenceV1 {
    scheduler_contract("schedule_control")
}

fn scheduler_contract(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: "scheduler".to_owned(),
        name: name.to_owned(),
        major: 1,
        revision: 1,
        schema_sha256: Sha256::digest(SCHEDULER_JOB_DESCRIPTOR_SET_V1).to_vec(),
    }
}

fn client_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: CALENDAR_CLIENT_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: calendar_client_routes_v1()
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

fn scheduler_due_capability() -> CapabilityDescriptorV1 {
    let contract = scheduler_job_contract_v1();
    let mut capability = event_capability(
        CALENDAR_SCHEDULER_DUE_CAPABILITY_ID_V1,
        ProvidedSurfaceKindV1::DurableConsumer,
        contract.clone(),
        vec![event_request(
            DurableEnvelopeKindV1::Command,
            contract.clone(),
            EventRouteDirectionV1::Consume,
        )],
    );
    capability.requests.insert(
        0,
        CapabilityRequestV1 {
            request: Some(Request::SchedulerJob(SchedulerJobRequestV1 {
                job_kind: Some(contract),
            })),
        },
    );
    capability
}

fn scheduler_receipt_capability() -> CapabilityDescriptorV1 {
    let contract = scheduler_receipt_contract_v1();
    event_capability(
        CALENDAR_SCHEDULER_RECEIPT_CAPABILITY_ID_V1,
        ProvidedSurfaceKindV1::DurablePublisher,
        contract.clone(),
        vec![
            event_request(
                DurableEnvelopeKindV1::Ack,
                contract.clone(),
                EventRouteDirectionV1::Publish,
            ),
            event_request(
                DurableEnvelopeKindV1::Result,
                contract,
                EventRouteDirectionV1::Publish,
            ),
        ],
    )
}

fn scheduler_schedule_command_capability() -> CapabilityDescriptorV1 {
    let contract = scheduler_schedule_control_contract_v1();
    event_capability(
        CALENDAR_SCHEDULER_SCHEDULE_COMMAND_CAPABILITY_ID_V1,
        ProvidedSurfaceKindV1::DurablePublisher,
        contract.clone(),
        vec![event_request(
            DurableEnvelopeKindV1::Command,
            contract,
            EventRouteDirectionV1::Publish,
        )],
    )
}

fn scheduler_schedule_result_capability() -> CapabilityDescriptorV1 {
    let contract = scheduler_schedule_control_contract_v1();
    event_capability(
        CALENDAR_SCHEDULER_SCHEDULE_RESULT_CAPABILITY_ID_V1,
        ProvidedSurfaceKindV1::DurableConsumer,
        contract.clone(),
        vec![event_request(
            DurableEnvelopeKindV1::Result,
            contract,
            EventRouteDirectionV1::Consume,
        )],
    )
}

fn storage_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: CALENDAR_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: CALENDAR_OWNER_ID_V1.to_owned(),
                connection_budget: STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: 5_000,
            })),
        }],
        ..Default::default()
    }
}

fn event_capability(
    capability_id: &str,
    kind: ProvidedSurfaceKindV1,
    contract: ContractReferenceV1,
    requests: Vec<CapabilityRequestV1>,
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
        requests,
        ..Default::default()
    }
}

fn event_request(
    kind: DurableEnvelopeKindV1,
    contract: ContractReferenceV1,
    direction: EventRouteDirectionV1,
) -> CapabilityRequestV1 {
    let consumes = direction == EventRouteDirectionV1::Consume;
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: kind as i32,
            contract: Some(contract),
            direction: direction as i32,
            max_in_flight: if consumes { 16 } else { 32 },
            subscription_requirement: if consumes {
                EventSubscriptionRequirementV1::Required as i32
            } else {
                EventSubscriptionRequirementV1::Unspecified as i32
            },
            max_deliver: if consumes { 10 } else { 0 },
            ack_wait_millis: if consumes { 30_000 } else { 0 },
        })),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use makosh_runtime_protocol::validation::descriptor::{
        validate_descriptor_v1, validate_settings_schema_v1,
    };

    use super::*;

    #[test]
    fn descriptor_is_exact_calendar_domain_and_scheduler_owned() {
        let descriptor = calendar_module_descriptor_v1("test");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&calendar_settings_schema_v1()).expect("settings");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Domain as i32);
        assert_eq!(descriptor.owner_id, CALENDAR_OWNER_ID_V1);
        assert_eq!(descriptor.capabilities.len(), 7);
        let ids = descriptor
            .capabilities
            .iter()
            .map(|value| value.capability_id.as_str())
            .collect::<BTreeSet<_>>();
        for expected in [
            CALENDAR_CLIENT_CAPABILITY_ID_V1,
            CALENDAR_LIFECYCLE_EVENT_CAPABILITY_ID_V1,
            CALENDAR_SCHEDULER_DUE_CAPABILITY_ID_V1,
            CALENDAR_SCHEDULER_RECEIPT_CAPABILITY_ID_V1,
            CALENDAR_SCHEDULER_SCHEDULE_COMMAND_CAPABILITY_ID_V1,
            CALENDAR_SCHEDULER_SCHEDULE_RESULT_CAPABILITY_ID_V1,
            CALENDAR_STORAGE_CAPABILITY_ID_V1,
        ] {
            assert!(ids.contains(expected), "{expected}");
        }
        let client = descriptor
            .capabilities
            .iter()
            .find(|value| value.capability_id == CALENDAR_CLIENT_CAPABILITY_ID_V1)
            .expect("client");
        assert_eq!(client.provides.len(), 16);
    }
}
