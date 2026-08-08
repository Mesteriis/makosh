//! Prepares the fenced Storage child from its exact signed platform binding.

use std::path::Path;
use std::time::Duration;

use makosh_kernel_control_store::PlatformManagedProcessLaunch;
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use crate::distribution::staged_contracts::StagedRuntimeContracts;
use crate::infrastructure::filesystem::prepare_owner_private_directory;
use crate::platform::macos::native_launch;
use crate::platform::storage::authorization::authorize_managed_binding;
use crate::platform::storage::binding::STORAGE_PROCESS_ID;
use crate::platform::storage::status;
use crate::platform::storage::topology;
use crate::platform::vault::status as vault_status;
use crate::runtime::lifecycle::control::ManagedRuntimeExpectation;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;
use crate::runtime::managed::execution::ManagedChildExecutionPolicy;

const STORAGE_MODULE_ID: &str = "storage";
const MAX_ATTEMPTS: u8 = 3;
const MAX_RUNTIME: Duration = Duration::from_secs(300);
const READY_BASE_SECONDS: u64 = 15;
const READY_SECONDS_PER_BINDING: u64 = 2;
const READY_MAX_SECONDS: u64 = 120;

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
    let binding = storage_binding(store)?;
    let topology = topology::current(store)?;
    let (desired_bindings, desired_bundles) = desired_configuration(store)?;
    let vault = vault_status::read_current(store, &supervisor.relay_port())?;
    let runtime_generation = next_runtime_generation(store)?;
    let (prepared, contracts) = prepare_launch(StorageLaunchInputV1 {
        kernel,
        binding: &binding,
        topology: &topology,
        desired_bindings: &desired_bindings,
        desired_bundles: &desired_bundles,
        vault_instance_id: store.snapshot().instance_id(),
        vault: &vault,
        runtime_dir,
        runtime_generation,
    })?;
    let record = PlatformManagedProcessLaunch::new(
        STORAGE_PROCESS_ID,
        binding.binding_revision(),
        store.snapshot().generation(),
        runtime_generation,
        store.snapshot().grant_epoch(),
    );
    if let Err(error) = store.record_platform_managed_process_launch(&record) {
        let _ = contracts.remove();
        let _ = prepared.remove();
        return Err(format!("{error:?}"));
    }
    let expectation = ManagedRuntimeExpectation::from_platform_fenced_launch(
        STORAGE_PROCESS_ID,
        STORAGE_MODULE_ID,
        &binding,
        &record,
    )?;
    supervisor.start_with_arguments_and_contracts(
        STORAGE_PROCESS_ID.to_owned(),
        prepared.into_staged_executable(),
        inherited_arguments(&contracts),
        expectation,
        ManagedChildExecutionPolicy::new(MAX_ATTEMPTS, MAX_RUNTIME)?,
        contracts,
    )?;
    supervisor.wait_until_ready_with_timeout(
        STORAGE_PROCESS_ID,
        storage_ready_timeout(desired_bindings.len()),
    )?;
    match status::wait_current(store, &supervisor.relay_port()) {
        Ok(status) if status.runtime_generation() == runtime_generation => Ok(runtime_generation),
        Ok(_) | Err(_) => {
            let _ = supervisor.stop(STORAGE_PROCESS_ID);
            Err("Storage runtime did not confirm its managed status".to_owned())
        }
    }
}

fn storage_ready_timeout(active_binding_count: usize) -> Duration {
    let active_binding_count = u64::try_from(active_binding_count).unwrap_or(u64::MAX);
    Duration::from_secs(
        READY_BASE_SECONDS
            .saturating_add(active_binding_count.saturating_mul(READY_SECONDS_PER_BINDING))
            .min(READY_MAX_SECONDS),
    )
}

fn desired_configuration(
    store: &SqliteControlStore,
) -> Result<
    (
        Vec<makosh_kernel_control_store::PlatformStorageBindingV1>,
        Vec<makosh_kernel_control_store::PlatformStorageBundleV1>,
    ),
    String,
> {
    let bindings = store
        .platform_storage_bindings()
        .map_err(|_| "Storage bindings are unavailable".to_owned())?
        .into_iter()
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .collect::<Vec<_>>();
    validate_desired_bindings(store, &bindings)?;
    load_bundles(store, &bindings).map(|bundles| (bindings, bundles))
}

fn ensure_inactive(supervisor: &ManagedRuntimeSupervisor) -> Result<(), String> {
    (!supervisor.is_active(STORAGE_PROCESS_ID)?)
        .then_some(())
        .ok_or_else(|| "Storage runtime is already active".to_owned())
}

fn load_bundles(
    store: &SqliteControlStore,
    bindings: &[makosh_kernel_control_store::PlatformStorageBindingV1],
) -> Result<Vec<makosh_kernel_control_store::PlatformStorageBundleV1>, String> {
    bindings
        .iter()
        .map(|binding| {
            store
                .platform_storage_bundle(binding.owner_id(), binding.storage_bundle_revision())
                .map_err(|_| "Storage bundle is unavailable".to_owned())?
                .ok_or_else(|| "Storage bundle is unavailable".to_owned())
        })
        .collect()
}

fn validate_desired_bindings(
    store: &SqliteControlStore,
    desired_bindings: &[makosh_kernel_control_store::PlatformStorageBindingV1],
) -> Result<(), String> {
    for binding in desired_bindings {
        let current = authorize_managed_binding(
            store,
            binding.registration_id(),
            binding.runtime_instance_id(),
            binding.runtime_generation(),
            binding.capability_id(),
        )?;
        if current.grant_epoch() != binding.grant_epoch()
            || current.owner_id() != binding.owner_id()
            || current.connection_budget() != binding.connection_budget()
            || current.statement_timeout_millis() != binding.statement_timeout_millis()
        {
            return Err("Storage binding authorization is stale".to_owned());
        }
    }
    Ok(())
}

pub(crate) fn current_launch(
    store: &SqliteControlStore,
) -> Result<PlatformManagedProcessLaunch, String> {
    let binding = storage_binding(store)?;
    let launch = store
        .platform_managed_process_launch(STORAGE_PROCESS_ID)
        .map_err(|_| "Storage runtime is unavailable".to_owned())?
        .ok_or_else(|| "Storage runtime is unavailable".to_owned())?;
    if launch.binding_revision() != binding.binding_revision()
        || launch.kernel_generation() != store.snapshot().generation()
        || launch.grant_epoch() != store.snapshot().grant_epoch()
    {
        return Err("Storage runtime is stale".to_owned());
    }
    Ok(launch)
}

fn storage_binding(
    store: &SqliteControlStore,
) -> Result<makosh_kernel_control_store::PlatformManagedProcessBinding, String> {
    store
        .platform_managed_process_binding(STORAGE_PROCESS_ID)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "Storage release binding is unavailable".to_owned())
}

struct StorageLaunchInputV1<'a> {
    kernel: &'a Path,
    binding: &'a makosh_kernel_control_store::PlatformManagedProcessBinding,
    topology: &'a makosh_kernel_control_store::PlatformStorageTopology,
    desired_bindings: &'a [makosh_kernel_control_store::PlatformStorageBindingV1],
    desired_bundles: &'a [makosh_kernel_control_store::PlatformStorageBundleV1],
    vault_instance_id: &'a str,
    vault: &'a vault_status::ManagedVaultStatus,
    runtime_dir: &'a Path,
    runtime_generation: u64,
}

fn prepare_launch(
    input: StorageLaunchInputV1<'_>,
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
            .join("storage")
            .join(format!("launch-{}", input.runtime_generation))
            .join("managed"),
    )?;
    let (pgbouncer_directory, pgbouncer_auth_directory) =
        match prepare_pgbouncer_directories(input.runtime_dir) {
            Ok(directories) => directories,
            Err(error) => {
                let _ = prepared.remove();
                return Err(error);
            }
        };
    let configuration =
        topology::encoded_managed_macos(topology::ManagedStorageConfigurationInputV1 {
            topology: input.topology,
            bindings: input.desired_bindings,
            bundles: input.desired_bundles,
            pgbouncer_database_config_path: &pgbouncer_directory.join("databases.ini"),
            pgbouncer_auth_file_path: &pgbouncer_auth_directory.join("users.txt"),
            vault_instance_id: input.vault_instance_id,
            vault_runtime_generation: input.vault.runtime_generation(),
            vault_hpke_public_key_x25519: input.vault.hpke_public_key_x25519(),
        })?;
    match StagedRuntimeContracts::stage_with_runtime_configuration(
        &input
            .runtime_dir
            .join("storage")
            .join(format!("launch-{}", input.runtime_generation))
            .join("contracts"),
        prepared.descriptor_bytes(),
        prepared.settings_schema_bytes(),
        Some(&configuration),
    ) {
        Ok(contracts) => Ok((prepared, contracts)),
        Err(error) => {
            let _ = prepared.remove();
            Err(error)
        }
    }
}

fn prepare_pgbouncer_directories(
    runtime_dir: &Path,
) -> Result<(std::path::PathBuf, std::path::PathBuf), String> {
    let pgbouncer_directory = runtime_dir.join("storage").join("pgbouncer");
    prepare_owner_private_directory(&pgbouncer_directory)?;
    let pgbouncer_auth_directory = pgbouncer_directory.join("auth");
    prepare_owner_private_directory(&pgbouncer_auth_directory)?;
    Ok((pgbouncer_directory, pgbouncer_auth_directory))
}

fn next_runtime_generation(store: &SqliteControlStore) -> Result<u64, String> {
    store
        .platform_managed_process_launch(STORAGE_PROCESS_ID)
        .map_err(|error| format!("{error:?}"))?
        .map_or(Ok(1), |record| {
            record
                .runtime_generation()
                .checked_add(1)
                .ok_or_else(|| "Storage runtime generation overflowed".to_owned())
        })
}

pub(crate) fn inherited_arguments(contracts: &StagedRuntimeContracts) -> Vec<String> {
    let mut arguments = vec![
        "serve-inherited".to_owned(),
        "--descriptor-path".to_owned(),
        contracts.descriptor_path().display().to_string(),
    ];
    if let Some(path) = contracts.settings_schema_path() {
        arguments.push("--settings-schema-path".to_owned());
        arguments.push(path.display().to_string());
    }
    if let Some(path) = contracts.runtime_configuration_path() {
        arguments.push("--configuration-path".to_owned());
        arguments.push(path.display().to_string());
    }
    arguments
}

#[cfg(test)]
mod tests {
    use super::{READY_MAX_SECONDS, storage_ready_timeout};

    #[test]
    fn readiness_deadline_is_bounded_by_exact_storage_workload() {
        assert_eq!(storage_ready_timeout(0).as_secs(), 15);
        assert_eq!(storage_ready_timeout(16).as_secs(), 47);
        assert_eq!(
            storage_ready_timeout(usize::MAX).as_secs(),
            READY_MAX_SECONDS
        );
    }
}
