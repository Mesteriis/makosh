//! Exact signed admission, Storage and native-resource lifecycle for managed Whisper STT.

use super::*;

use crate::platform::managed::signed_bundle::SignedRuntimeResource;
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_runtime_protocol::v1::{
    ManagedIntegrationRuntimeConfigurationV1, SettingValueV1, SettingsSnapshotV1,
    SettingsValueEntryV1, setting_value_v1::Value,
};
use makosh_whisper_stt_persistence::schema::{
    WHISPER_STT_STORAGE_BUNDLE_REVISION_V1, whisper_stt_storage_bundle_v1,
};
use makosh_whisper_stt_runtime::{
    WHISPER_STT_MODEL_ARTIFACT_ID_V1, WHISPER_STT_MODULE_ID_V1, WHISPER_STT_OWNER_ID_V1,
    WHISPER_STT_RUNNER_ARTIFACT_ID_V1, WHISPER_STT_STORAGE_CAPABILITY_ID_V1,
    whisper_stt_module_descriptor_v1, whisper_stt_settings_schema_bytes_v1,
};

const WHISPER_STT_RELEASE_ARTIFACT_ID_V1: &str = "whisper_stt.runtime.v1";
const WHISPER_STT_BUILD_ID_V1: &str = "managed-whisper-stt-live";
pub(super) const WHISPER_STT_CONFIGURATION_INSTANCE_ID_V1: &str = "whisper-stt-local";
pub(super) const WHISPER_STT_LOGICAL_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedWhisperSttRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedWhisperSttRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    capability_ids: Vec<String>,
}

#[derive(Clone, Copy)]
pub(super) enum WhisperSttBootstrapOverrideV1 {
    None,
    MissingOrDriftedRuntimeArtifact,
    MissingSettings,
    DriftedSettingsTarget,
    MissingStorage,
    StaleStorageFence,
    StopVaultAfterConfiguration,
}

pub(super) fn installed_whisper_stt_release_v1(root: &Path) -> InstalledSignedBundle {
    installed_whisper_stt_release_from_paths_v1(
        root,
        &binary("MAKOSH_WHISPER_STT_MODEL"),
        &binary("MAKOSH_WHISPER_STT_RUNNER"),
    )
}

pub(super) fn installed_whisper_stt_release_from_paths_v1(
    root: &Path,
    model: &Path,
    runner: &Path,
) -> InstalledSignedBundle {
    InstalledSignedBundle::install_with_runtime_resources(
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
            blob_release_artifact(),
            speech_to_text_release_artifact_v1(),
            whisper_stt_release_artifact_v1(),
        ],
        &[],
        &whisper_stt_runtime_resources_from_paths_v1(model, runner),
    )
    .expect("install signed Whisper STT release")
}

pub(super) fn admit_whisper_stt_runtime_v1(
    store: &SqliteControlStore,
) -> AdmittedWhisperSttRuntimeV1 {
    let descriptor = whisper_stt_module_descriptor_v1(WHISPER_STT_BUILD_ID_V1);
    assert_eq!(descriptor.module_id, WHISPER_STT_MODULE_ID_V1);
    assert_eq!(descriptor.owner_id, WHISPER_STT_OWNER_ID_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Whisper STT descriptor");
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
    .expect("approve exact Whisper STT capabilities");
    let settings = whisper_stt_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            WHISPER_STT_RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(
                std::fs::read(whisper_stt_binary()).expect("Whisper STT runtime binary"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record Whisper STT release binding");
    let bundle = whisper_stt_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                WHISPER_STT_OWNER_ID_V1,
                u64::from(WHISPER_STT_STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Whisper STT Storage bundle"),
        )
        .expect("persist Whisper STT Storage bundle");
    AdmittedWhisperSttRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_whisper_stt_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedWhisperSttRuntimeV1,
) -> AdmittedWhisperSttRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Whisper STT launch");
    let bundle = store
        .platform_storage_bundle(
            WHISPER_STT_OWNER_ID_V1,
            u64::from(WHISPER_STT_STORAGE_BUNDLE_REVISION_V1),
        )
        .expect("read Whisper STT Storage bundle")
        .expect("Whisper STT Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        WHISPER_STT_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(WHISPER_STT_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("Whisper STT Storage binding issue"),
    )
    .expect("issue Whisper STT Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Whisper STT Storage binding");
    admitted
}

pub(super) fn start_whisper_stt_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedWhisperSttRuntimeV1,
) -> StartedWhisperSttRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Whisper STT launch reservation");
    launch_reserved_whisper_stt_runtime_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        reservation,
        admitted,
        WhisperSttBootstrapOverrideV1::None,
        true,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn launch_whisper_stt_runtime_without_ready_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedWhisperSttRuntimeV1,
    bootstrap_override: WhisperSttBootstrapOverrideV1,
    test_stdio_capture_directory: &Path,
) -> StartedWhisperSttRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Whisper STT launch reservation");
    launch_reserved_whisper_stt_runtime_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        reservation,
        admitted,
        bootstrap_override,
        false,
        Some(test_stdio_capture_directory),
    )
}

pub(super) fn retry_whisper_stt_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    predecessor: StartedWhisperSttRuntimeV1,
) -> StartedWhisperSttRuntimeV1 {
    retry_whisper_stt_runtime_with_override_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        predecessor,
        WhisperSttBootstrapOverrideV1::None,
        true,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn retry_whisper_stt_runtime_without_ready_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    predecessor: StartedWhisperSttRuntimeV1,
    bootstrap_override: WhisperSttBootstrapOverrideV1,
    test_stdio_capture_directory: &Path,
) -> StartedWhisperSttRuntimeV1 {
    retry_whisper_stt_runtime_with_override_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        predecessor,
        bootstrap_override,
        false,
        Some(test_stdio_capture_directory),
    )
}

#[allow(clippy::too_many_arguments)]
fn retry_whisper_stt_runtime_with_override_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    predecessor: StartedWhisperSttRuntimeV1,
    bootstrap_override: WhisperSttBootstrapOverrideV1,
    wait_until_ready: bool,
    test_stdio_capture_directory: Option<&Path>,
) -> StartedWhisperSttRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &predecessor.registration_id)
        .expect("reload Whisper STT launch reservation");
    launch_reserved_whisper_stt_runtime_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        reservation,
        AdmittedWhisperSttRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
        bootstrap_override,
        wait_until_ready,
        test_stdio_capture_directory,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn launch_whisper_stt_successor_without_ready_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    predecessor: StartedWhisperSttRuntimeV1,
    bootstrap_override: WhisperSttBootstrapOverrideV1,
    test_stdio_capture_directory: &Path,
) -> StartedWhisperSttRuntimeV1 {
    let binding = whisper_stt_storage_binding_v1(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&binding).expect("derive Whisper STT successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        WHISPER_STT_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Whisper STT successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Whisper STT successor");
    launch_reserved_whisper_stt_runtime_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        reservation,
        AdmittedWhisperSttRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
        bootstrap_override,
        false,
        Some(test_stdio_capture_directory),
    )
}

pub(super) fn restart_whisper_stt_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    predecessor: StartedWhisperSttRuntimeV1,
) -> StartedWhisperSttRuntimeV1 {
    let previous_generation = predecessor.runtime_generation;
    let previous_instance = predecessor.runtime_instance_id.clone();
    let binding = whisper_stt_storage_binding_v1(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&binding).expect("derive Whisper STT successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        WHISPER_STT_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Whisper STT successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Whisper STT successor");
    let successor = launch_reserved_whisper_stt_runtime_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        reservation,
        AdmittedWhisperSttRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
        WhisperSttBootstrapOverrideV1::None,
        true,
        None,
    );
    assert_eq!(successor.runtime_generation, previous_generation + 1);
    assert_ne!(successor.runtime_instance_id, previous_instance);
    successor
}

#[allow(clippy::too_many_arguments)]
fn launch_reserved_whisper_stt_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedWhisperSttRuntimeV1,
    bootstrap_override: WhisperSttBootstrapOverrideV1,
    wait_until_ready: bool,
    test_stdio_capture_directory: Option<&Path>,
) -> StartedWhisperSttRuntimeV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = whisper_stt_storage_binding_v1(store, &admitted.registration_id);
    let topology =
        crate::platform::storage::topology::current(store).expect("Whisper STT Storage topology");
    let vault =
        vault_status::read_current(store, &supervisor.relay_port()).expect("live Vault status");
    let mut storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("Whisper STT Storage configuration");
    let mut settings = whisper_stt_settings_snapshot_v1();
    let include_storage = match bootstrap_override {
        WhisperSttBootstrapOverrideV1::None
        | WhisperSttBootstrapOverrideV1::MissingOrDriftedRuntimeArtifact
        | WhisperSttBootstrapOverrideV1::MissingSettings => true,
        WhisperSttBootstrapOverrideV1::DriftedSettingsTarget => {
            settings.target_id = "whisper-stt-drifted".to_owned();
            true
        }
        WhisperSttBootstrapOverrideV1::MissingStorage => false,
        WhisperSttBootstrapOverrideV1::StaleStorageFence => {
            storage.credential_revision = storage.credential_revision.saturating_add(1);
            true
        }
        WhisperSttBootstrapOverrideV1::StopVaultAfterConfiguration => {
            supervisor
                .stop(vault_binding::VAULT_PROCESS_ID)
                .expect("stop Vault after Whisper STT configuration");
            true
        }
    };
    let configuration = ManagedIntegrationRuntimeConfigurationV1 {
        major: 1,
        logical_owner_id: WHISPER_STT_OWNER_ID_V1.to_owned(),
        registration_id: admitted.registration_id.clone(),
        runtime_instance_id: runtime_instance_id.clone(),
        runtime_generation,
        grant_epoch,
        storage: include_storage.then_some(storage),
        event_hub_endpoint: String::new(),
        event_credential_revision: 0,
        configuration_instance_id: WHISPER_STT_CONFIGURATION_INSTANCE_ID_V1.to_owned(),
        runtime_artifacts: Vec::new(),
        integration_state_root: None,
        configuration_instances: Vec::new(),
        logical_human_owner_id: WHISPER_STT_LOGICAL_OWNER_ID_V1.to_owned(),
    };
    if let Some(directory) = test_stdio_capture_directory {
        unsafe {
            std::env::set_var(
                crate::runtime::managed::execution::MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV,
                directory,
            );
        }
    }
    let started = managed_launch::start_reserved_integration(
        supervisor,
        kernel_data,
        runtime_dir,
        reservation,
        managed_launch::ManagedIntegrationLaunchConfiguration {
            runtime: configuration,
            settings_snapshot_bytes: if matches!(
                bootstrap_override,
                WhisperSttBootstrapOverrideV1::MissingSettings
            ) {
                Vec::new()
            } else {
                settings.encode_to_vec()
            },
            granted_capability_ids: &admitted.capability_ids,
        },
    );
    if matches!(
        bootstrap_override,
        WhisperSttBootstrapOverrideV1::MissingOrDriftedRuntimeArtifact
            | WhisperSttBootstrapOverrideV1::MissingSettings
            | WhisperSttBootstrapOverrideV1::MissingStorage
    ) {
        assert!(
            started.is_err(),
            "Kernel must deny incomplete Whisper STT bootstrap"
        );
    } else {
        started.expect("start managed Whisper STT integration");
    }
    if wait_until_ready {
        supervisor
            .wait_until_ready(&admitted.registration_id)
            .unwrap_or_else(|error| {
                panic!(
                    "Whisper STT readiness: {error}; last_failure={:?}",
                    supervisor.last_failure(&admitted.registration_id)
                )
            });
    }
    StartedWhisperSttRuntimeV1 {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids: admitted.capability_ids,
    }
}

fn whisper_stt_settings_snapshot_v1() -> SettingsSnapshotV1 {
    SettingsSnapshotV1 {
        target_id: WHISPER_STT_CONFIGURATION_INSTANCE_ID_V1.to_owned(),
        revision: 1,
        values: vec![
            setting_entry_v1("whisper_stt.allowed_languages_mask", 14),
            setting_entry_v1("whisper_stt.maximum_source_bytes", 16 * 1024 * 1024),
            setting_entry_v1("whisper_stt.maximum_transcript_bytes", 4 * 1024 * 1024),
            setting_entry_v1("whisper_stt.thread_count", 4),
            setting_entry_v1("whisper_stt.timeout_millis", 30_000),
        ],
    }
}

fn setting_entry_v1(setting_id: &str, value: u64) -> SettingsValueEntryV1 {
    SettingsValueEntryV1 {
        setting_id: setting_id.to_owned(),
        value: Some(SettingValueV1 {
            value: Some(Value::UnsignedIntegerValue(value)),
        }),
    }
}

fn whisper_stt_storage_binding_v1(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(registration_id, WHISPER_STT_STORAGE_CAPABILITY_ID_V1)
        .expect("read Whisper STT Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active Whisper STT Storage binding")
}

pub(super) fn whisper_stt_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        WHISPER_STT_RELEASE_ARTIFACT_ID_V1,
        whisper_stt_binary(),
        whisper_stt_module_descriptor_v1(WHISPER_STT_BUILD_ID_V1).encode_to_vec(),
    )
    .with_settings_schema(whisper_stt_settings_schema_bytes_v1())
}

pub(super) fn whisper_stt_runtime_resources_v1() -> [SignedRuntimeResource; 2] {
    whisper_stt_runtime_resources_from_paths_v1(
        &binary("MAKOSH_WHISPER_STT_MODEL"),
        &binary("MAKOSH_WHISPER_STT_RUNNER"),
    )
}

fn whisper_stt_runtime_resources_from_paths_v1(
    model: &Path,
    runner: &Path,
) -> [SignedRuntimeResource; 2] {
    [
        SignedRuntimeResource::read_only_data(
            WHISPER_STT_MODEL_ARTIFACT_ID_V1,
            model.to_owned(),
            WHISPER_STT_MODULE_ID_V1,
        ),
        SignedRuntimeResource::native_executable(
            WHISPER_STT_RUNNER_ARTIFACT_ID_V1,
            runner.to_owned(),
            WHISPER_STT_MODULE_ID_V1,
        ),
    ]
}

pub(super) fn installed_whisper_stt_model_path_v1(root: &Path) -> PathBuf {
    root.join(
        "Макошь.app/Contents/Resources/makosh-kernel-release/distribution/data/whisper_stt.model.v1",
    )
}

pub(super) fn installed_whisper_stt_runner_path_v1(root: &Path) -> PathBuf {
    root.join(
        "Макошь.app/Contents/Resources/makosh-kernel-release/distribution/native-bin/whisper_stt.runner.v1",
    )
}

fn whisper_stt_binary() -> PathBuf {
    binary("MAKOSH_WHISPER_STT_RUNTIME_BIN")
}
