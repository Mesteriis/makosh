//! Prepares the owner-authorized Vault child from its durable release binding.

use std::path::Path;
use std::time::Duration;

use makosh_kernel_control_store::PlatformManagedProcessLaunch;
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use crate::distribution::staged_contracts::StagedRuntimeContracts;
use crate::identity::device::signer::{DeviceSigner, FileDeviceSigner};
use crate::platform::macos::native_launch;
use crate::platform::vault::binding::VAULT_PROCESS_ID;
use crate::platform::vault::status;
use crate::runtime::lifecycle::control::ManagedRuntimeExpectation;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;
use crate::runtime::managed::execution::ManagedChildExecutionPolicy;

const VAULT_MODULE_ID: &str = "vault";
const MAX_ATTEMPTS: u8 = 3;
const MAX_RUNTIME: Duration = Duration::from_secs(300);

pub fn current_launch(store: &SqliteControlStore) -> Result<PlatformManagedProcessLaunch, String> {
    let binding = store
        .platform_managed_process_binding(VAULT_PROCESS_ID)
        .map_err(|_| "Vault release binding is unavailable".to_owned())?
        .ok_or_else(|| "Vault release binding is unavailable".to_owned())?;
    let launch = store
        .platform_managed_process_launch(VAULT_PROCESS_ID)
        .map_err(|_| "Vault runtime is unavailable".to_owned())?
        .ok_or_else(|| "Vault runtime is unavailable".to_owned())?;
    if launch.binding_revision() != binding.binding_revision()
        || launch.kernel_generation() != store.snapshot().generation()
        || launch.grant_epoch() != store.snapshot().grant_epoch()
    {
        return Err("Vault runtime is stale".to_owned());
    }
    Ok(launch)
}

pub fn start(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
) -> Result<u64, String> {
    let kernel_executable =
        std::env::current_exe().map_err(|_| "Kernel executable path is unavailable".to_owned())?;
    start_from_kernel(supervisor, store, data_dir, &kernel_executable, runtime_dir)
}

pub(crate) fn start_from_kernel(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    data_dir: &Path,
    kernel_executable: &Path,
    runtime_dir: &Path,
) -> Result<u64, String> {
    if supervisor.is_active(VAULT_PROCESS_ID)? {
        return Err("Vault runtime is already active".to_owned());
    }
    let binding = vault_binding(store)?;
    let runtime_generation = next_runtime_generation(store)?;
    let (prepared, contracts) =
        prepare_launch(kernel_executable, &binding, runtime_dir, runtime_generation)?;
    let record = PlatformManagedProcessLaunch::new(
        VAULT_PROCESS_ID,
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
        VAULT_PROCESS_ID,
        VAULT_MODULE_ID,
        &binding,
        &record,
    )?;
    let arguments = inherited_arguments(
        data_dir,
        store.snapshot().instance_id(),
        runtime_generation,
        &contracts,
        FileDeviceSigner::open_for_instance(data_dir)?.public_key_sec1(),
    );
    supervisor.start_with_arguments_and_contracts(
        VAULT_PROCESS_ID.to_owned(),
        prepared.into_staged_executable(),
        arguments,
        expectation,
        ManagedChildExecutionPolicy::new(MAX_ATTEMPTS, MAX_RUNTIME)?,
        contracts,
    )?;
    supervisor.wait_until_ready(VAULT_PROCESS_ID)?;
    match status::read_current(store, &supervisor.relay_port()) {
        Ok(status) if status.runtime_generation() == runtime_generation => Ok(runtime_generation),
        Ok(_) | Err(_) => {
            let _ = supervisor.stop(VAULT_PROCESS_ID);
            Err("Vault runtime did not confirm its managed status".to_owned())
        }
    }
}

fn vault_binding(
    store: &SqliteControlStore,
) -> Result<makosh_kernel_control_store::PlatformManagedProcessBinding, String> {
    store
        .platform_managed_process_binding(VAULT_PROCESS_ID)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "Vault release binding is unavailable".to_owned())
}

fn prepare_launch(
    kernel_executable: &Path,
    binding: &makosh_kernel_control_store::PlatformManagedProcessBinding,
    runtime_dir: &Path,
    runtime_generation: u64,
) -> Result<
    (
        native_launch::PreparedPlatformManagedProcess,
        StagedRuntimeContracts,
    ),
    String,
> {
    let prepared = native_launch::prepare_bound_platform_process(
        kernel_executable,
        binding,
        &runtime_dir
            .join("vault")
            .join(format!("launch-{runtime_generation}"))
            .join("managed"),
    )?;
    match StagedRuntimeContracts::stage(
        &runtime_dir
            .join("vault")
            .join(format!("launch-{runtime_generation}"))
            .join("contracts"),
        prepared.descriptor_bytes(),
        prepared.settings_schema_bytes(),
    ) {
        Ok(contracts) => Ok((prepared, contracts)),
        Err(error) => {
            let _ = prepared.remove();
            Err(error)
        }
    }
}

fn next_runtime_generation(store: &SqliteControlStore) -> Result<u64, String> {
    store
        .platform_managed_process_launch(VAULT_PROCESS_ID)
        .map_err(|error| format!("{error:?}"))?
        .map_or(Ok(1), |record| {
            record
                .runtime_generation()
                .checked_add(1)
                .ok_or_else(|| "Vault runtime generation overflowed".to_owned())
        })
}

fn inherited_arguments(
    data_dir: &Path,
    instance_id: &str,
    runtime_generation: u64,
    contracts: &StagedRuntimeContracts,
    authorization_key_sec1: [u8; 65],
) -> Vec<String> {
    let mut arguments = vec![
        "serve-inherited".to_owned(),
        "--data-dir".to_owned(),
        data_dir.join("vault").display().to_string(),
        "--instance-id".to_owned(),
        instance_id.to_owned(),
        "--runtime-generation".to_owned(),
        runtime_generation.to_string(),
        "--descriptor-path".to_owned(),
        contracts.descriptor_path().display().to_string(),
    ];
    arguments.push("--kernel-authorization-key-sec1-hex".to_owned());
    arguments.push(
        authorization_key_sec1
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    );
    if let Some(path) = contracts.settings_schema_path() {
        arguments.push("--settings-schema-path".to_owned());
        arguments.push(path.display().to_string());
    }
    arguments
}
