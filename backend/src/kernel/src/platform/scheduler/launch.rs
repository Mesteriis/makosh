//! Stages one already-authorized Scheduler child from its exact runtime fences.

use std::path::Path;

use makosh_kernel_control_store::{PlatformStorageBindingStateV1, PlatformStorageBindingV1};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::{
    v1::{SchedulerRuntimeConfigurationV1, SchedulerRuntimeStorageBindingV1},
    validation::scheduler::validate_scheduler_runtime_configuration,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::distribution::staged_contracts::StagedRuntimeContracts;
use crate::platform::{
    events::{authority::status as events_status, catalog, topology},
    macos::{managed_launch::ManagedLaunchReservation, native_launch},
    storage,
    vault::status as vault_status,
};
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

const DISPATCH_BATCH_LIMIT: u32 = 32;
const RECEIPT_BATCH_LIMIT: u32 = 32;
const RECONCILE_INTERVAL_MILLIS: u32 = 1_000;

pub(crate) fn topology_fingerprint(
    store: &SqliteControlStore,
    scheduler_registration_id: &str,
    scheduler_grant_epoch: u64,
) -> Result<[u8; 32], String> {
    let event_topology = events_status::current_topology(store)?;
    let contracts = catalog::resolve_contracts(store)?;
    let topology = topology::plan(&contracts, &event_topology)?;
    let schedule_control = super::schedule_control::derive(
        store,
        &contracts,
        &topology,
        scheduler_registration_id,
        scheduler_grant_epoch,
    )?;
    let dispatch_publishers = topology::scheduler_dispatch_bindings(
        &topology,
        scheduler_registration_id,
        scheduler_grant_epoch,
    )
    .map_err(|_| "Scheduler dispatch topology is unavailable".to_owned())?;
    let receipt_consumers = topology::scheduler_receipt_bindings(
        &topology,
        scheduler_registration_id,
        scheduler_grant_epoch,
    )
    .map_err(|_| "Scheduler receipt topology is unavailable".to_owned())?;
    let mut digest = Sha256::new();
    digest.update(b"makosh.scheduler.runtime-topology.v1");
    digest.update(event_topology.revision().to_be_bytes());
    digest.update(event_topology.credential_revision().to_be_bytes());
    update_optional_message(
        &mut digest,
        b"schedule-control-binding",
        schedule_control.binding.as_ref(),
    );
    for grant in &schedule_control.grants {
        update_message(&mut digest, b"schedule-control-grant", grant);
    }
    for publisher in &dispatch_publishers {
        update_message(&mut digest, b"dispatch-publisher", publisher);
    }
    for consumer in &receipt_consumers {
        update_message(&mut digest, b"receipt-consumer", consumer);
    }
    Ok(digest.finalize().into())
}

fn update_optional_message<M: Message>(digest: &mut Sha256, label: &[u8], message: Option<&M>) {
    match message {
        Some(message) => update_message(digest, label, message),
        None => update_bytes(digest, label, &[]),
    }
}

fn update_message<M: Message>(digest: &mut Sha256, label: &[u8], message: &M) {
    update_bytes(digest, label, &message.encode_to_vec());
}

fn update_bytes(digest: &mut Sha256, label: &[u8], bytes: &[u8]) {
    digest.update((label.len() as u64).to_be_bytes());
    digest.update(label);
    digest.update((bytes.len() as u64).to_be_bytes());
    digest.update(bytes);
}

/// Launches Scheduler only after its caller has durably reserved and bound the exact identity.
pub(crate) fn start_from_reservation(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel: &Path,
    runtime_dir: &Path,
    reservation: ManagedLaunchReservation,
    storage_binding: &PlatformStorageBindingV1,
) -> Result<u64, String> {
    let storage_topology = storage::topology::current(store)?;
    validate_storage_binding(&reservation, storage_binding, &storage_topology)?;
    storage::provisioning::apply_reserved_binding(supervisor, store, storage_binding)?;
    let vault = vault_status::read_current(store, &supervisor.relay_port())?;
    let event_topology = events_status::current_topology(store)?;
    let contracts = catalog::resolve_contracts(store)?;
    let topology = topology::plan(&contracts, &event_topology)?;
    let schedule_control = super::schedule_control::derive(
        store,
        &contracts,
        &topology,
        reservation.registration_id(),
        reservation.grant_epoch(),
    )?;
    let configuration = runtime_configuration(
        &reservation,
        storage_binding,
        &storage_topology,
        store.snapshot().instance_id(),
        &vault,
        SchedulerRuntimeEventConfigurationV1 {
            event_topology: &event_topology,
            topology: &topology,
            schedule_control,
        },
    )?;
    start_staged_runtime(supervisor, kernel, runtime_dir, reservation, configuration)
}

fn start_staged_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    kernel: &Path,
    runtime_dir: &Path,
    reservation: ManagedLaunchReservation,
    configuration: Vec<u8>,
) -> Result<u64, String> {
    let prepared = native_launch::prepare_bound_managed_runtime(
        kernel,
        reservation.binding(),
        &runtime_dir
            .join("scheduler")
            .join(format!("launch-{}", reservation.runtime_generation()))
            .join("managed"),
    )?;
    let contracts = match stage_contracts(runtime_dir, &reservation, &prepared, &configuration) {
        Ok(contracts) => contracts,
        Err(error) => {
            let _ = prepared.remove();
            return Err(error);
        }
    };
    let arguments = match inherited_arguments(&contracts) {
        Ok(arguments) => arguments,
        Err(error) => {
            let _ = contracts.remove();
            let _ = prepared.remove();
            return Err(error);
        }
    };
    let runtime_generation = reservation.runtime_generation();
    let (registration_id, expectation, policy) = reservation.into_single_attempt_launch_parts()?;
    supervisor.start_with_arguments_and_contracts(
        registration_id.clone(),
        prepared.into_staged_executable(),
        arguments,
        expectation,
        policy,
        contracts,
    )?;
    if let Err(error) = supervisor.wait_until_ready(&registration_id) {
        let _ = supervisor.stop(&registration_id);
        let _ = supervisor.record_failure(&registration_id, error.clone());
        return Err(error);
    }
    Ok(runtime_generation)
}

fn stage_contracts(
    runtime_dir: &Path,
    reservation: &ManagedLaunchReservation,
    prepared: &native_launch::PreparedBundledManagedRuntime,
    configuration: &[u8],
) -> Result<StagedRuntimeContracts, String> {
    let Some(settings_schema) = prepared.settings_schema_bytes() else {
        return Err("Scheduler release lacks a settings schema".to_owned());
    };
    StagedRuntimeContracts::stage_with_runtime_configuration(
        &runtime_dir
            .join("scheduler")
            .join(format!("launch-{}", reservation.runtime_generation()))
            .join("contracts"),
        prepared.descriptor_bytes(),
        Some(settings_schema),
        Some(configuration),
    )
}

pub(crate) fn validate_storage_binding(
    reservation: &ManagedLaunchReservation,
    binding: &PlatformStorageBindingV1,
    topology: &makosh_kernel_control_store::PlatformStorageTopology,
) -> Result<(), String> {
    (binding.state() == PlatformStorageBindingStateV1::Active
        && binding.registration_id() == reservation.registration_id()
        && binding.runtime_instance_id() == reservation.runtime_instance_id()
        && binding.runtime_generation() == reservation.runtime_generation()
        && binding.grant_epoch() == reservation.grant_epoch()
        && binding.topology_revision() == topology.revision()
        && binding.storage_generation() == topology.storage_generation())
    .then_some(())
    .ok_or_else(|| "Scheduler Storage binding is stale".to_owned())
}

struct SchedulerRuntimeEventConfigurationV1<'a> {
    event_topology: &'a makosh_kernel_control_store::PlatformEventHubTopologyV1,
    topology: &'a topology::EventTopologyPlanV1,
    schedule_control: super::schedule_control::SchedulerScheduleControlConfigurationV1,
}

fn runtime_configuration(
    reservation: &ManagedLaunchReservation,
    storage_binding: &PlatformStorageBindingV1,
    storage_topology: &makosh_kernel_control_store::PlatformStorageTopology,
    vault_instance_id: &str,
    vault: &vault_status::ManagedVaultStatus,
    events: SchedulerRuntimeEventConfigurationV1<'_>,
) -> Result<Vec<u8>, String> {
    let configuration = SchedulerRuntimeConfigurationV1 {
        storage_binding: Some(SchedulerRuntimeStorageBindingV1 {
            database_id: storage_topology.database_id().to_owned(),
            pgbouncer_host: storage_topology.pgbouncer_endpoint().host().to_owned(),
            pgbouncer_port: u32::from(storage_topology.pgbouncer_endpoint().port()),
            runtime_principal: storage_binding.runtime_principal().to_owned(),
            storage_generation: storage_binding.storage_generation(),
            credential_revision: storage_binding.credential_lease_revision(),
            storage_instance_id: storage_topology.storage_instance_id().to_owned(),
            owner: storage_binding.owner_id().to_owned(),
            role_epoch: storage_binding.role_epoch(),
            pool_alias: format!(
                "runtime_{}_{}",
                storage_binding.registration_id(),
                storage_binding.runtime_generation()
            ),
            max_connections: u32::from(storage_binding.connection_budget()),
            statement_timeout_millis: storage_binding.statement_timeout_millis(),
            storage_bundle_revision: storage_binding.storage_bundle_revision(),
            storage_bundle_digest: storage_binding.storage_bundle_digest().to_vec(),
        }),
        vault_instance_id: vault_instance_id.to_owned(),
        vault_runtime_generation: vault.runtime_generation(),
        vault_hpke_public_key_x25519: vault.hpke_public_key_x25519().to_vec(),
        nats_endpoint: events.event_topology.nats_endpoint().to_owned(),
        event_credential_revision: events.event_topology.credential_revision(),
        dispatch_batch_limit: DISPATCH_BATCH_LIMIT,
        receipt_batch_limit: RECEIPT_BATCH_LIMIT,
        reconcile_interval_millis: RECONCILE_INTERVAL_MILLIS,
        receipt_consumers: topology::scheduler_receipt_bindings(
            events.topology,
            reservation.registration_id(),
            reservation.grant_epoch(),
        )
        .map_err(|_| "Scheduler receipt topology is unavailable".to_owned())?,
        dispatch_publishers: topology::scheduler_dispatch_bindings(
            events.topology,
            reservation.registration_id(),
            reservation.grant_epoch(),
        )
        .map_err(|_| "Scheduler dispatch topology is unavailable".to_owned())?,
        runtime_instance_id: reservation.runtime_instance_id().to_owned(),
        logical_owner_id: storage_binding.owner_id().to_owned(),
        schedule_control: events.schedule_control.binding,
        schedule_control_grants: events.schedule_control.grants,
    };
    validate_scheduler_runtime_configuration(&configuration)
        .map_err(|_| "Scheduler runtime configuration is invalid".to_owned())?;
    Ok(configuration.encode_to_vec())
}

fn inherited_arguments(contracts: &StagedRuntimeContracts) -> Result<Vec<String>, String> {
    let settings_schema = contracts
        .settings_schema_path()
        .ok_or_else(|| "Scheduler settings schema is unavailable".to_owned())?;
    let configuration = contracts
        .runtime_configuration_path()
        .ok_or_else(|| "Scheduler runtime configuration is unavailable".to_owned())?;
    Ok(vec![
        "serve-inherited".to_owned(),
        "--descriptor-path".to_owned(),
        contracts.descriptor_path().display().to_string(),
        "--settings-schema-path".to_owned(),
        settings_schema.display().to_string(),
        "--configuration-path".to_owned(),
        configuration.display().to_string(),
    ])
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::v1::SchedulerRuntimeScheduleControlGrantV1;

    use super::*;

    #[test]
    fn topology_digest_binds_the_source_runtime_generation() {
        let mut first = SchedulerRuntimeScheduleControlGrantV1 {
            source_module_id: "workflow".to_owned(),
            source_runtime_instance_id: vec![3; 16],
            source_runtime_generation: 1,
            source_grant_epoch: 2,
            source_owner: "workflow_owner".to_owned(),
            job_owner: "workflow_owner".to_owned(),
            job_name: "execute".to_owned(),
            job_major: 1,
            contract_name: "workflow_owner.execute".to_owned(),
            contract_revision: 1,
            contract_schema_sha256: vec![4; 32],
        };
        let first_digest = message_digest(&first);
        first.source_runtime_generation = 2;

        assert_ne!(first_digest, message_digest(&first));
    }

    fn message_digest(message: &impl Message) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(b"makosh.scheduler.runtime-topology.v1");
        update_message(&mut digest, b"schedule-control-grant", message);
        digest.finalize().into()
    }
}
