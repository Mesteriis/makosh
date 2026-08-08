//! Exact signed admission and owner-local lifecycle for Communication Summary.

use super::*;

use crate::platform::client_realtime::ClientRealtimePublishHandlerV1;
use makosh_communication_summary_api::{
    COMMUNICATION_SUMMARY_MODULE_ID_V1, COMMUNICATION_SUMMARY_OWNER_V1,
};
use makosh_communication_summary_persistence::{
    COMMUNICATION_SUMMARY_STORAGE_BUNDLE_REVISION_V1, communication_summary_storage_bundle_v1,
};
use makosh_communication_summary_runtime::{
    COMMUNICATION_SUMMARY_STORAGE_CAPABILITY_ID_V1, communication_summary_module_descriptor_v1,
    communication_summary_settings_schema_bytes_v1,
};
use makosh_gateway_runtime::InMemoryBrowserRealtimeSource;
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_runtime_protocol::v1::ManagedWorkflowRuntimeConfigurationV1;

const COMMUNICATION_SUMMARY_RELEASE_ARTIFACT_ID_V1: &str = "communication_summary.runtime.v1";
const COMMUNICATION_SUMMARY_BUILD_ID_V1: &str = "managed-communication-summary-live";
pub(super) const COMMUNICATION_SUMMARY_LOGICAL_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedCommunicationSummaryRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedCommunicationSummaryRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    capability_ids: Vec<String>,
}

pub(super) fn installed_communication_summary_ensemble_release_v1(
    root: &Path,
) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(ollama_ai_release_artifact_v1());
    artifacts.push(ai_inference_release_artifact_v1());
    artifacts.push(communication_summary_release_artifact_v1());
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed Communication Summary ensemble release")
}

fn communication_summary_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        COMMUNICATION_SUMMARY_RELEASE_ARTIFACT_ID_V1,
        communication_summary_binary(),
        communication_summary_module_descriptor_v1(COMMUNICATION_SUMMARY_BUILD_ID_V1)
            .encode_to_vec(),
    )
    .with_settings_schema(communication_summary_settings_schema_bytes_v1())
}

pub(super) fn admit_communication_summary_runtime_v1(
    store: &SqliteControlStore,
) -> AdmittedCommunicationSummaryRuntimeV1 {
    let descriptor = communication_summary_module_descriptor_v1(COMMUNICATION_SUMMARY_BUILD_ID_V1);
    assert_eq!(descriptor.module_id, COMMUNICATION_SUMMARY_MODULE_ID_V1);
    assert_eq!(descriptor.owner_id, COMMUNICATION_SUMMARY_OWNER_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Communication Summary descriptor");
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
    .expect("approve exact Communication Summary capabilities");

    let settings = communication_summary_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            COMMUNICATION_SUMMARY_RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(
                std::fs::read(communication_summary_binary())
                    .expect("Communication Summary runtime binary"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record Communication Summary release binding");

    let bundle = communication_summary_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                COMMUNICATION_SUMMARY_OWNER_V1,
                u64::from(COMMUNICATION_SUMMARY_STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Communication Summary Storage bundle"),
        )
        .expect("persist Communication Summary Storage bundle");

    AdmittedCommunicationSummaryRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn configure_communication_summary_realtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    realtime: InMemoryBrowserRealtimeSource,
) {
    supervisor
        .configure_client_realtime_handler(Arc::new(ClientRealtimePublishHandlerV1::new(
            Arc::clone(store),
            realtime,
        )))
        .expect("configure Communication Summary client realtime");
}

pub(super) fn prepare_communication_summary_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedCommunicationSummaryRuntimeV1,
) -> AdmittedCommunicationSummaryRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Communication Summary launch");
    let bundle = store
        .platform_storage_bundle(
            COMMUNICATION_SUMMARY_OWNER_V1,
            u64::from(COMMUNICATION_SUMMARY_STORAGE_BUNDLE_REVISION_V1),
        )
        .expect("read Communication Summary Storage bundle")
        .expect("Communication Summary Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        COMMUNICATION_SUMMARY_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(COMMUNICATION_SUMMARY_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("Communication Summary Storage binding issue"),
    )
    .expect("issue Communication Summary Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Communication Summary Storage binding");
    admitted
}

pub(super) fn start_communication_summary_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedCommunicationSummaryRuntimeV1,
) -> StartedCommunicationSummaryRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Communication Summary launch reservation");
    start_reserved_communication_summary_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        admitted,
    )
}

pub(super) fn restart_communication_summary_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedCommunicationSummaryRuntimeV1,
) -> StartedCommunicationSummaryRuntimeV1 {
    let previous_generation = predecessor.runtime_generation;
    let previous_instance = predecessor.runtime_instance_id.clone();
    let binding = communication_summary_storage_binding_v1(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&binding)
        .expect("derive Communication Summary Storage successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        COMMUNICATION_SUMMARY_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Communication Summary successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Communication Summary successor");
    let successor = start_reserved_communication_summary_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        AdmittedCommunicationSummaryRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
    );
    assert_eq!(successor.runtime_generation, previous_generation + 1);
    assert_ne!(successor.runtime_instance_id, previous_instance);
    successor
}

fn start_reserved_communication_summary_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedCommunicationSummaryRuntimeV1,
) -> StartedCommunicationSummaryRuntimeV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = communication_summary_storage_binding_v1(store, &admitted.registration_id);
    let topology =
        crate::platform::storage::topology::current(store).expect("Communication Summary topology");
    let vault =
        vault_status::read_current(store, &supervisor.relay_port()).expect("live Vault status");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("Communication Summary Storage configuration");
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
            logical_owner_id: COMMUNICATION_SUMMARY_LOGICAL_OWNER_ID_V1.to_owned(),
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
    .expect("start managed Communication Summary workflow");
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "Communication Summary readiness: {error}; last_failure={:?}",
                supervisor.last_failure(&admitted.registration_id)
            )
        });
    StartedCommunicationSummaryRuntimeV1 {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids: admitted.capability_ids,
    }
}

fn communication_summary_storage_binding_v1(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(
            registration_id,
            COMMUNICATION_SUMMARY_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read Communication Summary Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active Communication Summary Storage binding")
}

fn communication_summary_binary() -> PathBuf {
    binary("MAKOSH_COMMUNICATION_SUMMARY_RUNTIME_BIN")
}
