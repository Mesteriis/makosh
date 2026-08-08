//! Exact signed admission and owner-local Storage for Attachment Preview.

use super::*;

use crate::platform::client_realtime::ClientRealtimePublishHandlerV1;
use makosh_attachment_preview_api::{ATTACHMENT_PREVIEW_MODULE_ID_V1, ATTACHMENT_PREVIEW_OWNER_V1};
use makosh_attachment_preview_persistence::{
    ATTACHMENT_PREVIEW_STORAGE_BUNDLE_REVISION_V1, attachment_preview_storage_bundle_v1,
};
use makosh_attachment_preview_runtime::{
    ATTACHMENT_PREVIEW_STORAGE_CAPABILITY_ID_V1, attachment_preview_module_descriptor_v1,
    attachment_preview_settings_schema_bytes_v1,
};
use makosh_gateway_runtime::InMemoryBrowserRealtimeSource;
use makosh_runtime_protocol::v1::ManagedWorkflowRuntimeConfigurationV1;

const ATTACHMENT_PREVIEW_RELEASE_ARTIFACT_ID_V1: &str = "attachment_preview.runtime.v1";
const ATTACHMENT_PREVIEW_BUILD_ID_V1: &str = "managed-attachment-preview-live";
pub(super) const ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedAttachmentPreviewRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedAttachmentPreviewRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    capability_ids: Vec<String>,
}

pub(super) fn installed_attachment_preview_ensemble_release_v1(
    root: &Path,
) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(attachment_security_release_artifact());
    artifacts.push(attachment_preview_release_artifact_v1());
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed Attachment Preview ensemble release")
}

pub(super) fn admit_attachment_preview_runtime_v1(
    store: &SqliteControlStore,
) -> AdmittedAttachmentPreviewRuntimeV1 {
    let descriptor = attachment_preview_module_descriptor_v1(ATTACHMENT_PREVIEW_BUILD_ID_V1);
    assert_eq!(descriptor.module_id, ATTACHMENT_PREVIEW_MODULE_ID_V1);
    assert_eq!(descriptor.owner_id, ATTACHMENT_PREVIEW_OWNER_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Attachment Preview descriptor");
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
    .expect("approve exact Attachment Preview capabilities");
    let schema = attachment_preview_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            ATTACHMENT_PREVIEW_RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(
                std::fs::read(attachment_preview_binary())
                    .expect("Attachment Preview runtime binary"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&schema).into()),
        ))
        .expect("record Attachment Preview release binding");
    let bundle = attachment_preview_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                ATTACHMENT_PREVIEW_OWNER_V1,
                u64::from(ATTACHMENT_PREVIEW_STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Attachment Preview Storage bundle"),
        )
        .expect("persist Attachment Preview Storage bundle");
    AdmittedAttachmentPreviewRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_attachment_preview_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedAttachmentPreviewRuntimeV1,
) -> AdmittedAttachmentPreviewRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Attachment Preview launch");
    let bundle = store
        .platform_storage_bundle(
            ATTACHMENT_PREVIEW_OWNER_V1,
            u64::from(ATTACHMENT_PREVIEW_STORAGE_BUNDLE_REVISION_V1),
        )
        .expect("read Attachment Preview Storage bundle")
        .expect("Attachment Preview Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        ATTACHMENT_PREVIEW_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(ATTACHMENT_PREVIEW_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("Attachment Preview Storage binding issue"),
    )
    .expect("issue Attachment Preview Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Attachment Preview Storage binding");
    admitted
}

pub(super) fn configure_attachment_preview_realtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    realtime: InMemoryBrowserRealtimeSource,
) {
    supervisor
        .configure_client_realtime_handler(Arc::new(ClientRealtimePublishHandlerV1::new(
            Arc::clone(store),
            realtime,
        )))
        .expect("configure Attachment Preview client realtime");
}

pub(super) fn start_attachment_preview_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedAttachmentPreviewRuntimeV1,
) -> StartedAttachmentPreviewRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Attachment Preview launch reservation");
    start_reserved_attachment_preview_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        admitted.capability_ids,
    )
}

pub(super) fn restart_attachment_preview_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedAttachmentPreviewRuntimeV1,
) -> StartedAttachmentPreviewRuntimeV1 {
    let binding = store
        .platform_storage_binding(
            &predecessor.registration_id,
            ATTACHMENT_PREVIEW_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read predecessor Attachment Preview Storage binding")
        .expect("predecessor Attachment Preview Storage binding");
    let issue = storage_successor::issue_after(&binding)
        .expect("derive Attachment Preview Storage successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        ATTACHMENT_PREVIEW_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Attachment Preview successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Attachment Preview successor Storage binding");
    start_reserved_attachment_preview_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        predecessor.capability_ids,
    )
}

fn start_reserved_attachment_preview_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    capability_ids: Vec<String>,
) -> StartedAttachmentPreviewRuntimeV1 {
    let registration_id = reservation.registration_id().to_owned();
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = store
        .platform_storage_binding(
            &registration_id,
            ATTACHMENT_PREVIEW_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read Attachment Preview Storage binding")
        .expect("Attachment Preview Storage binding");
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
    .expect("build Attachment Preview Storage configuration");
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
            logical_owner_id: ATTACHMENT_PREVIEW_LOGICAL_OWNER_ID_V1.to_owned(),
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
    .expect("start managed Attachment Preview workflow");
    supervisor
        .wait_until_ready(&registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "Attachment Preview readiness: {error}; last_failure={:?}",
                supervisor.last_failure(&registration_id)
            )
        });
    StartedAttachmentPreviewRuntimeV1 {
        registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids,
    }
}

pub(super) fn attachment_preview_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        ATTACHMENT_PREVIEW_RELEASE_ARTIFACT_ID_V1,
        attachment_preview_binary(),
        attachment_preview_module_descriptor_v1(ATTACHMENT_PREVIEW_BUILD_ID_V1).encode_to_vec(),
    )
    .with_settings_schema(attachment_preview_settings_schema_bytes_v1())
}

fn attachment_preview_binary() -> PathBuf {
    binary("MAKOSH_ATTACHMENT_PREVIEW_RUNTIME_BIN")
}
