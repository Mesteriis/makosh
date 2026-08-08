//! Scheduler schedule-control runtime bindings derived from independent authorities.

use std::collections::BTreeSet;

use makosh_kernel_control_store::ModuleEventEnvelopeKindV1;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{
    SchedulerRuntimeScheduleControlBindingV1, SchedulerRuntimeScheduleControlGrantV1,
};

use crate::{
    platform::events::{
        catalog::EventCatalogContractV1,
        topology::{EventTopologyPlanV1, subject::EventStreamKindV1},
    },
    runtime::lifecycle::fence::current_managed_runtime_matches,
};

use super::scheduler_catalog;

const OWNER: &str = "scheduler";
const CONTRACT: &str = "schedule_control";
const MAJOR: u32 = 1;

pub(crate) struct SchedulerScheduleControlConfigurationV1 {
    pub(crate) binding: Option<SchedulerRuntimeScheduleControlBindingV1>,
    pub(crate) grants: Vec<SchedulerRuntimeScheduleControlGrantV1>,
}

pub(crate) fn derive(
    store: &SqliteControlStore,
    contracts: &[EventCatalogContractV1],
    topology: &EventTopologyPlanV1,
    scheduler_registration_id: &str,
    scheduler_grant_epoch: u64,
) -> Result<SchedulerScheduleControlConfigurationV1, String> {
    let binding = binding(
        contracts,
        topology,
        scheduler_registration_id,
        scheduler_grant_epoch,
    )?;
    let Some(binding) = binding else {
        return Ok(SchedulerScheduleControlConfigurationV1 {
            binding: None,
            grants: Vec::new(),
        });
    };
    let grants = grants(store, topology)?;
    if grants.is_empty() {
        return Ok(SchedulerScheduleControlConfigurationV1 {
            binding: None,
            grants,
        });
    }
    Ok(SchedulerScheduleControlConfigurationV1 {
        binding: Some(binding),
        grants,
    })
}

fn binding(
    contracts: &[EventCatalogContractV1],
    topology: &EventTopologyPlanV1,
    scheduler_registration_id: &str,
    scheduler_grant_epoch: u64,
) -> Result<Option<SchedulerRuntimeScheduleControlBindingV1>, String> {
    let command = topology.consumers().iter().find(|consumer| {
        consumer.registration_id() == scheduler_registration_id
            && consumer.grant_epoch() == scheduler_grant_epoch
            && subject_matches(
                consumer.subject().kind(),
                consumer.subject(),
                EventStreamKindV1::Command,
            )
    });
    let result_publisher = topology.publishers().iter().find(|publisher| {
        publisher.registration_id() == scheduler_registration_id
            && publisher.grant_epoch() == scheduler_grant_epoch
            && subject_matches(
                publisher.subject().kind(),
                publisher.subject(),
                EventStreamKindV1::Result,
            )
    });
    match (command, result_publisher) {
        (None, None) => return Ok(None),
        (Some(_), Some(_)) => {}
        _ => return Err("Scheduler schedule-control topology is incomplete".to_owned()),
    }
    let command = command.expect("complete topology");
    let result_contract = contracts
        .iter()
        .find(|contract| {
            contract.envelope_kind() == ModuleEventEnvelopeKindV1::Result
                && contract.owner() == OWNER
                && contract.name() == CONTRACT
                && contract.major() == MAJOR
        })
        .ok_or_else(|| "Scheduler schedule-control result contract is unavailable".to_owned())?;
    let delivery = command.delivery_policy();
    Ok(Some(SchedulerRuntimeScheduleControlBindingV1 {
        stream_name: "MAKOSH_COMMAND_V1".to_owned(),
        durable_name: command.durable_name().to_owned(),
        filter_subject: command.subject().as_str(),
        ack_wait_millis: delivery.ack_wait_millis(),
        max_deliver: u32::from(delivery.max_deliver()),
        max_ack_pending: u32::from(command.max_in_flight()),
        result_subject: result_publisher
            .expect("complete topology")
            .subject()
            .as_str(),
        command_contract_revision: command.contract().revision,
        command_schema_sha256: command.contract().schema_sha256.clone(),
        result_contract_revision: result_contract.revision(),
        result_schema_sha256: result_contract.schema_sha256().to_vec(),
    }))
}

fn grants(
    store: &SqliteControlStore,
    topology: &EventTopologyPlanV1,
) -> Result<Vec<SchedulerRuntimeScheduleControlGrantV1>, String> {
    let command_publishers = topology
        .publishers()
        .iter()
        .filter(|publisher| {
            subject_matches(
                publisher.subject().kind(),
                publisher.subject(),
                EventStreamKindV1::Command,
            )
        })
        .map(|publisher| {
            (
                publisher.registration_id().to_owned(),
                publisher.grant_epoch(),
            )
        })
        .collect::<BTreeSet<_>>();
    let result_consumers = topology
        .consumers()
        .iter()
        .filter(|consumer| {
            subject_matches(
                consumer.subject().kind(),
                consumer.subject(),
                EventStreamKindV1::Result,
            )
        })
        .map(|consumer| {
            (
                consumer.registration_id().to_owned(),
                consumer.grant_epoch(),
            )
        })
        .collect::<BTreeSet<_>>();
    let mut grants = Vec::new();
    for entry in scheduler_catalog::resolve(store)? {
        let source = (entry.registration_id().to_owned(), entry.grant_epoch());
        if !command_publishers.contains(&source) || !result_consumers.contains(&source) {
            continue;
        }
        let Some(launch) = store
            .effective_managed_launch_record(entry.registration_id())
            .map_err(|_| "Scheduler schedule-control source launch is unavailable".to_owned())?
        else {
            continue;
        };
        if launch.grant_epoch() != entry.grant_epoch()
            || !current_managed_runtime_matches(
                store,
                entry.registration_id(),
                launch.runtime_instance_id(),
                launch.runtime_generation(),
                launch.grant_epoch(),
            )
            .map_err(|_| "Scheduler schedule-control source fence is unavailable".to_owned())?
        {
            continue;
        }
        let request = entry.request();
        grants.push(SchedulerRuntimeScheduleControlGrantV1 {
            source_module_id: entry.module_id().to_owned(),
            source_runtime_instance_id: runtime_instance_id(launch.runtime_instance_id())?.to_vec(),
            source_runtime_generation: launch.runtime_generation(),
            source_grant_epoch: launch.grant_epoch(),
            source_owner: request.owner().to_owned(),
            job_owner: request.owner().to_owned(),
            job_name: request.name().to_owned(),
            job_major: request.major(),
            contract_name: format!("{}.{}", request.owner(), request.name()),
            contract_revision: request.revision(),
            contract_schema_sha256: request.schema_sha256().to_vec(),
        });
    }
    grants.sort_by(|left, right| {
        (
            left.source_module_id.as_str(),
            left.job_owner.as_str(),
            left.job_name.as_str(),
            left.job_major,
        )
            .cmp(&(
                right.source_module_id.as_str(),
                right.job_owner.as_str(),
                right.job_name.as_str(),
                right.job_major,
            ))
    });
    Ok(grants)
}

fn subject_matches(
    kind: EventStreamKindV1,
    subject: &crate::platform::events::topology::subject::EventSubjectV1,
    expected_kind: EventStreamKindV1,
) -> bool {
    kind == expected_kind
        && subject.owner() == OWNER
        && subject.contract() == CONTRACT
        && subject.major() == MAJOR
}

fn runtime_instance_id(value: &str) -> Result<[u8; 16], String> {
    (value.len() == 32)
        .then_some(())
        .ok_or_else(|| "Scheduler schedule-control runtime identity is invalid".to_owned())?;
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            std::str::from_utf8(pair)
                .ok()
                .and_then(|value| u8::from_str_radix(value, 16).ok())
        })
        .collect::<Option<Vec<_>>>()
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(|| "Scheduler schedule-control runtime identity is invalid".to_owned())
}

#[cfg(test)]
mod tests {
    use makosh_kernel_control_store::{
        BundledManagedLaunchBinding, ManagedLaunchRecord, ModuleDescriptorRegistrationRequestsV1,
        ModuleEventDeliveryPolicyV1, ModuleEventEnvelopeKindV1, ModuleEventRouteDirectionV1,
        ModuleEventRouteRequestInputV1, ModuleEventRouteRequestV1,
        ModuleEventSubscriptionRequirementV1, ModuleRegistration, ModuleRegistrationState,
        ModuleSchedulerJobRequestV1, PlatformEventHubTopologyV1, PlatformEventStreamBudgetV1,
    };

    use super::*;
    use crate::platform::events::{catalog, topology};

    #[test]
    fn derives_only_current_source_with_command_result_and_job_authorities() {
        let root = fixture_root("current");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("fixture directory");
        let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
            .expect("control store");
        register_scheduler(&store);
        register_source(&store);
        record_source_launch(&store);
        let contracts = catalog::resolve_contracts(&store).expect("contracts");
        let plan = topology::plan(&contracts, &event_hub()).expect("topology");

        let configuration =
            derive(&store, &contracts, &plan, "scheduler_registration", 2).expect("configuration");

        let binding = configuration.binding.expect("binding");
        assert_eq!(
            binding.filter_subject,
            "makosh.command.v1.scheduler.schedule_control.v1"
        );
        assert_eq!(
            binding.result_subject,
            "makosh.result.v1.scheduler.schedule_control.v1"
        );
        assert_eq!(configuration.grants.len(), 1);
        assert_eq!(configuration.grants[0].source_module_id, "delayed_delivery");
        assert_eq!(configuration.grants[0].source_runtime_generation, 3);
        assert_eq!(configuration.grants[0].source_grant_epoch, 2);
        std::fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn does_not_enable_schedule_control_without_a_current_source_launch() {
        let root = fixture_root("missing-launch");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("fixture directory");
        let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
            .expect("control store");
        register_scheduler(&store);
        register_source(&store);
        let contracts = catalog::resolve_contracts(&store).expect("contracts");
        let plan = topology::plan(&contracts, &event_hub()).expect("topology");

        let configuration =
            derive(&store, &contracts, &plan, "scheduler_registration", 2).expect("configuration");

        assert!(configuration.binding.is_none());
        assert!(configuration.grants.is_empty());
        std::fs::remove_dir_all(root).expect("remove fixture");
    }

    fn fixture_root(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "makosh-scheduler-control-topology-{}-{label}",
            std::process::id()
        ))
    }

    fn register_scheduler(store: &SqliteControlStore) {
        let registration = ModuleRegistration::new(
            "scheduler_registration",
            "scheduler",
            "scheduler",
            [1; 32],
            ModuleRegistrationState::Pending,
            1,
        );
        let capabilities = vec![
            "schedule_control_command".to_owned(),
            "schedule_control_result".to_owned(),
        ];
        let routes = vec![
            route(
                "scheduler_registration",
                "schedule_control_command",
                ModuleEventEnvelopeKindV1::Command,
                ModuleEventRouteDirectionV1::Consume,
            ),
            route(
                "scheduler_registration",
                "schedule_control_result",
                ModuleEventEnvelopeKindV1::Result,
                ModuleEventRouteDirectionV1::Publish,
            ),
        ];
        store
            .create_pending_registration_with_requests(
                &registration,
                &capabilities,
                &[],
                &routes,
                &[],
            )
            .expect("scheduler registration");
        store
            .approve_module_registration("scheduler_registration", &capabilities)
            .expect("scheduler approval");
    }

    fn register_source(store: &SqliteControlStore) {
        let registration = ModuleRegistration::new(
            "delayed_delivery_registration",
            "delayed_delivery",
            "communication_delayed_delivery",
            [2; 32],
            ModuleRegistrationState::Pending,
            1,
        );
        let capabilities = vec![
            "schedule_control_publish".to_owned(),
            "schedule_control_result".to_owned(),
            "scheduler_job".to_owned(),
        ];
        let routes = vec![
            route(
                "delayed_delivery_registration",
                "schedule_control_publish",
                ModuleEventEnvelopeKindV1::Command,
                ModuleEventRouteDirectionV1::Publish,
            ),
            route(
                "delayed_delivery_registration",
                "schedule_control_result",
                ModuleEventEnvelopeKindV1::Result,
                ModuleEventRouteDirectionV1::Consume,
            ),
        ];
        let jobs = [ModuleSchedulerJobRequestV1::new(
            "delayed_delivery_registration",
            "scheduler_job",
            "communication_delayed_delivery",
            "execute",
            1,
            1,
            [9; 32],
        )];
        store
            .create_pending_registration_with_descriptor_requests(
                &registration,
                &capabilities,
                ModuleDescriptorRegistrationRequestsV1 {
                    storage: &[],
                    events: &routes,
                    blobs: &[],
                    scheduler: &jobs,
                    vault_purposes: &[],
                    client_rpc_routes: &[],
                    client_blob_routes: &[],
                    client_realtime_routes: &[],
                    query_rpc_routes: &[],
                    request_rpc_routes: &[],
                    contract_dependencies: &[],
                },
            )
            .expect("source registration");
        store
            .approve_module_registration("delayed_delivery_registration", &capabilities)
            .expect("source approval");
    }

    fn record_source_launch(store: &SqliteControlStore) {
        store
            .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
                "delayed_delivery_registration",
                1,
                "distribution-1",
                "delayed-delivery-runtime",
                [3; 32],
                [2; 32],
                None,
            ))
            .expect("source binding");
        store
            .record_managed_launch(&ManagedLaunchRecord::new(
                "delayed_delivery_registration",
                "04040404040404040404040404040404",
                1,
                1,
                3,
                2,
            ))
            .expect("source launch");
    }

    fn route(
        registration_id: &str,
        capability_id: &str,
        envelope_kind: ModuleEventEnvelopeKindV1,
        direction: ModuleEventRouteDirectionV1,
    ) -> ModuleEventRouteRequestV1 {
        ModuleEventRouteRequestV1::new(ModuleEventRouteRequestInputV1 {
            registration_id: registration_id.to_owned(),
            capability_id: capability_id.to_owned(),
            envelope_kind,
            contract_owner: "scheduler".to_owned(),
            contract_name: "schedule_control".to_owned(),
            contract_major: 1,
            contract_revision: 1,
            contract_schema_sha256: [7; 32],
            direction,
            max_in_flight: 8,
            delivery_policy: matches!(direction, ModuleEventRouteDirectionV1::Consume).then(|| {
                ModuleEventDeliveryPolicyV1::new(
                    ModuleEventSubscriptionRequirementV1::Required,
                    5,
                    30_000,
                )
            }),
        })
    }

    fn event_hub() -> PlatformEventHubTopologyV1 {
        PlatformEventHubTopologyV1::new(
            1,
            "nats://127.0.0.1:4222",
            "event_hub",
            1,
            [
                ModuleEventEnvelopeKindV1::Command,
                ModuleEventEnvelopeKindV1::Result,
            ]
            .into_iter()
            .map(|kind| PlatformEventStreamBudgetV1::new(kind, 1_048_576, 3_600_000, 1))
            .collect(),
        )
    }
}
