//! Exact signed Persons domain admission and owner-local lifecycle.

use super::*;

use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_persons_api::{PERSONS_MODULE_ID_V1, PERSONS_OWNER_ID_V1};
use makosh_persons_persistence::{PERSONS_STORAGE_BUNDLE_REVISION_V1, persons_storage_bundle_v1};
use makosh_persons_runtime::{
    PERSONS_STORAGE_CAPABILITY_ID_V1, persons_module_descriptor_v1,
    persons_settings_schema_bytes_v1,
};

const PERSONS_RELEASE_ARTIFACT_ID_V1: &str = "persons.runtime.v1";
pub(super) const PERSONS_LOGICAL_HUMAN_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedPersonsRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedPersonsRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    capability_ids: Vec<String>,
}

pub(super) enum PersonsBootstrapOverrideV1 {
    None,
    StopVaultAfterConfiguration,
    UnavailableStoragePort(u16),
    UnavailableEventEndpoint(String),
}

pub(super) fn installed_persons_release_v1(root: &Path) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(persons_release_artifact_v1());
    InstalledSignedBundle::install(root, &artifacts).expect("install signed Persons release")
}

pub(super) fn persons_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        PERSONS_RELEASE_ARTIFACT_ID_V1,
        persons_binary(),
        persons_module_descriptor_v1("managed-persons-live").encode_to_vec(),
    )
    .with_settings_schema(persons_settings_schema_bytes_v1())
}

pub(super) fn admit_persons_runtime_v1(store: &SqliteControlStore) -> AdmittedPersonsRuntimeV1 {
    let descriptor = persons_module_descriptor_v1("managed-persons-live");
    assert_eq!(descriptor.module_id, PERSONS_MODULE_ID_V1);
    assert_eq!(descriptor.owner_id, PERSONS_OWNER_ID_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Persons descriptor");
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
    .expect("approve exact Persons capabilities");
    let settings = persons_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            PERSONS_RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(std::fs::read(persons_binary()).expect("Persons runtime binary")).into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record Persons release binding");
    let bundle = persons_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                PERSONS_OWNER_ID_V1,
                u64::from(PERSONS_STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Persons Storage bundle"),
        )
        .expect("persist Persons Storage bundle");
    AdmittedPersonsRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_persons_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedPersonsRuntimeV1,
) -> AdmittedPersonsRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Persons launch");
    let bundle = store
        .platform_storage_bundle(
            PERSONS_OWNER_ID_V1,
            u64::from(PERSONS_STORAGE_BUNDLE_REVISION_V1),
        )
        .expect("read Persons Storage bundle")
        .expect("Persons Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        PERSONS_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(PERSONS_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("Persons Storage binding issue"),
    )
    .expect("issue Persons Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Persons Storage binding");
    admitted
}

pub(super) fn start_persons_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedPersonsRuntimeV1,
) -> StartedPersonsRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Persons launch reservation");
    start_reserved_persons_runtime_v1(supervisor, store, runtime_dir, reservation, admitted)
}

pub(super) fn restart_persons_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedPersonsRuntimeV1,
) -> StartedPersonsRuntimeV1 {
    let previous_generation = predecessor.runtime_generation;
    let previous_instance = predecessor.runtime_instance_id.clone();
    let binding = persons_storage_binding_v1(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&binding).expect("derive Persons successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        PERSONS_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Persons successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Persons successor");
    let successor = start_reserved_persons_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        AdmittedPersonsRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
    );
    assert_eq!(successor.runtime_generation, previous_generation + 1);
    assert_ne!(successor.runtime_instance_id, previous_instance);
    successor
}

fn start_reserved_persons_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedPersonsRuntimeV1,
) -> StartedPersonsRuntimeV1 {
    launch_reserved_persons_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        admitted,
        PersonsBootstrapOverrideV1::None,
        true,
    )
}

pub(super) fn launch_persons_runtime_without_ready_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedPersonsRuntimeV1,
    bootstrap_override: PersonsBootstrapOverrideV1,
) -> StartedPersonsRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Persons launch reservation");
    launch_reserved_persons_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        admitted,
        bootstrap_override,
        false,
    )
}

pub(super) fn launch_persons_successor_without_ready_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedPersonsRuntimeV1,
    bootstrap_override: PersonsBootstrapOverrideV1,
) -> StartedPersonsRuntimeV1 {
    let binding = persons_storage_binding_v1(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&binding).expect("derive Persons successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        PERSONS_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Persons successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Persons successor");
    launch_reserved_persons_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        AdmittedPersonsRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
        bootstrap_override,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
fn launch_reserved_persons_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedPersonsRuntimeV1,
    bootstrap_override: PersonsBootstrapOverrideV1,
    wait_until_ready: bool,
) -> StartedPersonsRuntimeV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = persons_storage_binding_v1(store, &admitted.registration_id);
    let topology =
        crate::platform::storage::topology::current(store).expect("Persons Storage topology");
    let vault = vault_status::read_current(store, &supervisor.relay_port())
        .expect("live Vault status for Persons");
    let mut storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("Persons Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Persons Event Hub topology")
        .expect("Persons Event Hub topology");
    let mut event_hub_endpoint = events.nats_endpoint().to_owned();
    match bootstrap_override {
        PersonsBootstrapOverrideV1::None => {}
        PersonsBootstrapOverrideV1::StopVaultAfterConfiguration => {
            supervisor
                .stop(vault_binding::VAULT_PROCESS_ID)
                .expect("stop Vault after capturing Persons configuration");
        }
        PersonsBootstrapOverrideV1::UnavailableStoragePort(port) => {
            storage.pgbouncer_port = u32::from(port);
        }
        PersonsBootstrapOverrideV1::UnavailableEventEndpoint(endpoint) => {
            event_hub_endpoint = endpoint;
        }
    }
    managed_launch::start_reserved_domain(
        supervisor,
        runtime_dir,
        reservation,
        ManagedDomainRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: PERSONS_OWNER_ID_V1.to_owned(),
            registration_id: admitted.registration_id.clone(),
            runtime_instance_id: runtime_instance_id.clone(),
            runtime_generation,
            grant_epoch,
            storage: Some(storage),
            event_hub_endpoint,
            event_credential_revision: events.credential_revision(),
            logical_human_owner_id: PERSONS_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
        },
    )
    .expect("start managed Persons domain");
    if wait_until_ready {
        supervisor
            .wait_until_ready(&admitted.registration_id)
            .unwrap_or_else(|error| {
                panic!(
                    "Persons readiness: {error}; last_failure={:?}",
                    supervisor.last_failure(&admitted.registration_id)
                )
            });
    }
    StartedPersonsRuntimeV1 {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids: admitted.capability_ids,
    }
}

fn persons_storage_binding_v1(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(registration_id, PERSONS_STORAGE_CAPABILITY_ID_V1)
        .expect("read Persons Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active Persons Storage binding")
}

fn persons_binary() -> PathBuf {
    binary("MAKOSH_PERSONS_RUNTIME_BIN")
}
