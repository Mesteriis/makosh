//! Exact signed admission and owner-local Storage lifecycle for the Speech-to-Text engine.

use super::*;

use crate::modules::capability::module_request::ModuleRequestRouteHandlerV1;
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_runtime_protocol::v1::{ManagedEngineRuntimeConfigurationV1, SettingsSnapshotV1};
use makosh_speech_to_text_api::{SPEECH_TO_TEXT_MODULE_ID_V1, SPEECH_TO_TEXT_OWNER_V1};
use makosh_speech_to_text_persistence::schema::{
    SPEECH_TO_TEXT_STORAGE_BUNDLE_REVISION_V1, speech_to_text_storage_bundle_v1,
};
use makosh_speech_to_text_runtime::{
    SPEECH_TO_TEXT_STORAGE_CAPABILITY_ID_V1, speech_to_text_module_descriptor_v1,
    speech_to_text_settings_schema_bytes_v1,
};

const SPEECH_TO_TEXT_RELEASE_ARTIFACT_ID_V1: &str = "speech_to_text.runtime.v1";
const SPEECH_TO_TEXT_BUILD_ID_V1: &str = "managed-speech-to-text-live";
pub(super) const SPEECH_TO_TEXT_LOGICAL_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedSpeechToTextRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedSpeechToTextRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    capability_ids: Vec<String>,
}

pub(super) fn speech_to_text_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        SPEECH_TO_TEXT_RELEASE_ARTIFACT_ID_V1,
        speech_to_text_binary(),
        speech_to_text_module_descriptor_v1(SPEECH_TO_TEXT_BUILD_ID_V1).encode_to_vec(),
    )
    .with_settings_schema(speech_to_text_settings_schema_bytes_v1())
}

pub(super) fn configure_speech_to_text_module_request_router_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
) {
    supervisor
        .configure_module_request_handler(Arc::new(ModuleRequestRouteHandlerV1::new(
            Arc::clone(store),
            supervisor.relay_port(),
        )))
        .expect("configure Speech-to-Text module request router");
}

pub(super) fn admit_speech_to_text_runtime_v1(
    store: &SqliteControlStore,
) -> AdmittedSpeechToTextRuntimeV1 {
    let descriptor = speech_to_text_module_descriptor_v1(SPEECH_TO_TEXT_BUILD_ID_V1);
    assert_eq!(descriptor.module_id, SPEECH_TO_TEXT_MODULE_ID_V1);
    assert_eq!(descriptor.owner_id, SPEECH_TO_TEXT_OWNER_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Speech-to-Text descriptor");
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
    .expect("approve exact Speech-to-Text capabilities");
    let settings = speech_to_text_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            SPEECH_TO_TEXT_RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(
                std::fs::read(speech_to_text_binary()).expect("Speech-to-Text runtime binary"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record Speech-to-Text release binding");
    let bundle = speech_to_text_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                SPEECH_TO_TEXT_OWNER_V1,
                u64::from(SPEECH_TO_TEXT_STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Speech-to-Text Storage bundle"),
        )
        .expect("persist Speech-to-Text Storage bundle");
    AdmittedSpeechToTextRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_speech_to_text_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedSpeechToTextRuntimeV1,
) -> AdmittedSpeechToTextRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Speech-to-Text launch");
    let bundle = store
        .platform_storage_bundle(
            SPEECH_TO_TEXT_OWNER_V1,
            u64::from(SPEECH_TO_TEXT_STORAGE_BUNDLE_REVISION_V1),
        )
        .expect("read Speech-to-Text Storage bundle")
        .expect("Speech-to-Text Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        SPEECH_TO_TEXT_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(SPEECH_TO_TEXT_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("Speech-to-Text Storage binding issue"),
    )
    .expect("issue Speech-to-Text Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Speech-to-Text Storage binding");
    admitted
}

pub(super) fn start_speech_to_text_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedSpeechToTextRuntimeV1,
) -> StartedSpeechToTextRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Speech-to-Text launch reservation");
    start_reserved_speech_to_text_runtime_v1(supervisor, store, runtime_dir, reservation, admitted)
}

pub(super) fn restart_speech_to_text_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedSpeechToTextRuntimeV1,
) -> StartedSpeechToTextRuntimeV1 {
    let previous_generation = predecessor.runtime_generation;
    let previous_instance = predecessor.runtime_instance_id.clone();
    let binding = speech_to_text_storage_binding_v1(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&binding).expect("derive Speech-to-Text successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        SPEECH_TO_TEXT_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Speech-to-Text successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Speech-to-Text successor");
    let successor = start_reserved_speech_to_text_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        AdmittedSpeechToTextRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
    );
    assert_eq!(successor.runtime_generation, previous_generation + 1);
    assert_ne!(successor.runtime_instance_id, previous_instance);
    successor
}

fn start_reserved_speech_to_text_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedSpeechToTextRuntimeV1,
) -> StartedSpeechToTextRuntimeV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = speech_to_text_storage_binding_v1(store, &admitted.registration_id);
    let topology = crate::platform::storage::topology::current(store)
        .expect("Speech-to-Text Storage topology");
    let vault =
        vault_status::read_current(store, &supervisor.relay_port()).expect("live Vault status");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("Speech-to-Text Storage configuration");
    let configuration = ManagedEngineRuntimeConfigurationV1 {
        major: 1,
        logical_owner_id: SPEECH_TO_TEXT_OWNER_V1.to_owned(),
        registration_id: admitted.registration_id.clone(),
        runtime_instance_id: runtime_instance_id.clone(),
        runtime_generation,
        grant_epoch,
        storage: Some(storage),
        event_hub_endpoint: String::new(),
        event_credential_revision: 0,
        settings_revision: 1,
        logical_human_owner_id: SPEECH_TO_TEXT_LOGICAL_OWNER_ID_V1.to_owned(),
        runtime_artifacts: Vec::new(),
    };
    managed_launch::start_reserved_engine(
        supervisor,
        runtime_dir,
        reservation,
        configuration,
        SettingsSnapshotV1 {
            target_id: admitted.registration_id.clone(),
            revision: 1,
            values: Vec::new(),
        }
        .encode_to_vec(),
        &[],
    )
    .expect("start managed Speech-to-Text engine");
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "Speech-to-Text readiness: {error}; last_failure={:?}",
                supervisor.last_failure(&admitted.registration_id)
            )
        });
    StartedSpeechToTextRuntimeV1 {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        capability_ids: admitted.capability_ids,
    }
}

fn speech_to_text_storage_binding_v1(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(registration_id, SPEECH_TO_TEXT_STORAGE_CAPABILITY_ID_V1)
        .expect("read Speech-to-Text Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active Speech-to-Text Storage binding")
}

fn speech_to_text_binary() -> PathBuf {
    binary("MAKOSH_SPEECH_TO_TEXT_RUNTIME_BIN")
}
