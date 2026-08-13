//! Exact managed Workflow admission and lifecycle for bulk-action conformance.

use super::*;

use makosh_communication_bulk_action_api::{
    COMMUNICATION_BULK_ACTION_MODULE_ID_V1, COMMUNICATION_BULK_ACTION_OWNER_V1,
};
use makosh_communication_bulk_action_persistence::schema::{
    COMMUNICATION_BULK_ACTION_STORAGE_BUNDLE_REVISION_V3,
    communication_bulk_action_storage_bundle_v1,
};
use makosh_communication_bulk_action_runtime::admission::{
    COMMUNICATION_BULK_ACTION_STORAGE_CAPABILITY_ID_V1,
    communication_bulk_action_module_descriptor_v1,
    communication_bulk_action_settings_schema_bytes_v1,
};
use makosh_communication_delivery_intent_runtime::admission::{
    communication_delivery_intent_module_descriptor_v1,
    communication_delivery_intent_settings_schema_bytes_v1,
};
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_runtime_protocol::v1::ManagedWorkflowRuntimeConfigurationV1;

const BULK_ACTION_RELEASE_ARTIFACT_ID: &str = "workflow.communication_bulk_action";
const BULK_ACTION_RUNTIME_INSTANCE_ID: &str = "bulk-action-runtime-1";
pub(super) const BULK_ACTION_LOGICAL_OWNER_ID: &str = "owner-1";

pub(super) struct AdmittedBulkActionRuntime {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedBulkActionRuntime {
    pub(super) registration_id: String,
    pub(super) runtime_generation: u64,
    capability_ids: Vec<String>,
}

pub(super) fn installed_communications_bulk_action_release(root: &Path) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(
        SignedRuntimeArtifact::new(
            "workflow.communication_delivery_intent",
            binary("MAKOSH_COMMUNICATION_DELIVERY_INTENT_RUNTIME_BIN"),
            communication_delivery_intent_module_descriptor_v1("managed-delivery-intent-live")
                .encode_to_vec(),
        )
        .with_settings_schema(communication_delivery_intent_settings_schema_bytes_v1()),
    );
    artifacts.push(
        SignedRuntimeArtifact::new(
            BULK_ACTION_RELEASE_ARTIFACT_ID,
            bulk_action_binary(),
            communication_bulk_action_module_descriptor_v1("managed-bulk-action-live")
                .encode_to_vec(),
        )
        .with_settings_schema(communication_bulk_action_settings_schema_bytes_v1()),
    );
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed Communications and bulk-action release")
}

pub(super) fn admit_bulk_action_runtime(store: &SqliteControlStore) -> AdmittedBulkActionRuntime {
    let descriptor = communication_bulk_action_module_descriptor_v1("managed-bulk-action-live");
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact bulk-action descriptor");
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
    .expect("approve exact bulk-action capabilities");
    let schema = communication_bulk_action_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            BULK_ACTION_RELEASE_ARTIFACT_ID,
            Sha256::digest(
                std::fs::read(bulk_action_binary()).expect("bulk-action runtime binary bytes"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&schema).into()),
        ))
        .expect("record bulk-action release binding");
    let bundle = communication_bulk_action_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                COMMUNICATION_BULK_ACTION_OWNER_V1,
                u64::from(COMMUNICATION_BULK_ACTION_STORAGE_BUNDLE_REVISION_V3),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("record bulk-action Storage bundle"),
        )
        .expect("persist bulk-action Storage bundle");
    AdmittedBulkActionRuntime {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_bulk_action_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedBulkActionRuntime,
) -> AdmittedBulkActionRuntime {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve bulk-action managed launch");
    let bundle = store
        .platform_storage_bundle(
            COMMUNICATION_BULK_ACTION_OWNER_V1,
            u64::from(COMMUNICATION_BULK_ACTION_STORAGE_BUNDLE_REVISION_V3),
        )
        .expect("read bulk-action Storage bundle")
        .expect("bulk-action Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        COMMUNICATION_BULK_ACTION_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(COMMUNICATION_BULK_ACTION_STORAGE_BUNDLE_REVISION_V3),
            *bundle.digest(),
        )
        .expect("bulk-action Storage binding issue"),
    )
    .expect("issue bulk-action Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision bulk-action Storage binding");
    admitted
}

pub(super) fn start_bulk_action_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedBulkActionRuntime,
) -> StartedBulkActionRuntime {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load bulk-action managed launch reservation");
    let runtime_generation = reservation.runtime_generation();
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let binding = bulk_action_storage_binding(store, &admitted.registration_id);
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
    .expect("build bulk-action Storage configuration");
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
            logical_owner_id: BULK_ACTION_LOGICAL_OWNER_ID.to_owned(),
            registration_id: admitted.registration_id.clone(),
            runtime_instance_id,
            runtime_generation,
            grant_epoch: store
                .module_registration(&admitted.registration_id)
                .expect("read bulk-action registration")
                .expect("bulk-action registration")
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
    .expect("start managed bulk-action workflow");
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .expect("wait for bulk-action readiness");
    StartedBulkActionRuntime {
        registration_id: admitted.registration_id,
        runtime_generation,
        capability_ids: admitted.capability_ids,
    }
}

pub(super) fn restart_bulk_action_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedBulkActionRuntime,
) -> StartedBulkActionRuntime {
    let predecessor_generation = predecessor.runtime_generation;
    let predecessor_binding = bulk_action_storage_binding(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&predecessor_binding)
        .expect("derive bulk-action successor Storage fences");
    let (_, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        COMMUNICATION_BULK_ACTION_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve successor bulk-action launch and Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision successor bulk-action Storage binding");
    let successor = start_bulk_action_runtime(
        supervisor,
        store,
        runtime_dir,
        AdmittedBulkActionRuntime {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
    );
    assert_eq!(
        successor.runtime_generation,
        predecessor_generation + 1,
        "bulk-action restart must use the next managed runtime generation"
    );
    successor
}

fn bulk_action_storage_binding(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(
            registration_id,
            COMMUNICATION_BULK_ACTION_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read bulk-action Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active bulk-action Storage binding")
}

fn bulk_action_binary() -> PathBuf {
    binary("MAKOSH_COMMUNICATION_BULK_ACTION_RUNTIME_BIN")
}
