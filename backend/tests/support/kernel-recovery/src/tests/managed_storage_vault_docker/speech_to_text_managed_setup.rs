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
    pub(super) grant_epoch: u64,
    capability_ids: Vec<String>,
}

#[derive(Clone, Copy)]
pub(super) enum SpeechToTextBootstrapOverrideV1 {
    None,
    MissingSettings,
    DriftedSettingsRevision,
    MissingStorage,
    StaleStorageFence,
    StopVaultAfterConfiguration,
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
    launch_reserved_speech_to_text_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        admitted,
        SpeechToTextBootstrapOverrideV1::None,
        true,
        None,
    )
}

pub(super) fn launch_speech_to_text_runtime_without_ready_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedSpeechToTextRuntimeV1,
    bootstrap_override: SpeechToTextBootstrapOverrideV1,
    test_stdio_capture_directory: &Path,
) -> StartedSpeechToTextRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Speech-to-Text launch reservation");
    launch_reserved_speech_to_text_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        admitted,
        bootstrap_override,
        false,
        Some(test_stdio_capture_directory),
    )
}

pub(super) fn retry_speech_to_text_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedSpeechToTextRuntimeV1,
) -> StartedSpeechToTextRuntimeV1 {
    retry_speech_to_text_runtime_with_override_v1(
        supervisor,
        store,
        runtime_dir,
        predecessor,
        SpeechToTextBootstrapOverrideV1::None,
        true,
        None,
    )
}

pub(super) fn retry_speech_to_text_runtime_without_ready_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedSpeechToTextRuntimeV1,
    bootstrap_override: SpeechToTextBootstrapOverrideV1,
    test_stdio_capture_directory: &Path,
) -> StartedSpeechToTextRuntimeV1 {
    retry_speech_to_text_runtime_with_override_v1(
        supervisor,
        store,
        runtime_dir,
        predecessor,
        bootstrap_override,
        false,
        Some(test_stdio_capture_directory),
    )
}

#[allow(clippy::too_many_arguments)]
fn retry_speech_to_text_runtime_with_override_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedSpeechToTextRuntimeV1,
    bootstrap_override: SpeechToTextBootstrapOverrideV1,
    wait_until_ready: bool,
    test_stdio_capture_directory: Option<&Path>,
) -> StartedSpeechToTextRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &predecessor.registration_id)
        .expect("reload Speech-to-Text launch reservation");
    launch_reserved_speech_to_text_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        AdmittedSpeechToTextRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
        bootstrap_override,
        wait_until_ready,
        test_stdio_capture_directory,
    )
}

pub(super) fn launch_speech_to_text_successor_without_ready_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedSpeechToTextRuntimeV1,
    bootstrap_override: SpeechToTextBootstrapOverrideV1,
    test_stdio_capture_directory: &Path,
) -> StartedSpeechToTextRuntimeV1 {
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
    launch_reserved_speech_to_text_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        AdmittedSpeechToTextRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
        bootstrap_override,
        false,
        Some(test_stdio_capture_directory),
    )
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
    let successor = launch_reserved_speech_to_text_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        AdmittedSpeechToTextRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
        SpeechToTextBootstrapOverrideV1::None,
        true,
        None,
    );
    assert_eq!(successor.runtime_generation, previous_generation + 1);
    assert_ne!(successor.runtime_instance_id, previous_instance);
    successor
}

#[allow(clippy::too_many_arguments)]
fn launch_reserved_speech_to_text_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedSpeechToTextRuntimeV1,
    bootstrap_override: SpeechToTextBootstrapOverrideV1,
    wait_until_ready: bool,
    test_stdio_capture_directory: Option<&Path>,
) -> StartedSpeechToTextRuntimeV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = speech_to_text_storage_binding_v1(store, &admitted.registration_id);
    let topology = crate::platform::storage::topology::current(store)
        .expect("Speech-to-Text Storage topology");
    let vault =
        vault_status::read_current(store, &supervisor.relay_port()).expect("live Vault status");
    let mut storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("Speech-to-Text Storage configuration");
    let mut settings = SettingsSnapshotV1 {
        target_id: admitted.registration_id.clone(),
        revision: 1,
        values: Vec::new(),
    };
    let include_storage = match bootstrap_override {
        SpeechToTextBootstrapOverrideV1::None
        | SpeechToTextBootstrapOverrideV1::MissingSettings => true,
        SpeechToTextBootstrapOverrideV1::DriftedSettingsRevision => {
            settings.revision = 2;
            true
        }
        SpeechToTextBootstrapOverrideV1::MissingStorage => false,
        SpeechToTextBootstrapOverrideV1::StaleStorageFence => {
            storage.credential_revision = storage.credential_revision.saturating_add(1);
            true
        }
        SpeechToTextBootstrapOverrideV1::StopVaultAfterConfiguration => {
            supervisor
                .stop(vault_binding::VAULT_PROCESS_ID)
                .expect("stop Vault after Speech-to-Text configuration");
            true
        }
    };
    let configuration = ManagedEngineRuntimeConfigurationV1 {
        major: 1,
        logical_owner_id: SPEECH_TO_TEXT_OWNER_V1.to_owned(),
        registration_id: admitted.registration_id.clone(),
        runtime_instance_id: runtime_instance_id.clone(),
        runtime_generation,
        grant_epoch,
        storage: include_storage.then_some(storage),
        event_hub_endpoint: String::new(),
        event_credential_revision: 0,
        settings_revision: 1,
        logical_human_owner_id: SPEECH_TO_TEXT_LOGICAL_OWNER_ID_V1.to_owned(),
        runtime_artifacts: Vec::new(),
    };
    if let Some(directory) = test_stdio_capture_directory {
        unsafe {
            std::env::set_var(
                crate::runtime::managed::execution::MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV,
                directory,
            );
        }
    }
    let started = managed_launch::start_reserved_engine(
        supervisor,
        runtime_dir,
        reservation,
        configuration,
        if matches!(
            bootstrap_override,
            SpeechToTextBootstrapOverrideV1::MissingSettings
        ) {
            Vec::new()
        } else {
            settings.encode_to_vec()
        },
        &[],
    );
    if matches!(
        bootstrap_override,
        SpeechToTextBootstrapOverrideV1::MissingSettings
            | SpeechToTextBootstrapOverrideV1::MissingStorage
    ) {
        assert!(
            started.is_err(),
            "Kernel must deny incomplete Speech-to-Text bootstrap"
        );
    } else {
        started.expect("start managed Speech-to-Text engine");
    }
    if wait_until_ready {
        supervisor
            .wait_until_ready(&admitted.registration_id)
            .unwrap_or_else(|error| {
                panic!(
                    "Speech-to-Text readiness: {error}; last_failure={:?}",
                    supervisor.last_failure(&admitted.registration_id)
                )
            });
    }
    StartedSpeechToTextRuntimeV1 {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
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
