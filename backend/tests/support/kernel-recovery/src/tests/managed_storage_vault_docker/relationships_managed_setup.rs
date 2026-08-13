//! Exact signed Relationships domain admission for the owner lifecycle contour.

use super::*;

use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_relationships_api::{RELATIONSHIPS_MODULE_ID_V1, RELATIONSHIPS_OWNER_ID_V1};
use makosh_relationships_persistence::{
    RELATIONSHIPS_STORAGE_BUNDLE_REVISION_V1, relationships_storage_bundle_v1,
};
use makosh_relationships_runtime::{
    relationships_module_descriptor_v1, relationships_settings_schema_bytes_v1,
};

const RELATIONSHIPS_RELEASE_ARTIFACT_ID_V1: &str = "relationships.runtime.v1";
pub(super) const RELATIONSHIPS_LOGICAL_HUMAN_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedRelationshipsRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedRelationshipsRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    capability_ids: Vec<String>,
}

pub(super) fn installed_relationships_release_v1(root: &Path) -> InstalledSignedBundle {
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
                RELATIONSHIPS_RELEASE_ARTIFACT_ID_V1,
                relationships_binary_v1(),
                relationships_module_descriptor_v1("managed-relationships-live").encode_to_vec(),
            )
            .with_settings_schema(relationships_settings_schema_bytes_v1()),
        ],
    )
    .expect("install signed Relationships release")
}

pub(super) fn admit_relationships_runtime_v1(
    store: &SqliteControlStore,
) -> AdmittedRelationshipsRuntimeV1 {
    let descriptor = relationships_module_descriptor_v1("managed-relationships-live");
    assert_eq!(descriptor.module_id, RELATIONSHIPS_MODULE_ID_V1);
    assert_eq!(descriptor.owner_id, RELATIONSHIPS_OWNER_ID_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Relationships descriptor");
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
    .expect("approve exact Relationships capabilities");
    let settings = relationships_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            RELATIONSHIPS_RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(
                std::fs::read(relationships_binary_v1()).expect("Relationships runtime binary"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record Relationships release binding");
    let bundle = relationships_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                RELATIONSHIPS_OWNER_ID_V1,
                u64::from(RELATIONSHIPS_STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Relationships Storage bundle"),
        )
        .expect("persist Relationships Storage bundle");
    AdmittedRelationshipsRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_relationships_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedRelationshipsRuntimeV1,
) -> AdmittedRelationshipsRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Relationships launch");
    let bundle = store
        .platform_storage_bundle(
            RELATIONSHIPS_OWNER_ID_V1,
            u64::from(RELATIONSHIPS_STORAGE_BUNDLE_REVISION_V1),
        )
        .expect("read Relationships bundle")
        .expect("Relationships bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        makosh_relationships_api::RELATIONSHIPS_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(RELATIONSHIPS_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("Relationships Storage binding issue"),
    )
    .expect("issue Relationships Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Relationships Storage binding");
    admitted
}

pub(super) fn start_relationships_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedRelationshipsRuntimeV1,
) -> StartedRelationshipsRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Relationships reservation");
    start_reserved_relationships_runtime_v1(supervisor, store, runtime_dir, reservation, admitted)
}

pub(super) fn restart_relationships_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedRelationshipsRuntimeV1,
) -> StartedRelationshipsRuntimeV1 {
    let previous_generation = predecessor.runtime_generation;
    let previous_instance = predecessor.runtime_instance_id.clone();
    let binding = relationships_storage_binding_v1(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&binding).expect("derive Relationships successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        makosh_relationships_api::RELATIONSHIPS_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Relationships successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Relationships successor binding");
    let successor = start_reserved_relationships_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        AdmittedRelationshipsRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
    );
    assert_eq!(successor.runtime_generation, previous_generation + 1);
    assert_ne!(successor.runtime_instance_id, previous_instance);
    successor
}

fn start_reserved_relationships_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedRelationshipsRuntimeV1,
) -> StartedRelationshipsRuntimeV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = relationships_storage_binding_v1(store, &admitted.registration_id);
    let topology =
        crate::platform::storage::topology::current(store).expect("Relationships Storage topology");
    let vault =
        vault_status::read_current(store, &supervisor.relay_port()).expect("live Vault status");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("Relationships Storage configuration");
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
            logical_owner_id: RELATIONSHIPS_OWNER_ID_V1.to_owned(),
            registration_id: admitted.registration_id.clone(),
            runtime_instance_id: runtime_instance_id.clone(),
            runtime_generation,
            grant_epoch,
            storage: Some(storage),
            event_hub_endpoint: events.nats_endpoint().to_owned(),
            event_credential_revision: events.credential_revision(),
            logical_human_owner_id: RELATIONSHIPS_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
        },
    )
    .expect("start managed Relationships domain");
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "Relationships readiness: {error}; last_failure={:?}",
                supervisor.last_failure(&admitted.registration_id)
            )
        });
    StartedRelationshipsRuntimeV1 {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids: admitted.capability_ids,
    }
}

fn relationships_storage_binding_v1(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(
            registration_id,
            makosh_relationships_api::RELATIONSHIPS_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read Relationships binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active Relationships binding")
}

fn relationships_binary_v1() -> PathBuf {
    binary("MAKOSH_RELATIONSHIPS_RUNTIME_BIN")
}
