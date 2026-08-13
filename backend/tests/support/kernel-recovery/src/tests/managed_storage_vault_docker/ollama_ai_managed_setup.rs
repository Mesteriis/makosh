//! Exact signed admission and owner-local storage lifecycle for managed Ollama conformance.

use super::*;

use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_ollama_ai_api::{
    OLLAMA_AI_MODULE_ID_V1, OLLAMA_AI_STORAGE_CAPABILITY_ID_V1, OLLAMA_OWNER_ID_V1,
    ollama_ai_settings_schema_bytes_v1,
};
use makosh_ollama_ai_persistence::schema::{
    OLLAMA_AI_STORAGE_BUNDLE_REVISION_V1, ollama_ai_storage_bundle_v1,
};
use makosh_ollama_ai_runtime::ollama_ai_module_descriptor_v1;
use makosh_runtime_protocol::v1::{
    ManagedIntegrationRuntimeConfigurationV1, SettingValueV1, SettingsSnapshotV1,
    SettingsValueEntryV1, setting_value_v1::Value,
};

const OLLAMA_AI_RELEASE_ARTIFACT_ID_V1: &str = "ollama_ai.runtime.v1";
pub(super) const OLLAMA_AI_CONFIGURATION_INSTANCE_ID_V1: &str = "ollama-local";
pub(super) const OLLAMA_AI_LOGICAL_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedOllamaAiRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedOllamaAiRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    capability_ids: Vec<String>,
}

#[derive(Clone, Copy)]
pub(super) enum OllamaAiBootstrapOverrideV1 {
    None,
    MissingSettings,
    DriftedSettingsTarget,
    MissingStorage,
    StaleStorageFence,
    StopVaultAfterConfiguration,
}

pub(super) fn installed_ollama_ai_release_v1(root: &Path) -> InstalledSignedBundle {
    InstalledSignedBundle::install(
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
            ollama_ai_release_artifact_v1(),
        ],
    )
    .expect("install signed Ollama AI release")
}

pub(super) fn ollama_ai_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        OLLAMA_AI_RELEASE_ARTIFACT_ID_V1,
        ollama_ai_binary(),
        ollama_ai_module_descriptor_v1("managed-ollama-ai-negative").encode_to_vec(),
    )
    .with_settings_schema(ollama_ai_settings_schema_bytes_v1())
}

pub(super) fn admit_ollama_ai_runtime_v1(store: &SqliteControlStore) -> AdmittedOllamaAiRuntimeV1 {
    let descriptor = ollama_ai_module_descriptor_v1("managed-ollama-ai-negative");
    assert_eq!(descriptor.module_id, OLLAMA_AI_MODULE_ID_V1);
    assert_eq!(descriptor.owner_id, OLLAMA_OWNER_ID_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Ollama AI descriptor");
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
    .expect("approve exact Ollama AI capabilities");

    let settings = ollama_ai_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            OLLAMA_AI_RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(std::fs::read(ollama_ai_binary()).expect("Ollama AI runtime binary"))
                .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record Ollama AI release binding");

    let bundle = ollama_ai_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                OLLAMA_OWNER_ID_V1,
                u64::from(OLLAMA_AI_STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Ollama AI Storage bundle"),
        )
        .expect("persist Ollama AI Storage bundle");

    AdmittedOllamaAiRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_ollama_ai_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedOllamaAiRuntimeV1,
) -> AdmittedOllamaAiRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Ollama AI launch");
    let bundle = store
        .platform_storage_bundle(
            OLLAMA_OWNER_ID_V1,
            u64::from(OLLAMA_AI_STORAGE_BUNDLE_REVISION_V1),
        )
        .expect("read Ollama AI Storage bundle")
        .expect("Ollama AI Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        OLLAMA_AI_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(OLLAMA_AI_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("Ollama AI Storage binding issue"),
    )
    .expect("issue Ollama AI Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Ollama AI Storage binding");
    admitted
}

pub(super) fn start_ollama_ai_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedOllamaAiRuntimeV1,
    ollama_port: u16,
) -> StartedOllamaAiRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Ollama AI launch reservation");
    launch_reserved_ollama_ai_runtime_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        reservation,
        admitted,
        ollama_port,
        OllamaAiBootstrapOverrideV1::None,
        true,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn launch_ollama_ai_runtime_without_ready_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedOllamaAiRuntimeV1,
    ollama_port: u16,
    bootstrap_override: OllamaAiBootstrapOverrideV1,
    test_stdio_capture_directory: &Path,
) -> StartedOllamaAiRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Ollama AI launch reservation");
    launch_reserved_ollama_ai_runtime_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        reservation,
        admitted,
        ollama_port,
        bootstrap_override,
        false,
        Some(test_stdio_capture_directory),
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn launch_ollama_ai_successor_without_ready_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    predecessor: StartedOllamaAiRuntimeV1,
    ollama_port: u16,
    bootstrap_override: OllamaAiBootstrapOverrideV1,
    test_stdio_capture_directory: &Path,
) -> StartedOllamaAiRuntimeV1 {
    let binding = ollama_ai_storage_binding_v1(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&binding).expect("derive Ollama AI successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        OLLAMA_AI_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Ollama AI successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Ollama AI successor");
    launch_reserved_ollama_ai_runtime_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        reservation,
        AdmittedOllamaAiRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
        ollama_port,
        bootstrap_override,
        false,
        Some(test_stdio_capture_directory),
    )
}

pub(super) fn restart_ollama_ai_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    predecessor: StartedOllamaAiRuntimeV1,
    ollama_port: u16,
) -> StartedOllamaAiRuntimeV1 {
    let previous_generation = predecessor.runtime_generation;
    let previous_instance = predecessor.runtime_instance_id.clone();
    let binding = ollama_ai_storage_binding_v1(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&binding).expect("derive Ollama AI successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        OLLAMA_AI_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Ollama AI successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Ollama AI successor");
    let successor = launch_reserved_ollama_ai_runtime_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        reservation,
        AdmittedOllamaAiRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
        ollama_port,
        OllamaAiBootstrapOverrideV1::None,
        true,
        None,
    );
    assert_eq!(successor.runtime_generation, previous_generation + 1);
    assert_ne!(successor.runtime_instance_id, previous_instance);
    successor
}

#[allow(clippy::too_many_arguments)]
fn launch_reserved_ollama_ai_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedOllamaAiRuntimeV1,
    ollama_port: u16,
    bootstrap_override: OllamaAiBootstrapOverrideV1,
    wait_until_ready: bool,
    test_stdio_capture_directory: Option<&Path>,
) -> StartedOllamaAiRuntimeV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = ollama_ai_storage_binding_v1(store, &admitted.registration_id);
    let topology =
        crate::platform::storage::topology::current(store).expect("Ollama AI Storage topology");
    let vault =
        vault_status::read_current(store, &supervisor.relay_port()).expect("live Vault status");
    let mut storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("Ollama AI Storage configuration");
    let mut settings = ollama_ai_settings_snapshot_v1(ollama_port);
    let include_storage = match bootstrap_override {
        OllamaAiBootstrapOverrideV1::None => true,
        OllamaAiBootstrapOverrideV1::MissingSettings => true,
        OllamaAiBootstrapOverrideV1::DriftedSettingsTarget => {
            settings.target_id = "ollama-drifted".to_owned();
            true
        }
        OllamaAiBootstrapOverrideV1::MissingStorage => false,
        OllamaAiBootstrapOverrideV1::StaleStorageFence => {
            storage.credential_revision = storage.credential_revision.saturating_add(1);
            true
        }
        OllamaAiBootstrapOverrideV1::StopVaultAfterConfiguration => {
            supervisor
                .stop(vault_binding::VAULT_PROCESS_ID)
                .expect("stop Vault after Ollama AI configuration");
            true
        }
    };
    let configuration = ManagedIntegrationRuntimeConfigurationV1 {
        major: 1,
        logical_owner_id: OLLAMA_OWNER_ID_V1.to_owned(),
        registration_id: admitted.registration_id.clone(),
        runtime_instance_id: runtime_instance_id.clone(),
        runtime_generation,
        grant_epoch,
        storage: include_storage.then_some(storage),
        event_hub_endpoint: String::new(),
        event_credential_revision: 0,
        configuration_instance_id: OLLAMA_AI_CONFIGURATION_INSTANCE_ID_V1.to_owned(),
        runtime_artifacts: Vec::new(),
        integration_state_root: None,
        configuration_instances: Vec::new(),
        logical_human_owner_id: OLLAMA_AI_LOGICAL_OWNER_ID_V1.to_owned(),
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
                OllamaAiBootstrapOverrideV1::MissingSettings
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
        OllamaAiBootstrapOverrideV1::MissingSettings | OllamaAiBootstrapOverrideV1::MissingStorage
    ) {
        assert!(
            started.is_err(),
            "Kernel must deny incomplete Ollama AI bootstrap"
        );
    } else {
        started.expect("start managed Ollama AI integration");
    }
    if wait_until_ready {
        supervisor
            .wait_until_ready(&admitted.registration_id)
            .unwrap_or_else(|error| {
                panic!(
                    "Ollama AI readiness: {error}; last_failure={:?}",
                    supervisor.last_failure(&admitted.registration_id)
                )
            });
    }
    StartedOllamaAiRuntimeV1 {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids: admitted.capability_ids,
    }
}

fn ollama_ai_settings_snapshot_v1(ollama_port: u16) -> SettingsSnapshotV1 {
    SettingsSnapshotV1 {
        target_id: OLLAMA_AI_CONFIGURATION_INSTANCE_ID_V1.to_owned(),
        revision: 1,
        values: vec![
            setting_entry_v1(
                "ollama.chat_model",
                Value::StringValue("makosh-conformance:latest".to_owned()),
            ),
            setting_entry_v1(
                "ollama.port",
                Value::UnsignedIntegerValue(u64::from(ollama_port)),
            ),
            setting_entry_v1("ollama.timeout_millis", Value::UnsignedIntegerValue(30_000)),
        ],
    }
}

fn setting_entry_v1(setting_id: &str, value: Value) -> SettingsValueEntryV1 {
    SettingsValueEntryV1 {
        setting_id: setting_id.to_owned(),
        value: Some(SettingValueV1 { value: Some(value) }),
    }
}

fn ollama_ai_storage_binding_v1(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(registration_id, OLLAMA_AI_STORAGE_CAPABILITY_ID_V1)
        .expect("read Ollama AI Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active Ollama AI Storage binding")
}

fn ollama_ai_binary() -> PathBuf {
    binary("MAKOSH_OLLAMA_AI_RUNTIME_BIN")
}
