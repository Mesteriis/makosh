//! Exact signed admission and owner-local storage lifecycle for managed AI inference.

use super::*;

use crate::modules::capability::module_request::ModuleRequestRouteHandlerV1;
use makosh_ai_contracts::{AI_INFERENCE_MODULE_ID_V1, AI_OWNER_V1};
use makosh_ai_inference_persistence::schema::{
    AI_INFERENCE_STORAGE_BUNDLE_REVISION_V1, ai_inference_storage_bundle_v1,
};
use makosh_ai_inference_runtime::{
    AI_INFERENCE_STORAGE_CAPABILITY_ID_V1, ai_inference_module_descriptor_v1,
    ai_inference_settings_schema_bytes_v1,
};
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_runtime_protocol::v1::{ManagedEngineRuntimeConfigurationV1, SettingsSnapshotV1};

const AI_INFERENCE_RELEASE_ARTIFACT_ID_V1: &str = "ai_inference.runtime.v1";
const AI_INFERENCE_BUILD_ID_V1: &str = "managed-ai-inference-negative";
pub(super) const AI_INFERENCE_LOGICAL_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedAiInferenceRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedAiInferenceRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    capability_ids: Vec<String>,
}

pub(super) fn installed_ai_inference_release_v1(root: &Path) -> InstalledSignedBundle {
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
            blob_release_artifact(),
            ollama_ai_release_artifact_v1(),
            ai_inference_release_artifact_v1(),
        ],
    )
    .expect("install signed AI inference release")
}

pub(super) fn ai_inference_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        AI_INFERENCE_RELEASE_ARTIFACT_ID_V1,
        ai_inference_binary(),
        ai_inference_module_descriptor_v1(AI_INFERENCE_BUILD_ID_V1).encode_to_vec(),
    )
    .with_settings_schema(ai_inference_settings_schema_bytes_v1())
}

pub(super) fn configure_ai_module_request_router_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
) {
    supervisor
        .configure_module_request_handler(Arc::new(ModuleRequestRouteHandlerV1::new(
            Arc::clone(store),
            supervisor.relay_port(),
        )))
        .expect("configure AI module request router");
}

pub(super) fn admit_ai_inference_runtime_v1(
    store: &SqliteControlStore,
) -> AdmittedAiInferenceRuntimeV1 {
    let descriptor = ai_inference_module_descriptor_v1(AI_INFERENCE_BUILD_ID_V1);
    assert_eq!(descriptor.module_id, AI_INFERENCE_MODULE_ID_V1);
    assert_eq!(descriptor.owner_id, AI_OWNER_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact AI inference descriptor");
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
    .expect("approve exact AI inference capabilities");

    let settings = ai_inference_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            AI_INFERENCE_RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(
                std::fs::read(ai_inference_binary()).expect("AI inference runtime binary"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record AI inference release binding");

    let bundle = ai_inference_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                AI_OWNER_V1,
                u64::from(AI_INFERENCE_STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("AI inference Storage bundle"),
        )
        .expect("persist AI inference Storage bundle");

    AdmittedAiInferenceRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_ai_inference_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedAiInferenceRuntimeV1,
) -> AdmittedAiInferenceRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve AI inference launch");
    let bundle = store
        .platform_storage_bundle(
            AI_OWNER_V1,
            u64::from(AI_INFERENCE_STORAGE_BUNDLE_REVISION_V1),
        )
        .expect("read AI inference Storage bundle")
        .expect("AI inference Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        AI_INFERENCE_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(AI_INFERENCE_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("AI inference Storage binding issue"),
    )
    .expect("issue AI inference Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision AI inference Storage binding");
    admitted
}

pub(super) fn start_ai_inference_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedAiInferenceRuntimeV1,
) -> StartedAiInferenceRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load AI inference launch reservation");
    start_reserved_ai_inference_runtime_v1(supervisor, store, runtime_dir, reservation, admitted)
}

pub(super) fn restart_ai_inference_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedAiInferenceRuntimeV1,
) -> StartedAiInferenceRuntimeV1 {
    let previous_generation = predecessor.runtime_generation;
    let previous_instance = predecessor.runtime_instance_id.clone();
    let binding = ai_inference_storage_binding_v1(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&binding).expect("derive AI inference successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        AI_INFERENCE_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve AI inference successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision AI inference successor");
    let successor = start_reserved_ai_inference_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        AdmittedAiInferenceRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
    );
    assert_eq!(successor.runtime_generation, previous_generation + 1);
    assert_ne!(successor.runtime_instance_id, previous_instance);
    successor
}

fn start_reserved_ai_inference_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedAiInferenceRuntimeV1,
) -> StartedAiInferenceRuntimeV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = ai_inference_storage_binding_v1(store, &admitted.registration_id);
    let topology =
        crate::platform::storage::topology::current(store).expect("AI inference Storage topology");
    let vault =
        vault_status::read_current(store, &supervisor.relay_port()).expect("live Vault status");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("AI inference Storage configuration");
    let configuration = ManagedEngineRuntimeConfigurationV1 {
        major: 1,
        logical_owner_id: AI_OWNER_V1.to_owned(),
        registration_id: admitted.registration_id.clone(),
        runtime_instance_id: runtime_instance_id.clone(),
        runtime_generation,
        grant_epoch,
        storage: Some(storage),
        event_hub_endpoint: String::new(),
        event_credential_revision: 0,
        settings_revision: 1,
        logical_human_owner_id: AI_INFERENCE_LOGICAL_OWNER_ID_V1.to_owned(),
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
    .expect("start managed AI inference engine");
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "AI inference readiness: {error}; last_failure={:?}",
                supervisor.last_failure(&admitted.registration_id)
            )
        });
    StartedAiInferenceRuntimeV1 {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        capability_ids: admitted.capability_ids,
    }
}

fn ai_inference_storage_binding_v1(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(registration_id, AI_INFERENCE_STORAGE_CAPABILITY_ID_V1)
        .expect("read AI inference Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active AI inference Storage binding")
}

fn ai_inference_binary() -> PathBuf {
    binary("MAKOSH_AI_INFERENCE_RUNTIME_BIN")
}
