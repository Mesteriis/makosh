//! Exact managed Workflow admission for delayed communication delivery conformance.

use super::*;

use makosh_communication_delayed_delivery_api::{
    COMMUNICATION_DELAYED_DELIVERY_MODULE_ID_V1, COMMUNICATION_DELAYED_DELIVERY_OWNER_V1,
};
use makosh_communication_delayed_delivery_persistence::schema::{
    COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V4,
    communication_delayed_delivery_storage_bundle_v1,
};
use makosh_communication_delayed_delivery_runtime::{
    COMMUNICATION_DELAYED_DELIVERY_STORAGE_CAPABILITY_ID_V1,
    communication_delayed_delivery_module_descriptor_v1,
    communication_delayed_delivery_settings_schema_bytes_v1,
};
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_runtime_protocol::v1::ManagedWorkflowRuntimeConfigurationV1;

const DELAYED_DELIVERY_RELEASE_ARTIFACT_ID: &str = "communication_delayed_delivery.runtime.v1";
pub(super) const DELAYED_DELIVERY_LOGICAL_OWNER_ID: &str = "owner-1";

pub(super) struct AdmittedDelayedDeliveryRuntime {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedDelayedDeliveryRuntime {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    capability_ids: Vec<String>,
}

pub(super) fn installed_delayed_delivery_conformance_release(root: &Path) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(scheduler_release_artifact());
    artifacts.push(delivery_intent_release_artifact());
    artifacts.push(delayed_delivery_release_artifact());
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed delayed-delivery conformance release")
}

fn delayed_delivery_release_artifact() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        DELAYED_DELIVERY_RELEASE_ARTIFACT_ID,
        delayed_delivery_binary(),
        communication_delayed_delivery_module_descriptor_v1("managed-delayed-delivery-live")
            .encode_to_vec(),
    )
    .with_settings_schema(communication_delayed_delivery_settings_schema_bytes_v1())
}

pub(super) fn admit_delayed_delivery_runtime(
    store: &SqliteControlStore,
) -> AdmittedDelayedDeliveryRuntime {
    let descriptor =
        communication_delayed_delivery_module_descriptor_v1("managed-delayed-delivery-live");
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact delayed-delivery descriptor");
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
    .expect("approve exact delayed-delivery capabilities");
    let settings = communication_delayed_delivery_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            DELAYED_DELIVERY_RELEASE_ARTIFACT_ID,
            Sha256::digest(
                std::fs::read(delayed_delivery_binary())
                    .expect("delayed-delivery runtime binary bytes"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record delayed-delivery release binding");
    let bundle = communication_delayed_delivery_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                COMMUNICATION_DELAYED_DELIVERY_OWNER_V1,
                u64::from(COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V4),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("record delayed-delivery Storage bundle"),
        )
        .expect("persist delayed-delivery Storage bundle");
    AdmittedDelayedDeliveryRuntime {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_delayed_delivery_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedDelayedDeliveryRuntime,
) -> AdmittedDelayedDeliveryRuntime {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve delayed-delivery managed launch");
    let bundle = store
        .platform_storage_bundle(
            COMMUNICATION_DELAYED_DELIVERY_OWNER_V1,
            u64::from(COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V4),
        )
        .expect("read delayed-delivery Storage bundle")
        .expect("delayed-delivery Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        COMMUNICATION_DELAYED_DELIVERY_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(COMMUNICATION_DELAYED_DELIVERY_STORAGE_BUNDLE_REVISION_V4),
            *bundle.digest(),
        )
        .expect("delayed-delivery Storage binding issue"),
    )
    .expect("issue delayed-delivery Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision delayed-delivery Storage binding");
    admitted
}

pub(super) fn start_delayed_delivery_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedDelayedDeliveryRuntime,
) -> StartedDelayedDeliveryRuntime {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load delayed-delivery managed launch reservation");
    let runtime_generation = reservation.runtime_generation();
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let grant_epoch = store
        .module_registration(&admitted.registration_id)
        .expect("read delayed-delivery registration")
        .expect("delayed-delivery registration")
        .grant_epoch();
    let binding = delayed_delivery_storage_binding(store, &admitted.registration_id);
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
    .expect("build delayed-delivery Storage configuration");
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
            logical_owner_id: DELAYED_DELIVERY_LOGICAL_OWNER_ID.to_owned(),
            registration_id: admitted.registration_id.clone(),
            runtime_instance_id: runtime_instance_id.clone(),
            runtime_generation,
            grant_epoch,
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
    .expect("start managed delayed-delivery workflow");
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .expect("wait for delayed-delivery readiness");
    StartedDelayedDeliveryRuntime {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids: admitted.capability_ids,
    }
}

pub(super) fn restart_delayed_delivery_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedDelayedDeliveryRuntime,
) -> StartedDelayedDeliveryRuntime {
    let predecessor_generation = predecessor.runtime_generation;
    let predecessor_instance_id = predecessor.runtime_instance_id.clone();
    let predecessor_binding = delayed_delivery_storage_binding(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&predecessor_binding)
        .expect("derive delayed-delivery successor Storage fences");
    let (_, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        COMMUNICATION_DELAYED_DELIVERY_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve successor delayed-delivery launch and Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision successor delayed-delivery Storage binding");
    let successor = start_delayed_delivery_runtime(
        supervisor,
        store,
        runtime_dir,
        AdmittedDelayedDeliveryRuntime {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
    );
    assert_eq!(
        successor.runtime_generation,
        predecessor_generation + 1,
        "delayed-delivery restart must use the next managed runtime generation"
    );
    assert_ne!(
        successor.runtime_instance_id, predecessor_instance_id,
        "delayed-delivery restart must use a new managed runtime instance"
    );
    successor
}

fn delayed_delivery_storage_binding(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(
            registration_id,
            COMMUNICATION_DELAYED_DELIVERY_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read delayed-delivery Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active delayed-delivery Storage binding")
}

fn delayed_delivery_binary() -> PathBuf {
    binary("MAKOSH_COMMUNICATION_DELAYED_DELIVERY_RUNTIME_BIN")
}
