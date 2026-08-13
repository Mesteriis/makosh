//! Exact signed Calendar domain admission for the owner lifecycle contour.

use super::*;

use makosh_calendar_api::{CALENDAR_MODULE_ID_V1, CALENDAR_OWNER_ID_V1};
use makosh_calendar_persistence::{
    CALENDAR_STORAGE_BUNDLE_REVISION_V1, calendar_storage_bundle_v1,
};
use makosh_calendar_runtime::{calendar_module_descriptor_v1, calendar_settings_schema_bytes_v1};
use makosh_kernel_control_store::PlatformStorageBindingStateV1;

const CALENDAR_RELEASE_ARTIFACT_ID_V1: &str = "calendar.runtime.v1";
pub(super) const CALENDAR_LOGICAL_HUMAN_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedCalendarRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedCalendarRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    capability_ids: Vec<String>,
}

pub(super) fn installed_calendar_release_v1(root: &Path) -> InstalledSignedBundle {
    let artifacts = vec![
        SignedRuntimeArtifact::new(
            "platform.storage",
            storage_binary(),
            descriptor("storage").encode_to_vec(),
        ),
        SignedRuntimeArtifact::new(
            "platform.vault",
            vault_binary(),
            descriptor("vault").encode_to_vec(),
        ),
        scheduler_release_artifact(),
        calendar_release_artifact_v1(),
    ];
    InstalledSignedBundle::install(root, &artifacts).expect("install signed Calendar release")
}

fn calendar_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        CALENDAR_RELEASE_ARTIFACT_ID_V1,
        calendar_binary_v1(),
        calendar_module_descriptor_v1("managed-calendar-live").encode_to_vec(),
    )
    .with_settings_schema(calendar_settings_schema_bytes_v1())
}

pub(super) fn configured_calendar_store_v1(root: &Path, kernel: &Path) -> SqliteControlStore {
    let store = configured_store(root, kernel);
    record_scheduler_runtime_for_calendar(&store);
    store
}

pub(super) fn admit_calendar_runtime_v1(store: &SqliteControlStore) -> AdmittedCalendarRuntimeV1 {
    let descriptor = calendar_module_descriptor_v1("managed-calendar-live");
    assert_eq!(descriptor.module_id, CALENDAR_MODULE_ID_V1);
    assert_eq!(descriptor.owner_id, CALENDAR_OWNER_ID_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Calendar descriptor");
    let capability_ids = descriptor
        .capabilities
        .iter()
        .map(|capability| capability.capability_id.clone())
        .collect::<Vec<_>>();
    crate::modules::registration::registry::approve_after_owner_authorization(
        store,
        registration.registration_id(),
        &capability_ids,
    )
    .expect("approve exact Calendar capabilities");
    let settings = calendar_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            CALENDAR_RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(std::fs::read(calendar_binary_v1()).expect("Calendar runtime binary"))
                .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record Calendar release binding");
    let bundle = calendar_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                CALENDAR_OWNER_ID_V1,
                u64::from(CALENDAR_STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Calendar Storage bundle"),
        )
        .expect("persist Calendar Storage bundle");
    AdmittedCalendarRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_calendar_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedCalendarRuntimeV1,
) -> AdmittedCalendarRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Calendar launch");
    let bundle = store
        .platform_storage_bundle(
            CALENDAR_OWNER_ID_V1,
            u64::from(CALENDAR_STORAGE_BUNDLE_REVISION_V1),
        )
        .expect("read Calendar Storage bundle")
        .expect("Calendar Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        makosh_calendar_api::CALENDAR_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(CALENDAR_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("Calendar Storage binding issue"),
    )
    .expect("issue Calendar Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Calendar Storage binding");
    admitted
}

pub(super) fn start_calendar_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedCalendarRuntimeV1,
) -> StartedCalendarRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Calendar launch reservation");
    start_reserved_calendar_runtime_v1(supervisor, store, runtime_dir, reservation, admitted)
}

pub(super) fn restart_calendar_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedCalendarRuntimeV1,
) -> StartedCalendarRuntimeV1 {
    let predecessor_generation = predecessor.runtime_generation;
    let predecessor_instance = predecessor.runtime_instance_id.clone();
    let binding = calendar_storage_binding_v1(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&binding).expect("derive Calendar successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        makosh_calendar_api::CALENDAR_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Calendar successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Calendar successor Storage binding");
    let successor = start_reserved_calendar_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        AdmittedCalendarRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
    );
    assert_eq!(successor.runtime_generation, predecessor_generation + 1);
    assert_ne!(successor.runtime_instance_id, predecessor_instance);
    successor
}

fn start_reserved_calendar_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedCalendarRuntimeV1,
) -> StartedCalendarRuntimeV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = calendar_storage_binding_v1(store, &admitted.registration_id);
    let topology =
        crate::platform::storage::topology::current(store).expect("Calendar Storage topology");
    let vault = vault_status::read_current(store, &supervisor.relay_port())
        .expect("live Vault status for Calendar");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("Calendar Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Calendar Event Hub topology")
        .expect("Calendar Event Hub topology");
    managed_launch::start_reserved_domain(
        supervisor,
        runtime_dir,
        reservation,
        ManagedDomainRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: CALENDAR_OWNER_ID_V1.to_owned(),
            registration_id: admitted.registration_id.clone(),
            runtime_instance_id: runtime_instance_id.clone(),
            runtime_generation,
            grant_epoch,
            storage: Some(storage),
            event_hub_endpoint: events.nats_endpoint().to_owned(),
            event_credential_revision: events.credential_revision(),
            logical_human_owner_id: CALENDAR_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
        },
    )
    .expect("start managed Calendar domain");
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "Calendar readiness: {error}; last_failure={:?}",
                supervisor.last_failure(&admitted.registration_id)
            )
        });
    StartedCalendarRuntimeV1 {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids: admitted.capability_ids,
    }
}

fn calendar_storage_binding_v1(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(
            registration_id,
            makosh_calendar_api::CALENDAR_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read Calendar Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active Calendar Storage binding")
}

fn calendar_binary_v1() -> PathBuf {
    binary("MAKOSH_CALENDAR_RUNTIME_BIN")
}
