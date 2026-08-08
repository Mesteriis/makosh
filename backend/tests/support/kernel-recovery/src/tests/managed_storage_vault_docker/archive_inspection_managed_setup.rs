//! Exact signed admission, owner-local Storage and Gateway realtime for Archive Inspection.

use super::*;

use crate::platform::client_realtime::ClientRealtimePublishHandlerV1;
use makosh_attachment_archive_inspection_api::{
    ATTACHMENT_ARCHIVE_INSPECTION_MODULE_ID_V1, ATTACHMENT_ARCHIVE_INSPECTION_OWNER_V1,
};
use makosh_attachment_archive_inspection_persistence::{
    ATTACHMENT_ARCHIVE_INSPECTION_STORAGE_BUNDLE_REVISION_V1,
    attachment_archive_inspection_storage_bundle_v1,
};
use makosh_attachment_archive_inspection_runtime::{
    admission::{
        ATTACHMENT_ARCHIVE_INSPECTION_STORAGE_CAPABILITY_ID,
        attachment_archive_inspection_module_descriptor_v1,
    },
    settings::attachment_archive_inspection_settings_schema_bytes_v1,
};
use makosh_gateway_runtime::InMemoryBrowserRealtimeSource;
use makosh_runtime_protocol::v1::{
    ManagedEngineRuntimeConfigurationV1, SettingValueV1, SettingsSnapshotV1, SettingsValueEntryV1,
    setting_value_v1::Value,
};

const ARCHIVE_INSPECTION_RELEASE_ARTIFACT_ID_V1: &str = "attachment_archive_inspection.runtime.v1";
const ARCHIVE_INSPECTION_BUILD_ID_V1: &str = "managed-archive-inspection-live";
const ARCHIVE_INSPECTION_SETTINGS_REVISION_V1: u64 = 1;
pub(super) const ARCHIVE_INSPECTION_LOGICAL_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedArchiveInspectionRuntimeV1 {
    registration_id: String,
}

pub(super) struct StartedArchiveInspectionRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
}

pub(super) fn installed_archive_inspection_ensemble_release_v1(
    root: &Path,
) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(attachment_security_release_artifact());
    artifacts.push(archive_inspection_release_artifact_v1());
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed Archive Inspection ensemble release")
}

fn archive_inspection_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        ARCHIVE_INSPECTION_RELEASE_ARTIFACT_ID_V1,
        archive_inspection_binary(),
        attachment_archive_inspection_module_descriptor_v1(ARCHIVE_INSPECTION_BUILD_ID_V1)
            .encode_to_vec(),
    )
    .with_settings_schema(attachment_archive_inspection_settings_schema_bytes_v1())
}

pub(super) fn admit_archive_inspection_runtime_v1(
    store: &SqliteControlStore,
) -> AdmittedArchiveInspectionRuntimeV1 {
    let descriptor =
        attachment_archive_inspection_module_descriptor_v1(ARCHIVE_INSPECTION_BUILD_ID_V1);
    assert_eq!(
        descriptor.module_id,
        ATTACHMENT_ARCHIVE_INSPECTION_MODULE_ID_V1
    );
    assert_eq!(descriptor.owner_id, ATTACHMENT_ARCHIVE_INSPECTION_OWNER_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Archive Inspection descriptor");
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
    .expect("approve exact Archive Inspection capabilities");
    let schema = attachment_archive_inspection_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            ARCHIVE_INSPECTION_RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(
                std::fs::read(archive_inspection_binary())
                    .expect("Archive Inspection runtime binary"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&schema).into()),
        ))
        .expect("record Archive Inspection release binding");
    let bundle = attachment_archive_inspection_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                ATTACHMENT_ARCHIVE_INSPECTION_OWNER_V1,
                u64::from(ATTACHMENT_ARCHIVE_INSPECTION_STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Archive Inspection Storage bundle"),
        )
        .expect("persist Archive Inspection Storage bundle");
    AdmittedArchiveInspectionRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
    }
}

pub(super) fn configure_archive_inspection_realtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    realtime: InMemoryBrowserRealtimeSource,
) {
    supervisor
        .configure_client_realtime_handler(Arc::new(ClientRealtimePublishHandlerV1::new(
            Arc::clone(store),
            realtime,
        )))
        .expect("configure Archive Inspection client realtime");
}

pub(super) fn prepare_archive_inspection_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedArchiveInspectionRuntimeV1,
) -> AdmittedArchiveInspectionRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Archive Inspection launch");
    let bundle = store
        .platform_storage_bundle(
            ATTACHMENT_ARCHIVE_INSPECTION_OWNER_V1,
            u64::from(ATTACHMENT_ARCHIVE_INSPECTION_STORAGE_BUNDLE_REVISION_V1),
        )
        .expect("read Archive Inspection Storage bundle")
        .expect("Archive Inspection Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        ATTACHMENT_ARCHIVE_INSPECTION_STORAGE_CAPABILITY_ID,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(ATTACHMENT_ARCHIVE_INSPECTION_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("Archive Inspection Storage binding issue"),
    )
    .expect("issue Archive Inspection Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Archive Inspection Storage binding");
    admitted
}

pub(super) fn start_archive_inspection_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedArchiveInspectionRuntimeV1,
) -> StartedArchiveInspectionRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Archive Inspection launch reservation");
    start_reserved_archive_inspection_runtime_v1(supervisor, store, runtime_dir, reservation)
}

pub(super) fn restart_archive_inspection_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedArchiveInspectionRuntimeV1,
) -> StartedArchiveInspectionRuntimeV1 {
    let binding = store
        .platform_storage_binding(
            &predecessor.registration_id,
            ATTACHMENT_ARCHIVE_INSPECTION_STORAGE_CAPABILITY_ID,
        )
        .expect("read predecessor Archive Inspection Storage binding")
        .expect("predecessor Archive Inspection Storage binding");
    let issue = storage_successor::issue_after(&binding)
        .expect("derive Archive Inspection Storage successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        ATTACHMENT_ARCHIVE_INSPECTION_STORAGE_CAPABILITY_ID,
        issue,
    )
    .expect("reserve Archive Inspection successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Archive Inspection successor Storage binding");
    start_reserved_archive_inspection_runtime_v1(supervisor, store, runtime_dir, reservation)
}

fn start_reserved_archive_inspection_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
) -> StartedArchiveInspectionRuntimeV1 {
    let registration_id = reservation.registration_id().to_owned();
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = store
        .platform_storage_binding(
            &registration_id,
            ATTACHMENT_ARCHIVE_INSPECTION_STORAGE_CAPABILITY_ID,
        )
        .expect("read Archive Inspection Storage binding")
        .expect("Archive Inspection Storage binding");
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
    .expect("build Archive Inspection Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let configuration = ManagedEngineRuntimeConfigurationV1 {
        major: 1,
        logical_owner_id: ATTACHMENT_ARCHIVE_INSPECTION_OWNER_V1.to_owned(),
        registration_id: registration_id.clone(),
        runtime_instance_id: runtime_instance_id.clone(),
        runtime_generation,
        grant_epoch,
        storage: Some(storage),
        event_hub_endpoint: events.nats_endpoint().to_owned(),
        event_credential_revision: events.credential_revision(),
        settings_revision: ARCHIVE_INSPECTION_SETTINGS_REVISION_V1,
        logical_human_owner_id: ARCHIVE_INSPECTION_LOGICAL_OWNER_ID_V1.to_owned(),
        runtime_artifacts: Vec::new(),
    };
    managed_launch::start_reserved_engine(
        supervisor,
        runtime_dir,
        reservation,
        configuration,
        archive_inspection_settings_snapshot(&registration_id).encode_to_vec(),
        &[],
    )
    .expect("start managed Archive Inspection engine");
    StartedArchiveInspectionRuntimeV1 {
        registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
    }
}

fn archive_inspection_settings_snapshot(registration_id: &str) -> SettingsSnapshotV1 {
    fn entry(setting_id: &str, value: u64) -> SettingsValueEntryV1 {
        SettingsValueEntryV1 {
            setting_id: setting_id.to_owned(),
            value: Some(SettingValueV1 {
                value: Some(Value::UnsignedIntegerValue(value)),
            }),
        }
    }

    SettingsSnapshotV1 {
        target_id: registration_id.to_owned(),
        revision: ARCHIVE_INSPECTION_SETTINGS_REVISION_V1,
        values: vec![
            entry(
                "attachment_archive_inspection.max_archive_bytes",
                100 * 1024 * 1024,
            ),
            entry("attachment_archive_inspection.max_depth", 3),
            entry("attachment_archive_inspection.max_entries", 1_000),
            entry(
                "attachment_archive_inspection.max_entry_uncompressed_bytes",
                256 * 1024 * 1024,
            ),
            entry("attachment_archive_inspection.max_path_bytes", 1_024),
            entry(
                "attachment_archive_inspection.max_total_uncompressed_bytes",
                1024 * 1024 * 1024,
            ),
        ],
    }
}

fn archive_inspection_binary() -> PathBuf {
    binary("MAKOSH_ATTACHMENT_ARCHIVE_INSPECTION_RUNTIME_BIN")
}
