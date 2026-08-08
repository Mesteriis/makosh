//! Local developer bootstrap for the signed platform foundation.
//!
//! It starts only runtimes for which the private development profile has a
//! complete local configuration. Event Hub and Scheduler remain owner-configured
//! until their NATS and module-grant records are provisioned.

use makosh_kernel_control_store::{
    ModuleDescriptorRegistrationRequestsV1, ModuleEventDeliveryPolicyV1, ModuleEventEnvelopeKindV1,
    ModuleEventRouteDirectionV1, ModuleEventRouteRequestInputV1, ModuleEventRouteRequestV1,
    ModuleEventSubscriptionRequirementV1, ModuleRegistration, ModuleRegistrationState,
    ModuleStorageRequestV1, PlatformEventHubTopologyV1, PlatformEventStreamBudgetV1,
    PlatformEventsAuthorityConfigurationV1, PlatformStorageBindingStateV1, PlatformStorageBundleV1,
    PlatformStorageEndpointV1, PlatformStorageTopology, PlatformStorageTopologyInputV1,
    StorageDeploymentProfileV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_scheduler_protocol::SCHEDULER_JOB_DESCRIPTOR_SET_V1;
use makosh_storage_protocol::v1::StorageBundleV1;
use prost::Message;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::Command;

use crate::platform::{
    blob::{binding as blob_binding, launch as blob_launch},
    events::authority::{binding as events_binding, launch as events_launch},
    macos::managed_launch,
    macos::native_launch,
    storage::{binding as storage_binding, launch as storage_launch},
    telemetry::{binding as telemetry_binding, launch as telemetry_launch},
    vault::{binding as vault_binding, launch as vault_launch},
};
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;
use crate::{
    distribution::bundled_launch,
    platform::storage::issuance::{StorageBindingIssueV1, issue_managed},
};

mod vault;

/// Starts the signed local platform runtimes that have a complete local
/// configuration. It is called only for an explicitly selected development
/// Gateway profile.
pub(crate) fn start_local_foundation(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    data_dir: &Path,
    runtime_dir: &Path,
) -> Result<(), String> {
    let vault_binding = ensure_vault_binding(store)?;
    vault::ensure_initialized_vault(
        data_dir,
        runtime_dir,
        &vault_binding,
        store.snapshot().instance_id(),
    )?;
    ensure_blob_binding(store)?;
    ensure_telemetry_binding(store)?;
    restore_scheduler_developer_grants(supervisor, store)?;
    ensure_storage_binding(store)?;
    ensure_storage_topology(store)?;
    ensure_events_binding(store)?;
    ensure_events_configuration(store, data_dir)?;
    ensure_event_hub_topology(store)?;

    if !supervisor.is_active(vault_binding::VAULT_PROCESS_ID)? {
        vault_launch::start(supervisor, store, data_dir, runtime_dir)
            .map_err(|error| format!("developer Vault startup failed: {error}"))?;
    }
    if !supervisor.is_active(blob_binding::BLOB_PROCESS_ID)? {
        blob_launch::start(supervisor, store, data_dir, runtime_dir)
            .map_err(|error| format!("developer Blob startup failed: {error}"))?;
    }
    if !supervisor.is_active(telemetry_binding::TELEMETRY_PROCESS_ID)? {
        telemetry_launch::start(supervisor, store, data_dir, runtime_dir)
            .map_err(|error| format!("developer Telemetry startup failed: {error}"))?;
    }
    if !supervisor.is_active(storage_binding::STORAGE_PROCESS_ID)? {
        storage_launch::start(supervisor, store, runtime_dir)
            .map_err(|error| format!("developer Storage startup failed: {error}"))?;
    }
    if !supervisor.is_active(events_binding::EVENTS_AUTHORITY_PROCESS_ID)? {
        events_launch::start(supervisor, store, runtime_dir)
            .map_err(|error| format!("developer Event Hub startup failed: {error}"))?;
    }
    ensure_scheduler(supervisor, store, runtime_dir)?;
    Ok(())
}

fn restore_scheduler_developer_grants(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
) -> Result<(), String> {
    const REGISTRATION_ID: &str = "scheduler_developer";
    let capabilities = scheduler_capabilities();
    if store
        .module_registration(REGISTRATION_ID)
        .map_err(|_| "developer Scheduler registration is unavailable".to_owned())?
        .is_some_and(|registration| registration.state() != ModuleRegistrationState::Approved)
    {
        store
            .approve_module_registration(REGISTRATION_ID, &capabilities)
            .map_err(|_| "developer Scheduler grants cannot be restored".to_owned())?;
    }
    if !supervisor.is_active(REGISTRATION_ID)?
        && let Some(binding) = store
            .platform_storage_binding(REGISTRATION_ID, "storage.scheduler")
            .map_err(|_| "developer Scheduler Storage binding is unavailable".to_owned())?
        && binding.state() == PlatformStorageBindingStateV1::Active
    {
        store
            .begin_platform_storage_binding_revocation(
                REGISTRATION_ID,
                "storage.scheduler",
                binding.binding_revision(),
            )
            .map_err(|_| "developer Scheduler Storage binding cannot be fenced".to_owned())?;
    }
    Ok(())
}

fn ensure_vault_binding(
    store: &SqliteControlStore,
) -> Result<makosh_kernel_control_store::PlatformManagedProcessBinding, String> {
    // The developer bundle may be rebuilt between local launches. Rebind the
    // exact installed artifact rather than retaining a stale digest from a
    // prior temporary bundle.
    vault_binding::bind_current_installed_release(store)
}

fn ensure_blob_binding(store: &SqliteControlStore) -> Result<(), String> {
    blob_binding::bind_current_installed_release(store)?;
    Ok(())
}

fn ensure_telemetry_binding(store: &SqliteControlStore) -> Result<(), String> {
    telemetry_binding::bind_current_installed_release(store)?;
    Ok(())
}

fn ensure_storage_binding(store: &SqliteControlStore) -> Result<(), String> {
    storage_binding::bind_current_installed_release(store)?;
    Ok(())
}

fn ensure_storage_topology(store: &SqliteControlStore) -> Result<(), String> {
    let existing = store
        .platform_storage_topology()
        .map_err(|_| "developer Storage topology is unavailable".to_owned())?;
    if existing.is_some() {
        return Ok(());
    }
    // The development operator owns this private localhost contour. The
    // database and pooler image identities are explicit dev-profile markers;
    // production topology continues to require owner-authorized image pins.
    let postgres_digest: [u8; 32] =
        Sha256::digest(b"postgres:16-alpine/dev-authenticated-v1").into();
    let pgbouncer_digest: [u8; 32] =
        Sha256::digest(b"edoburu/pgbouncer:v1.25.2-p0/dev-authenticated-v1").into();
    let topology = PlatformStorageTopology::new(PlatformStorageTopologyInputV1 {
        revision: 1,
        storage_generation: 1,
        storage_instance_id: "storage_main".to_owned(),
        database_id: "makosh_storage_authenticated".to_owned(),
        deployment_profile: StorageDeploymentProfileV1::MacosTauriEmbedded,
        postgres_endpoint: PlatformStorageEndpointV1::new("127.0.0.1", 35_532),
        pgbouncer_endpoint: PlatformStorageEndpointV1::new("127.0.0.1", 36_532),
        postgres_artifact_sha256: postgres_digest,
        pgbouncer_artifact_sha256: pgbouncer_digest,
    })
    .with_pgbouncer_backend_endpoint(PlatformStorageEndpointV1::new("postgres", 5_432));
    store
        .record_platform_storage_topology(&topology)
        .map_err(|_| "developer Storage topology cannot be recorded".to_owned())
}

fn ensure_events_binding(store: &SqliteControlStore) -> Result<(), String> {
    events_binding::bind_current_installed_release(store).map(|_| ())
}

fn ensure_events_configuration(store: &SqliteControlStore, data_dir: &Path) -> Result<(), String> {
    if store
        .platform_events_authority_configuration()
        .map_err(|_| "developer Event Hub configuration is unavailable".to_owned())?
        .is_some()
    {
        return Ok(());
    }
    let account_public_key = std::fs::read_to_string(
        data_dir
            .join("developer-platform-credentials")
            .join("nats-account-public-key"),
    )
    .map_err(|_| "developer Event Hub identity is unavailable".to_owned())?;
    let configuration =
        PlatformEventsAuthorityConfigurationV1::new(1, account_public_key.trim(), 1);
    store
        .record_platform_events_authority_configuration(&configuration)
        .map_err(|_| "developer Event Hub configuration cannot be recorded".to_owned())
}

fn ensure_event_hub_topology(store: &SqliteControlStore) -> Result<(), String> {
    if store
        .platform_event_hub_topology()
        .map_err(|_| "developer Event Hub topology is unavailable".to_owned())?
        .is_some()
    {
        return Ok(());
    }
    let budgets = [
        ModuleEventEnvelopeKindV1::Command,
        ModuleEventEnvelopeKindV1::Event,
        ModuleEventEnvelopeKindV1::Observation,
        ModuleEventEnvelopeKindV1::Result,
        ModuleEventEnvelopeKindV1::Ack,
    ]
    .into_iter()
    .map(|kind| PlatformEventStreamBudgetV1::new(kind, 1_048_576, 3_600_000, 1))
    .collect();
    let topology =
        PlatformEventHubTopologyV1::new(1, "nats://127.0.0.1:43225", "event_hub", 1, budgets);
    store
        .record_platform_event_hub_topology(&topology)
        .map_err(|_| "developer Event Hub topology cannot be recorded".to_owned())
}

fn ensure_scheduler(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
) -> Result<(), String> {
    const REGISTRATION_ID: &str = "scheduler_developer";
    const STORAGE_CAPABILITY: &str = "storage.scheduler";
    const DISPATCH_CAPABILITY: &str = "events.scheduler.dispatch";
    const ACK_CAPABILITY: &str = "events.scheduler.ack";
    const RESULT_CAPABILITY: &str = "events.scheduler.result";

    if supervisor.is_active(REGISTRATION_ID)? {
        return Ok(());
    }
    let kernel =
        std::env::current_exe().map_err(|_| "Kernel executable path is unavailable".to_owned())?;
    let bundle = native_launch::verify_selected_installed_bundle_artifact_ids(
        &kernel,
        "aarch64-apple-darwin",
        &["platform.scheduler"],
    )?;
    let artifact = bundle
        .artifacts()
        .iter()
        .find(|artifact| artifact.artifact_id() == "platform.scheduler")
        .ok_or_else(|| "developer Scheduler release is unavailable".to_owned())?;
    let descriptor = artifact
        .module_descriptor_bytes()
        .ok_or_else(|| "developer Scheduler descriptor is unavailable".to_owned())?;
    ensure_scheduler_registration(store, descriptor)?;
    bundled_launch::admit(store, REGISTRATION_ID, &bundle, "platform.scheduler")?;
    let (bundle_revision, bundle_digest) =
        ensure_scheduler_storage_bundle(store, &kernel, runtime_dir)?;
    let reservation = managed_launch::reserve(supervisor, store, REGISTRATION_ID)?;
    let binding_issue = match store
        .platform_storage_binding(REGISTRATION_ID, STORAGE_CAPABILITY)
        .map_err(|_| "developer Scheduler Storage binding is unavailable".to_owned())?
    {
        Some(binding) => crate::platform::storage::successor::issue_after_with_bundle(
            &binding,
            bundle_revision,
            bundle_digest,
        )?,
        None => StorageBindingIssueV1::new(1, 1, bundle_revision, bundle_digest)?,
    };
    let binding = issue_managed(
        store,
        REGISTRATION_ID,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        STORAGE_CAPABILITY,
        binding_issue,
    )?;
    // Scheduler's consumers are part of the approved topology. Reconcile only
    // after its registration and grants are durable, but before its runtime can
    // attempt to attach a receipt consumer.
    crate::platform::events::reconciliation::apply(store, &supervisor.relay_port())?;
    crate::platform::scheduler::launch::start_from_reservation(
        supervisor,
        store,
        &kernel,
        runtime_dir,
        reservation,
        &binding,
    )
    .map(|_| ())
    .map_err(|error| format!("developer Scheduler startup failed: {error}"))
}

fn ensure_scheduler_registration(
    store: &SqliteControlStore,
    descriptor: &[u8],
) -> Result<(), String> {
    const REGISTRATION_ID: &str = "scheduler_developer";
    const STORAGE_CAPABILITY: &str = "storage.scheduler";
    let capabilities = scheduler_capabilities();
    let descriptor_sha256: [u8; 32] = Sha256::digest(descriptor).into();
    let registration = store
        .module_registration(REGISTRATION_ID)
        .map_err(|_| "developer Scheduler registration is unavailable".to_owned())?;
    if registration.is_none() {
        let registration = ModuleRegistration::new(
            REGISTRATION_ID,
            "scheduler",
            "scheduler",
            descriptor_sha256,
            ModuleRegistrationState::Pending,
            1,
        );
        let storage =
            ModuleStorageRequestV1::new(REGISTRATION_ID, STORAGE_CAPABILITY, "scheduler", 4, 5_000);
        let routes = scheduler_event_routes();
        store
            .create_pending_registration_with_requests(
                &registration,
                &capabilities,
                std::slice::from_ref(&storage),
                &routes,
                &[],
            )
            .map_err(|_| "developer Scheduler registration cannot be recorded")?;
        return store
            .approve_module_registration(REGISTRATION_ID, &capabilities)
            .map(|_| ())
            .map_err(|_| "developer Scheduler grants cannot be recorded".to_owned());
    }
    let current = registration.expect("checked registration");
    let routes = scheduler_event_routes();
    let routes_are_current = scheduler_event_routes_are_current(store, &routes)?;
    if current.state() == ModuleRegistrationState::Approved
        && (current.descriptor_sha256() != &descriptor_sha256 || !routes_are_current)
    {
        let next_epoch = current
            .grant_epoch()
            .checked_add(1)
            .ok_or_else(|| "developer Scheduler grant epoch overflowed".to_owned())?;
        let upgraded = ModuleRegistration::new(
            REGISTRATION_ID,
            "scheduler",
            "scheduler",
            descriptor_sha256,
            ModuleRegistrationState::Approved,
            next_epoch,
        );
        let storage =
            ModuleStorageRequestV1::new(REGISTRATION_ID, STORAGE_CAPABILITY, "scheduler", 4, 5_000);
        return store
            .upgrade_approved_registration_with_all_descriptor_requests(
                &upgraded,
                &capabilities,
                ModuleDescriptorRegistrationRequestsV1 {
                    storage: std::slice::from_ref(&storage),
                    events: &routes,
                    blobs: &[],
                    scheduler: &[],
                    vault_purposes: &[],
                    client_rpc_routes: &[],
                    client_blob_routes: &[],
                    client_realtime_routes: &[],
                    query_rpc_routes: &[],
                    request_rpc_routes: &[],
                    contract_dependencies: &[],
                },
            )
            .map_err(|_| "developer Scheduler registration cannot be upgraded".to_owned());
    }
    (current.state() != ModuleRegistrationState::Approved)
        .then(|| store.approve_module_registration(REGISTRATION_ID, &capabilities))
        .transpose()
        .map(|_| ())
        .map_err(|_| "developer Scheduler grants cannot be restored".to_owned())
}

fn scheduler_event_routes_are_current(
    store: &SqliteControlStore,
    expected: &[ModuleEventRouteRequestV1],
) -> Result<bool, String> {
    for route in expected {
        let current = store
            .module_event_route_requests("scheduler_developer", route.capability_id())
            .map_err(|_| "developer Scheduler event routes are unavailable".to_owned())?;
        if current.as_slice() != std::slice::from_ref(route) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn scheduler_capabilities() -> [String; 6] {
    [
        "events.scheduler.ack",
        "events.scheduler.dispatch",
        "events.scheduler.result",
        "events.scheduler.schedule_control.command",
        "events.scheduler.schedule_control.result",
        "storage.scheduler",
    ]
    .map(str::to_owned)
}

fn scheduler_event_routes() -> [ModuleEventRouteRequestV1; 5] {
    let scheduler_schema: [u8; 32] = Sha256::digest(SCHEDULER_JOB_DESCRIPTOR_SET_V1).into();
    [
        scheduler_event_route(
            "events.scheduler.dispatch",
            ModuleEventEnvelopeKindV1::Command,
            "platform",
            "maintenance",
            ModuleEventRouteDirectionV1::Publish,
            [7; 32],
        ),
        scheduler_event_route(
            "events.scheduler.ack",
            ModuleEventEnvelopeKindV1::Ack,
            "scheduler",
            "job_receipt",
            ModuleEventRouteDirectionV1::Consume,
            scheduler_schema,
        ),
        scheduler_event_route(
            "events.scheduler.result",
            ModuleEventEnvelopeKindV1::Result,
            "scheduler",
            "job_receipt",
            ModuleEventRouteDirectionV1::Consume,
            scheduler_schema,
        ),
        scheduler_event_route(
            "events.scheduler.schedule_control.command",
            ModuleEventEnvelopeKindV1::Command,
            "scheduler",
            "schedule_control",
            ModuleEventRouteDirectionV1::Consume,
            scheduler_schema,
        ),
        scheduler_event_route(
            "events.scheduler.schedule_control.result",
            ModuleEventEnvelopeKindV1::Result,
            "scheduler",
            "schedule_control",
            ModuleEventRouteDirectionV1::Publish,
            scheduler_schema,
        ),
    ]
}

fn ensure_scheduler_storage_bundle(
    store: &SqliteControlStore,
    kernel: &Path,
    runtime_dir: &Path,
) -> Result<(u64, [u8; 32]), String> {
    let bytes = export_scheduler_storage_bundle(kernel, runtime_dir)?;
    let exported = StorageBundleV1::decode(bytes.as_slice())
        .map_err(|_| "developer Scheduler Storage bundle is invalid".to_owned())?;
    if exported.major != 1
        || exported.owner_id != "scheduler"
        || exported.bundle_id != "scheduler_state"
        || exported.revision == 0
    {
        return Err("developer Scheduler Storage bundle is invalid".to_owned());
    }
    let revision = u64::from(exported.revision);
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    let bundle = PlatformStorageBundleV1::new("scheduler", revision, digest, bytes)
        .map_err(|_| "developer Scheduler Storage bundle is invalid".to_owned())?;
    match store
        .platform_storage_bundle("scheduler", bundle.revision())
        .map_err(|_| "developer Scheduler Storage bundle is unavailable".to_owned())?
    {
        Some(existing) if existing.digest() == bundle.digest() => Ok((revision, digest)),
        Some(_) => Err(
            "developer Scheduler Storage bundle revision conflicts with the persisted contract"
                .to_owned(),
        ),
        None => {
            store
                .record_platform_storage_bundle(&bundle)
                .map_err(|_| "developer Scheduler Storage bundle cannot be recorded".to_owned())?;
            Ok((revision, digest))
        }
    }
}

fn scheduler_event_route(
    capability: &str,
    kind: ModuleEventEnvelopeKindV1,
    owner: &str,
    contract: &str,
    direction: ModuleEventRouteDirectionV1,
    schema_sha256: [u8; 32],
) -> ModuleEventRouteRequestV1 {
    ModuleEventRouteRequestV1::new(ModuleEventRouteRequestInputV1 {
        registration_id: "scheduler_developer".to_owned(),
        capability_id: capability.to_owned(),
        envelope_kind: kind,
        contract_owner: owner.to_owned(),
        contract_name: contract.to_owned(),
        contract_major: 1,
        contract_revision: 1,
        contract_schema_sha256: schema_sha256,
        direction,
        max_in_flight: 16,
        delivery_policy: matches!(direction, ModuleEventRouteDirectionV1::Consume).then(|| {
            ModuleEventDeliveryPolicyV1::new(
                ModuleEventSubscriptionRequirementV1::Required,
                3,
                2_000,
            )
        }),
    })
}

fn export_scheduler_storage_bundle(kernel: &Path, runtime_dir: &Path) -> Result<Vec<u8>, String> {
    let staged = native_launch::verify_selected_installed_release(
        kernel,
        "aarch64-apple-darwin",
        "platform.scheduler",
        &runtime_dir
            .join("developer-bootstrap")
            .join("scheduler-bundle"),
    )?;
    let output = Command::new(staged.path())
        .arg("export-storage-bundle")
        .output()
        .map_err(|_| "developer Scheduler Storage bundle is unavailable".to_owned());
    let _ = staged.remove();
    let output = output?;
    (output.status.success() && !output.stdout.is_empty())
        .then_some(output.stdout)
        .ok_or_else(|| "developer Scheduler Storage bundle is unavailable".to_owned())
}
