//! Stages and launches the verified Blob service as a managed child.

use std::path::Path;
use std::time::Duration;

use makosh_kernel_control_store::PlatformManagedProcessLaunch;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::BlobRuntimeConfigurationV1;
use makosh_runtime_protocol::validation::blob::validate_blob_runtime_configuration;
use prost::Message;

use crate::distribution::staged_contracts::StagedRuntimeContracts;
use crate::identity::device::signer::{DeviceSigner, FileDeviceSigner};
use crate::infrastructure::filesystem::prepare_owner_private_directory;
use crate::platform::blob::binding::BLOB_PROCESS_ID;
use crate::platform::macos::native_launch;
use crate::platform::vault::status as vault_status;
use crate::runtime::lifecycle::control::ManagedRuntimeExpectation;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;
use crate::runtime::managed::execution::ManagedChildExecutionPolicy;

const BLOB_MODULE_ID: &str = "blob";
const MAX_ATTEMPTS: u8 = 3;
const MAX_RUNTIME: Duration = Duration::from_secs(300);
const MAXIMUM_BLOB_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const CUSTODY_RELEASE_GRACE_PERIOD_MS: u64 = 24 * 60 * 60 * 1_000;

#[must_use]
pub(crate) fn data_socket_path(runtime_dir: &Path) -> std::path::PathBuf {
    // Place runtime sockets under the short runtime cache path to stay below
    // platform pathname limits.
    runtime_dir.join("blob").join("data.sock")
}

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
    ensure_inactive(supervisor)?;
    let binding = blob_binding(store)?;
    let vault = vault_status::read_current(store, &supervisor.relay_port())?;
    let runtime_generation = next_runtime_generation(store)?;
    let (prepared, contracts) = prepare_launch(
        kernel,
        &binding,
        data_dir,
        runtime_dir,
        store.snapshot().instance_id(),
        &vault,
        runtime_generation,
    )?;
    let launch = PlatformManagedProcessLaunch::new(
        BLOB_PROCESS_ID,
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
        BLOB_PROCESS_ID,
        BLOB_MODULE_ID,
        &binding,
        &launch,
    )?;
    supervisor.start_with_arguments_and_contracts(
        BLOB_PROCESS_ID.to_owned(),
        prepared.into_staged_executable(),
        inherited_arguments(&contracts)?,
        expectation,
        ManagedChildExecutionPolicy::new(MAX_ATTEMPTS, MAX_RUNTIME)?,
        contracts,
    )?;
    supervisor.wait_until_ready(BLOB_PROCESS_ID)?;
    match super::status::read_current(store, &supervisor.relay_port()) {
        Ok(status) if status.runtime_generation() == runtime_generation => Ok(runtime_generation),
        Ok(_) | Err(_) => {
            let _ = supervisor.stop(BLOB_PROCESS_ID);
            Err("Blob runtime did not confirm its managed status".to_owned())
        }
    }
}

pub(crate) fn current_launch(
    store: &SqliteControlStore,
) -> Result<PlatformManagedProcessLaunch, String> {
    let binding = blob_binding(store)?;
    let launch = store
        .platform_managed_process_launch(BLOB_PROCESS_ID)
        .map_err(|_| "Blob runtime is unavailable".to_owned())?
        .ok_or_else(|| "Blob runtime is unavailable".to_owned())?;
    if launch.binding_revision() != binding.binding_revision()
        || launch.kernel_generation() != store.snapshot().generation()
        || launch.grant_epoch() != store.snapshot().grant_epoch()
    {
        return Err("Blob runtime is stale".to_owned());
    }
    Ok(launch)
}

fn ensure_inactive(supervisor: &ManagedRuntimeSupervisor) -> Result<(), String> {
    (!supervisor.is_active(BLOB_PROCESS_ID)?)
        .then_some(())
        .ok_or_else(|| "Blob runtime is already active".to_owned())
}

fn blob_binding(
    store: &SqliteControlStore,
) -> Result<makosh_kernel_control_store::PlatformManagedProcessBinding, String> {
    store
        .platform_managed_process_binding(BLOB_PROCESS_ID)
        .map_err(|_| "Blob release binding is unavailable".to_owned())?
        .ok_or_else(|| "Blob release binding is unavailable".to_owned())
}

fn prepare_launch(
    kernel: &Path,
    binding: &makosh_kernel_control_store::PlatformManagedProcessBinding,
    data_dir: &Path,
    runtime_dir: &Path,
    vault_instance_id: &str,
    vault: &vault_status::ManagedVaultStatus,
    runtime_generation: u64,
) -> Result<
    (
        native_launch::PreparedPlatformManagedProcess,
        StagedRuntimeContracts,
    ),
    String,
> {
    let service_data_dir = data_dir.join("blob");
    prepare_owner_private_directory(&service_data_dir)?;
    // Unix socket paths have a small kernel-defined byte limit.
    let data_socket_path = data_socket_path(runtime_dir);
    let prepared = native_launch::prepare_bound_platform_process(
        kernel,
        binding,
        &runtime_dir
            .join("blob")
            .join(format!("launch-{runtime_generation}"))
            .join("managed"),
    )?;
    let configuration = match runtime_configuration(
        &service_data_dir,
        &data_socket_path,
        data_dir,
        vault_instance_id,
        vault,
    ) {
        Ok(configuration) => configuration,
        Err(error) => {
            let _ = prepared.remove();
            return Err(error);
        }
    };
    match StagedRuntimeContracts::stage_with_runtime_configuration(
        &runtime_dir
            .join("blob")
            .join(format!("launch-{runtime_generation}"))
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

fn runtime_configuration(
    data_dir: &Path,
    data_socket_path: &Path,
    kernel_data_dir: &Path,
    vault_instance_id: &str,
    vault: &vault_status::ManagedVaultStatus,
) -> Result<Vec<u8>, String> {
    let signer = FileDeviceSigner::open_for_instance(kernel_data_dir)?;
    let configuration = BlobRuntimeConfigurationV1 {
        data_dir: data_dir.display().to_string(),
        maximum_blob_bytes: MAXIMUM_BLOB_BYTES,
        vault_instance_id: vault_instance_id.to_owned(),
        vault_runtime_generation: vault.runtime_generation(),
        vault_hpke_public_key_x25519: vault.hpke_public_key_x25519().to_vec(),
        data_socket_path: data_socket_path.display().to_string(),
        kernel_instance_id: vault_instance_id.to_owned(),
        kernel_authorization_public_key_sec1: signer.public_key_sec1().to_vec(),
        custody_release_grace_period_ms: CUSTODY_RELEASE_GRACE_PERIOD_MS,
    };
    validate_blob_runtime_configuration(&configuration)
        .map_err(|_| "Blob runtime configuration is invalid".to_owned())?;
    Ok(configuration.encode_to_vec())
}

fn next_runtime_generation(store: &SqliteControlStore) -> Result<u64, String> {
    store
        .platform_managed_process_launch(BLOB_PROCESS_ID)
        .map_err(|_| "Blob runtime is unavailable".to_owned())?
        .map_or(Ok(1), |launch| {
            launch
                .runtime_generation()
                .checked_add(1)
                .ok_or_else(|| "Blob runtime generation overflowed".to_owned())
        })
}

fn inherited_arguments(contracts: &StagedRuntimeContracts) -> Result<Vec<String>, String> {
    let configuration_path = contracts
        .runtime_configuration_path()
        .ok_or_else(|| "Blob runtime configuration is unavailable".to_owned())?;
    let mut arguments = vec![
        "serve-inherited".to_owned(),
        "--descriptor-path".to_owned(),
        contracts.descriptor_path().display().to_string(),
    ];
    if let Some(path) = contracts.settings_schema_path() {
        arguments.push("--settings-schema-path".to_owned());
        arguments.push(path.display().to_string());
    }
    arguments.push("--configuration-path".to_owned());
    arguments.push(configuration_path.display().to_string());
    Ok(arguments)
}
