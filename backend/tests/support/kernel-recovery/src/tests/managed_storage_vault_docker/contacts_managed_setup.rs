//! Exact signed Contacts domain admission and owner-local lifecycle.

use super::*;

use makosh_contacts_command_api::{CONTACTS_MODULE_ID_V1, CONTACTS_OWNER_ID_V1};
use makosh_contacts_persistence::{
    CONTACTS_STORAGE_BUNDLE_REVISION_V1, contacts_storage_bundle_v1,
};
use makosh_contacts_runtime::{
    CONTACTS_STORAGE_CAPABILITY_ID_V1, contacts_module_descriptor_v1,
    contacts_settings_schema_bytes_v1,
};
use makosh_kernel_control_store::PlatformStorageBindingStateV1;

const CONTACTS_RELEASE_ARTIFACT_ID_V1: &str = "contacts.runtime.v1";
pub(super) const CONTACTS_LOGICAL_HUMAN_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedContactsRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedContactsRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    capability_ids: Vec<String>,
}

pub(super) fn installed_contacts_release_v1(root: &Path) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(contacts_release_artifact_v1());
    InstalledSignedBundle::install(root, &artifacts).expect("install signed Contacts release")
}

pub(super) fn contacts_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        CONTACTS_RELEASE_ARTIFACT_ID_V1,
        contacts_binary(),
        contacts_module_descriptor_v1("managed-contacts-live").encode_to_vec(),
    )
    .with_settings_schema(contacts_settings_schema_bytes_v1())
}

pub(super) fn admit_contacts_runtime_v1(store: &SqliteControlStore) -> AdmittedContactsRuntimeV1 {
    let descriptor = contacts_module_descriptor_v1("managed-contacts-live");
    assert_eq!(descriptor.module_id, CONTACTS_MODULE_ID_V1);
    assert_eq!(descriptor.owner_id, CONTACTS_OWNER_ID_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Contacts descriptor");
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
    .expect("approve exact Contacts capabilities");
    let settings = contacts_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            CONTACTS_RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(std::fs::read(contacts_binary()).expect("Contacts runtime binary"))
                .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record Contacts release binding");
    let bundle = contacts_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                CONTACTS_OWNER_ID_V1,
                u64::from(CONTACTS_STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Contacts Storage bundle"),
        )
        .expect("persist Contacts Storage bundle");
    AdmittedContactsRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_contacts_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedContactsRuntimeV1,
) -> AdmittedContactsRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Contacts launch");
    let bundle = store
        .platform_storage_bundle(
            CONTACTS_OWNER_ID_V1,
            u64::from(CONTACTS_STORAGE_BUNDLE_REVISION_V1),
        )
        .expect("read Contacts Storage bundle")
        .expect("Contacts Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        CONTACTS_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(CONTACTS_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("Contacts Storage binding issue"),
    )
    .expect("issue Contacts Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Contacts Storage binding");
    admitted
}

pub(super) fn start_contacts_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedContactsRuntimeV1,
) -> StartedContactsRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Contacts launch reservation");
    start_reserved_contacts_runtime_v1(supervisor, store, runtime_dir, reservation, admitted)
}

pub(super) fn restart_contacts_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedContactsRuntimeV1,
) -> StartedContactsRuntimeV1 {
    let previous_generation = predecessor.runtime_generation;
    let previous_instance = predecessor.runtime_instance_id.clone();
    let binding = contacts_storage_binding_v1(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&binding).expect("derive Contacts successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        CONTACTS_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Contacts successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Contacts successor");
    let successor = start_reserved_contacts_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        AdmittedContactsRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
    );
    assert_eq!(successor.runtime_generation, previous_generation + 1);
    assert_ne!(successor.runtime_instance_id, previous_instance);
    successor
}

fn start_reserved_contacts_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedContactsRuntimeV1,
) -> StartedContactsRuntimeV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = contacts_storage_binding_v1(store, &admitted.registration_id);
    let topology =
        crate::platform::storage::topology::current(store).expect("Contacts Storage topology");
    let vault = vault_status::read_current(store, &supervisor.relay_port())
        .expect("live Vault status for Contacts");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("Contacts Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Contacts Event Hub topology")
        .expect("Contacts Event Hub topology");
    managed_launch::start_reserved_domain(
        supervisor,
        runtime_dir,
        reservation,
        ManagedDomainRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: CONTACTS_OWNER_ID_V1.to_owned(),
            registration_id: admitted.registration_id.clone(),
            runtime_instance_id: runtime_instance_id.clone(),
            runtime_generation,
            grant_epoch,
            storage: Some(storage),
            event_hub_endpoint: events.nats_endpoint().to_owned(),
            event_credential_revision: events.credential_revision(),
            logical_human_owner_id: CONTACTS_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
        },
    )
    .expect("start managed Contacts domain");
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "Contacts readiness: {error}; last_failure={:?}",
                supervisor.last_failure(&admitted.registration_id)
            )
        });
    StartedContactsRuntimeV1 {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids: admitted.capability_ids,
    }
}

fn contacts_storage_binding_v1(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(registration_id, CONTACTS_STORAGE_CAPABILITY_ID_V1)
        .expect("read Contacts Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active Contacts Storage binding")
}

fn contacts_binary() -> PathBuf {
    binary("MAKOSH_CONTACTS_RUNTIME_BIN")
}
