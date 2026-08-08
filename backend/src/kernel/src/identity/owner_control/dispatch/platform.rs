//! Owner-authorized lifecycle commands for Kernel-managed platform processes.

use std::path::Path;

use makosh_gateway_protocol::v1::{
    BeginManagedStorageBindingRevocationRequestV1, BeginManagedStorageBindingRevocationResponseV1,
    BindPlatformStorageReleaseRequestV1, BindPlatformStorageReleaseResponseV1,
    BindPlatformTelemetryReleaseRequestV1, BindPlatformTelemetryReleaseResponseV1,
    BindPlatformVaultReleaseRequestV1, BindPlatformVaultReleaseResponseV1,
    ConfigurePlatformStorageTopologyRequestV1, ConfigurePlatformStorageTopologyResponseV1,
    GetManagedStorageBindingStatusRequestV1, GetManagedStorageBindingStatusResponseV1,
    GetPlatformTelemetryDiagnosticsRequestV1, GetPlatformTelemetryDiagnosticsResponseV1,
    IssueExternalStorageBindingRequestV1, IssueExternalStorageBindingResponseV1,
    IssueManagedStorageBindingRequestV1, IssueManagedStorageBindingResponseV1,
    StartPlatformStorageRuntimeRequestV1, StartPlatformStorageRuntimeResponseV1,
    StartPlatformTelemetryRuntimeRequestV1, StartPlatformTelemetryRuntimeResponseV1,
    StartPlatformVaultRuntimeRequestV1, StartPlatformVaultRuntimeResponseV1,
    owner_control_request_v1::Operation,
};
use makosh_kernel_control_store::{
    PlatformStorageBindingStateV1, PlatformStorageEndpointV1, PlatformStorageTopology,
    PlatformStorageTopologyInputV1, StorageDeploymentProfileV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::DeploymentProfileV1;

use super::{OwnerControlSessions, OwnerResult};
use crate::platform::storage::binding as storage_binding;
use crate::platform::storage::issuance::{StorageBindingIssueV1, issue_external, issue_managed};
use crate::platform::storage::launch as storage_launch;
use crate::platform::storage::revocation as storage_revocation;
use crate::platform::telemetry::binding as telemetry_binding;
use crate::platform::telemetry::diagnostics as telemetry_diagnostics;
use crate::platform::telemetry::launch as telemetry_launch;
use crate::platform::vault::binding as vault_binding;
use crate::platform::vault::launch as vault_launch;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

mod blob;
mod event_hub;
mod events;
mod external_binding;
mod storage_bundle;

pub(super) fn route(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    operation: Operation,
) -> Result<OwnerResult, String> {
    match operation {
        Operation::BindPlatformBlobRelease(request) => {
            blob::bind_release(store, supervisor, sessions, request)
        }
        Operation::StartPlatformBlobRuntime(request) => {
            blob::start(store, data_dir, runtime_dir, supervisor, sessions, request)
        }
        Operation::BindPlatformEventsAuthorityRelease(request) => {
            events::bind_release(store, supervisor, sessions, request)
        }
        Operation::ConfigurePlatformEventsAuthority(request) => {
            events::configure(store, supervisor, sessions, request)
        }
        Operation::StartPlatformEventsAuthorityRuntime(request) => {
            events::start(store, runtime_dir, supervisor, sessions, request)
        }
        Operation::ApplyPlatformEventsAuthorityAccountJwt(request) => {
            events::apply_account_jwt(supervisor, sessions, store, request)
        }
        Operation::ConfigurePlatformEventHubTopology(request) => {
            event_hub::configure(store, sessions, request)
        }
        Operation::ReconcilePlatformEventHubTopology(request) => {
            event_hub::reconcile(store, supervisor, sessions, request)
        }
        operation => route_foundation(
            store,
            data_dir,
            runtime_dir,
            supervisor,
            sessions,
            operation,
        ),
    }
}

fn route_foundation(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    operation: Operation,
) -> Result<OwnerResult, String> {
    match operation {
        Operation::BindPlatformVaultRelease(request) => {
            bind_vault_release(store, supervisor, sessions, request)
        }
        Operation::StartPlatformVaultRuntime(request) => {
            start_vault(store, data_dir, runtime_dir, supervisor, sessions, request)
        }
        Operation::BindPlatformTelemetryRelease(request) => {
            bind_telemetry_release(store, supervisor, sessions, request)
        }
        Operation::StartPlatformTelemetryRuntime(request) => {
            start_telemetry(store, data_dir, runtime_dir, supervisor, sessions, request)
        }
        Operation::GetPlatformTelemetryDiagnostics(request) => {
            telemetry_diagnostics(store, supervisor, sessions, request)
        }
        Operation::BindPlatformStorageRelease(request) => {
            bind_storage_release(store, supervisor, sessions, request)
        }
        Operation::StartPlatformStorageRuntime(request) => {
            start_storage(store, runtime_dir, supervisor, sessions, request)
        }
        Operation::ConfigurePlatformStorageTopology(request) => {
            configure_storage_topology(store, supervisor, sessions, request)
        }
        Operation::IssueManagedStorageBinding(request) => {
            issue_managed_storage_binding(store, supervisor, sessions, request)
        }
        Operation::GetManagedStorageBindingStatus(request) => {
            get_managed_storage_binding_status(store, sessions, request)
        }
        Operation::IssueExternalStorageBinding(request) => {
            issue_external_storage_binding(store, sessions, request)
        }
        Operation::BeginManagedStorageBindingRevocation(request) => {
            begin_managed_storage_binding_revocation(store, supervisor, sessions, request)
        }
        Operation::BeginExternalStorageBindingRevocation(request) => {
            external_binding::begin(store, sessions, request)
        }
        Operation::AdmitStorageBundle(request) => storage_bundle::admit(store, sessions, request),
        Operation::AdmitBundledStorageArtifact(request) => {
            storage_bundle::admit_bundled(store, sessions, request)
        }
        _ => Err("owner control operation is unavailable".to_owned()),
    }
}

fn bind_vault_release(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: BindPlatformVaultReleaseRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        let binding = vault_binding::bind_current_installed_release(store)?;
        stop_if_active(supervisor, vault_binding::VAULT_PROCESS_ID)?;
        Ok(binding)
    })()
    .map(|binding| {
        OwnerResult::BindPlatformVaultRelease(BindPlatformVaultReleaseResponseV1 {
            process_id: binding.process_id().to_owned(),
            binding_revision: binding.binding_revision(),
            distribution_id: binding.distribution_id().to_owned(),
            artifact_id: binding.artifact_id().to_owned(),
        })
    })
}

fn start_vault(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: StartPlatformVaultRuntimeRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        vault_launch::start(supervisor, store, data_dir, runtime_dir)
    })()
    .map(|runtime_generation| {
        OwnerResult::StartPlatformVaultRuntime(StartPlatformVaultRuntimeResponseV1 {
            process_id: vault_binding::VAULT_PROCESS_ID.to_owned(),
            runtime_generation,
            launch_state: "accepted".to_owned(),
        })
    })
}

fn bind_telemetry_release(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: BindPlatformTelemetryReleaseRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        let binding = telemetry_binding::bind_current_installed_release(store)?;
        stop_if_active(supervisor, telemetry_binding::TELEMETRY_PROCESS_ID)?;
        Ok(binding)
    })()
    .map(|binding| {
        OwnerResult::BindPlatformTelemetryRelease(BindPlatformTelemetryReleaseResponseV1 {
            process_id: binding.process_id().to_owned(),
            binding_revision: binding.binding_revision(),
            distribution_id: binding.distribution_id().to_owned(),
            artifact_id: binding.artifact_id().to_owned(),
        })
    })
}

fn start_telemetry(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: StartPlatformTelemetryRuntimeRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        telemetry_launch::start(supervisor, store, data_dir, runtime_dir)
    })()
    .map(|runtime_generation| {
        OwnerResult::StartPlatformTelemetryRuntime(StartPlatformTelemetryRuntimeResponseV1 {
            process_id: telemetry_binding::TELEMETRY_PROCESS_ID.to_owned(),
            runtime_generation,
            launch_state: "accepted".to_owned(),
        })
    })
}

fn telemetry_diagnostics(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: GetPlatformTelemetryDiagnosticsRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        telemetry_diagnostics::read(supervisor)
    })()
    .map(|diagnostics| {
        OwnerResult::GetPlatformTelemetryDiagnostics(GetPlatformTelemetryDiagnosticsResponseV1 {
            process_id: telemetry_binding::TELEMETRY_PROCESS_ID.to_owned(),
            segment_count: diagnostics.segment_count(),
            total_bytes: diagnostics.total_bytes(),
        })
    })
}

fn bind_storage_release(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: BindPlatformStorageReleaseRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        let binding = storage_binding::bind_current_installed_release(store)?;
        stop_if_active(supervisor, storage_binding::STORAGE_PROCESS_ID)?;
        Ok(binding)
    })()
    .map(|binding| {
        OwnerResult::BindPlatformStorageRelease(BindPlatformStorageReleaseResponseV1 {
            process_id: binding.process_id().to_owned(),
            binding_revision: binding.binding_revision(),
            distribution_id: binding.distribution_id().to_owned(),
            artifact_id: binding.artifact_id().to_owned(),
        })
    })
}

fn start_storage(
    store: &SqliteControlStore,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: StartPlatformStorageRuntimeRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        storage_launch::start(supervisor, store, runtime_dir)
    })()
    .map(|runtime_generation| {
        OwnerResult::StartPlatformStorageRuntime(StartPlatformStorageRuntimeResponseV1 {
            process_id: storage_binding::STORAGE_PROCESS_ID.to_owned(),
            runtime_generation,
            launch_state: "accepted".to_owned(),
        })
    })
}

fn configure_storage_topology(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: ConfigurePlatformStorageTopologyRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        let topology = PlatformStorageTopology::new(PlatformStorageTopologyInputV1 {
            revision: next_topology_revision(store)?,
            storage_generation: request.storage_generation,
            storage_instance_id: request.storage_instance_id,
            database_id: request.database_id,
            deployment_profile: deployment_profile(request.deployment_profile)?,
            postgres_endpoint: endpoint(request.postgres_host, request.postgres_port)?,
            pgbouncer_endpoint: endpoint(request.pgbouncer_host, request.pgbouncer_port)?,
            postgres_artifact_sha256: exact_digest(request.postgres_artifact_sha256)?,
            pgbouncer_artifact_sha256: exact_digest(request.pgbouncer_artifact_sha256)?,
        })
        .with_pgbouncer_backend_endpoint(endpoint(
            request.pgbouncer_backend_host,
            request.pgbouncer_backend_port,
        )?);
        stop_if_active(supervisor, storage_binding::STORAGE_PROCESS_ID)?;
        store
            .record_platform_storage_topology(&topology)
            .map_err(|_| "Storage topology cannot be updated".to_owned())?;
        Ok(topology)
    })()
    .map(|topology| {
        OwnerResult::ConfigurePlatformStorageTopology(ConfigurePlatformStorageTopologyResponseV1 {
            topology_revision: topology.revision(),
            storage_generation: topology.storage_generation(),
        })
    })
}

fn issue_managed_storage_binding(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: IssueManagedStorageBindingRequestV1,
) -> Result<OwnerResult, String> {
    let digest: [u8; 32] = request
        .storage_bundle_digest
        .try_into()
        .map_err(|_| "Storage bundle digest is invalid".to_owned())?;
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        let issue = StorageBindingIssueV1::new(
            request.role_epoch,
            request.credential_lease_revision,
            request.storage_bundle_revision,
            digest,
        )?;
        let binding = issue_managed(
            store,
            &request.registration_id,
            &request.runtime_instance_id,
            request.runtime_generation,
            &request.capability_id,
            issue,
        )?;
        crate::platform::storage::provisioning::apply_reserved_binding(
            supervisor, store, &binding,
        )?;
        Ok(binding)
    })()
    .map(|binding| {
        OwnerResult::IssueManagedStorageBinding(IssueManagedStorageBindingResponseV1 {
            registration_id: binding.registration_id().to_owned(),
            capability_id: binding.capability_id().to_owned(),
            binding_revision: binding.binding_revision(),
            topology_revision: binding.topology_revision(),
            storage_generation: binding.storage_generation(),
        })
    })
}

fn get_managed_storage_binding_status(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: GetManagedStorageBindingStatusRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        store
            .effective_bundled_managed_launch_binding(&request.registration_id)
            .map_err(|_| "Managed launch binding status is unavailable".to_owned())?
            .ok_or_else(|| "Managed launch binding status is unavailable".to_owned())?;
        store
            .platform_storage_binding(&request.registration_id, &request.capability_id)
            .map_err(|_| "Storage binding status is unavailable".to_owned())?
            .ok_or_else(|| "Storage binding status is unavailable".to_owned())
    })()
    .map(|binding| {
        OwnerResult::GetManagedStorageBindingStatus(GetManagedStorageBindingStatusResponseV1 {
            registration_id: binding.registration_id().to_owned(),
            capability_id: binding.capability_id().to_owned(),
            binding_revision: binding.binding_revision(),
            role_epoch: binding.role_epoch(),
            credential_lease_revision: binding.credential_lease_revision(),
            binding_state: match binding.state() {
                PlatformStorageBindingStateV1::Active => "active",
                PlatformStorageBindingStateV1::Revoking => "revoking",
            }
            .to_owned(),
        })
    })
}

fn issue_external_storage_binding(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: IssueExternalStorageBindingRequestV1,
) -> Result<OwnerResult, String> {
    let digest: [u8; 32] = request
        .storage_bundle_digest
        .try_into()
        .map_err(|_| "Storage bundle digest is invalid".to_owned())?;
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        let issue = StorageBindingIssueV1::new(
            request.role_epoch,
            request.credential_lease_revision,
            request.storage_bundle_revision,
            digest,
        )?;
        issue_external(
            store,
            &request.registration_id,
            &request.runtime_instance_id,
            request.runtime_generation,
            &request.capability_id,
            issue,
        )
    })()
    .map(|binding| {
        OwnerResult::IssueExternalStorageBinding(IssueExternalStorageBindingResponseV1 {
            registration_id: binding.registration_id().to_owned(),
            capability_id: binding.capability_id().to_owned(),
            binding_revision: binding.binding_revision(),
            topology_revision: binding.topology_revision(),
            storage_generation: binding.storage_generation(),
        })
    })
}

fn begin_managed_storage_binding_revocation(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: BeginManagedStorageBindingRevocationRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        let binding = store
            .begin_platform_storage_binding_revocation(
                &request.registration_id,
                &request.capability_id,
                request.binding_revision,
            )
            .map_err(|_| "Storage binding cannot be reserved for revocation".to_owned())?;
        if storage_revocation::fence_reserved_binding(supervisor, store, &binding).is_err() {
            stop_if_active(supervisor, storage_binding::STORAGE_PROCESS_ID)?;
            return Err("Storage binding revocation is incomplete".to_owned());
        }
        Ok(binding)
    })()
    .map(|binding| {
        OwnerResult::BeginManagedStorageBindingRevocation(
            BeginManagedStorageBindingRevocationResponseV1 {
                registration_id: binding.registration_id().to_owned(),
                capability_id: binding.capability_id().to_owned(),
                binding_revision: binding.binding_revision(),
            },
        )
    })
}

fn endpoint(host: String, port: u32) -> Result<PlatformStorageEndpointV1, String> {
    let port = u16::try_from(port).ok().filter(|value| *value != 0);
    let endpoint = PlatformStorageEndpointV1::new(host, port.unwrap_or_default());
    valid_endpoint(&endpoint)
        .then_some(endpoint)
        .ok_or_else(|| "Storage endpoint is invalid".to_owned())
}

fn valid_endpoint(endpoint: &PlatformStorageEndpointV1) -> bool {
    endpoint.port() > 0
        && !endpoint.host().is_empty()
        && endpoint.host().len() <= 253
        && endpoint.host().bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_uppercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'-' | b':')
        })
}

fn next_topology_revision(store: &SqliteControlStore) -> Result<u64, String> {
    store
        .platform_storage_topology()
        .map_err(|_| "Storage topology is unavailable".to_owned())?
        .map_or(Ok(1), |topology| {
            topology
                .revision()
                .checked_add(1)
                .ok_or_else(|| "Storage topology revision overflowed".to_owned())
        })
}

fn deployment_profile(value: i32) -> Result<StorageDeploymentProfileV1, String> {
    match DeploymentProfileV1::try_from(value) {
        Ok(DeploymentProfileV1::MacosTauriEmbedded) => {
            Ok(StorageDeploymentProfileV1::MacosTauriEmbedded)
        }
        Ok(DeploymentProfileV1::LinuxDockerServer) => {
            Ok(StorageDeploymentProfileV1::LinuxDockerServer)
        }
        _ => Err("Storage deployment profile is invalid".to_owned()),
    }
}

fn exact_digest(value: Vec<u8>) -> Result<[u8; 32], String> {
    let digest: [u8; 32] = value
        .try_into()
        .map_err(|_| "Storage artifact digest is invalid".to_owned())?;
    if digest.iter().all(|byte| *byte == 0) {
        return Err("Storage artifact digest is invalid".to_owned());
    }
    Ok(digest)
}

fn stop_if_active(supervisor: &ManagedRuntimeSupervisor, process_id: &str) -> Result<(), String> {
    if supervisor.is_active(process_id)? {
        supervisor.stop(process_id)?;
    }
    Ok(())
}
