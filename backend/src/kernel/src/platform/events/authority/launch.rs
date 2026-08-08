//! Stages and launches the verified Events credential authority child.

use std::path::Path;
use std::time::Duration;

use makosh_kernel_control_store::{
    PlatformEventHubTopologyV1, PlatformEventsAuthorityConfigurationV1,
    PlatformManagedProcessLaunch,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::{
    v1::EventsAuthorityRuntimeConfigurationV1,
    validation::events_authority::validate_events_authority_runtime_configuration,
};
use prost::Message;

use crate::distribution::staged_contracts::StagedRuntimeContracts;
use crate::platform::events::authority::{binding::EVENTS_AUTHORITY_PROCESS_ID, status};
use crate::platform::macos::native_launch;
use crate::platform::vault::status as vault_status;
use crate::runtime::lifecycle::control::ManagedRuntimeExpectation;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;
use crate::runtime::managed::execution::ManagedChildExecutionPolicy;

const EVENTS_AUTHORITY_MODULE_ID: &str = "events";
const MAX_ATTEMPTS: u8 = 3;
const MAX_RUNTIME: Duration = Duration::from_secs(300);

pub fn start(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
) -> Result<u64, String> {
    let kernel =
        std::env::current_exe().map_err(|_| "Kernel executable path is unavailable".to_owned())?;
    start_from_kernel(supervisor, store, &kernel, runtime_dir)
}

pub(crate) fn start_from_kernel(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel: &Path,
    runtime_dir: &Path,
) -> Result<u64, String> {
    ensure_inactive(supervisor)?;
    let binding = authority_binding(store)?;
    let configuration = status::current_configuration(store)?;
    let topology = status::current_topology(store)?;
    let vault = vault_status::read_current(store, &supervisor.relay_port())?;
    let runtime_generation = next_runtime_generation(store)?;
    let (prepared, contracts) = prepare_launch(EventsAuthorityLaunchInputV1 {
        kernel,
        binding: &binding,
        runtime_dir,
        vault_instance_id: store.snapshot().instance_id(),
        configuration: &configuration,
        topology: &topology,
        vault: &vault,
        runtime_generation,
    })?;
    let launch = PlatformManagedProcessLaunch::new(
        EVENTS_AUTHORITY_PROCESS_ID,
        binding.binding_revision(),
        store.snapshot().generation(),
        runtime_generation,
        store.snapshot().grant_epoch(),
    );
    if let Err(error) = store.record_platform_managed_process_launch(&launch) {
        let _ = contracts.remove();
        let _ = prepared.remove();
        return Err(format!("{error:?}"));
    }
    let expectation = ManagedRuntimeExpectation::from_platform_fenced_launch(
        EVENTS_AUTHORITY_PROCESS_ID,
        EVENTS_AUTHORITY_MODULE_ID,
        &binding,
        &launch,
    )?;
    supervisor.start_with_arguments_and_contracts(
        EVENTS_AUTHORITY_PROCESS_ID.to_owned(),
        prepared.into_staged_executable(),
        inherited_arguments(&contracts)?,
        expectation,
        ManagedChildExecutionPolicy::new(MAX_ATTEMPTS, MAX_RUNTIME)?,
        contracts,
    )?;
    supervisor.wait_until_ready(EVENTS_AUTHORITY_PROCESS_ID)?;
    match status::read_current(store, &supervisor.relay_port()) {
        Ok(status) if status.runtime_generation() == runtime_generation => Ok(runtime_generation),
        Ok(_) | Err(_) => {
            let _ = supervisor.stop(EVENTS_AUTHORITY_PROCESS_ID);
            Err("Events authority did not confirm its managed status".to_owned())
        }
    }
}

pub(crate) fn current_launch(
    store: &SqliteControlStore,
) -> Result<PlatformManagedProcessLaunch, String> {
    let binding = authority_binding(store)?;
    let launch = store
        .platform_managed_process_launch(EVENTS_AUTHORITY_PROCESS_ID)
        .map_err(|_| "Events authority runtime is unavailable".to_owned())?
        .ok_or_else(|| "Events authority runtime is unavailable".to_owned())?;
    if launch.binding_revision() != binding.binding_revision()
        || launch.kernel_generation() != store.snapshot().generation()
        || launch.grant_epoch() != store.snapshot().grant_epoch()
    {
        return Err("Events authority runtime is stale".to_owned());
    }
    Ok(launch)
}

fn ensure_inactive(supervisor: &ManagedRuntimeSupervisor) -> Result<(), String> {
    (!supervisor.is_active(EVENTS_AUTHORITY_PROCESS_ID)?)
        .then_some(())
        .ok_or_else(|| "Events authority runtime is already active".to_owned())
}

fn authority_binding(
    store: &SqliteControlStore,
) -> Result<makosh_kernel_control_store::PlatformManagedProcessBinding, String> {
    store
        .platform_managed_process_binding(EVENTS_AUTHORITY_PROCESS_ID)
        .map_err(|_| "Events authority release binding is unavailable".to_owned())?
        .ok_or_else(|| "Events authority release binding is unavailable".to_owned())
}

struct EventsAuthorityLaunchInputV1<'a> {
    kernel: &'a Path,
    binding: &'a makosh_kernel_control_store::PlatformManagedProcessBinding,
    runtime_dir: &'a Path,
    vault_instance_id: &'a str,
    configuration: &'a PlatformEventsAuthorityConfigurationV1,
    topology: &'a PlatformEventHubTopologyV1,
    vault: &'a vault_status::ManagedVaultStatus,
    runtime_generation: u64,
}

fn prepare_launch(
    input: EventsAuthorityLaunchInputV1<'_>,
) -> Result<
    (
        native_launch::PreparedPlatformManagedProcess,
        StagedRuntimeContracts,
    ),
    String,
> {
    let prepared = native_launch::prepare_bound_platform_process(
        input.kernel,
        input.binding,
        &input
            .runtime_dir
            .join("events")
            .join("authority")
            .join(format!("launch-{}", input.runtime_generation))
            .join("managed"),
    )?;
    let Some(settings_schema) = prepared.settings_schema_bytes() else {
        let _ = prepared.remove();
        return Err("Events authority release lacks a settings schema".to_owned());
    };
    let runtime_configuration = runtime_configuration(
        input.vault_instance_id,
        input.configuration,
        input.topology,
        input.vault,
    )?;
    match StagedRuntimeContracts::stage_with_runtime_configuration(
        &input
            .runtime_dir
            .join("events")
            .join("authority")
            .join(format!("launch-{}", input.runtime_generation))
            .join("contracts"),
        prepared.descriptor_bytes(),
        Some(settings_schema),
        Some(&runtime_configuration),
    ) {
        Ok(contracts) => Ok((prepared, contracts)),
        Err(error) => {
            let _ = prepared.remove();
            Err(error)
        }
    }
}

fn runtime_configuration(
    vault_instance_id: &str,
    configuration: &PlatformEventsAuthorityConfigurationV1,
    topology: &PlatformEventHubTopologyV1,
    vault: &vault_status::ManagedVaultStatus,
) -> Result<Vec<u8>, String> {
    let configuration = EventsAuthorityRuntimeConfigurationV1 {
        account_public_key: configuration.account_public_key().to_owned(),
        vault_instance_id: vault_instance_id.to_owned(),
        vault_runtime_generation: vault.runtime_generation(),
        vault_hpke_public_key_x25519: vault.hpke_public_key_x25519().to_vec(),
        signer_credential_revision: configuration.signer_credential_revision(),
        nats_endpoint: topology.nats_endpoint().to_owned(),
        nats_username: topology.nats_username().to_owned(),
        event_hub_credential_revision: topology.credential_revision(),
    };
    validate_events_authority_runtime_configuration(&configuration)
        .map_err(|_| "Events authority configuration is invalid".to_owned())?;
    Ok(configuration.encode_to_vec())
}

fn next_runtime_generation(store: &SqliteControlStore) -> Result<u64, String> {
    store
        .platform_managed_process_launch(EVENTS_AUTHORITY_PROCESS_ID)
        .map_err(|_| "Events authority runtime is unavailable".to_owned())?
        .map_or(Ok(1), |launch| {
            launch
                .runtime_generation()
                .checked_add(1)
                .ok_or_else(|| "Events authority runtime generation overflowed".to_owned())
        })
}

pub(crate) fn inherited_arguments(
    contracts: &StagedRuntimeContracts,
) -> Result<Vec<String>, String> {
    let schema_path = contracts
        .settings_schema_path()
        .ok_or_else(|| "Events authority settings schema is unavailable".to_owned())?;
    let configuration_path = contracts
        .runtime_configuration_path()
        .ok_or_else(|| "Events authority configuration is unavailable".to_owned())?;
    Ok(vec![
        "serve-inherited".to_owned(),
        "--descriptor-path".to_owned(),
        contracts.descriptor_path().display().to_string(),
        "--settings-schema-path".to_owned(),
        schema_path.display().to_string(),
        "--configuration-path".to_owned(),
        configuration_path.display().to_string(),
    ])
}
