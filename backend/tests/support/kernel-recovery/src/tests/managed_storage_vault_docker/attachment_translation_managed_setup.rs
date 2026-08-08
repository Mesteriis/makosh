//! Exact signed admission and owner-local lifecycle for Attachment Translation.

use super::*;

use crate::platform::client_realtime::ClientRealtimePublishHandlerV1;
use makosh_attachment_translation_api::{
    ATTACHMENT_TRANSLATION_MODULE_ID_V1, ATTACHMENT_TRANSLATION_OWNER_V1,
};
use makosh_attachment_translation_persistence::{
    ATTACHMENT_TRANSLATION_STORAGE_BUNDLE_REVISION_V1, attachment_translation_storage_bundle_v1,
};
use makosh_attachment_translation_runtime::{
    ATTACHMENT_TRANSLATION_STORAGE_CAPABILITY_ID_V1, attachment_translation_module_descriptor_v1,
    attachment_translation_settings_schema_bytes_v1,
};
use makosh_gateway_runtime::InMemoryBrowserRealtimeSource;
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_runtime_protocol::v1::ManagedWorkflowRuntimeConfigurationV1;

const ATTACHMENT_TRANSLATION_RELEASE_ARTIFACT_ID_V1: &str = "attachment_translation.runtime.v1";
const ATTACHMENT_TRANSLATION_BUILD_ID_V1: &str = "managed-attachment-translation-live";
pub(super) const ATTACHMENT_TRANSLATION_LOGICAL_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedAttachmentTranslationRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedAttachmentTranslationRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    capability_ids: Vec<String>,
}

pub(super) fn installed_attachment_translation_ensemble_release_v1(
    root: &Path,
) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(attachment_security_release_artifact());
    artifacts.push(attachment_text_extraction_release_artifact_v1());
    artifacts.push(ollama_ai_release_artifact_v1());
    artifacts.push(ai_inference_release_artifact_v1());
    artifacts.push(attachment_translation_release_artifact_v1());
    InstalledSignedBundle::install_with_runtime_resources(
        root,
        &artifacts,
        &[],
        &attachment_text_extraction_runtime_resources_v1(),
    )
    .expect("install signed Attachment Translation ensemble release")
}

fn attachment_translation_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        ATTACHMENT_TRANSLATION_RELEASE_ARTIFACT_ID_V1,
        attachment_translation_binary(),
        attachment_translation_module_descriptor_v1(ATTACHMENT_TRANSLATION_BUILD_ID_V1)
            .encode_to_vec(),
    )
    .with_settings_schema(attachment_translation_settings_schema_bytes_v1())
}

pub(super) fn admit_attachment_translation_runtime_v1(
    store: &SqliteControlStore,
) -> AdmittedAttachmentTranslationRuntimeV1 {
    let descriptor =
        attachment_translation_module_descriptor_v1(ATTACHMENT_TRANSLATION_BUILD_ID_V1);
    assert_eq!(descriptor.module_id, ATTACHMENT_TRANSLATION_MODULE_ID_V1);
    assert_eq!(descriptor.owner_id, ATTACHMENT_TRANSLATION_OWNER_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Attachment Translation descriptor");
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
    .expect("approve exact Attachment Translation capabilities");

    let settings = attachment_translation_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            ATTACHMENT_TRANSLATION_RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(
                std::fs::read(attachment_translation_binary())
                    .expect("Attachment Translation runtime binary"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record Attachment Translation release binding");

    let bundle = attachment_translation_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                ATTACHMENT_TRANSLATION_OWNER_V1,
                u64::from(ATTACHMENT_TRANSLATION_STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Attachment Translation Storage bundle"),
        )
        .expect("persist Attachment Translation Storage bundle");

    AdmittedAttachmentTranslationRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn configure_attachment_translation_realtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    realtime: InMemoryBrowserRealtimeSource,
) {
    supervisor
        .configure_client_realtime_handler(Arc::new(ClientRealtimePublishHandlerV1::new(
            Arc::clone(store),
            realtime,
        )))
        .expect("configure Attachment Translation client realtime");
}

pub(super) fn prepare_attachment_translation_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedAttachmentTranslationRuntimeV1,
) -> AdmittedAttachmentTranslationRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Attachment Translation launch");
    let bundle = store
        .platform_storage_bundle(
            ATTACHMENT_TRANSLATION_OWNER_V1,
            u64::from(ATTACHMENT_TRANSLATION_STORAGE_BUNDLE_REVISION_V1),
        )
        .expect("read Attachment Translation Storage bundle")
        .expect("Attachment Translation Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        ATTACHMENT_TRANSLATION_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(ATTACHMENT_TRANSLATION_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("Attachment Translation Storage binding issue"),
    )
    .expect("issue Attachment Translation Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Attachment Translation Storage binding");
    admitted
}

pub(super) fn start_attachment_translation_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedAttachmentTranslationRuntimeV1,
) -> StartedAttachmentTranslationRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Attachment Translation launch reservation");
    start_reserved_attachment_translation_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        admitted,
    )
}

pub(super) fn restart_attachment_translation_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedAttachmentTranslationRuntimeV1,
) -> StartedAttachmentTranslationRuntimeV1 {
    let previous_generation = predecessor.runtime_generation;
    let previous_instance = predecessor.runtime_instance_id.clone();
    let binding = attachment_translation_storage_binding_v1(store, &predecessor.registration_id);
    let issue = storage_successor::issue_after(&binding)
        .expect("derive Attachment Translation Storage successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        ATTACHMENT_TRANSLATION_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Attachment Translation successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Attachment Translation successor");
    let successor = start_reserved_attachment_translation_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        AdmittedAttachmentTranslationRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
    );
    assert_eq!(successor.runtime_generation, previous_generation + 1);
    assert_ne!(successor.runtime_instance_id, previous_instance);
    successor
}

fn start_reserved_attachment_translation_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedAttachmentTranslationRuntimeV1,
) -> StartedAttachmentTranslationRuntimeV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = attachment_translation_storage_binding_v1(store, &admitted.registration_id);
    let topology = crate::platform::storage::topology::current(store)
        .expect("Attachment Translation topology");
    let vault =
        vault_status::read_current(store, &supervisor.relay_port()).expect("live Vault status");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("Attachment Translation Storage configuration");
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
            logical_owner_id: ATTACHMENT_TRANSLATION_LOGICAL_OWNER_ID_V1.to_owned(),
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
    .expect("start managed Attachment Translation workflow");
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "Attachment Translation readiness: {error}; last_failure={:?}",
                supervisor.last_failure(&admitted.registration_id)
            )
        });
    StartedAttachmentTranslationRuntimeV1 {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids: admitted.capability_ids,
    }
}

fn attachment_translation_storage_binding_v1(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(
            registration_id,
            ATTACHMENT_TRANSLATION_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read Attachment Translation Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active Attachment Translation Storage binding")
}

fn attachment_translation_binary() -> PathBuf {
    binary("MAKOSH_ATTACHMENT_TRANSLATION_RUNTIME_BIN")
}
