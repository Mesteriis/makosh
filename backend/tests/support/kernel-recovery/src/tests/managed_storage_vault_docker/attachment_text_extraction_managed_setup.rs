//! Exact signed admission, Storage and OCR-resource staging for Attachment Text Extraction.

use super::*;

use std::os::unix::fs::PermissionsExt;

use crate::platform::client_realtime::ClientRealtimePublishHandlerV1;
use crate::platform::managed::signed_bundle::SignedRuntimeResource;

use makosh_attachment_text_extraction_api::{
    ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1, ATTACHMENT_TEXT_EXTRACTION_OWNER_V1,
};
use makosh_attachment_text_extraction_persistence::{
    ATTACHMENT_TEXT_EXTRACTION_STORAGE_BUNDLE_REVISION_V1,
    attachment_text_extraction_storage_bundle_v1,
};
use makosh_attachment_text_extraction_runtime::{
    ATTACHMENT_TEXT_EXTRACTION_OCR_ENGLISH_ARTIFACT_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_OCR_RUNNER_ARTIFACT_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_OCR_RUSSIAN_ARTIFACT_ID_V1,
    ATTACHMENT_TEXT_EXTRACTION_STORAGE_CAPABILITY_ID_V1,
    attachment_text_extraction_module_descriptor_v1,
    attachment_text_extraction_settings_schema_bytes_v1,
};
use makosh_gateway_runtime::InMemoryBrowserRealtimeSource;
use makosh_runtime_protocol::v1::ManagedWorkflowRuntimeConfigurationV1;

const ATTACHMENT_TEXT_EXTRACTION_RELEASE_ARTIFACT_ID_V1: &str =
    "attachment_text_extraction.runtime.v1";
const ATTACHMENT_TEXT_EXTRACTION_BUILD_ID_V1: &str = "managed-text-extraction-live";
pub(super) const ATTACHMENT_TEXT_EXTRACTION_LOGICAL_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedAttachmentTextExtractionRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedAttachmentTextExtractionRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    pub(super) capability_ids: Vec<String>,
}

pub(super) fn installed_attachment_text_extraction_ensemble_release_v1(
    root: &Path,
) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(attachment_security_release_artifact());
    artifacts.push(attachment_text_extraction_release_artifact_v1());
    InstalledSignedBundle::install_with_runtime_resources(
        root,
        &artifacts,
        &[],
        &attachment_text_extraction_runtime_resources_v1(),
    )
    .expect("install signed Attachment Text Extraction ensemble release")
}

pub(super) fn admit_attachment_text_extraction_runtime_v1(
    store: &SqliteControlStore,
) -> AdmittedAttachmentTextExtractionRuntimeV1 {
    let descriptor =
        attachment_text_extraction_module_descriptor_v1(ATTACHMENT_TEXT_EXTRACTION_BUILD_ID_V1);
    assert_eq!(
        descriptor.module_id,
        ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1
    );
    assert_eq!(descriptor.owner_id, ATTACHMENT_TEXT_EXTRACTION_OWNER_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Attachment Text Extraction descriptor");
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
    .expect("approve exact Attachment Text Extraction capabilities");
    let schema = attachment_text_extraction_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            ATTACHMENT_TEXT_EXTRACTION_RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(
                std::fs::read(attachment_text_extraction_binary())
                    .expect("Attachment Text Extraction runtime binary"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&schema).into()),
        ))
        .expect("record Attachment Text Extraction release binding");
    let bundle = attachment_text_extraction_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                ATTACHMENT_TEXT_EXTRACTION_OWNER_V1,
                u64::from(ATTACHMENT_TEXT_EXTRACTION_STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Attachment Text Extraction Storage bundle"),
        )
        .expect("persist Attachment Text Extraction Storage bundle");
    AdmittedAttachmentTextExtractionRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_attachment_text_extraction_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedAttachmentTextExtractionRuntimeV1,
) -> AdmittedAttachmentTextExtractionRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Attachment Text Extraction launch");
    let bundle = store
        .platform_storage_bundle(
            ATTACHMENT_TEXT_EXTRACTION_OWNER_V1,
            u64::from(ATTACHMENT_TEXT_EXTRACTION_STORAGE_BUNDLE_REVISION_V1),
        )
        .expect("read Attachment Text Extraction Storage bundle")
        .expect("Attachment Text Extraction Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        ATTACHMENT_TEXT_EXTRACTION_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(ATTACHMENT_TEXT_EXTRACTION_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("Attachment Text Extraction Storage binding issue"),
    )
    .expect("issue Attachment Text Extraction Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Attachment Text Extraction Storage binding");
    admitted
}

pub(super) fn configure_attachment_text_extraction_realtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    realtime: InMemoryBrowserRealtimeSource,
) {
    supervisor
        .configure_client_realtime_handler(Arc::new(ClientRealtimePublishHandlerV1::new(
            Arc::clone(store),
            realtime,
        )))
        .expect("configure Attachment Text Extraction client realtime");
}

pub(super) fn start_attachment_text_extraction_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedAttachmentTextExtractionRuntimeV1,
) -> StartedAttachmentTextExtractionRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Attachment Text Extraction launch reservation");
    start_reserved_attachment_text_extraction_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        admitted.capability_ids,
    )
}

pub(super) fn remove_staged_attachment_text_extraction_ocr_runner_v1(
    runtime_dir: &Path,
    runtime_generation: u64,
) {
    let artifacts = runtime_dir
        .join("managed")
        .join(format!("launch-{runtime_generation}"))
        .join("runtime-artifacts");
    let executable = std::fs::read_dir(&artifacts)
        .expect("read staged Attachment Text Extraction runtime artifacts")
        .map(|entry| entry.expect("staged Attachment Text Extraction runtime artifact"))
        .filter(|entry| {
            entry
                .metadata()
                .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        executable.len(),
        1,
        "Attachment Text Extraction must stage exactly one OCR executable"
    );
    std::fs::remove_file(executable[0].path())
        .expect("remove staged OCR runner for parser-unavailable conformance");
}

pub(super) fn restart_attachment_text_extraction_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedAttachmentTextExtractionRuntimeV1,
) -> StartedAttachmentTextExtractionRuntimeV1 {
    let binding = store
        .platform_storage_binding(
            &predecessor.registration_id,
            ATTACHMENT_TEXT_EXTRACTION_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read predecessor Attachment Text Extraction Storage binding")
        .expect("predecessor Attachment Text Extraction Storage binding");
    let issue = storage_successor::issue_after(&binding)
        .expect("derive Attachment Text Extraction Storage successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        ATTACHMENT_TEXT_EXTRACTION_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Attachment Text Extraction successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Attachment Text Extraction successor Storage binding");
    start_reserved_attachment_text_extraction_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        predecessor.capability_ids,
    )
}

fn start_reserved_attachment_text_extraction_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    capability_ids: Vec<String>,
) -> StartedAttachmentTextExtractionRuntimeV1 {
    let registration_id = reservation.registration_id().to_owned();
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = store
        .platform_storage_binding(
            &registration_id,
            ATTACHMENT_TEXT_EXTRACTION_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read Attachment Text Extraction Storage binding")
        .expect("Attachment Text Extraction Storage binding");
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
    .expect("build Attachment Text Extraction Storage configuration");
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
            logical_owner_id: ATTACHMENT_TEXT_EXTRACTION_LOGICAL_OWNER_ID_V1.to_owned(),
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
    .expect("start managed Attachment Text Extraction workflow");
    supervisor
        .wait_until_ready(&registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "Attachment Text Extraction readiness: {error}; last_failure={:?}",
                supervisor.last_failure(&registration_id)
            )
        });
    StartedAttachmentTextExtractionRuntimeV1 {
        registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids,
    }
}

pub(super) fn attachment_text_extraction_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        ATTACHMENT_TEXT_EXTRACTION_RELEASE_ARTIFACT_ID_V1,
        attachment_text_extraction_binary(),
        attachment_text_extraction_module_descriptor_v1(ATTACHMENT_TEXT_EXTRACTION_BUILD_ID_V1)
            .encode_to_vec(),
    )
    .with_settings_schema(attachment_text_extraction_settings_schema_bytes_v1())
}

pub(super) fn attachment_text_extraction_runtime_resources_v1() -> [SignedRuntimeResource; 3] {
    [
        SignedRuntimeResource::read_only_data(
            ATTACHMENT_TEXT_EXTRACTION_OCR_ENGLISH_ARTIFACT_ID_V1,
            binary("MAKOSH_ATTACHMENT_TEXT_EXTRACTION_OCR_ENG"),
            ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1,
        ),
        SignedRuntimeResource::native_executable(
            ATTACHMENT_TEXT_EXTRACTION_OCR_RUNNER_ARTIFACT_ID_V1,
            binary("MAKOSH_ATTACHMENT_TEXT_EXTRACTION_OCR_RUNNER"),
            ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1,
        ),
        SignedRuntimeResource::read_only_data(
            ATTACHMENT_TEXT_EXTRACTION_OCR_RUSSIAN_ARTIFACT_ID_V1,
            binary("MAKOSH_ATTACHMENT_TEXT_EXTRACTION_OCR_RUS"),
            ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1,
        ),
    ]
}

fn attachment_text_extraction_binary() -> PathBuf {
    binary("MAKOSH_ATTACHMENT_TEXT_EXTRACTION_RUNTIME_BIN")
}
