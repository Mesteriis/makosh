//! Exact managed Workflow admission and lifecycle for delivery-intent conformance.

use super::*;

use crate::modules::capability::module_request::ModuleRequestRouteHandlerV1;
use makosh_communication_delivery_intent_api::{
    COMMUNICATION_DELIVERY_INTENT_MODULE_ID_V1, COMMUNICATION_DELIVERY_INTENT_OWNER_V1,
};
use makosh_communication_delivery_intent_persistence::schema::{
    COMMUNICATION_DELIVERY_INTENT_STORAGE_BUNDLE_REVISION_V5,
    communication_delivery_intent_storage_bundle_v1,
};
use makosh_communication_delivery_intent_runtime::admission::{
    COMMUNICATION_DELIVERY_INTENT_STORAGE_CAPABILITY_ID_V1,
    communication_delivery_intent_module_descriptor_v1,
    communication_delivery_intent_settings_schema_bytes_v1,
};
use makosh_gateway_runtime::InMemoryBrowserRealtimeSource;
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_runtime_protocol::v1::ManagedWorkflowRuntimeConfigurationV1;

use crate::modules::capability::module_query::ModuleQueryRouteHandlerV1;
use crate::platform::client_realtime::ClientRealtimePublishHandlerV1;
use crate::runtime::lifecycle::control::ManagedRuntimeModuleRequestHandler;

const DELIVERY_INTENT_RELEASE_ARTIFACT_ID: &str = "workflow.communication_delivery_intent";
const DELIVERY_INTENT_RUNTIME_INSTANCE_ID: &str = "delivery-intent-runtime-1";
pub(super) const DELIVERY_INTENT_LOGICAL_OWNER_ID: &str = "owner-1";

pub(super) struct AdmittedDeliveryIntentRuntime {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedDeliveryIntentRuntime {
    pub(super) registration_id: String,
    pub(super) runtime_generation: u64,
    capability_ids: Vec<String>,
}

pub(super) fn installed_communications_delivery_intent_release(
    root: &Path,
) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(delivery_intent_release_artifact());
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed Communications and delivery-intent release")
}

pub(super) fn delivery_intent_release_artifact() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        DELIVERY_INTENT_RELEASE_ARTIFACT_ID,
        delivery_intent_binary(),
        communication_delivery_intent_module_descriptor_v1("managed-delivery-intent-live")
            .encode_to_vec(),
    )
    .with_settings_schema(communication_delivery_intent_settings_schema_bytes_v1())
}

pub(super) fn admit_delivery_intent_runtime(
    store: &SqliteControlStore,
) -> AdmittedDeliveryIntentRuntime {
    let descriptor =
        communication_delivery_intent_module_descriptor_v1("managed-delivery-intent-live");
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact delivery-intent descriptor");
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
    .expect("approve exact delivery-intent capabilities");
    let schema = communication_delivery_intent_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            DELIVERY_INTENT_RELEASE_ARTIFACT_ID,
            Sha256::digest(
                std::fs::read(delivery_intent_binary())
                    .expect("delivery-intent runtime binary bytes"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&schema).into()),
        ))
        .expect("record delivery-intent release binding");
    let bundle = communication_delivery_intent_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                COMMUNICATION_DELIVERY_INTENT_OWNER_V1,
                u64::from(COMMUNICATION_DELIVERY_INTENT_STORAGE_BUNDLE_REVISION_V5),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("record delivery-intent Storage bundle"),
        )
        .expect("persist delivery-intent Storage bundle");
    AdmittedDeliveryIntentRuntime {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_delivery_intent_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedDeliveryIntentRuntime,
) -> AdmittedDeliveryIntentRuntime {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve delivery-intent managed launch");
    let bundle = store
        .platform_storage_bundle(
            COMMUNICATION_DELIVERY_INTENT_OWNER_V1,
            u64::from(COMMUNICATION_DELIVERY_INTENT_STORAGE_BUNDLE_REVISION_V5),
        )
        .expect("read delivery-intent Storage bundle")
        .expect("delivery-intent Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        COMMUNICATION_DELIVERY_INTENT_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(COMMUNICATION_DELIVERY_INTENT_STORAGE_BUNDLE_REVISION_V5),
            *bundle.digest(),
        )
        .expect("delivery-intent Storage binding issue"),
    )
    .expect("issue delivery-intent Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision delivery-intent Storage binding");
    admitted
}

pub(super) fn configure_delivery_intent_runtime_routes(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    client_realtime: InMemoryBrowserRealtimeSource,
) {
    configure_delivery_intent_runtime_routes_with_request_handler(
        supervisor,
        store,
        client_realtime,
        delivery_intent_request_route_handler(supervisor, store),
    );
}

pub(super) fn delivery_intent_request_route_handler(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
) -> Arc<dyn ManagedRuntimeModuleRequestHandler> {
    Arc::new(ModuleRequestRouteHandlerV1::new(
        Arc::clone(store),
        supervisor.relay_port(),
    ))
}

pub(super) fn configure_delivery_intent_runtime_routes_with_request_handler(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    client_realtime: InMemoryBrowserRealtimeSource,
    request_handler: Arc<dyn ManagedRuntimeModuleRequestHandler>,
) {
    supervisor
        .configure_module_request_handler(request_handler)
        .expect("configure managed module request handler");
    supervisor
        .configure_module_query_handler(Arc::new(ModuleQueryRouteHandlerV1::new(
            Arc::clone(store),
            supervisor.relay_port(),
        )))
        .expect("configure managed module query handler");
    supervisor
        .configure_client_realtime_handler(Arc::new(ClientRealtimePublishHandlerV1::new(
            Arc::clone(store),
            client_realtime,
        )))
        .expect("configure managed client realtime handler");
}

pub(super) fn start_delivery_intent_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedDeliveryIntentRuntime,
) -> StartedDeliveryIntentRuntime {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load delivery-intent managed launch reservation");
    let runtime_generation = reservation.runtime_generation();
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let binding = delivery_intent_storage_binding(store, &admitted.registration_id);
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
    .expect("build delivery-intent Storage configuration");
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
            logical_owner_id: DELIVERY_INTENT_LOGICAL_OWNER_ID.to_owned(),
            registration_id: admitted.registration_id.clone(),
            runtime_instance_id,
            runtime_generation,
            grant_epoch: store
                .module_registration(&admitted.registration_id)
                .expect("read delivery-intent registration")
                .expect("delivery-intent registration")
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
    .expect("start managed delivery-intent workflow");
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .expect("wait for delivery-intent readiness");
    StartedDeliveryIntentRuntime {
        registration_id: admitted.registration_id,
        runtime_generation,
        capability_ids: admitted.capability_ids,
    }
}

pub(super) fn restart_delivery_intent_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedDeliveryIntentRuntime,
) -> StartedDeliveryIntentRuntime {
    let predecessor_generation = predecessor.runtime_generation;
    let predecessor_binding = delivery_intent_storage_binding(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&predecessor_binding)
        .expect("derive delivery-intent successor Storage fences");
    let (_, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        COMMUNICATION_DELIVERY_INTENT_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve successor delivery-intent launch and Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision successor delivery-intent Storage binding");
    let successor = start_delivery_intent_runtime(
        supervisor,
        store,
        runtime_dir,
        AdmittedDeliveryIntentRuntime {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
    );
    assert_eq!(
        successor.runtime_generation,
        predecessor_generation + 1,
        "delivery-intent restart must use the next managed runtime generation"
    );
    successor
}

fn delivery_intent_storage_binding(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(
            registration_id,
            COMMUNICATION_DELIVERY_INTENT_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read delivery-intent Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active delivery-intent Storage binding")
}

pub(super) fn delivery_intent_binary() -> PathBuf {
    binary("MAKOSH_COMMUNICATION_DELIVERY_INTENT_RUNTIME_BIN")
}
