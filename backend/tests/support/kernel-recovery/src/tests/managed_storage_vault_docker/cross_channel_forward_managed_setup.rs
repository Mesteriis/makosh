//! Exact managed Workflow admission and lifecycle for cross-channel-forward conformance.

use super::*;

use makosh_communication_cross_channel_forward_api::{
    COMMUNICATION_CROSS_CHANNEL_FORWARD_MODULE_ID_V1, COMMUNICATION_CROSS_CHANNEL_FORWARD_OWNER_V1,
};
use makosh_communication_cross_channel_forward_persistence::schema::{
    COMMUNICATION_CROSS_CHANNEL_FORWARD_STORAGE_BUNDLE_REVISION_V3,
    communication_cross_channel_forward_storage_bundle_v1,
};
use makosh_communication_cross_channel_forward_runtime::{
    COMMUNICATION_CROSS_CHANNEL_FORWARD_STORAGE_CAPABILITY_ID_V1,
    communication_cross_channel_forward_module_descriptor_v1,
    communication_cross_channel_forward_settings_schema_bytes_v1,
};
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_runtime_protocol::v1::ManagedWorkflowRuntimeConfigurationV1;

const CROSS_CHANNEL_FORWARD_RELEASE_ARTIFACT_ID: &str =
    "workflow.communication_cross_channel_forward";
const CROSS_CHANNEL_FORWARD_RUNTIME_INSTANCE_ID: &str = "cross-channel-forward-runtime-1";
pub(super) const CROSS_CHANNEL_FORWARD_LOGICAL_OWNER_ID: &str = "owner-1";

pub(super) struct AdmittedCrossChannelForwardRuntime {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedCrossChannelForwardRuntime {
    pub(super) registration_id: String,
    pub(super) runtime_generation: u64,
    capability_ids: Vec<String>,
}

pub(super) fn installed_cross_channel_forward_release(root: &Path) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(super::delivery_intent_managed_setup::delivery_intent_release_artifact());
    artifacts.push(cross_channel_forward_release_artifact());
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed Communications, delivery-intent and cross-channel-forward release")
}

pub(super) fn cross_channel_forward_release_artifact() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        CROSS_CHANNEL_FORWARD_RELEASE_ARTIFACT_ID,
        cross_channel_forward_binary(),
        communication_cross_channel_forward_module_descriptor_v1(
            "managed-cross-channel-forward-live",
        )
        .encode_to_vec(),
    )
    .with_settings_schema(communication_cross_channel_forward_settings_schema_bytes_v1())
}

pub(super) fn admit_cross_channel_forward_runtime(
    store: &SqliteControlStore,
) -> AdmittedCrossChannelForwardRuntime {
    let descriptor = communication_cross_channel_forward_module_descriptor_v1(
        "managed-cross-channel-forward-live",
    );
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact cross-channel-forward descriptor");
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
    .expect("approve exact cross-channel-forward capabilities");
    let schema = communication_cross_channel_forward_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            CROSS_CHANNEL_FORWARD_RELEASE_ARTIFACT_ID,
            Sha256::digest(
                std::fs::read(cross_channel_forward_binary())
                    .expect("cross-channel-forward runtime binary bytes"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&schema).into()),
        ))
        .expect("record cross-channel-forward release binding");
    let bundle = communication_cross_channel_forward_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                COMMUNICATION_CROSS_CHANNEL_FORWARD_OWNER_V1,
                u64::from(COMMUNICATION_CROSS_CHANNEL_FORWARD_STORAGE_BUNDLE_REVISION_V3),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("record cross-channel-forward Storage bundle"),
        )
        .expect("persist cross-channel-forward Storage bundle");
    AdmittedCrossChannelForwardRuntime {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_cross_channel_forward_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedCrossChannelForwardRuntime,
) -> AdmittedCrossChannelForwardRuntime {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve cross-channel-forward managed launch");
    let bundle = store
        .platform_storage_bundle(
            COMMUNICATION_CROSS_CHANNEL_FORWARD_OWNER_V1,
            u64::from(COMMUNICATION_CROSS_CHANNEL_FORWARD_STORAGE_BUNDLE_REVISION_V3),
        )
        .expect("read cross-channel-forward Storage bundle")
        .expect("cross-channel-forward Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        COMMUNICATION_CROSS_CHANNEL_FORWARD_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(COMMUNICATION_CROSS_CHANNEL_FORWARD_STORAGE_BUNDLE_REVISION_V3),
            *bundle.digest(),
        )
        .expect("cross-channel-forward Storage binding issue"),
    )
    .expect("issue cross-channel-forward Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision cross-channel-forward Storage binding");
    admitted
}

pub(super) fn start_cross_channel_forward_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedCrossChannelForwardRuntime,
) -> StartedCrossChannelForwardRuntime {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load cross-channel-forward managed launch reservation");
    let runtime_generation = reservation.runtime_generation();
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let binding = cross_channel_forward_storage_binding(store, &admitted.registration_id);
    let topology =
        crate::platform::storage::topology::current(store).expect("read Storage topology");
    let vault = vault_status::read_current(store, &supervisor.relay_port())
        .expect("read live Vault status");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("build cross-channel-forward Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    managed_launch::start_reserved_workflow(
        supervisor,
        runtime_dir,
        reservation,
        ManagedWorkflowRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: CROSS_CHANNEL_FORWARD_LOGICAL_OWNER_ID.to_owned(),
            registration_id: admitted.registration_id.clone(),
            runtime_instance_id,
            runtime_generation,
            grant_epoch: store
                .module_registration(&admitted.registration_id)
                .expect("read cross-channel-forward registration")
                .expect("cross-channel-forward registration")
                .grant_epoch(),
            storage: Some(storage),
            event_hub_endpoint: events.nats_endpoint().to_owned(),
            event_credential_revision: events.credential_revision(),
            runtime_artifacts: Vec::new(),
            configuration_instance_id: String::new(),
            settings_revision: 0,
            configuration_instances: Vec::new(),
        },
        &[],
    )
    .expect("start managed cross-channel-forward workflow");
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .expect("wait for cross-channel-forward readiness");
    StartedCrossChannelForwardRuntime {
        registration_id: admitted.registration_id,
        runtime_generation,
        capability_ids: admitted.capability_ids,
    }
}

pub(super) fn restart_cross_channel_forward_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedCrossChannelForwardRuntime,
) -> StartedCrossChannelForwardRuntime {
    let predecessor_generation = predecessor.runtime_generation;
    let predecessor_binding =
        cross_channel_forward_storage_binding(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&predecessor_binding)
        .expect("derive cross-channel-forward successor Storage fences");
    let (_, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        COMMUNICATION_CROSS_CHANNEL_FORWARD_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve successor cross-channel-forward launch and Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision successor cross-channel-forward Storage binding");
    let successor = start_cross_channel_forward_runtime(
        supervisor,
        store,
        runtime_dir,
        AdmittedCrossChannelForwardRuntime {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
    );
    assert_eq!(
        successor.runtime_generation,
        predecessor_generation + 1,
        "cross-channel-forward restart must use the next managed runtime generation"
    );
    successor
}

fn cross_channel_forward_storage_binding(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(
            registration_id,
            COMMUNICATION_CROSS_CHANNEL_FORWARD_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read cross-channel-forward Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active cross-channel-forward Storage binding")
}

pub(super) fn cross_channel_forward_database_id(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    registration_id: &str,
) -> String {
    let binding = cross_channel_forward_storage_binding(store, registration_id);
    let topology =
        crate::platform::storage::topology::current(store).expect("read Storage topology");
    let vault = vault_status::read_current(store, &supervisor.relay_port())
        .expect("read live Vault status");
    crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("build cross-channel-forward Storage configuration")
    .database_id
}

pub(super) fn cross_channel_forward_binary() -> PathBuf {
    binary("MAKOSH_COMMUNICATION_CROSS_CHANNEL_FORWARD_RUNTIME_BIN")
}
