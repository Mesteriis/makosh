//! Prepares one fenced macOS managed runtime launch and hands it to the Kernel supervisor.

use std::path::{Path, PathBuf};
use std::time::Duration;

use makosh_kernel_control_store::{ManagedLaunchRecord, PlatformStorageBindingStateV1};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::{
    managed_control::select_managed_control_transport,
    v1::{
        ManagedDomainRuntimeConfigurationV1, ManagedEngineRuntimeConfigurationV1,
        ManagedIntegrationHostBridgeConfigurationV1, ManagedIntegrationRuntimeConfigurationV1,
        ManagedStorageRuntimeConfigurationV1, ManagedWorkflowRuntimeConfigurationV1, ModuleKindV1,
    },
    validation::{
        descriptor::{decode_descriptor_v1, decode_settings_snapshot_v1},
        integration_host_bridge::validate_managed_integration_host_bridge_configuration,
        managed_domain_runtime::validate_managed_domain_runtime_configuration,
        managed_engine_runtime::validate_managed_engine_runtime_configuration,
        managed_integration_runtime::validate_managed_integration_runtime_configuration,
        managed_workflow_runtime::validate_managed_workflow_runtime_configuration,
    },
};
use prost::Message;

use crate::distribution::staged_artifact::StagedNativeArtifact;
use crate::distribution::staged_contracts::StagedRuntimeContracts;
use crate::infrastructure::filesystem::new_instance_id;
use crate::platform::macos::host_bridge_descriptor;
use crate::platform::macos::native_launch;
use crate::platform::{storage, vault::status as vault_status};
use crate::runtime::lifecycle::control::ManagedRuntimeExpectation;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;
use crate::runtime::managed::execution::ManagedChildExecutionPolicy;

const MAX_ATTEMPTS: u8 = 3;
const MAX_RUNTIME: Duration = Duration::from_secs(300);

pub(crate) struct ManagedLaunchReservation {
    registration_id: String,
    binding: makosh_kernel_control_store::BundledManagedLaunchBinding,
    record: ManagedLaunchRecord,
    expectation: ManagedRuntimeExpectation,
    policy: ManagedChildExecutionPolicy,
}

pub(crate) struct ManagedIntegrationLaunchConfiguration<'a> {
    pub runtime: ManagedIntegrationRuntimeConfigurationV1,
    pub settings_snapshot_bytes: Vec<u8>,
    pub granted_capability_ids: &'a [String],
}

struct PreparedRuntimeContractInput {
    runtime_configuration_bytes: Vec<u8>,
    settings_snapshot_bytes: Option<Vec<u8>>,
    host_bridge_configuration: Option<ManagedIntegrationHostBridgeConfigurationV1>,
    cleanup: Option<Box<dyn FnOnce() + Send>>,
    expected_module_kind: ModuleKindV1,
}

impl ManagedLaunchReservation {
    #[must_use]
    pub(crate) fn registration_id(&self) -> &str {
        &self.registration_id
    }

    #[must_use]
    pub(crate) fn binding(&self) -> &makosh_kernel_control_store::BundledManagedLaunchBinding {
        &self.binding
    }

    #[must_use]
    pub(crate) fn runtime_instance_id(&self) -> &str {
        self.record.runtime_instance_id()
    }

    #[must_use]
    pub(crate) fn runtime_generation(&self) -> u64 {
        self.record.runtime_generation()
    }

    #[must_use]
    pub(crate) fn grant_epoch(&self) -> u64 {
        self.record.grant_epoch()
    }

    pub(crate) fn into_launch_parts(
        self,
    ) -> (
        String,
        ManagedRuntimeExpectation,
        ManagedChildExecutionPolicy,
    ) {
        (self.registration_id, self.expectation, self.policy)
    }

    /// Scheduler restarts need a successor runtime identity and cannot reuse this reservation.
    pub(crate) fn into_single_attempt_launch_parts(
        self,
    ) -> Result<
        (
            String,
            ManagedRuntimeExpectation,
            ManagedChildExecutionPolicy,
        ),
        String,
    > {
        let policy = ManagedChildExecutionPolicy::new(1, self.policy.max_runtime())?;
        Ok((self.registration_id, self.expectation, policy))
    }
}

pub fn start(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    registration_id: &str,
) -> Result<u64, String> {
    let reservation = reserve(supervisor, store, registration_id)?;
    cleanup_abandoned_launch_directory(&managed_launch_directory(runtime_dir, &reservation))?;
    let kernel_executable = selected_kernel_executable()?;
    let staged = native_launch::stage_bound_installed_release(
        &kernel_executable,
        reservation.binding(),
        &managed_launch_directory(runtime_dir, &reservation),
    )?;
    let runtime_generation = reservation.runtime_generation();
    let (registration_id, expectation, policy) = reservation.into_launch_parts();
    supervisor.start(registration_id, staged, expectation, policy)?;
    Ok(runtime_generation)
}

fn selected_kernel_executable() -> Result<std::path::PathBuf, String> {
    #[cfg(test)]
    if let Some(executable) = std::env::var_os("MAKOSH_TEST_KERNEL_EXECUTABLE") {
        return Ok(std::path::PathBuf::from(executable));
    }
    std::env::current_exe().map_err(|_| "Kernel executable path is unavailable".to_owned())
}

/// Verifies the exact signed descriptor kind without reserving, fencing or
/// otherwise mutating managed runtime state.
pub(crate) fn require_bound_module_kind(
    store: &SqliteControlStore,
    runtime_dir: &Path,
    registration_id: &str,
    expected_module_kind: ModuleKindV1,
) -> Result<(), String> {
    let registration = store
        .module_registration(registration_id)
        .map_err(|_| "managed runtime registration is unavailable".to_owned())?
        .ok_or_else(|| "managed runtime registration is unavailable".to_owned())?;
    let binding = store
        .effective_bundled_managed_launch_binding(registration_id)
        .map_err(|_| "managed launch binding is unavailable".to_owned())?
        .ok_or_else(|| "managed launch binding is unavailable".to_owned())?;
    let prepared = native_launch::prepare_bound_managed_runtime(
        &selected_kernel_executable()?,
        &binding,
        &runtime_dir
            .join("managed")
            .join(format!("kind-preflight-{}", new_instance_id()?)),
    )?;
    let result = decode_descriptor_v1(prepared.descriptor_bytes())
        .map_err(|_| "managed runtime descriptor is invalid".to_owned())
        .and_then(|descriptor| {
            if descriptor.module_id != registration.module_id()
                || descriptor.module_kind != expected_module_kind as i32
            {
                Err("managed runtime module kind does not match launch path".to_owned())
            } else {
                Ok(())
            }
        });
    let cleanup = prepared.remove();
    result.and(cleanup)
}

pub fn start_with_storage_configuration(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    registration_id: &str,
) -> Result<u64, String> {
    let reservation = reserve(supervisor, store, registration_id)?;
    let storage_binding = store
        .platform_storage_bindings()
        .map_err(|_| "managed runtime Storage binding is unavailable".to_owned())?
        .into_iter()
        .find(|binding| {
            binding.registration_id() == registration_id
                && binding.state() == PlatformStorageBindingStateV1::Active
        })
        .ok_or_else(|| "managed runtime Storage binding is unavailable".to_owned())?;
    let topology = storage::topology::current(store)?;
    let vault = vault_status::read_current(store, &supervisor.relay_port())?;
    let configuration = storage::topology::to_managed_runtime_configuration(
        &topology,
        &storage_binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )?;
    start_staged_with_configuration(supervisor, store, runtime_dir, reservation, configuration)
}

fn start_staged_with_configuration(
    supervisor: &ManagedRuntimeSupervisor,
    _store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: ManagedLaunchReservation,
    configuration: ManagedStorageRuntimeConfigurationV1,
) -> Result<u64, String> {
    start_staged_with_configurations(supervisor, runtime_dir, reservation, configuration, None)
}

/// Starts one already-reserved provider integration from a Kernel-staged,
/// provider-neutral configuration. Provider settings and credentials are not
/// represented here.
pub(crate) fn start_reserved_integration(
    supervisor: &ManagedRuntimeSupervisor,
    data_dir: &Path,
    runtime_dir: &Path,
    reservation: ManagedLaunchReservation,
    launch: ManagedIntegrationLaunchConfiguration<'_>,
) -> Result<u64, String> {
    let ManagedIntegrationLaunchConfiguration {
        runtime: mut configuration,
        settings_snapshot_bytes,
        granted_capability_ids,
    } = launch;
    if configuration.registration_id != reservation.registration_id()
        || configuration.runtime_instance_id != reservation.runtime_instance_id()
        || configuration.runtime_generation != reservation.runtime_generation()
        || configuration.grant_epoch != reservation.grant_epoch()
    {
        return Err("managed integration runtime configuration is stale".to_owned());
    }
    cleanup_abandoned_launch_directory(&managed_launch_directory(runtime_dir, &reservation))?;
    let prepared = prepare_integration_runtime(
        data_dir,
        runtime_dir,
        &reservation,
        &mut configuration,
        granted_capability_ids,
    )?;
    if validate_managed_integration_runtime_configuration(&configuration).is_err() {
        prepared.remove();
        return Err("managed integration runtime configuration is invalid".to_owned());
    }
    let (prepared_runtime, staged_runtime_artifacts) = prepared.into_launch_parts();
    start_prepared_with_configuration_bytes(
        supervisor,
        runtime_dir,
        reservation,
        prepared_runtime,
        PreparedRuntimeContractInput {
            runtime_configuration_bytes: configuration.encode_to_vec(),
            settings_snapshot_bytes: Some(settings_snapshot_bytes),
            host_bridge_configuration: None,
            cleanup: staged_runtime_artifact_cleanup(staged_runtime_artifacts),
            expected_module_kind: ModuleKindV1::Integration,
        },
    )
}

/// Starts one already-reserved business domain from a Kernel-staged domain
/// configuration. It is deliberately separate from integration launch so no
/// provider configuration instance or host bridge can enter a domain runtime.
pub(crate) fn start_reserved_domain(
    supervisor: &ManagedRuntimeSupervisor,
    runtime_dir: &Path,
    reservation: ManagedLaunchReservation,
    configuration: ManagedDomainRuntimeConfigurationV1,
) -> Result<u64, String> {
    validate_managed_domain_runtime_configuration(&configuration)
        .map_err(|_| "managed domain runtime configuration is invalid".to_owned())?;
    if configuration.registration_id != reservation.registration_id()
        || configuration.runtime_instance_id != reservation.runtime_instance_id()
        || configuration.runtime_generation != reservation.runtime_generation()
        || configuration.grant_epoch != reservation.grant_epoch()
    {
        return Err("managed domain runtime configuration is stale".to_owned());
    }
    start_staged_with_configuration_bytes(
        supervisor,
        runtime_dir,
        reservation,
        PreparedRuntimeContractInput {
            runtime_configuration_bytes: configuration.encode_to_vec(),
            settings_snapshot_bytes: None,
            host_bridge_configuration: None,
            cleanup: None,
            expected_module_kind: ModuleKindV1::Domain,
        },
    )
}

/// Starts one already-reserved engine from a Kernel-staged engine
/// configuration and typed settings snapshot. Provider configuration,
/// integration state and host bridges are not representable on this path.
pub(crate) fn start_reserved_engine(
    supervisor: &ManagedRuntimeSupervisor,
    runtime_dir: &Path,
    reservation: ManagedLaunchReservation,
    mut configuration: ManagedEngineRuntimeConfigurationV1,
    settings_snapshot_bytes: Vec<u8>,
    granted_capability_ids: &[String],
) -> Result<u64, String> {
    if configuration.registration_id != reservation.registration_id()
        || configuration.runtime_instance_id != reservation.runtime_instance_id()
        || configuration.runtime_generation != reservation.runtime_generation()
        || configuration.grant_epoch != reservation.grant_epoch()
    {
        return Err("managed engine runtime configuration is stale".to_owned());
    }
    cleanup_abandoned_launch_directory(&managed_launch_directory(runtime_dir, &reservation))?;
    let prepared = prepare_runtime_with_artifacts(
        runtime_dir,
        &reservation,
        granted_capability_ids,
        "managed engine",
    )?;
    configuration.runtime_artifacts = prepared.runtime_artifact_bindings().to_vec();
    if validate_managed_engine_runtime_configuration(&configuration).is_err() {
        prepared.remove();
        return Err("managed engine runtime configuration is invalid".to_owned());
    }
    let (prepared_runtime, staged_runtime_artifacts) = prepared.into_launch_parts();
    start_prepared_with_configuration_bytes(
        supervisor,
        runtime_dir,
        reservation,
        prepared_runtime,
        PreparedRuntimeContractInput {
            runtime_configuration_bytes: configuration.encode_to_vec(),
            settings_snapshot_bytes: Some(settings_snapshot_bytes),
            host_bridge_configuration: None,
            cleanup: staged_runtime_artifact_cleanup(staged_runtime_artifacts),
            expected_module_kind: ModuleKindV1::Engine,
        },
    )
}

/// Starts one already-reserved workflow from its own provider-neutral
/// configuration. Workflow launch is not an alias for domain or integration
/// launch and cannot carry settings snapshots or host-bridge routes.
pub(crate) fn start_reserved_workflow(
    supervisor: &ManagedRuntimeSupervisor,
    runtime_dir: &Path,
    reservation: ManagedLaunchReservation,
    configuration: ManagedWorkflowRuntimeConfigurationV1,
    granted_capability_ids: &[String],
) -> Result<u64, String> {
    start_reserved_workflow_inner(
        supervisor,
        runtime_dir,
        reservation,
        configuration,
        None,
        granted_capability_ids,
    )
}

pub(crate) fn start_reserved_workflow_with_settings(
    supervisor: &ManagedRuntimeSupervisor,
    runtime_dir: &Path,
    reservation: ManagedLaunchReservation,
    configuration: ManagedWorkflowRuntimeConfigurationV1,
    settings_snapshot_bytes: Vec<u8>,
    granted_capability_ids: &[String],
) -> Result<u64, String> {
    start_reserved_workflow_inner(
        supervisor,
        runtime_dir,
        reservation,
        configuration,
        Some(settings_snapshot_bytes),
        granted_capability_ids,
    )
}

fn start_reserved_workflow_inner(
    supervisor: &ManagedRuntimeSupervisor,
    runtime_dir: &Path,
    reservation: ManagedLaunchReservation,
    mut configuration: ManagedWorkflowRuntimeConfigurationV1,
    settings_snapshot_bytes: Option<Vec<u8>>,
    granted_capability_ids: &[String],
) -> Result<u64, String> {
    if configuration.registration_id != reservation.registration_id()
        || configuration.runtime_instance_id != reservation.runtime_instance_id()
        || configuration.runtime_generation != reservation.runtime_generation()
        || configuration.grant_epoch != reservation.grant_epoch()
    {
        return Err("managed workflow runtime configuration is stale".to_owned());
    }
    cleanup_abandoned_launch_directory(&managed_launch_directory(runtime_dir, &reservation))?;
    let prepared = prepare_runtime_with_artifacts(
        runtime_dir,
        &reservation,
        granted_capability_ids,
        "managed workflow",
    )?;
    configuration.runtime_artifacts = prepared.runtime_artifact_bindings().to_vec();
    if validate_managed_workflow_runtime_configuration(&configuration).is_err() {
        prepared.remove();
        return Err("managed workflow runtime configuration is invalid".to_owned());
    }
    let (prepared_runtime, staged_runtime_artifacts) = prepared.into_launch_parts();
    start_prepared_with_configuration_bytes(
        supervisor,
        runtime_dir,
        reservation,
        prepared_runtime,
        PreparedRuntimeContractInput {
            runtime_configuration_bytes: configuration.encode_to_vec(),
            settings_snapshot_bytes,
            host_bridge_configuration: None,
            cleanup: staged_runtime_artifact_cleanup(staged_runtime_artifacts),
            expected_module_kind: ModuleKindV1::Workflow,
        },
    )
}

pub(crate) fn start_staged_with_host_bridge_configuration(
    supervisor: &ManagedRuntimeSupervisor,
    data_dir: &Path,
    runtime_dir: &Path,
    reservation: ManagedLaunchReservation,
    launch: ManagedIntegrationLaunchConfiguration<'_>,
    host_bridge_configuration: ManagedIntegrationHostBridgeConfigurationV1,
) -> Result<u64, String> {
    let ManagedIntegrationLaunchConfiguration {
        runtime: mut configuration,
        settings_snapshot_bytes,
        granted_capability_ids,
    } = launch;
    validate_managed_integration_host_bridge_configuration(&host_bridge_configuration)
        .map_err(|_| "managed integration host bridge configuration is invalid".to_owned())?;
    if configuration.registration_id != reservation.registration_id()
        || configuration.runtime_instance_id != reservation.runtime_instance_id()
        || configuration.runtime_generation != reservation.runtime_generation()
        || configuration.grant_epoch != reservation.grant_epoch()
        || host_bridge_configuration.registration_id != reservation.registration_id()
        || host_bridge_configuration.runtime_instance_id != reservation.runtime_instance_id()
        || host_bridge_configuration.runtime_generation != reservation.runtime_generation()
        || host_bridge_configuration.grant_epoch != reservation.grant_epoch()
    {
        return Err("managed integration host bridge configuration is stale".to_owned());
    }
    cleanup_abandoned_launch_directory(&managed_launch_directory(runtime_dir, &reservation))?;
    let prepared = prepare_integration_runtime(
        data_dir,
        runtime_dir,
        &reservation,
        &mut configuration,
        granted_capability_ids,
    )?;
    if validate_managed_integration_runtime_configuration(&configuration).is_err() {
        prepared.remove();
        return Err("managed integration runtime configuration is invalid".to_owned());
    }
    let descriptor = match host_bridge_descriptor::publish(runtime_dir, &host_bridge_configuration)
    {
        Ok(descriptor) => descriptor,
        Err(error) => {
            prepared.remove();
            return Err(error);
        }
    };
    let host_bridge_socket_path = PathBuf::from(&host_bridge_configuration.socket_path);
    let (prepared_runtime, staged_runtime_artifacts) = prepared.into_launch_parts();
    let cleanup = combine_cleanup(
        staged_runtime_artifact_cleanup(staged_runtime_artifacts),
        Some(Box::new(move || {
            descriptor.remove(&host_bridge_socket_path);
        })),
    );
    start_prepared_with_configuration_bytes(
        supervisor,
        runtime_dir,
        reservation,
        prepared_runtime,
        PreparedRuntimeContractInput {
            runtime_configuration_bytes: configuration.encode_to_vec(),
            settings_snapshot_bytes: Some(settings_snapshot_bytes),
            host_bridge_configuration: Some(host_bridge_configuration),
            cleanup,
            expected_module_kind: ModuleKindV1::Integration,
        },
    )
}

fn start_staged_with_configurations(
    supervisor: &ManagedRuntimeSupervisor,
    runtime_dir: &Path,
    reservation: ManagedLaunchReservation,
    configuration: ManagedStorageRuntimeConfigurationV1,
    host_bridge_configuration: Option<ManagedIntegrationHostBridgeConfigurationV1>,
) -> Result<u64, String> {
    start_staged_with_configuration_bytes(
        supervisor,
        runtime_dir,
        reservation,
        PreparedRuntimeContractInput {
            runtime_configuration_bytes: configuration.encode_to_vec(),
            settings_snapshot_bytes: None,
            host_bridge_configuration,
            cleanup: None,
            expected_module_kind: ModuleKindV1::Platform,
        },
    )
}

fn start_staged_with_configuration_bytes(
    supervisor: &ManagedRuntimeSupervisor,
    runtime_dir: &Path,
    reservation: ManagedLaunchReservation,
    contracts: PreparedRuntimeContractInput,
) -> Result<u64, String> {
    let kernel_executable = selected_kernel_executable()?;
    let launch_directory = managed_launch_directory(runtime_dir, &reservation);
    cleanup_abandoned_launch_directory(&launch_directory)?;
    let prepared = native_launch::prepare_bound_managed_runtime(
        &kernel_executable,
        reservation.binding(),
        &launch_directory,
    )?;
    start_prepared_with_configuration_bytes(
        supervisor,
        runtime_dir,
        reservation,
        prepared,
        contracts,
    )
}

fn start_prepared_with_configuration_bytes(
    supervisor: &ManagedRuntimeSupervisor,
    runtime_dir: &Path,
    reservation: ManagedLaunchReservation,
    prepared: native_launch::PreparedBundledManagedRuntime,
    input: PreparedRuntimeContractInput,
) -> Result<u64, String> {
    let PreparedRuntimeContractInput {
        runtime_configuration_bytes,
        settings_snapshot_bytes,
        host_bridge_configuration,
        cleanup,
        expected_module_kind,
    } = input;
    log_developer_launch_contracts(
        expected_module_kind,
        &runtime_configuration_bytes,
        settings_snapshot_bytes.as_deref(),
        host_bridge_configuration.as_ref(),
    );
    let launch_directory = managed_launch_directory(runtime_dir, &reservation);
    let preflight = (|| {
        let descriptor = decode_descriptor_v1(prepared.descriptor_bytes())
            .map_err(|_| "managed runtime descriptor is invalid".to_owned())?;
        if descriptor.module_kind != expected_module_kind as i32 {
            return Err("managed runtime module kind does not match launch path".to_owned());
        }
        let control_transport = select_managed_control_transport(&descriptor)
            .map_err(|_| "managed runtime control transport is not exact".to_owned())?;
        let host_bridge_configuration_bytes = host_bridge_configuration
            .as_ref()
            .map(prost::Message::encode_to_vec);
        let contracts = match (
            settings_snapshot_bytes,
            host_bridge_configuration_bytes.as_deref(),
        ) {
            (Some(settings_snapshot_bytes), Some(host_bridge_configuration_bytes)) => {
                StagedRuntimeContracts::stage_with_runtime_host_bridge_and_settings_snapshot(
                    &launch_directory.join("contracts"),
                    prepared.descriptor_bytes(),
                    prepared.settings_schema_bytes(),
                    Some(&settings_snapshot_bytes),
                    Some(&runtime_configuration_bytes),
                    Some(host_bridge_configuration_bytes),
                )?
            }
            (Some(settings_snapshot_bytes), None) => {
                StagedRuntimeContracts::stage_with_runtime_configuration_and_settings_snapshot(
                    &launch_directory.join("contracts"),
                    prepared.descriptor_bytes(),
                    prepared.settings_schema_bytes(),
                    &settings_snapshot_bytes,
                    &runtime_configuration_bytes,
                )?
            }
            (None, Some(host_bridge_configuration_bytes)) => {
                StagedRuntimeContracts::stage_with_runtime_and_host_bridge_configuration(
                    &launch_directory.join("contracts"),
                    prepared.descriptor_bytes(),
                    prepared.settings_schema_bytes(),
                    Some(&runtime_configuration_bytes),
                    Some(host_bridge_configuration_bytes),
                )?
            }
            (None, None) => {
                StagedRuntimeContracts::stage_with_runtime_and_host_bridge_configuration(
                    &launch_directory.join("contracts"),
                    prepared.descriptor_bytes(),
                    prepared.settings_schema_bytes(),
                    Some(&runtime_configuration_bytes),
                    None,
                )?
            }
        };
        let mut arguments = vec![
            "serve-inherited".to_owned(),
            "--descriptor-path".to_owned(),
            contracts.descriptor_path().display().to_string(),
        ];
        let settings_schema_path = contracts
            .settings_schema_path()
            .ok_or_else(|| "managed runtime settings schema is unavailable".to_owned())?;
        arguments.push("--settings-schema-path".to_owned());
        arguments.push(settings_schema_path.display().to_string());
        if let Some(path) = contracts.settings_snapshot_path() {
            arguments.push("--settings-snapshot-path".to_owned());
            arguments.push(path.display().to_string());
        }
        let configuration_path = contracts
            .runtime_configuration_path()
            .ok_or_else(|| "managed runtime configuration is unavailable".to_owned())?;
        arguments.push("--runtime-configuration-path".to_owned());
        arguments.push(configuration_path.display().to_string());
        arguments.push("--runtime-instance-id".to_owned());
        arguments.push(reservation.runtime_instance_id().to_owned());
        if let Some(path) = contracts.host_bridge_configuration_path() {
            arguments.push("--host-bridge-configuration-path".to_owned());
            arguments.push(path.display().to_string());
        }
        Ok::<_, String>((control_transport, contracts, arguments))
    })();
    let (control_transport, contracts, arguments) = match preflight {
        Ok(preflight) => preflight,
        Err(error) => {
            let _ = prepared.remove();
            if let Some(cleanup) = cleanup {
                cleanup();
            }
            return Err(error);
        }
    };
    let runtime_generation = reservation.runtime_generation();
    let (registration_id, expectation, policy) = reservation.into_launch_parts();
    supervisor.start_with_arguments_contracts_and_cleanup(
        crate::runtime::lifecycle::supervisor::ManagedRuntimeLaunchRequest {
            registration_id,
            staged_executable: prepared.into_staged_executable(),
            arguments,
            expectation,
            policy,
            control_transport,
            contracts: Some(contracts),
            cleanup,
        },
    )?;
    Ok(runtime_generation)
}

fn log_developer_launch_contracts(
    expected_module_kind: ModuleKindV1,
    runtime_configuration_bytes: &[u8],
    settings_snapshot_bytes: Option<&[u8]>,
    host_bridge_configuration: Option<&ManagedIntegrationHostBridgeConfigurationV1>,
) {
    if !tracing::enabled!(tracing::Level::DEBUG) {
        return;
    }
    let settings_snapshot = settings_snapshot_bytes.map(decode_settings_snapshot_v1);
    match expected_module_kind {
        ModuleKindV1::Domain => tracing::debug!(
            event = "managed_runtime.launch.contracts",
            module.kind = "domain",
            payload.runtime_configuration = ?ManagedDomainRuntimeConfigurationV1::decode(runtime_configuration_bytes),
            payload.settings_snapshot = ?settings_snapshot,
            payload.host_bridge_configuration = ?host_bridge_configuration,
        ),
        ModuleKindV1::Engine => tracing::debug!(
            event = "managed_runtime.launch.contracts",
            module.kind = "engine",
            payload.runtime_configuration = ?ManagedEngineRuntimeConfigurationV1::decode(runtime_configuration_bytes),
            payload.settings_snapshot = ?settings_snapshot,
            payload.host_bridge_configuration = ?host_bridge_configuration,
        ),
        ModuleKindV1::Integration => tracing::debug!(
            event = "managed_runtime.launch.contracts",
            module.kind = "integration",
            payload.runtime_configuration = ?ManagedIntegrationRuntimeConfigurationV1::decode(runtime_configuration_bytes),
            payload.settings_snapshot = ?settings_snapshot,
            payload.host_bridge_configuration = ?host_bridge_configuration,
        ),
        ModuleKindV1::Workflow => tracing::debug!(
            event = "managed_runtime.launch.contracts",
            module.kind = "workflow",
            payload.runtime_configuration = ?ManagedWorkflowRuntimeConfigurationV1::decode(runtime_configuration_bytes),
            payload.settings_snapshot = ?settings_snapshot,
            payload.host_bridge_configuration = ?host_bridge_configuration,
        ),
        _ => tracing::debug!(
            event = "managed_runtime.launch.contracts",
            module.kind = "unsupported",
            payload.runtime_configuration_bytes = runtime_configuration_bytes.len(),
            payload.settings_snapshot = ?settings_snapshot,
            payload.host_bridge_configuration = ?host_bridge_configuration,
        ),
    }
}

fn prepare_runtime_with_artifacts(
    runtime_dir: &Path,
    reservation: &ManagedLaunchReservation,
    granted_capability_ids: &[String],
    runtime_kind: &str,
) -> Result<native_launch::PreparedBundledManagedRuntimeWithArtifacts, String> {
    let kernel_executable = selected_kernel_executable()?;
    let prepared = native_launch::prepare_bound_managed_runtime_with_artifacts(
        &kernel_executable,
        reservation.binding(),
        &managed_launch_directory(runtime_dir, reservation),
        granted_capability_ids,
    )?;
    if prepared.state_layout_revision().is_some() {
        prepared.remove();
        return Err(format!(
            "{runtime_kind} cannot receive integration state resources"
        ));
    }
    Ok(prepared)
}

fn prepare_integration_runtime(
    data_dir: &Path,
    runtime_dir: &Path,
    reservation: &ManagedLaunchReservation,
    configuration: &mut ManagedIntegrationRuntimeConfigurationV1,
    granted_capability_ids: &[String],
) -> Result<native_launch::PreparedBundledManagedRuntimeWithArtifacts, String> {
    let kernel_executable = selected_kernel_executable()?;
    let prepared = native_launch::prepare_bound_managed_runtime_with_artifacts(
        &kernel_executable,
        reservation.binding(),
        &managed_launch_directory(runtime_dir, reservation),
        granted_capability_ids,
    )?;
    configuration.runtime_artifacts = prepared.runtime_artifact_bindings().to_vec();
    configuration.integration_state_root = match prepared.state_layout_revision() {
        Some(state_layout_revision) => {
            match crate::platform::integration_state::prepare(
                data_dir,
                &configuration.logical_owner_id,
                &configuration.registration_id,
                &configuration.configuration_instance_id,
                state_layout_revision,
            ) {
                Ok(root) => Some(root),
                Err(error) => {
                    prepared.remove();
                    return Err(error);
                }
            }
        }
        None => None,
    };
    Ok(prepared)
}

fn managed_launch_directory(runtime_dir: &Path, reservation: &ManagedLaunchReservation) -> PathBuf {
    runtime_dir.join("managed").join(format!(
        "launch-{}-{}",
        reservation.runtime_generation(),
        reservation.runtime_instance_id()
    ))
}

fn cleanup_abandoned_launch_directory(launch_directory: &Path) -> Result<(), String> {
    let metadata = match std::fs::symlink_metadata(launch_directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("managed launch cleanup requires a non-symlink directory".to_owned());
    }
    std::fs::remove_dir_all(launch_directory).map_err(|error| error.to_string())
}

fn staged_runtime_artifact_cleanup(
    artifacts: Vec<StagedNativeArtifact>,
) -> Option<Box<dyn FnOnce() + Send>> {
    if artifacts.is_empty() {
        return None;
    }
    Some(Box::new(move || {
        native_launch::cleanup_staged_runtime_artifacts(artifacts);
    }))
}

fn combine_cleanup(
    first: Option<Box<dyn FnOnce() + Send>>,
    second: Option<Box<dyn FnOnce() + Send>>,
) -> Option<Box<dyn FnOnce() + Send>> {
    match (first, second) {
        (None, None) => None,
        (Some(cleanup), None) | (None, Some(cleanup)) => Some(cleanup),
        (Some(first), Some(second)) => Some(Box::new(move || {
            first();
            second();
        })),
    }
}

#[cfg(test)]
mod abandoned_launch_cleanup_tests {
    use super::{cleanup_abandoned_launch_directory, new_instance_id};

    #[test]
    fn retry_removes_only_the_exact_abandoned_launch_directory() {
        let root = std::env::temp_dir().join(format!(
            "makosh-managed-launch-cleanup-test-{}",
            new_instance_id().expect("test instance")
        ));
        let launch = root.join("managed").join("launch-7-instance");
        std::fs::create_dir_all(launch.join("contracts")).expect("stale launch directory");
        std::fs::write(launch.join("contracts").join("descriptor.pb"), b"stale")
            .expect("stale contract");
        let sibling = root.join("managed").join("keep");
        std::fs::create_dir_all(&sibling).expect("sibling");

        cleanup_abandoned_launch_directory(&launch).expect("cleanup");

        assert!(!launch.exists());
        assert!(sibling.is_dir());
        std::fs::remove_dir_all(root).expect("test cleanup");
    }
}

pub(crate) fn reserve(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    registration_id: &str,
) -> Result<ManagedLaunchReservation, String> {
    if supervisor.is_active(registration_id)? {
        return Err("managed runtime is already active for this registration".to_owned());
    }
    let registration = store
        .module_registration(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "managed launch registration does not exist".to_owned())?;
    let binding = store
        .effective_bundled_managed_launch_binding(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "managed launch binding is unavailable".to_owned())?;
    let runtime_generation = next_runtime_generation(store, registration_id)?;
    let record = ManagedLaunchRecord::new(
        registration_id,
        new_instance_id()?,
        binding.binding_revision(),
        store.snapshot().generation(),
        runtime_generation,
        registration.grant_epoch(),
    );
    let expectation =
        ManagedRuntimeExpectation::from_fenced_launch(&registration, &binding, &record)?;
    let policy = ManagedChildExecutionPolicy::new(MAX_ATTEMPTS, MAX_RUNTIME)?;
    store
        .record_managed_launch(&record)
        .map_err(|error| format!("{error:?}"))?;
    Ok(ManagedLaunchReservation {
        registration_id: registration_id.to_owned(),
        binding,
        record,
        expectation,
        policy,
    })
}

/// Reconstructs an unstarted durable reservation after a separate owner-control step.
pub(crate) fn load(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    registration_id: &str,
) -> Result<ManagedLaunchReservation, String> {
    if supervisor.is_active(registration_id)? {
        return Err("managed runtime is already active for this registration".to_owned());
    }
    let registration = store
        .module_registration(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "managed launch registration does not exist".to_owned())?;
    let binding = store
        .effective_bundled_managed_launch_binding(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "managed launch binding is unavailable".to_owned())?;
    let record = store
        .effective_managed_launch_record(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "managed launch reservation is unavailable".to_owned())?;
    if record.registration_id() != registration_id
        || record.binding_revision() != binding.binding_revision()
        || record.kernel_generation() != store.snapshot().generation()
        || record.grant_epoch() != registration.grant_epoch()
    {
        return Err("managed launch reservation is stale".to_owned());
    }
    let expectation =
        ManagedRuntimeExpectation::from_fenced_launch(&registration, &binding, &record)?;
    let policy = ManagedChildExecutionPolicy::new(MAX_ATTEMPTS, MAX_RUNTIME)?;
    Ok(ManagedLaunchReservation {
        registration_id: registration_id.to_owned(),
        binding,
        record,
        expectation,
        policy,
    })
}

fn next_runtime_generation(
    store: &SqliteControlStore,
    registration_id: &str,
) -> Result<u64, String> {
    store
        .managed_launch_generation_high_watermark(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .map_or(Ok(1), |runtime_generation| {
            runtime_generation
                .checked_add(1)
                .ok_or_else(|| "managed runtime generation overflowed".to_owned())
        })
}
