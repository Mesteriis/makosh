//! Exact signed admission and owner-local lifecycle for Communication Translation.

use super::*;

use crate::platform::client_realtime::ClientRealtimePublishHandlerV1;
use makosh_communication_translation_api::{
    COMMUNICATION_TRANSLATION_MODULE_ID_V1, COMMUNICATION_TRANSLATION_OWNER_V1,
};
use makosh_communication_translation_persistence::{
    COMMUNICATION_TRANSLATION_STORAGE_BUNDLE_REVISION_V1,
    communication_translation_storage_bundle_v1,
};
use makosh_communication_translation_runtime::{
    COMMUNICATION_TRANSLATION_STORAGE_CAPABILITY_ID_V1,
    communication_translation_module_descriptor_v1,
    communication_translation_settings_schema_bytes_v1,
};
use makosh_gateway_runtime::InMemoryBrowserRealtimeSource;
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_runtime_protocol::v1::ManagedWorkflowRuntimeConfigurationV1;

const COMMUNICATION_TRANSLATION_RELEASE_ARTIFACT_ID_V1: &str =
    "communication_translation.runtime.v1";
const COMMUNICATION_TRANSLATION_BUILD_ID_V1: &str = "managed-communication-translation-live";
pub(super) const COMMUNICATION_TRANSLATION_LOGICAL_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedCommunicationTranslationRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedCommunicationTranslationRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    capability_ids: Vec<String>,
}

pub(super) fn installed_communication_translation_ensemble_release_v1(
    root: &Path,
) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(ollama_ai_release_artifact_v1());
    artifacts.push(ai_inference_release_artifact_v1());
    artifacts.push(communication_translation_release_artifact_v1());
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed Communication Translation ensemble release")
}

fn communication_translation_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        COMMUNICATION_TRANSLATION_RELEASE_ARTIFACT_ID_V1,
        communication_translation_binary(),
        communication_translation_module_descriptor_v1(COMMUNICATION_TRANSLATION_BUILD_ID_V1)
            .encode_to_vec(),
    )
    .with_settings_schema(communication_translation_settings_schema_bytes_v1())
}

pub(super) fn admit_communication_translation_runtime_v1(
    store: &SqliteControlStore,
) -> AdmittedCommunicationTranslationRuntimeV1 {
    let descriptor =
        communication_translation_module_descriptor_v1(COMMUNICATION_TRANSLATION_BUILD_ID_V1);
    assert_eq!(descriptor.module_id, COMMUNICATION_TRANSLATION_MODULE_ID_V1);
    assert_eq!(descriptor.owner_id, COMMUNICATION_TRANSLATION_OWNER_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Communication Translation descriptor");
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
    .expect("approve exact Communication Translation capabilities");

    let settings = communication_translation_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            COMMUNICATION_TRANSLATION_RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(
                std::fs::read(communication_translation_binary())
                    .expect("Communication Translation runtime binary"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record Communication Translation release binding");

    let bundle = communication_translation_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                COMMUNICATION_TRANSLATION_OWNER_V1,
                u64::from(COMMUNICATION_TRANSLATION_STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Communication Translation Storage bundle"),
        )
        .expect("persist Communication Translation Storage bundle");

    AdmittedCommunicationTranslationRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn configure_communication_translation_realtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    realtime: InMemoryBrowserRealtimeSource,
) {
    supervisor
        .configure_client_realtime_handler(Arc::new(ClientRealtimePublishHandlerV1::new(
            Arc::clone(store),
            realtime,
        )))
        .expect("configure Communication Translation client realtime");
}

pub(super) fn prepare_communication_translation_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedCommunicationTranslationRuntimeV1,
) -> AdmittedCommunicationTranslationRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Communication Translation launch");
    let bundle = store
        .platform_storage_bundle(
            COMMUNICATION_TRANSLATION_OWNER_V1,
            u64::from(COMMUNICATION_TRANSLATION_STORAGE_BUNDLE_REVISION_V1),
        )
        .expect("read Communication Translation Storage bundle")
        .expect("Communication Translation Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        COMMUNICATION_TRANSLATION_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(COMMUNICATION_TRANSLATION_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("Communication Translation Storage binding issue"),
    )
    .expect("issue Communication Translation Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Communication Translation Storage binding");
    admitted
}

pub(super) fn start_communication_translation_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedCommunicationTranslationRuntimeV1,
) -> StartedCommunicationTranslationRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Communication Translation launch reservation");
    start_reserved_communication_translation_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        admitted,
    )
}

pub(super) fn restart_communication_translation_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedCommunicationTranslationRuntimeV1,
) -> StartedCommunicationTranslationRuntimeV1 {
    let previous_generation = predecessor.runtime_generation;
    let previous_instance = predecessor.runtime_instance_id.clone();
    let binding = communication_translation_storage_binding_v1(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&binding)
        .expect("derive Communication Translation Storage successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        COMMUNICATION_TRANSLATION_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Communication Translation successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Communication Translation successor");
    let successor = start_reserved_communication_translation_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        AdmittedCommunicationTranslationRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
    );
    assert_eq!(successor.runtime_generation, previous_generation + 1);
    assert_ne!(successor.runtime_instance_id, previous_instance);
    successor
}

fn start_reserved_communication_translation_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedCommunicationTranslationRuntimeV1,
) -> StartedCommunicationTranslationRuntimeV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = communication_translation_storage_binding_v1(store, &admitted.registration_id);
    let topology = crate::platform::storage::topology::current(store)
        .expect("Communication Translation topology");
    let vault =
        vault_status::read_current(store, &supervisor.relay_port()).expect("live Vault status");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("Communication Translation Storage configuration");
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
            logical_owner_id: COMMUNICATION_TRANSLATION_LOGICAL_OWNER_ID_V1.to_owned(),
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
    .expect("start managed Communication Translation workflow");
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "Communication Translation readiness: {error}; last_failure={:?}",
                supervisor.last_failure(&admitted.registration_id)
            )
        });
    StartedCommunicationTranslationRuntimeV1 {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids: admitted.capability_ids,
    }
}

fn communication_translation_storage_binding_v1(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(
            registration_id,
            COMMUNICATION_TRANSLATION_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read Communication Translation Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active Communication Translation Storage binding")
}

fn communication_translation_binary() -> PathBuf {
    binary("MAKOSH_COMMUNICATION_TRANSLATION_RUNTIME_BIN")
}
