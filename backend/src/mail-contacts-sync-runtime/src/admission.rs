use makosh_contacts_command_api::{
    bind_mail_address_book_provider_link_contract_reference_v1,
    bind_mail_address_book_provider_link_publish_request_v1,
    bind_mail_address_book_provider_link_rejected_consume_request_v1,
    bind_mail_address_book_provider_link_rejected_contract_reference_v1,
    contact_upsert_rejected_contract_reference_v1, contact_upserted_contract_reference_v1,
    mail_address_book_provider_link_bound_consume_request_v1,
    mail_address_book_provider_link_bound_contract_reference_v1,
    upsert_contact_command_contract_reference_v1, upsert_contact_command_publish_request_v1,
};
use makosh_contacts_mail_sync_source_api::{
    contact_changed_for_mail_sync_consume_request_v1,
    contact_changed_for_mail_sync_contract_reference_v1,
    contact_mail_sync_source_prepare_contract_reference_v1,
    contact_mail_sync_source_prepare_publish_request_v1,
    contact_mail_sync_source_prepared_consume_request_v1,
    contact_mail_sync_source_prepared_contract_reference_v1,
    contact_mail_sync_source_rejected_consume_request_v1,
    contact_mail_sync_source_rejected_contract_reference_v1,
};
use makosh_mail_address_book_contract::MailAddressBookContractV1;
use makosh_mail_contacts_sync_api::{
    MAIL_CONTACTS_SYNC_CAPABILITY_ID_V1, MAIL_CONTACTS_SYNC_COMMAND_CONNECT_PATH_V1,
    MAIL_CONTACTS_SYNC_MODULE_ID_V1, MAIL_CONTACTS_SYNC_OWNER_ID_V1,
    MAIL_CONTACTS_SYNC_QUERY_CONNECT_PATH_V1, mail_contacts_sync_query_contract_v1,
    mail_contacts_sync_realtime_contract_v1, mail_contacts_sync_start_contract_v1,
};
use makosh_runtime_protocol::v1::{
    CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1, ClientRpcRouteV1,
    ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1, EventRouteRequestV1,
    EventSubscriptionRequirementV1, ModuleDescriptorV1, ModuleKindV1, ProtocolRangeV1,
    ProvidedSurfaceKindV1, ProvidedSurfaceV1, RuntimeBudgetRequestV1, SchedulerJobRequestV1,
    SettingsSchemaRefV1, StorageNamespaceRequestV1, capability_request_v1::Request,
};
use makosh_scheduler_protocol::SCHEDULER_JOB_DESCRIPTOR_SET_V1;
use sha2::{Digest, Sha256};

use crate::settings::mail_contacts_sync_settings_schema_bytes_v1;

pub const MAIL_CONTACTS_SYNC_STORAGE_CAPABILITY_ID_V1: &str = "mail_contacts_sync.storage.v1";
pub const MAIL_CONTACTS_SYNC_SCHEDULER_CAPABILITY_ID_V1: &str = "mail_contacts_sync.scheduler.v1";
const STORAGE_CONNECTION_BUDGET_V1: u32 = 4;

#[must_use]
pub fn mail_contacts_sync_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings = mail_contacts_sync_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: MAIL_CONTACTS_SYNC_MODULE_ID_V1.to_owned(),
        owner_id: MAIL_CONTACTS_SYNC_OWNER_ID_V1.to_owned(),
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
        display_name: "Mail Contacts Sync".to_owned(),
    }
}

fn capabilities() -> Vec<CapabilityDescriptorV1> {
    vec![
        client_capability(),
        event_capability(
            "mail_contacts_sync.contacts.changed.v1",
            ProvidedSurfaceKindV1::DurableConsumer,
            contact_changed_for_mail_sync_contract_reference_v1(),
            contact_changed_for_mail_sync_consume_request_v1(),
        ),
        event_capability(
            "mail_contacts_sync.contacts.command.v1",
            ProvidedSurfaceKindV1::DurablePublisher,
            upsert_contact_command_contract_reference_v1(),
            upsert_contact_command_publish_request_v1(),
        ),
        event_capability(
            "mail_contacts_sync.contacts.provider-link-bound.v1",
            ProvidedSurfaceKindV1::DurableConsumer,
            mail_address_book_provider_link_bound_contract_reference_v1(),
            mail_address_book_provider_link_bound_consume_request_v1(),
        ),
        event_capability(
            "mail_contacts_sync.contacts.provider-link-command.v1",
            ProvidedSurfaceKindV1::DurablePublisher,
            bind_mail_address_book_provider_link_contract_reference_v1(),
            bind_mail_address_book_provider_link_publish_request_v1(),
        ),
        event_capability(
            "mail_contacts_sync.contacts.provider-link-rejected.v1",
            ProvidedSurfaceKindV1::DurableConsumer,
            bind_mail_address_book_provider_link_rejected_contract_reference_v1(),
            bind_mail_address_book_provider_link_rejected_consume_request_v1(),
        ),
        event_capability(
            "mail_contacts_sync.contacts.rejected.v1",
            ProvidedSurfaceKindV1::DurableConsumer,
            contact_upsert_rejected_contract_reference_v1(),
            consume_result(contact_upsert_rejected_contract_reference_v1()),
        ),
        event_capability(
            "mail_contacts_sync.contacts.source-prepare.v1",
            ProvidedSurfaceKindV1::DurablePublisher,
            contact_mail_sync_source_prepare_contract_reference_v1(),
            contact_mail_sync_source_prepare_publish_request_v1(),
        ),
        event_capability(
            "mail_contacts_sync.contacts.source-prepared.v1",
            ProvidedSurfaceKindV1::DurableConsumer,
            contact_mail_sync_source_prepared_contract_reference_v1(),
            contact_mail_sync_source_prepared_consume_request_v1(),
        ),
        event_capability(
            "mail_contacts_sync.contacts.source-rejected.v1",
            ProvidedSurfaceKindV1::DurableConsumer,
            contact_mail_sync_source_rejected_contract_reference_v1(),
            contact_mail_sync_source_rejected_consume_request_v1(),
        ),
        event_capability(
            "mail_contacts_sync.contacts.upserted.v1",
            ProvidedSurfaceKindV1::DurableConsumer,
            contact_upserted_contract_reference_v1(),
            consume_result(contact_upserted_contract_reference_v1()),
        ),
        mail_capability(
            "mail_contacts_sync.mail.entry-observed.v1",
            MailAddressBookContractV1::EntryObserved,
            EventRouteDirectionV1::Consume,
        ),
        mail_capability(
            "mail_contacts_sync.mail.entry-upsert-rejected.v1",
            MailAddressBookContractV1::EntryUpsertRejected,
            EventRouteDirectionV1::Consume,
        ),
        mail_capability(
            "mail_contacts_sync.mail.entry-upserted.v1",
            MailAddressBookContractV1::EntryUpserted,
            EventRouteDirectionV1::Consume,
        ),
        mail_capability(
            "mail_contacts_sync.mail.fetch-page.v1",
            MailAddressBookContractV1::FetchPageCommand,
            EventRouteDirectionV1::Publish,
        ),
        mail_capability(
            "mail_contacts_sync.mail.page-completed.v1",
            MailAddressBookContractV1::PageCompleted,
            EventRouteDirectionV1::Consume,
        ),
        mail_capability(
            "mail_contacts_sync.mail.page-rejected.v1",
            MailAddressBookContractV1::PageRejected,
            EventRouteDirectionV1::Consume,
        ),
        mail_capability(
            "mail_contacts_sync.mail.upsert-entry.v1",
            MailAddressBookContractV1::UpsertEntryCommand,
            EventRouteDirectionV1::Publish,
        ),
        scheduler_receipt_capability(),
        scheduler_capability(),
        storage_capability(),
    ]
}

fn client_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: MAIL_CONTACTS_SYNC_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            client_rpc(
                mail_contacts_sync_start_contract_v1(),
                MAIL_CONTACTS_SYNC_COMMAND_CONNECT_PATH_V1,
            ),
            client_rpc(
                mail_contacts_sync_query_contract_v1(),
                MAIL_CONTACTS_SYNC_QUERY_CONNECT_PATH_V1,
            ),
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRealtime as i32,
                contract: Some(mail_contacts_sync_realtime_contract_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        ..Default::default()
    }
}

fn client_rpc(contract: ContractReferenceV1, path: &str) -> ProvidedSurfaceV1 {
    ProvidedSurfaceV1 {
        kind: ProvidedSurfaceKindV1::ClientRpc as i32,
        contract: Some(contract),
        client_rpc_route: Some(ClientRpcRouteV1 {
            path: path.to_owned(),
        }),
        client_blob_route: None,
    }
}

fn mail_capability(
    capability_id: &str,
    contract: MailAddressBookContractV1,
    direction: EventRouteDirectionV1,
) -> CapabilityDescriptorV1 {
    let kind = if direction == EventRouteDirectionV1::Publish {
        ProvidedSurfaceKindV1::DurablePublisher
    } else {
        ProvidedSurfaceKindV1::DurableConsumer
    };
    let request = if direction == EventRouteDirectionV1::Publish {
        contract.publish_request()
    } else {
        contract.consume_request()
    };
    event_capability(capability_id, kind, contract.reference(), request)
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

fn consume_result(contract: ContractReferenceV1) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: DurableEnvelopeKindV1::Result as i32,
            contract: Some(contract),
            direction: EventRouteDirectionV1::Consume as i32,
            max_in_flight: 32,
            subscription_requirement: EventSubscriptionRequirementV1::Required as i32,
            max_deliver: 10,
            ack_wait_millis: 30_000,
        })),
    }
}

fn scheduler_capability() -> CapabilityDescriptorV1 {
    let contract = scheduler_job_contract_v1();
    CapabilityDescriptorV1 {
        capability_id: MAIL_CONTACTS_SYNC_SCHEDULER_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::DurableConsumer as i32,
            contract: Some(contract.clone()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![
            CapabilityRequestV1 {
                request: Some(Request::SchedulerJob(SchedulerJobRequestV1 {
                    job_kind: Some(contract.clone()),
                })),
            },
            CapabilityRequestV1 {
                request: Some(Request::EventRoute(EventRouteRequestV1 {
                    envelope_kind: DurableEnvelopeKindV1::Command as i32,
                    contract: Some(contract),
                    direction: EventRouteDirectionV1::Consume as i32,
                    max_in_flight: 1,
                    subscription_requirement: EventSubscriptionRequirementV1::Required as i32,
                    max_deliver: 8,
                    ack_wait_millis: 30_000,
                })),
            },
        ],
        ..Default::default()
    }
}

pub(crate) fn scheduler_job_contract_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: MAIL_CONTACTS_SYNC_OWNER_ID_V1.to_owned(),
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

fn scheduler_receipt_capability() -> CapabilityDescriptorV1 {
    let contract = scheduler_receipt_contract_v1();
    CapabilityDescriptorV1 {
        capability_id: "mail_contacts_sync.scheduler.receipt.v1".to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
            contract: Some(contract.clone()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![
            publish_route(DurableEnvelopeKindV1::Ack, contract.clone()),
            publish_route(DurableEnvelopeKindV1::Result, contract),
        ],
        ..Default::default()
    }
}

fn publish_route(
    kind: DurableEnvelopeKindV1,
    contract: ContractReferenceV1,
) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: kind as i32,
            contract: Some(contract),
            direction: EventRouteDirectionV1::Publish as i32,
            max_in_flight: 32,
            subscription_requirement: EventSubscriptionRequirementV1::Unspecified as i32,
            max_deliver: 0,
            ack_wait_millis: 0,
        })),
    }
}

fn storage_capability() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: MAIL_CONTACTS_SYNC_STORAGE_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: MAIL_CONTACTS_SYNC_OWNER_ID_V1.to_owned(),
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
    use crate::mail_contacts_sync_settings_schema_v1;

    #[test]
    fn descriptor_is_workflow_owned_and_requests_only_public_contracts() {
        let descriptor = mail_contacts_sync_module_descriptor_v1("test-build");
        validate_descriptor_v1(&descriptor).expect("descriptor");
        validate_settings_schema_v1(&mail_contacts_sync_settings_schema_v1()).expect("settings");
        assert_eq!(descriptor.module_kind, ModuleKindV1::Workflow as i32);
        assert_eq!(descriptor.owner_id, MAIL_CONTACTS_SYNC_OWNER_ID_V1);
        assert_eq!(descriptor.capabilities.len(), 21);
        for capability_id in [
            "mail_contacts_sync.contacts.provider-link-bound.v1",
            "mail_contacts_sync.contacts.provider-link-command.v1",
            "mail_contacts_sync.contacts.provider-link-rejected.v1",
        ] {
            assert!(
                descriptor
                    .capabilities
                    .iter()
                    .any(|capability| capability.capability_id == capability_id),
                "missing capability {capability_id}"
            );
        }
        assert!(descriptor.capabilities.iter().all(|capability| {
            capability.capability_id.starts_with("mail_contacts_sync.")
                || capability.capability_id == MAIL_CONTACTS_SYNC_CAPABILITY_ID_V1
        }));
    }
}
