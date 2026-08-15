//! Request dispatch for owner-private control IPC.

mod platform;
mod scheduler;

use std::path::Path;

use makosh_gateway_protocol::v1::{
    ApplyManagedIntegrationSettingsRequestV1, ApplyManagedIntegrationSettingsResponseV1,
    ApproveModuleRegistrationRequestV1, ApproveModuleRegistrationResponseV1,
    BeginBrowserPairingRequestV1, BeginBrowserPairingResponseV1,
    BeginOwnerControlSessionResponseV1, BindBundledManagedReleaseRequestV1,
    BindBundledManagedReleaseResponseV1, BindExternalRuntimeIdentityRequestV1,
    BindExternalRuntimeIdentityResponseV1, CompleteOwnerControlSessionRequestV1,
    CompleteOwnerControlSessionResponseV1, GetModuleRegistrationStatusRequestV1,
    GetModuleRegistrationStatusResponseV1, OwnerControlRequestV1, OwnerControlResponseV1,
    ProposeBundledManagedArtifactRequestV1, ProposeBundledManagedArtifactResponseV1,
    ReserveBundledManagedRuntimeRequestV1, ReserveBundledManagedRuntimeResponseV1,
    StartBundledManagedRuntimeRequestV1, StartBundledManagedRuntimeResponseV1,
    StartReservedDomainRuntimeRequestV1, StartReservedDomainRuntimeResponseV1,
    StartReservedEngineRuntimeRequestV1, StartReservedEngineRuntimeResponseV1,
    StartReservedIntegrationRuntimeRequestV1, StartReservedIntegrationRuntimeResponseV1,
    StartReservedWorkflowRuntimeRequestV1, StartReservedWorkflowRuntimeResponseV1,
    TransitionModuleRegistrationRequestV1, TransitionModuleRegistrationResponseV1,
    UpdateOperatorSettingsRequestV1, UpdateOperatorSettingsResponseV1,
    UpgradeBundledManagedRegistrationRequestV1, UpgradeBundledManagedRegistrationResponseV1,
};
use makosh_kernel_control_store::{ModuleRegistrationState, PlatformStorageBindingStateV1};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::{
    v1::{ManagedDomainRuntimeConfigurationV1, ManagedEngineRuntimeConfigurationV1},
    validation::{
        managed_domain_runtime::validate_managed_domain_runtime_configuration,
        managed_engine_runtime::validate_managed_engine_runtime_configuration,
    },
};

use crate::identity::owner_control::sessions::OwnerControlSessions;
use crate::modules::registration::registry as module_registry;
use crate::modules::settings::managed_application as managed_settings_application;
use crate::modules::settings::mutation as settings_operator_mutation;
use crate::platform::gateway::BrowserPairingAdmissionV1;
use crate::platform::macos::bundled_release as macos_bundled_release_binding;
use crate::platform::macos::managed_launch as macos_managed_runtime_launch;
use crate::runtime::lifecycle::integration_launch as managed_integration_launch;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;
use crate::runtime::lifecycle::workflow_launch as managed_workflow_launch;

pub(super) type OwnerResult = makosh_gateway_protocol::v1::owner_control_response_v1::Result;

pub(super) fn handle(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    browser_pairing: Option<&BrowserPairingAdmissionV1>,
    sessions: &mut OwnerControlSessions,
    request: OwnerControlRequestV1,
) -> OwnerControlResponseV1 {
    response(route(
        store,
        data_dir,
        runtime_dir,
        supervisor,
        browser_pairing,
        sessions,
        request,
    ))
}

fn route(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    browser_pairing: Option<&BrowserPairingAdmissionV1>,
    sessions: &mut OwnerControlSessions,
    request: OwnerControlRequestV1,
) -> Result<OwnerResult, String> {
    let Some(operation) = request.operation else {
        return Err("owner control operation is unavailable".to_owned());
    };

    route_operation(
        store,
        data_dir,
        runtime_dir,
        supervisor,
        browser_pairing,
        sessions,
        operation,
    )
}

fn route_operation(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    browser_pairing: Option<&BrowserPairingAdmissionV1>,
    sessions: &mut OwnerControlSessions,
    operation: makosh_gateway_protocol::v1::owner_control_request_v1::Operation,
) -> Result<OwnerResult, String> {
    use makosh_gateway_protocol::v1::owner_control_request_v1::Operation;

    match operation {
        Operation::GetModuleRegistrationStatus(request) => status(store, request),
        Operation::ApproveModuleRegistration(request) => approve(store, sessions, request),
        Operation::TransitionModuleRegistration(request) => {
            transition(store, supervisor, sessions, request)
        }
        Operation::BeginOwnerSession(_) => begin(store, sessions),
        Operation::CompleteOwnerSession(request) => complete(store, sessions, request),
        Operation::BeginBrowserPairing(request) => {
            begin_browser_pairing(store, sessions, browser_pairing, request)
        }
        Operation::UpdateOperatorSettings(request) => update_settings(store, sessions, request),
        Operation::ApplyManagedIntegrationSettings(request) => apply_managed_integration_settings(
            store,
            data_dir,
            runtime_dir,
            supervisor,
            sessions,
            request,
        ),
        Operation::BindExternalRuntimeIdentity(request) => {
            bind_external_identity(store, sessions, request)
        }
        Operation::BindBundledManagedRelease(request) => {
            bind_managed_release(store, supervisor, sessions, request)
        }
        Operation::ProposeBundledManagedArtifact(request) => {
            propose_bundled_artifact(store, sessions, request)
        }
        Operation::UpgradeBundledManagedRegistration(request) => {
            upgrade_bundled_registration(store, supervisor, sessions, request)
        }
        Operation::StartBundledManagedRuntime(request) => {
            start_managed_runtime(store, runtime_dir, supervisor, sessions, request)
        }
        Operation::ReserveBundledManagedRuntime(request) => {
            reserve_managed_runtime(store, supervisor, sessions, request)
        }
        Operation::StartReservedIntegrationRuntime(request) => start_reserved_integration_runtime(
            store,
            data_dir,
            runtime_dir,
            supervisor,
            sessions,
            request,
        ),
        Operation::StartReservedDomainRuntime(request) => {
            start_reserved_domain_runtime(store, runtime_dir, supervisor, sessions, request)
        }
        Operation::StartReservedEngineRuntime(request) => {
            start_reserved_engine_runtime(store, runtime_dir, supervisor, sessions, request)
        }
        Operation::StartReservedWorkflowRuntime(request) => {
            start_reserved_workflow_runtime(store, runtime_dir, supervisor, sessions, request)
        }
        Operation::StartReservedSchedulerRuntime(request) => {
            scheduler::start_reserved(store, runtime_dir, supervisor, sessions, request)
        }
        Operation::UpsertSchedulerSchedule(request) => {
            scheduler::upsert(store, supervisor, sessions, request)
        }
        Operation::RestartSchedulerRuntime(request) => {
            scheduler::restart(store, runtime_dir, supervisor, sessions, request)
        }
        operation => platform::route(
            store,
            data_dir,
            runtime_dir,
            supervisor,
            sessions,
            operation,
        ),
    }
}

fn begin_browser_pairing(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    browser_pairing: Option<&BrowserPairingAdmissionV1>,
    request: BeginBrowserPairingRequestV1,
) -> Result<OwnerResult, String> {
    let owner = sessions.authorized_owner(store, &request.owner_session_id)?;
    let browser_pairing =
        browser_pairing.ok_or_else(|| "browser Gateway pairing is unavailable".to_owned())?;
    let pairing = browser_pairing.begin(owner.owner_id(), owner.device_id(), unix_millis()?)?;
    Ok(OwnerResult::BeginBrowserPairing(
        BeginBrowserPairingResponseV1 {
            pairing_id: pairing.pairing_id().to_owned(),
            expires_at_unix_millis: pairing.expires_at_unix_millis(),
        },
    ))
}

fn reserve_managed_runtime(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: ReserveBundledManagedRuntimeRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        macos_managed_runtime_launch::reserve(supervisor, store, &request.registration_id)
    })()
    .map(|reservation| {
        OwnerResult::ReserveBundledManagedRuntime(ReserveBundledManagedRuntimeResponseV1 {
            registration_id: reservation.registration_id().to_owned(),
            runtime_instance_id: reservation.runtime_instance_id().to_owned(),
            runtime_generation: reservation.runtime_generation(),
            grant_epoch: reservation.grant_epoch(),
        })
    })
}

fn start_reserved_integration_runtime(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: StartReservedIntegrationRuntimeRequestV1,
) -> Result<OwnerResult, String> {
    sessions.authorize(store, &request.owner_session_id)?;
    let Some(configuration_instance_id) =
        managed_integration_launch::startup_configuration_instance(
            store,
            &request.registration_id,
            &request.configuration_instance_id,
        )?
    else {
        return Ok(OwnerResult::StartReservedIntegrationRuntime(
            StartReservedIntegrationRuntimeResponseV1 {
                registration_id: request.registration_id,
                runtime_generation: 0,
                launch_state: "unconfigured".to_owned(),
                host_bridge_socket_path: None,
            },
        ));
    };
    managed_integration_launch::launch_reserved(
        store,
        data_dir,
        runtime_dir,
        supervisor,
        &request.registration_id,
        &request.storage_capability_id,
        &configuration_instance_id,
        request.request_host_bridge,
        None,
    )
    .map(|(runtime_generation, host_bridge_socket_path)| {
        OwnerResult::StartReservedIntegrationRuntime(StartReservedIntegrationRuntimeResponseV1 {
            registration_id: request.registration_id,
            runtime_generation,
            launch_state: "accepted".to_owned(),
            host_bridge_socket_path,
        })
    })
}

fn apply_managed_integration_settings(
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: ApplyManagedIntegrationSettingsRequestV1,
) -> Result<OwnerResult, String> {
    sessions.authorize(store, &request.owner_session_id)?;
    let prepared = managed_settings_application::prepare(
        store,
        supervisor,
        &request.registration_id,
        &request.configuration_instance_id,
        &request.storage_capability_id,
        request.expected_desired_revision,
    )?;
    let launched = managed_integration_launch::launch_reserved(
        store,
        data_dir,
        runtime_dir,
        supervisor,
        &request.registration_id,
        &request.storage_capability_id,
        &request.configuration_instance_id,
        request.request_host_bridge,
        Some(prepared.snapshot_bytes().to_vec()),
    );
    let (runtime_generation, host_bridge_socket_path) = match launched {
        Ok(launched) => launched,
        Err(error) => {
            managed_settings_application::block_after_launch_failure(
                store,
                &request.registration_id,
                &request.configuration_instance_id,
                prepared.revision(),
            );
            return Err(error);
        }
    };
    managed_settings_application::wait_for_ready_and_confirm(
        store,
        supervisor,
        &request.registration_id,
        &request.configuration_instance_id,
        prepared.revision(),
    )?;
    Ok(OwnerResult::ApplyManagedIntegrationSettings(
        ApplyManagedIntegrationSettingsResponseV1 {
            registration_id: request.registration_id,
            effective_revision: prepared.revision(),
            runtime_generation,
            apply_state: "current".to_owned(),
            host_bridge_socket_path,
        },
    ))
}

fn start_reserved_domain_runtime(
    store: &SqliteControlStore,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: StartReservedDomainRuntimeRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        let logical_human_owner = sessions.authorized_owner(store, &request.owner_session_id)?;
        let reservation =
            macos_managed_runtime_launch::load(supervisor, store, &request.registration_id)?;
        let registration = store
            .module_registration(&request.registration_id)
            .map_err(|_| "managed domain registration is unavailable".to_owned())?
            .ok_or_else(|| "managed domain registration is unavailable".to_owned())?;
        let binding = store
            .platform_storage_binding(&request.registration_id, &request.storage_capability_id)
            .map_err(|_| "managed domain Storage binding is unavailable".to_owned())?
            .filter(|value| value.state() == PlatformStorageBindingStateV1::Active)
            .ok_or_else(|| "managed domain Storage binding is unavailable".to_owned())?;
        let storage_topology = crate::platform::storage::topology::current(store)?;
        let vault = crate::platform::vault::status::read_current(store, &supervisor.relay_port())?;
        let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
            &storage_topology,
            &binding,
            store.snapshot().instance_id(),
            vault.runtime_generation(),
            vault.hpke_public_key_x25519(),
        )?;
        let (event_hub_endpoint, event_credential_revision) =
            domain_event_hub_configuration(store, &request.registration_id)?;
        let configuration = ManagedDomainRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: registration.owner_id().to_owned(),
            registration_id: request.registration_id.clone(),
            runtime_instance_id: reservation.runtime_instance_id().to_owned(),
            runtime_generation: reservation.runtime_generation(),
            grant_epoch: reservation.grant_epoch(),
            storage: Some(storage),
            event_hub_endpoint,
            event_credential_revision,
            logical_human_owner_id: logical_human_owner.owner_id().to_owned(),
        };
        validate_managed_domain_runtime_configuration(&configuration)
            .map_err(|_| "managed domain runtime configuration is invalid".to_owned())?;
        macos_managed_runtime_launch::start_reserved_domain(
            supervisor,
            runtime_dir,
            reservation,
            configuration,
        )
    })()
    .map(|runtime_generation| {
        OwnerResult::StartReservedDomainRuntime(StartReservedDomainRuntimeResponseV1 {
            registration_id: request.registration_id,
            runtime_generation,
            launch_state: "accepted".to_owned(),
        })
    })
}

fn domain_event_hub_configuration(
    store: &SqliteControlStore,
    registration_id: &str,
) -> Result<(String, u64), String> {
    capability_scoped_event_hub_configuration(store, registration_id, "managed domain")
}

fn engine_event_hub_configuration(
    store: &SqliteControlStore,
    registration_id: &str,
) -> Result<(String, u64), String> {
    capability_scoped_event_hub_configuration(store, registration_id, "managed engine")
}

fn capability_scoped_event_hub_configuration(
    store: &SqliteControlStore,
    registration_id: &str,
    runtime_kind: &str,
) -> Result<(String, u64), String> {
    let snapshot = store
        .module_grant_snapshot(registration_id)
        .map_err(|_| format!("{runtime_kind} grants are unavailable"))?
        .ok_or_else(|| format!("{runtime_kind} grants are unavailable"))?;
    let grants = snapshot
        .effective_grants()
        .ok_or_else(|| format!("{runtime_kind} grants are unavailable"))?;
    let mut requires_event_hub = false;
    for capability_id in grants.capability_ids() {
        if !store
            .module_event_route_requests(registration_id, capability_id)
            .map_err(|_| format!("{runtime_kind} Event Hub routes are unavailable"))?
            .is_empty()
        {
            requires_event_hub = true;
            break;
        }
    }
    if !requires_event_hub {
        return Ok((String::new(), 0));
    }
    let topology = store
        .platform_event_hub_topology()
        .map_err(|_| "Event Hub topology is unavailable".to_owned())?
        .ok_or_else(|| "Event Hub topology is unavailable".to_owned())?;
    Ok((
        topology.nats_endpoint().to_owned(),
        topology.credential_revision(),
    ))
}

fn start_reserved_engine_runtime(
    store: &SqliteControlStore,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: StartReservedEngineRuntimeRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        let logical_human_owner = sessions.authorized_owner(store, &request.owner_session_id)?;
        let reservation =
            macos_managed_runtime_launch::load(supervisor, store, &request.registration_id)?;
        let registration = store
            .module_registration(&request.registration_id)
            .map_err(|_| "managed engine registration is unavailable".to_owned())?
            .ok_or_else(|| "managed engine registration is unavailable".to_owned())?;
        let binding = store
            .platform_storage_binding(&request.registration_id, &request.storage_capability_id)
            .map_err(|_| "managed engine Storage binding is unavailable".to_owned())?
            .filter(|value| value.state() == PlatformStorageBindingStateV1::Active)
            .ok_or_else(|| "managed engine Storage binding is unavailable".to_owned())?;
        let storage_topology = crate::platform::storage::topology::current(store)?;
        let vault = crate::platform::vault::status::read_current(store, &supervisor.relay_port())?;
        let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
            &storage_topology,
            &binding,
            store.snapshot().instance_id(),
            vault.runtime_generation(),
            vault.hpke_public_key_x25519(),
        )?;
        let (event_hub_endpoint, event_credential_revision) =
            engine_event_hub_configuration(store, &request.registration_id)?;
        let settings_snapshot = managed_integration_launch::admitted_settings_snapshot(
            store,
            &request.registration_id,
        )?;
        let granted_capability_ids =
            effective_granted_capability_ids(store, &request.registration_id, "managed engine")?;
        let configuration = ManagedEngineRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: registration.owner_id().to_owned(),
            registration_id: request.registration_id.clone(),
            runtime_instance_id: reservation.runtime_instance_id().to_owned(),
            runtime_generation: reservation.runtime_generation(),
            grant_epoch: reservation.grant_epoch(),
            storage: Some(storage),
            event_hub_endpoint,
            event_credential_revision,
            settings_revision: settings_snapshot.revision,
            logical_human_owner_id: logical_human_owner.owner_id().to_owned(),
            runtime_artifacts: Vec::new(),
        };
        validate_managed_engine_runtime_configuration(&configuration)
            .map_err(|_| "managed engine runtime configuration is invalid".to_owned())?;
        macos_managed_runtime_launch::start_reserved_engine(
            supervisor,
            runtime_dir,
            reservation,
            configuration,
            settings_snapshot.bytes,
            &granted_capability_ids,
        )
    })()
    .map(|runtime_generation| {
        OwnerResult::StartReservedEngineRuntime(StartReservedEngineRuntimeResponseV1 {
            registration_id: request.registration_id,
            runtime_generation,
            launch_state: "accepted".to_owned(),
        })
    })
}

fn start_reserved_workflow_runtime(
    store: &SqliteControlStore,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: StartReservedWorkflowRuntimeRequestV1,
) -> Result<OwnerResult, String> {
    let logical_owner = sessions.authorized_owner(store, &request.owner_session_id)?;
    if request.configuration_instance_id.is_empty()
        && managed_workflow_launch::configuration_required_but_unavailable(
            store,
            &request.registration_id,
        )?
    {
        return Ok(OwnerResult::StartReservedWorkflowRuntime(
            StartReservedWorkflowRuntimeResponseV1 {
                registration_id: request.registration_id,
                runtime_generation: 0,
                launch_state: "unconfigured".to_owned(),
            },
        ));
    }
    managed_workflow_launch::launch_reserved(
        store,
        runtime_dir,
        supervisor,
        logical_owner.owner_id(),
        &request.registration_id,
        &request.storage_capability_id,
        &request.configuration_instance_id,
        None,
    )
    .map(|runtime_generation| {
        OwnerResult::StartReservedWorkflowRuntime(StartReservedWorkflowRuntimeResponseV1 {
            registration_id: request.registration_id,
            runtime_generation,
            launch_state: "accepted".to_owned(),
        })
    })
}

fn effective_granted_capability_ids(
    store: &SqliteControlStore,
    registration_id: &str,
    runtime_kind: &str,
) -> Result<Vec<String>, String> {
    store
        .module_grant_snapshot(registration_id)
        .map_err(|_| format!("{runtime_kind} grants are unavailable"))?
        .and_then(|snapshot| {
            snapshot
                .effective_grants()
                .map(|grants| grants.capability_ids().to_vec())
        })
        .ok_or_else(|| format!("{runtime_kind} grants are unavailable"))
}

fn status(
    store: &SqliteControlStore,
    request: GetModuleRegistrationStatusRequestV1,
) -> Result<OwnerResult, String> {
    module_registry::status(store, &request.registration_id).map(|status| {
        let attestation = status.external_runtime_attestation();
        OwnerResult::GetModuleRegistrationStatus(GetModuleRegistrationStatusResponseV1 {
            registration_id: status.registration().registration_id().to_owned(),
            module_id: status.registration().module_id().to_owned(),
            owner_id: status.registration().owner_id().to_owned(),
            registration_state: status.registration().state().as_str().to_owned(),
            grant_epoch: status.registration().grant_epoch(),
            effective_capability_count: status.effective_capability_count() as u32,
            external_runtime_id: attestation
                .map_or_else(String::new, |item| item.runtime_id().to_owned()),
            external_runtime_generation: attestation.map_or(0, |item| item.runtime_generation()),
        })
    })
}

fn approve(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: ApproveModuleRegistrationRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        module_registry::approve_after_owner_authorization(
            store,
            &request.registration_id,
            &request.capability_id,
        )
    })()
    .map(|grants| {
        OwnerResult::ApproveModuleRegistration(ApproveModuleRegistrationResponseV1 {
            registration_id: grants.registration_id().to_owned(),
            grant_epoch: grants.grant_epoch(),
            effective_capability_count: grants.capability_ids().len() as u32,
        })
    })
}

fn transition(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: TransitionModuleRegistrationRequestV1,
) -> Result<OwnerResult, String> {
    let next = transition_target(&request.target_state)?;
    sessions.authorize(store, &request.owner_session_id)?;
    let registration = module_registry::transition_after_owner_authorization(
        store,
        &request.registration_id,
        next,
    )?;
    let storage_revocation = crate::platform::storage::revocation::fence_registration_bindings(
        supervisor,
        store,
        registration.registration_id(),
    );
    let runtime_stop = supervisor.stop_if_active(registration.registration_id());
    let storage_revocation = storage_revocation.or_else(|_| {
        let retry = crate::platform::storage::revocation::fence_registration_bindings(
            supervisor,
            store,
            registration.registration_id(),
        );
        if retry.is_err() {
            supervisor.stop_if_active(crate::platform::storage::binding::STORAGE_PROCESS_ID)?;
        }
        retry
    });
    storage_revocation?;
    runtime_stop?;
    Ok(OwnerResult::TransitionModuleRegistration(
        TransitionModuleRegistrationResponseV1 {
            registration_id: registration.registration_id().to_owned(),
            registration_state: registration.state().as_str().to_owned(),
            grant_epoch: registration.grant_epoch(),
        },
    ))
}

fn begin(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
) -> Result<OwnerResult, String> {
    sessions.begin(store).map(|challenge| {
        OwnerResult::BeginOwnerSession(BeginOwnerControlSessionResponseV1 {
            challenge_id: challenge.challenge_id().to_owned(),
            challenge_bytes: challenge.bytes().to_vec(),
            kernel_instance_id: challenge.kernel_instance_id().to_owned(),
            owner_id: challenge.owner_id().to_owned(),
            device_id: challenge.device_id().to_owned(),
            control_store_generation: challenge.control_store_generation(),
            expires_at_unix_millis: challenge.expires_at_unix_millis(),
        })
    })
}

fn complete(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: CompleteOwnerControlSessionRequestV1,
) -> Result<OwnerResult, String> {
    sessions
        .complete(store, &request.challenge_id, &request.signature_raw)
        .map(|session| {
            OwnerResult::CompleteOwnerSession(CompleteOwnerControlSessionResponseV1 {
                owner_session_id: session.session_id().to_owned(),
                expires_at_unix_millis: session.expires_at_unix_millis(),
            })
        })
}

fn update_settings(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: UpdateOperatorSettingsRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        settings_operator_mutation::commit_after_owner_authorization(
            store,
            &request.registration_id,
            request.expected_revision,
            &request.snapshot_bytes,
        )
    })()
    .map(|desired_revision| {
        OwnerResult::UpdateOperatorSettings(UpdateOperatorSettingsResponseV1 {
            registration_id: request.registration_id,
            desired_revision,
            apply_state: "pending_validation".to_owned(),
        })
    })
}

fn bind_external_identity(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: BindExternalRuntimeIdentityRequestV1,
) -> Result<OwnerResult, String> {
    let public_key_sec1: [u8; 65] = request
        .public_key_sec1
        .try_into()
        .map_err(|_| "external runtime public key is invalid".to_owned())?;
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        module_registry::bind_external_runtime_identity_after_owner_authorization(
            store,
            &request.registration_id,
            public_key_sec1,
        )
    })()
    .map(|registration| {
        OwnerResult::BindExternalRuntimeIdentity(BindExternalRuntimeIdentityResponseV1 {
            registration_id: registration.registration_id().to_owned(),
            grant_epoch: registration.grant_epoch(),
        })
    })
}

fn bind_managed_release(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: BindBundledManagedReleaseRequestV1,
) -> Result<OwnerResult, String> {
    sessions.authorize(store, &request.owner_session_id)?;
    let binding = macos_bundled_release_binding::bind_current_installed_release(
        store,
        &request.registration_id,
        &request.artifact_id,
    )?;
    supervisor.stop_if_active(binding.registration_id())?;
    Ok(OwnerResult::BindBundledManagedRelease(
        BindBundledManagedReleaseResponseV1 {
            registration_id: binding.registration_id().to_owned(),
            binding_revision: binding.binding_revision(),
            distribution_id: binding.distribution_id().to_owned(),
            artifact_id: binding.artifact_id().to_owned(),
        },
    ))
}

fn propose_bundled_artifact(
    store: &SqliteControlStore,
    sessions: &mut OwnerControlSessions,
    request: ProposeBundledManagedArtifactRequestV1,
) -> Result<OwnerResult, String> {
    sessions.authorize(store, &request.owner_session_id)?;
    let operation_id: [u8; 16] = request
        .idempotency_key
        .as_slice()
        .try_into()
        .map_err(|_| "bundled artifact proposal idempotency key is invalid".to_owned())?;
    let proposal = macos_bundled_release_binding::propose_current_installed_artifact(
        store,
        &request.artifact_id,
        &request.expected_distribution_id,
        request.expected_distribution_generation,
        operation_id,
    )?;
    let receipt = proposal.receipt();
    let registration = receipt.registration();
    Ok(OwnerResult::ProposeBundledManagedArtifact(
        ProposeBundledManagedArtifactResponseV1 {
            registration_id: registration.registration_id().to_owned(),
            module_id: registration.module_id().to_owned(),
            owner_id: registration.owner_id().to_owned(),
            descriptor_sha256: registration.descriptor_sha256().to_vec(),
            requested_capability_ids: proposal.requested_capability_ids().to_vec(),
            distribution_id: request.expected_distribution_id,
            distribution_generation: request.expected_distribution_generation,
            artifact_id: request.artifact_id,
            replayed: receipt.replayed(),
        },
    ))
}

fn upgrade_bundled_registration(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: UpgradeBundledManagedRegistrationRequestV1,
) -> Result<OwnerResult, String> {
    sessions.authorize(store, &request.owner_session_id)?;
    let upgrade = macos_bundled_release_binding::upgrade_current_installed_artifact(
        store,
        &request.registration_id,
        &request.artifact_id,
        &request.expected_distribution_id,
        request.expected_distribution_generation,
    )?;
    supervisor.stop_if_active(&request.registration_id)?;
    let registration = upgrade.registration();
    Ok(OwnerResult::UpgradeBundledManagedRegistration(
        UpgradeBundledManagedRegistrationResponseV1 {
            registration_id: registration.registration_id().to_owned(),
            grant_epoch: registration.grant_epoch(),
            descriptor_sha256: registration.descriptor_sha256().to_vec(),
            effective_capability_count: u32::try_from(upgrade.requested_capability_ids().len())
                .map_err(|_| "managed registration capability count is invalid".to_owned())?,
        },
    ))
}

fn start_managed_runtime(
    store: &SqliteControlStore,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    sessions: &mut OwnerControlSessions,
    request: StartBundledManagedRuntimeRequestV1,
) -> Result<OwnerResult, String> {
    (|| {
        sessions.authorize(store, &request.owner_session_id)?;
        macos_managed_runtime_launch::start(
            supervisor,
            store,
            runtime_dir,
            &request.registration_id,
        )
    })()
    .map(|runtime_generation| {
        OwnerResult::StartBundledManagedRuntime(StartBundledManagedRuntimeResponseV1 {
            registration_id: request.registration_id,
            runtime_generation,
            launch_state: "accepted".to_owned(),
        })
    })
}

fn transition_target(value: &str) -> Result<ModuleRegistrationState, String> {
    match value {
        "suspended" => Ok(ModuleRegistrationState::Suspended),
        "revoked" => Ok(ModuleRegistrationState::Revoked),
        _ => Err("owner control transition is unavailable".to_owned()),
    }
}

fn response(result: Result<OwnerResult, String>) -> OwnerControlResponseV1 {
    match result {
        Ok(result) => OwnerControlResponseV1 {
            result: Some(result),
            error_code: String::new(),
        },
        Err(error) => {
            tracing::warn!(
                event = "owner_control.operation.denied",
                error.class = "owner_control",
                error.message = %error,
            );
            OwnerControlResponseV1 {
                result: None,
                error_code: "operation_denied".to_owned(),
            }
        }
    }
}

fn unix_millis() -> Result<u64, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .map_err(|_| "owner control clock is unavailable".to_owned())
}
