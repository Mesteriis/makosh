//! Exact signed Organizations domain admission for the owner lifecycle contour.

use super::*;

use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_organizations_api::{ORGANIZATIONS_MODULE_ID_V1, ORGANIZATIONS_OWNER_ID_V1};
use makosh_organizations_persistence::{
    ORGANIZATIONS_STORAGE_BUNDLE_REVISION_V1, organizations_storage_bundle_v1,
};
use makosh_organizations_runtime::{
    organizations_module_descriptor_v1, organizations_settings_schema_bytes_v1,
};

const ORGANIZATIONS_RELEASE_ARTIFACT_ID_V1: &str = "organizations.runtime.v1";
pub(super) const ORGANIZATIONS_LOGICAL_HUMAN_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedOrganizationsRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedOrganizationsRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    capability_ids: Vec<String>,
}

pub(super) fn installed_organizations_release_v1(root: &Path) -> InstalledSignedBundle {
    InstalledSignedBundle::install(
        root,
        &[
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
            SignedRuntimeArtifact::new(
                ORGANIZATIONS_RELEASE_ARTIFACT_ID_V1,
                organizations_binary_v1(),
                organizations_module_descriptor_v1("managed-organizations-live").encode_to_vec(),
            )
            .with_settings_schema(organizations_settings_schema_bytes_v1()),
        ],
    )
    .expect("install signed Organizations release")
}

pub(super) fn admit_organizations_runtime_v1(
    store: &SqliteControlStore,
) -> AdmittedOrganizationsRuntimeV1 {
    let descriptor = organizations_module_descriptor_v1("managed-organizations-live");
    assert_eq!(descriptor.module_id, ORGANIZATIONS_MODULE_ID_V1);
    assert_eq!(descriptor.owner_id, ORGANIZATIONS_OWNER_ID_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Organizations descriptor");
    let capability_ids = descriptor
        .capabilities
        .iter()
        .map(|value| value.capability_id.clone())
        .collect::<Vec<_>>();
    crate::modules::registration::registry::approve_after_owner_authorization(
        store,
        registration.registration_id(),
        &capability_ids,
    )
    .expect("approve exact Organizations capabilities");
    let settings = organizations_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            ORGANIZATIONS_RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(
                std::fs::read(organizations_binary_v1()).expect("Organizations runtime binary"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record Organizations release binding");
    let bundle = organizations_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                ORGANIZATIONS_OWNER_ID_V1,
                u64::from(ORGANIZATIONS_STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Organizations Storage bundle"),
        )
        .expect("persist Organizations Storage bundle");
    AdmittedOrganizationsRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_organizations_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedOrganizationsRuntimeV1,
) -> AdmittedOrganizationsRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Organizations launch");
    let bundle = store
        .platform_storage_bundle(
            ORGANIZATIONS_OWNER_ID_V1,
            u64::from(ORGANIZATIONS_STORAGE_BUNDLE_REVISION_V1),
        )
        .expect("read Organizations bundle")
        .expect("Organizations bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        makosh_organizations_api::ORGANIZATIONS_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(ORGANIZATIONS_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("Organizations Storage binding issue"),
    )
    .expect("issue Organizations Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Organizations Storage binding");
    admitted
}

pub(super) fn start_organizations_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedOrganizationsRuntimeV1,
) -> StartedOrganizationsRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Organizations reservation");
    start_reserved_organizations_runtime_v1(supervisor, store, runtime_dir, reservation, admitted)
}

pub(super) fn restart_organizations_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedOrganizationsRuntimeV1,
) -> StartedOrganizationsRuntimeV1 {
    let previous_generation = predecessor.runtime_generation;
    let previous_instance = predecessor.runtime_instance_id.clone();
    let binding = organizations_storage_binding_v1(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&binding).expect("derive Organizations successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        makosh_organizations_api::ORGANIZATIONS_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Organizations successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Organizations successor binding");
    let successor = start_reserved_organizations_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        AdmittedOrganizationsRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
    );
    assert_eq!(successor.runtime_generation, previous_generation + 1);
    assert_ne!(successor.runtime_instance_id, previous_instance);
    successor
}

fn start_reserved_organizations_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedOrganizationsRuntimeV1,
) -> StartedOrganizationsRuntimeV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = organizations_storage_binding_v1(store, &admitted.registration_id);
    let topology =
        crate::platform::storage::topology::current(store).expect("Organizations Storage topology");
    let vault =
        vault_status::read_current(store, &supervisor.relay_port()).expect("live Vault status");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("Organizations Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    managed_launch::start_reserved_domain(
        supervisor,
        runtime_dir,
        reservation,
        ManagedDomainRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: ORGANIZATIONS_OWNER_ID_V1.to_owned(),
            registration_id: admitted.registration_id.clone(),
            runtime_instance_id: runtime_instance_id.clone(),
            runtime_generation,
            grant_epoch,
            storage: Some(storage),
            event_hub_endpoint: events.nats_endpoint().to_owned(),
            event_credential_revision: events.credential_revision(),
            logical_human_owner_id: ORGANIZATIONS_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
        },
    )
    .expect("start managed Organizations domain");
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "Organizations readiness: {error}; last_failure={:?}",
                supervisor.last_failure(&admitted.registration_id)
            )
        });
    StartedOrganizationsRuntimeV1 {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids: admitted.capability_ids,
    }
}

fn organizations_storage_binding_v1(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(
            registration_id,
            makosh_organizations_api::ORGANIZATIONS_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read Organizations binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active Organizations binding")
}

fn organizations_binary_v1() -> PathBuf {
    binary("MAKOSH_ORGANIZATIONS_RUNTIME_BIN")
}
