//! Staged, fenced launch of the owner-authorized Telemetry Collector.

use std::path::Path;
use std::time::Duration;

use makosh_kernel_control_store::PlatformManagedProcessLaunch;
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use crate::distribution::staged_artifact::StagedNativeArtifact;
use crate::distribution::staged_contracts::StagedRuntimeContracts;
use crate::infrastructure::filesystem::prepare_owner_private_directory;
use crate::platform::macos::native_launch;
use crate::platform::telemetry::binding::TELEMETRY_PROCESS_ID;
use crate::runtime::lifecycle::control::ManagedRuntimeExpectation;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;
use crate::runtime::managed::execution::ManagedChildExecutionPolicy;

const TELEMETRY_MODULE_ID: &str = "telemetry";
const MAX_ATTEMPTS: u8 = 3;
const MAX_RUNTIME: Duration = Duration::from_secs(300);

pub fn start(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
) -> Result<u64, String> {
    let kernel =
        std::env::current_exe().map_err(|_| "Kernel executable path is unavailable".to_owned())?;
    start_from_kernel(supervisor, store, &kernel, data_dir, runtime_dir)
}

pub(crate) fn start_from_kernel(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel: &Path,
    data_dir: &Path,
    runtime_dir: &Path,
) -> Result<u64, String> {
    if supervisor.is_active(TELEMETRY_PROCESS_ID)? {
        return Err("Telemetry Collector is already active".to_owned());
    }
    let binding = store
        .platform_managed_process_binding(TELEMETRY_PROCESS_ID)
        .map_err(|_| "Telemetry release binding is unavailable".to_owned())?
        .ok_or_else(|| "Telemetry release binding is unavailable".to_owned())?;
    let generation = next_generation(store)?;
    let service_runtime_dir = prepare_service_directories(data_dir, runtime_dir)?;
    let (artifact, contracts) = prepare(kernel, &binding, runtime_dir, generation)?;
    let arguments = match inherited_arguments(data_dir, &service_runtime_dir, &contracts) {
        Ok(arguments) => arguments,
        Err(error) => {
            let _ = contracts.remove();
            let _ = artifact.remove();
            return Err(error);
        }
    };
    let launch = PlatformManagedProcessLaunch::new(
        TELEMETRY_PROCESS_ID,
        binding.binding_revision(),
        store.snapshot().generation(),
        generation,
        store.snapshot().grant_epoch(),
    );
    if let Err(error) = store.record_platform_managed_process_launch(&launch) {
        let _ = contracts.remove();
        let _ = artifact.remove();
        return Err(format!("{error:?}"));
    }
    let expectation = ManagedRuntimeExpectation::from_platform_fenced_launch(
        TELEMETRY_PROCESS_ID,
        TELEMETRY_MODULE_ID,
        &binding,
        &launch,
    )?;
    supervisor.start_with_arguments_and_contracts(
        TELEMETRY_PROCESS_ID.to_owned(),
        artifact,
        arguments,
        expectation,
        ManagedChildExecutionPolicy::new(MAX_ATTEMPTS, MAX_RUNTIME)?,
        contracts,
    )?;
    match supervisor.wait_until_ready(TELEMETRY_PROCESS_ID) {
        Ok(()) => Ok(generation),
        Err(error) => {
            let _ = supervisor.stop(TELEMETRY_PROCESS_ID);
            Err(error)
        }
    }
}

fn prepare_service_directories(
    data_dir: &Path,
    runtime_dir: &Path,
) -> Result<std::path::PathBuf, String> {
    let data_directory = data_dir.join("telemetry");
    prepare_owner_private_directory(&data_directory)?;
    // Use runtime_dir directly for the collector socket path:
    // "<runtime_dir>/telemetry.sock" keeps Unix socket paths below
    // platform pathname limits.
    let runtime_directory = runtime_dir.to_owned();
    prepare_owner_private_directory(&runtime_directory)?;
    Ok(runtime_directory)
}

fn prepare(
    kernel: &Path,
    binding: &makosh_kernel_control_store::PlatformManagedProcessBinding,
    runtime_dir: &Path,
    runtime_generation: u64,
) -> Result<(StagedNativeArtifact, StagedRuntimeContracts), String> {
    prepare_owner_private_directory(&runtime_dir.join("telemetry"))?;
    let prepared = native_launch::prepare_bound_platform_process(
        kernel,
        binding,
        &runtime_dir
            .join("telemetry")
            .join(format!("launch-{runtime_generation}"))
            .join("managed"),
    )?;
    let contracts = match StagedRuntimeContracts::stage(
        &runtime_dir
            .join("telemetry")
            .join(format!("launch-{runtime_generation}"))
            .join("contracts"),
        prepared.descriptor_bytes(),
        prepared.settings_schema_bytes(),
    ) {
        Ok(contracts) => contracts,
        Err(error) => {
            let _ = prepared.remove();
            return Err(error);
        }
    };
    Ok((prepared.into_staged_executable(), contracts))
}

fn next_generation(store: &SqliteControlStore) -> Result<u64, String> {
    store
        .platform_managed_process_launch(TELEMETRY_PROCESS_ID)
        .map_err(|_| "Telemetry runtime is unavailable".to_owned())?
        .map_or(Ok(1), |record| {
            record
                .runtime_generation()
                .checked_add(1)
                .ok_or_else(|| "Telemetry runtime generation overflowed".to_owned())
        })
}

fn inherited_arguments(
    data_dir: &Path,
    service_runtime_dir: &Path,
    contracts: &StagedRuntimeContracts,
) -> Result<Vec<String>, String> {
    let settings_schema_path = contracts
        .settings_schema_path()
        .ok_or_else(|| "Telemetry release artifact lacks a settings schema".to_owned())?;
    Ok(vec![
        "serve-inherited".to_owned(),
        "--data-dir".to_owned(),
        data_dir.join("telemetry").display().to_string(),
        "--runtime-dir".to_owned(),
        service_runtime_dir.display().to_string(),
        "--descriptor-path".to_owned(),
        contracts.descriptor_path().display().to_string(),
        "--settings-schema-path".to_owned(),
        settings_schema_path.display().to_string(),
    ])
}
