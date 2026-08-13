use makosh_mail_address_book_contract::MailPersonSourceContractV1;
use makosh_mail_persons_sync_api::{MAIL_PERSONS_SYNC_OWNER_V1, MailPersonsSyncContractV1};
use makosh_persons_api::{
    persons_command_contract_reference_v1, persons_command_rejected_contract_reference_v1,
    persons_command_succeeded_contract_reference_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1, ContractReferenceV1,
    DurableEnvelopeKindV1, EventRouteDirectionV1, EventRouteRequestV1,
    EventSubscriptionRequirementV1, ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1,
    ProvidedSurfaceKindV1, ProvidedSurfaceV1, RuntimeBudgetRequestV1, SchedulerJobRequestV1,
    SettingsSchemaRefV1, SettingsSchemaV1, StorageNamespaceRequestV1,
    capability_request_v1::Request,
};
use makosh_scheduler_protocol::SCHEDULER_JOB_DESCRIPTOR_SET_V1;
use prost::Message;
use sha2::{Digest, Sha256};

pub const MAIL_PERSONS_SYNC_MODULE_ID_V1: &str = "makosh-mail-persons-sync-runtime";
pub const MAIL_PERSONS_SYNC_STORAGE_CAPABILITY_ID_V1: &str = "mail_persons_sync.storage.v1";
const STORAGE_CONNECTION_BUDGET_V1: u32 = 4;

#[must_use]
pub fn mail_persons_sync_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn mail_persons_sync_settings_schema_bytes_v1() -> Vec<u8> {
    mail_persons_sync_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn mail_persons_sync_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = mail_persons_sync_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 2,
        module_id: MAIL_PERSONS_SYNC_MODULE_ID_V1.to_owned(),
        owner_id: MAIL_PERSONS_SYNC_OWNER_V1.to_owned(),
        module_kind: ModuleKindV1::Workflow as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: capabilities(),
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
        display_name: "Mail Persons Sync".to_owned(),
    }
}

fn capabilities() -> Vec<CapabilityDescriptorV1> {
    vec![
        mail_consumer(
            "mail_persons_sync.mail.account-ready.v1",
            MailPersonSourceContractV1::AccountReady,
        ),
        mail_consumer(
            "mail_persons_sync.mail.account-retired.v1",
            MailPersonSourceContractV1::AccountRetired,
        ),
        event_capability(
            "mail_persons_sync.mail.fetch-page.v1",
            ProvidedSurfaceKindV1::DurablePublisher,
            MailPersonSourceContractV1::FetchPageCommand.reference(),
            MailPersonSourceContractV1::FetchPageCommand.publish_request(),
        ),
        mail_consumer(
            "mail_persons_sync.mail.page-completed.v1",
            MailPersonSourceContractV1::PageCompleted,
        ),
        mail_consumer(
            "mail_persons_sync.mail.page-rejected.v1",
            MailPersonSourceContractV1::PageRejected,
        ),
        mail_consumer(
            "mail_persons_sync.mail.source-observed.v1",
            MailPersonSourceContractV1::SourceObserved,
        ),
        mail_consumer(
            "mail_persons_sync.mail.source-removed.v1",
            MailPersonSourceContractV1::SourceRemoved,
        ),
        mail_consumer(
            "mail_persons_sync.mail.source-updated.v1",
            MailPersonSourceContractV1::SourceUpdated,
        ),
        workflow_publisher(
            "mail_persons_sync.page-receipt.v1",
            MailPersonsSyncContractV1::PageReceipt,
        ),
        event_capability(
            "mail_persons_sync.persons.command-rejected.v1",
            ProvidedSurfaceKindV1::DurableConsumer,
            persons_command_rejected_contract_reference_v1(),
            event_request(
                DurableEnvelopeKindV1::Result,
                persons_command_rejected_contract_reference_v1(),
                EventRouteDirectionV1::Consume,
            ),
        ),
        event_capability(
            "mail_persons_sync.persons.command-succeeded.v1",
            ProvidedSurfaceKindV1::DurableConsumer,
            persons_command_succeeded_contract_reference_v1(),
            event_request(
                DurableEnvelopeKindV1::Result,
                persons_command_succeeded_contract_reference_v1(),
                EventRouteDirectionV1::Consume,
            ),
        ),
        event_capability(
            "mail_persons_sync.persons.command.v1",
            ProvidedSurfaceKindV1::DurablePublisher,
            persons_command_contract_reference_v1(),
            event_request(
                DurableEnvelopeKindV1::Command,
                persons_command_contract_reference_v1(),
                EventRouteDirectionV1::Publish,
            ),
        ),
        workflow_publisher(
            "mail_persons_sync.run-result.v1",
            MailPersonsSyncContractV1::RunResult,
        ),
        scheduler_receipt_capability(),
        scheduler_capability(),
        scheduler_schedule_command_capability(),
        scheduler_schedule_result_capability(),
        storage_capability(),
    ]
}

fn mail_consumer(id: &str, contract: MailPersonSourceContractV1) -> CapabilityDescriptorV1 {
    event_capability(
        id,
        ProvidedSurfaceKindV1::DurableConsumer,
        contract.reference(),
        contract.consume_request(),
    )
}

fn workflow_publisher(id: &str, contract: MailPersonsSyncContractV1) -> CapabilityDescriptorV1 {
    let reference = contract.reference();
    event_capability(
        id,
        ProvidedSurfaceKindV1::DurablePublisher,
        reference.clone(),
        event_request(
            DurableEnvelopeKindV1::Result,
            reference,
            EventRouteDirectionV1::Publish,
        ),
    )
}

fn event_capability(
    id: &str,
    kind: ProvidedSurfaceKindV1,
    contract: ContractReferenceV1,
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

pub(crate) fn scheduler_job_contract_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: MAIL_PERSONS_SYNC_OWNER_V1.to_owned(),
        name: "scheduled_sync".to_owned(),
        major: 1,
        revision: 1,
        schema_sha256: Sha256::digest(SCHEDULER_JOB_DESCRIPTOR_SET_V1).to_vec(),
    }
}

pub(crate) fn scheduler_receipt_contract_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: "scheduler".to_owned(),
        name: "job_receipt".to_owned(),
        major: 1,
        revision: 1,
        schema_sha256: Sha256::digest(SCHEDULER_JOB_DESCRIPTOR_SET_V1).to_vec(),
    }
}

pub(crate) fn scheduler_schedule_control_contract_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: "scheduler".to_owned(),
        name: "schedule_control".to_owned(),
        major: 1,
        revision: 1,
        schema_sha256: Sha256::digest(SCHEDULER_JOB_DESCRIPTOR_SET_V1).to_vec(),
    }
}

fn scheduler_capability() -> CapabilityDescriptorV1 {
    let contract = scheduler_job_contract_v1();
    let mut capability = event_capability(
        "mail_persons_sync.scheduler.v1",
        ProvidedSurfaceKindV1::DurableConsumer,
        contract.clone(),
        event_request(
            DurableEnvelopeKindV1::Command,
            contract.clone(),
            EventRouteDirectionV1::Consume,
        ),
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
    CapabilityDescriptorV1 {
        capability_id: "mail_persons_sync.scheduler.receipt.v1".to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
            contract: Some(contract.clone()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![
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
        ..Default::default()
    }
}

fn scheduler_schedule_command_capability() -> CapabilityDescriptorV1 {
    let contract = scheduler_schedule_control_contract_v1();
    event_capability(
        "mail_persons_sync.scheduler_schedule_command.v1",
        ProvidedSurfaceKindV1::DurablePublisher,
        contract.clone(),
        event_request(
            DurableEnvelopeKindV1::Command,
            contract,
            EventRouteDirectionV1::Publish,
        ),
    )
}

fn scheduler_schedule_result_capability() -> CapabilityDescriptorV1 {
    let contract = scheduler_schedule_control_contract_v1();
    event_capability(
        "mail_persons_sync.scheduler_schedule_result.v1",
        ProvidedSurfaceKindV1::DurableConsumer,
        contract.clone(),
        event_request(
            DurableEnvelopeKindV1::Result,
            contract,
            EventRouteDirectionV1::Consume,
        ),
    )
}

fn storage_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: MAIL_PERSONS_SYNC_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: MAIL_PERSONS_SYNC_OWNER_V1.to_owned(),
                connection_budget: STORAGE_CONNECTION_BUDGET_V1,
                timeout_millis: 5_000,
            })),
        }],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn descriptor_has_exact_account_binding_and_scheduler_control_contour() {
        let descriptor = mail_persons_sync_module_descriptor_v1("test-build");
        assert_eq!(descriptor.descriptor_revision, 2);
        assert_eq!(descriptor.capabilities.len(), 18);
        let ids = descriptor
            .capabilities
            .iter()
            .map(|capability| capability.capability_id.as_str())
            .collect::<BTreeSet<_>>();
        for expected in [
            "mail_persons_sync.mail.account-ready.v1",
            "mail_persons_sync.mail.account-retired.v1",
            "mail_persons_sync.scheduler_schedule_command.v1",
            "mail_persons_sync.scheduler_schedule_result.v1",
        ] {
            assert!(ids.contains(expected), "missing {expected}");
        }
        let receipt = descriptor
            .capabilities
            .iter()
            .find(|capability| capability.capability_id == "mail_persons_sync.scheduler.receipt.v1")
            .expect("scheduler receipt capability");
        assert_eq!(
            receipt.requests.len(),
            2,
            "acceptance ACK and terminal Result routes"
        );
    }
}
