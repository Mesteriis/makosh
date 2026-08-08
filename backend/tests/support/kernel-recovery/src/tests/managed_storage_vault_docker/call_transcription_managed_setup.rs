//! Exact signed admission and owner-local Storage for Call Transcription.

use super::*;

use crate::platform::{
    client_realtime::ClientRealtimePublishHandlerV1, managed::signed_bundle::SignedRuntimeResource,
};
use makosh_call_transcription_api::{MODULE_ID_V1, OWNER_ID_V1};
use makosh_call_transcription_persistence::{
    CALL_TRANSCRIPTION_STORAGE_BUNDLE_REVISION_V1, call_transcription_storage_bundle_v1,
};
use makosh_call_transcription_runtime::admission::{
    STORAGE_CAPABILITY_ID_V1, module_descriptor_v1, settings_schema_bytes_v1,
};
use makosh_gateway_runtime::InMemoryBrowserRealtimeSource;
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_runtime_protocol::v1::ManagedWorkflowRuntimeConfigurationV1;

const RELEASE_ARTIFACT_ID_V1: &str = "call_transcription.runtime.v1";
const BUILD_ID_V1: &str = "managed-call-transcription-live";
pub(super) const CALL_TRANSCRIPTION_LOGICAL_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedCallTranscriptionRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

#[derive(Clone)]
pub(super) struct StartedCallTranscriptionRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    capability_ids: Vec<String>,
}

pub(super) fn installed_call_transcription_ensemble_release_v1(
    root: &Path,
) -> InstalledSignedBundle {
    let artifacts = [
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
        desktop_recording_release_artifact_v1(),
        speech_to_text_release_artifact_v1(),
        whisper_stt_release_artifact_v1(),
        call_transcription_release_artifact_v1(),
    ];
    let resources: [SignedRuntimeResource; 2] = whisper_stt_runtime_resources_v1();
    InstalledSignedBundle::install_with_runtime_resources(root, &artifacts, &[], &resources)
        .expect("install signed Call Transcription ensemble release")
}

pub(super) fn admit_call_transcription_runtime_v1(
    store: &SqliteControlStore,
) -> AdmittedCallTranscriptionRuntimeV1 {
    let descriptor = module_descriptor_v1(BUILD_ID_V1);
    assert_eq!(descriptor.module_id, MODULE_ID_V1);
    assert_eq!(descriptor.owner_id, OWNER_ID_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Call Transcription descriptor");
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
    .expect("approve exact Call Transcription capabilities");
    let settings = settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(
                std::fs::read(call_transcription_binary())
                    .expect("Call Transcription runtime binary"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record Call Transcription release binding");
    let bundle = call_transcription_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                OWNER_ID_V1,
                u64::from(CALL_TRANSCRIPTION_STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Call Transcription Storage bundle"),
        )
        .expect("persist Call Transcription Storage bundle");
    AdmittedCallTranscriptionRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn configure_call_transcription_realtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    realtime: InMemoryBrowserRealtimeSource,
) {
    supervisor
        .configure_client_realtime_handler(Arc::new(ClientRealtimePublishHandlerV1::new(
            Arc::clone(store),
            realtime,
        )))
        .expect("configure Call Transcription client realtime");
}

pub(super) fn prepare_call_transcription_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedCallTranscriptionRuntimeV1,
) -> AdmittedCallTranscriptionRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Call Transcription launch");
    let bundle = store
        .platform_storage_bundle(
            OWNER_ID_V1,
            u64::from(CALL_TRANSCRIPTION_STORAGE_BUNDLE_REVISION_V1),
        )
        .expect("read Call Transcription Storage bundle")
        .expect("Call Transcription Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(CALL_TRANSCRIPTION_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("Call Transcription Storage binding issue"),
    )
    .expect("issue Call Transcription Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Call Transcription Storage binding");
    admitted
}

pub(super) fn start_call_transcription_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedCallTranscriptionRuntimeV1,
) -> StartedCallTranscriptionRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Call Transcription launch reservation");
    start_reserved_call_transcription_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        admitted.registration_id,
        admitted.capability_ids,
    )
}

pub(super) fn restart_call_transcription_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedCallTranscriptionRuntimeV1,
) -> StartedCallTranscriptionRuntimeV1 {
    let binding = call_transcription_storage_binding_v1(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&binding)
        .expect("derive Call Transcription Storage successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Call Transcription successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Call Transcription successor Storage binding");
    start_reserved_call_transcription_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        predecessor.registration_id,
        predecessor.capability_ids,
    )
}

fn start_reserved_call_transcription_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    registration_id: String,
    capability_ids: Vec<String>,
) -> StartedCallTranscriptionRuntimeV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = call_transcription_storage_binding_v1(store, &registration_id);
    let topology = crate::platform::storage::topology::current(store)
        .expect("Call Transcription Storage topology");
    let vault =
        vault_status::read_current(store, &supervisor.relay_port()).expect("live Vault status");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("Call Transcription Storage configuration");
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
            logical_owner_id: CALL_TRANSCRIPTION_LOGICAL_OWNER_ID_V1.to_owned(),
            registration_id: registration_id.clone(),
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
        &capability_ids,
    )
    .expect("start managed Call Transcription workflow");
    supervisor
        .wait_until_ready(&registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "Call Transcription readiness: {error}; last_failure={:?}",
                supervisor.last_failure(&registration_id)
            )
        });
    StartedCallTranscriptionRuntimeV1 {
        registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids,
    }
}

fn call_transcription_storage_binding_v1(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(registration_id, STORAGE_CAPABILITY_ID_V1)
        .expect("read Call Transcription Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active Call Transcription Storage binding")
}

fn call_transcription_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        RELEASE_ARTIFACT_ID_V1,
        call_transcription_binary(),
        module_descriptor_v1(BUILD_ID_V1).encode_to_vec(),
    )
    .with_settings_schema(settings_schema_bytes_v1())
}

fn call_transcription_binary() -> PathBuf {
    binary("MAKOSH_CALL_TRANSCRIPTION_RUNTIME_BIN")
}
