//! Exact signed admission and owner-local Storage for retained Preview evidence replay.

use super::*;

use makosh_attachment_preview_evidence_replay_api::{
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_MODULE_ID_V1, ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_OWNER_V1,
};
use makosh_attachment_preview_evidence_replay_persistence::attachment_preview_evidence_replay_storage_bundle_v1;
use makosh_attachment_preview_evidence_replay_runtime::{
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_STORAGE_CAPABILITY_ID_V1,
    attachment_preview_evidence_replay_module_descriptor_v1,
    attachment_preview_evidence_replay_settings_schema_bytes_v1,
};
use makosh_runtime_protocol::v1::ManagedWorkflowRuntimeConfigurationV1;

const REPLAY_RELEASE_ARTIFACT_ID_V1: &str = "attachment_preview_evidence_replay.runtime.v1";
const REPLAY_BUILD_ID_V1: &str = "managed-attachment-preview-evidence-replay-live";

pub(super) struct AdmittedAttachmentPreviewEvidenceReplayRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedAttachmentPreviewEvidenceReplayRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    capability_ids: Vec<String>,
}

pub(super) fn installed_attachment_preview_replay_ensemble_release_v1(
    root: &Path,
) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(mail_release_artifact());
    artifacts.push(attachment_security_release_artifact());
    artifacts.push(attachment_preview_release_artifact_v1());
    artifacts.push(attachment_preview_evidence_replay_release_artifact_v1());
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed retained Preview evidence replay ensemble release")
}

pub(super) fn admit_attachment_preview_evidence_replay_runtime_v1(
    store: &SqliteControlStore,
) -> AdmittedAttachmentPreviewEvidenceReplayRuntimeV1 {
    let descriptor = attachment_preview_evidence_replay_module_descriptor_v1(REPLAY_BUILD_ID_V1);
    assert_eq!(
        descriptor.module_id,
        ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_MODULE_ID_V1
    );
    assert_eq!(
        descriptor.owner_id,
        ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_OWNER_V1
    );
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact retained Preview evidence replay descriptor");
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
    .expect("approve exact retained Preview evidence replay capabilities");
    let schema = attachment_preview_evidence_replay_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            REPLAY_RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(
                std::fs::read(attachment_preview_evidence_replay_binary_v1())
                    .expect("retained Preview evidence replay runtime binary"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&schema).into()),
        ))
        .expect("record retained Preview evidence replay release binding");
    let bundle = attachment_preview_evidence_replay_storage_bundle_v1();
    let bundle_revision = bundle.revision;
    let bundle = bundle.encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_OWNER_V1,
                u64::from(bundle_revision),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("retained Preview evidence replay Storage bundle"),
        )
        .expect("persist retained Preview evidence replay Storage bundle");
    AdmittedAttachmentPreviewEvidenceReplayRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_attachment_preview_evidence_replay_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedAttachmentPreviewEvidenceReplayRuntimeV1,
) -> AdmittedAttachmentPreviewEvidenceReplayRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve retained Preview evidence replay launch");
    let bundle = attachment_preview_evidence_replay_storage_bundle_v1();
    let bundle_revision = bundle.revision;
    let bundle = store
        .platform_storage_bundle(
            ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_OWNER_V1,
            u64::from(bundle_revision),
        )
        .expect("read retained Preview evidence replay Storage bundle")
        .expect("retained Preview evidence replay Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(1, 1, u64::from(bundle_revision), *bundle.digest())
            .expect("retained Preview evidence replay Storage binding issue"),
    )
    .expect("issue retained Preview evidence replay Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision retained Preview evidence replay Storage binding");
    admitted
}

pub(super) fn start_attachment_preview_evidence_replay_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedAttachmentPreviewEvidenceReplayRuntimeV1,
) -> StartedAttachmentPreviewEvidenceReplayRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load retained Preview evidence replay launch reservation");
    start_reserved_attachment_preview_evidence_replay_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        admitted.capability_ids,
    )
}

pub(super) fn restart_attachment_preview_evidence_replay_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedAttachmentPreviewEvidenceReplayRuntimeV1,
) -> StartedAttachmentPreviewEvidenceReplayRuntimeV1 {
    let admitted =
        reserve_attachment_preview_evidence_replay_successor_v1(supervisor, store, predecessor);
    start_attachment_preview_evidence_replay_runtime_v1(supervisor, store, runtime_dir, admitted)
}

pub(super) fn reserve_attachment_preview_evidence_replay_successor_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    predecessor: StartedAttachmentPreviewEvidenceReplayRuntimeV1,
) -> AdmittedAttachmentPreviewEvidenceReplayRuntimeV1 {
    let binding = store
        .platform_storage_binding(
            &predecessor.registration_id,
            ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read predecessor retained Preview evidence replay Storage binding")
        .expect("predecessor retained Preview evidence replay Storage binding");
    let issue = storage_successor::issue_after(&binding)
        .expect("derive retained Preview evidence replay Storage successor");
    let (_reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve retained Preview evidence replay successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision retained Preview evidence replay successor Storage binding");
    AdmittedAttachmentPreviewEvidenceReplayRuntimeV1 {
        registration_id: predecessor.registration_id,
        capability_ids: predecessor.capability_ids,
    }
}

fn start_reserved_attachment_preview_evidence_replay_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    capability_ids: Vec<String>,
) -> StartedAttachmentPreviewEvidenceReplayRuntimeV1 {
    let registration_id = reservation.registration_id().to_owned();
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = store
        .platform_storage_binding(
            &registration_id,
            ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read retained Preview evidence replay Storage binding")
        .expect("retained Preview evidence replay Storage binding");
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
    .expect("build retained Preview evidence replay Storage configuration");
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
    .expect("start managed retained Preview evidence replay workflow");
    supervisor
        .wait_until_ready(&registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "retained Preview evidence replay readiness: {error}; last_failure={:?}",
                supervisor.last_failure(&registration_id)
            )
        });
    StartedAttachmentPreviewEvidenceReplayRuntimeV1 {
        registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids,
    }
}

fn attachment_preview_evidence_replay_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        REPLAY_RELEASE_ARTIFACT_ID_V1,
        attachment_preview_evidence_replay_binary_v1(),
        attachment_preview_evidence_replay_module_descriptor_v1(REPLAY_BUILD_ID_V1).encode_to_vec(),
    )
    .with_settings_schema(attachment_preview_evidence_replay_settings_schema_bytes_v1())
}

fn attachment_preview_evidence_replay_binary_v1() -> PathBuf {
    binary("MAKOSH_ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_BIN")
}
