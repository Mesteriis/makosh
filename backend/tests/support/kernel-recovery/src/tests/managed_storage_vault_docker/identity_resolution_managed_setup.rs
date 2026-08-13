//! Exact signed Identity Resolution engine admission and owner-local Storage lifecycle.

use super::*;

use makosh_identity_resolution_api::{
    IDENTITY_RESOLUTION_MODULE_ID_V1, IDENTITY_RESOLUTION_OWNER_ID_V1,
    IDENTITY_RESOLUTION_STORAGE_CAPABILITY_ID_V1,
};
use makosh_identity_resolution_persistence::identity_resolution_storage_bundle_v1;
use makosh_identity_resolution_runtime::{
    identity_resolution_module_descriptor_v1, identity_resolution_settings_schema_bytes_v1,
};
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_runtime_protocol::v1::{ManagedEngineRuntimeConfigurationV1, SettingsSnapshotV1};

const IDENTITY_RESOLUTION_ARTIFACT_ID_V1: &str = "identity_resolution.runtime.v1";
const IDENTITY_RESOLUTION_BUILD_ID_V1: &str = "managed-identity-resolution-live";
pub(super) const IDENTITY_RESOLUTION_HUMAN_OWNER_V1: &str = "owner-1";

pub(super) struct AdmittedIdentityResolutionV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedIdentityResolutionV1 {
    pub(super) registration_id: String,
}

pub(super) fn identity_resolution_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        IDENTITY_RESOLUTION_ARTIFACT_ID_V1,
        identity_resolution_binary(),
        identity_resolution_module_descriptor_v1(IDENTITY_RESOLUTION_BUILD_ID_V1).encode_to_vec(),
    )
    .with_settings_schema(identity_resolution_settings_schema_bytes_v1())
}

pub(super) fn admit_identity_resolution_v1(
    store: &SqliteControlStore,
) -> AdmittedIdentityResolutionV1 {
    let descriptor = identity_resolution_module_descriptor_v1(IDENTITY_RESOLUTION_BUILD_ID_V1);
    assert_eq!(descriptor.module_id, IDENTITY_RESOLUTION_MODULE_ID_V1);
    assert_eq!(descriptor.owner_id, IDENTITY_RESOLUTION_OWNER_ID_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register Identity Resolution descriptor");
    let capabilities = descriptor
        .capabilities
        .iter()
        .map(|capability| capability.capability_id.clone())
        .collect::<Vec<_>>();
    crate::modules::registration::registry::approve_after_owner_authorization(
        store,
        registration.registration_id(),
        &capabilities,
    )
    .expect("approve Identity Resolution capabilities");
    let settings = identity_resolution_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            IDENTITY_RESOLUTION_ARTIFACT_ID_V1,
            Sha256::digest(
                std::fs::read(identity_resolution_binary()).expect("Identity Resolution binary"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record Identity Resolution release binding");
    let bundle = identity_resolution_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                IDENTITY_RESOLUTION_OWNER_ID_V1,
                1,
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Identity Resolution storage bundle"),
        )
        .expect("persist Identity Resolution bundle");
    AdmittedIdentityResolutionV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids: capabilities,
    }
}

pub(super) fn prepare_identity_resolution_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedIdentityResolutionV1,
) -> AdmittedIdentityResolutionV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Identity Resolution launch");
    let bundle = store
        .platform_storage_bundle(IDENTITY_RESOLUTION_OWNER_ID_V1, 1)
        .expect("read Identity Resolution bundle")
        .expect("Identity Resolution bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        IDENTITY_RESOLUTION_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(1, 1, 1, *bundle.digest())
            .expect("Identity Resolution storage issue"),
    )
    .expect("issue Identity Resolution storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Identity Resolution storage");
    admitted
}

pub(super) fn start_identity_resolution_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedIdentityResolutionV1,
) -> StartedIdentityResolutionV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Identity Resolution reservation");
    let binding = store
        .platform_storage_binding(
            &admitted.registration_id,
            IDENTITY_RESOLUTION_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read Identity Resolution binding")
        .filter(|value| value.state() == PlatformStorageBindingStateV1::Active)
        .expect("active Identity Resolution binding");
    let topology = crate::platform::storage::topology::current(store).expect("Storage topology");
    let vault = vault_status::read_current(store, &supervisor.relay_port()).expect("Vault status");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("Identity Resolution storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("Event topology")
        .expect("Event topology");
    let registration_id = admitted.registration_id;
    let capability_ids = admitted.capability_ids;
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    managed_launch::start_reserved_engine(
        supervisor,
        runtime_dir,
        reservation,
        ManagedEngineRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: IDENTITY_RESOLUTION_OWNER_ID_V1.to_owned(),
            logical_human_owner_id: IDENTITY_RESOLUTION_HUMAN_OWNER_V1.to_owned(),
            registration_id: registration_id.clone(),
            runtime_instance_id,
            runtime_generation,
            grant_epoch,
            storage: Some(storage),
            event_hub_endpoint: events.nats_endpoint().to_owned(),
            event_credential_revision: events.credential_revision(),
            settings_revision: 1,
            runtime_artifacts: Vec::new(),
        },
        SettingsSnapshotV1 {
            target_id: registration_id.clone(),
            revision: 1,
            values: Vec::new(),
        }
        .encode_to_vec(),
        &capability_ids,
    )
    .expect("start Identity Resolution engine");
    supervisor
        .wait_until_ready(&registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "Identity Resolution readiness: {error}; last={:?}",
                supervisor.last_failure(&registration_id)
            )
        });
    StartedIdentityResolutionV1 { registration_id }
}

fn identity_resolution_binary() -> PathBuf {
    binary("MAKOSH_IDENTITY_RESOLUTION_RUNTIME_BIN")
}
