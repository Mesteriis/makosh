//! Exact signed admission and owner-local lifecycle for Mail Contacts Sync.

use super::*;

use crate::platform::client_realtime::ClientRealtimePublishHandlerV1;
use makosh_gateway_runtime::InMemoryBrowserRealtimeSource;
use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_mail_contacts_sync_api::{
    MAIL_CONTACTS_SYNC_MODULE_ID_V1, MAIL_CONTACTS_SYNC_OWNER_ID_V1,
};
use makosh_mail_contacts_sync_persistence::{
    MAIL_CONTACTS_SYNC_STORAGE_BUNDLE_REVISION_V1, mail_contacts_sync_storage_bundle_v1,
};
use makosh_mail_contacts_sync_runtime::{
    MAIL_CONTACTS_SYNC_STORAGE_CAPABILITY_ID_V1, mail_contacts_sync_module_descriptor_v1,
    mail_contacts_sync_settings_schema_bytes_v1,
};
use makosh_runtime_protocol::v1::{
    ManagedWorkflowConfigurationInstanceV1, ManagedWorkflowRuntimeConfigurationV1, SettingValueV1,
    SettingsSnapshotV1, SettingsValueEntryV1, setting_value_v1::Value,
};

const MAIL_CONTACTS_SYNC_RELEASE_ARTIFACT_ID_V1: &str = "mail_contacts_sync.runtime.v1";
const MAIL_CONTACTS_SYNC_BUILD_ID_V1: &str = "managed-mail-contacts-sync-live";
pub(super) const MAIL_CONTACTS_SYNC_CONFIGURATION_ID_V1: &str = "sync-account-1";
pub(super) const MAIL_CONTACTS_SYNC_LOGICAL_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedMailContactsSyncRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedMailContactsSyncRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
}

pub(super) fn installed_mail_contacts_sync_ensemble_release_v1(
    root: &Path,
) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(mail_release_artifact());
    artifacts.push(contacts_release_artifact_v1());
    artifacts.push(mail_contacts_sync_release_artifact_v1());
    artifacts.push(scheduler_release_artifact());
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed Mail Contacts Sync ensemble release")
}

fn mail_contacts_sync_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        MAIL_CONTACTS_SYNC_RELEASE_ARTIFACT_ID_V1,
        mail_contacts_sync_binary(),
        mail_contacts_sync_module_descriptor_v1(MAIL_CONTACTS_SYNC_BUILD_ID_V1).encode_to_vec(),
    )
    .with_settings_schema(mail_contacts_sync_settings_schema_bytes_v1())
}

pub(super) fn admit_mail_contacts_sync_runtime_v1(
    store: &SqliteControlStore,
) -> AdmittedMailContactsSyncRuntimeV1 {
    let descriptor = mail_contacts_sync_module_descriptor_v1(MAIL_CONTACTS_SYNC_BUILD_ID_V1);
    assert_eq!(descriptor.module_id, MAIL_CONTACTS_SYNC_MODULE_ID_V1);
    assert_eq!(descriptor.owner_id, MAIL_CONTACTS_SYNC_OWNER_ID_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Mail Contacts Sync descriptor");
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
    .expect("approve exact Mail Contacts Sync capabilities");

    let settings = mail_contacts_sync_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            MAIL_CONTACTS_SYNC_RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(
                std::fs::read(mail_contacts_sync_binary())
                    .expect("Mail Contacts Sync runtime binary"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record Mail Contacts Sync release binding");

    let bundle = mail_contacts_sync_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                MAIL_CONTACTS_SYNC_OWNER_ID_V1,
                u64::from(MAIL_CONTACTS_SYNC_STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Mail Contacts Sync Storage bundle"),
        )
        .expect("persist Mail Contacts Sync Storage bundle");

    AdmittedMailContactsSyncRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_mail_contacts_sync_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedMailContactsSyncRuntimeV1,
) -> AdmittedMailContactsSyncRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Mail Contacts Sync launch");
    let bundle = store
        .platform_storage_bundle(
            MAIL_CONTACTS_SYNC_OWNER_ID_V1,
            u64::from(MAIL_CONTACTS_SYNC_STORAGE_BUNDLE_REVISION_V1),
        )
        .expect("read Mail Contacts Sync Storage bundle")
        .expect("Mail Contacts Sync Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        MAIL_CONTACTS_SYNC_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(MAIL_CONTACTS_SYNC_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("Mail Contacts Sync Storage binding issue"),
    )
    .expect("issue Mail Contacts Sync Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Mail Contacts Sync Storage binding");
    admitted
}

pub(super) fn configure_mail_contacts_sync_realtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    realtime: InMemoryBrowserRealtimeSource,
) {
    supervisor
        .configure_client_realtime_handler(Arc::new(ClientRealtimePublishHandlerV1::new(
            Arc::clone(store),
            realtime,
        )))
        .expect("configure Mail Contacts Sync client realtime");
}

pub(super) fn start_mail_contacts_sync_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedMailContactsSyncRuntimeV1,
) -> StartedMailContactsSyncRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Mail Contacts Sync launch reservation");
    start_reserved_mail_contacts_sync_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        admitted,
    )
}

fn start_reserved_mail_contacts_sync_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedMailContactsSyncRuntimeV1,
) -> StartedMailContactsSyncRuntimeV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = mail_contacts_sync_storage_binding_v1(store, &admitted.registration_id);
    let topology = crate::platform::storage::topology::current(store)
        .expect("Mail Contacts Sync Storage topology");
    let vault = vault_status::read_current(store, &supervisor.relay_port())
        .expect("live Vault status for Mail Contacts Sync");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("Mail Contacts Sync Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Mail Contacts Sync Event Hub topology")
        .expect("Mail Contacts Sync Event Hub topology");
    let settings = mail_contacts_sync_settings_snapshot_v1();
    let settings_revision = settings.revision;
    let settings_bytes = settings.encode_to_vec();
    managed_launch::start_reserved_workflow_with_settings(
        supervisor,
        runtime_dir,
        reservation,
        ManagedWorkflowRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: MAIL_CONTACTS_SYNC_LOGICAL_OWNER_ID_V1.to_owned(),
            registration_id: admitted.registration_id.clone(),
            runtime_instance_id: runtime_instance_id.clone(),
            runtime_generation,
            grant_epoch,
            storage: Some(storage),
            event_hub_endpoint: events.nats_endpoint().to_owned(),
            event_credential_revision: events.credential_revision(),
            runtime_artifacts: Vec::new(),
            configuration_instance_id: MAIL_CONTACTS_SYNC_CONFIGURATION_ID_V1.to_owned(),
            settings_revision,
            configuration_instances: vec![ManagedWorkflowConfigurationInstanceV1 {
                configuration_instance_id: MAIL_CONTACTS_SYNC_CONFIGURATION_ID_V1.to_owned(),
                settings_snapshot_bytes: settings_bytes.clone(),
            }],
        },
        settings_bytes,
        &[],
    )
    .expect("start managed Mail Contacts Sync workflow");
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "Mail Contacts Sync readiness: {error}; last_failure={:?}",
                supervisor.last_failure(&admitted.registration_id)
            )
        });
    StartedMailContactsSyncRuntimeV1 {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
    }
}

fn mail_contacts_sync_storage_binding_v1(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(registration_id, MAIL_CONTACTS_SYNC_STORAGE_CAPABILITY_ID_V1)
        .expect("read Mail Contacts Sync Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active Mail Contacts Sync Storage binding")
}

fn mail_contacts_sync_settings_snapshot_v1() -> SettingsSnapshotV1 {
    SettingsSnapshotV1 {
        target_id: MAIL_CONTACTS_SYNC_CONFIGURATION_ID_V1.to_owned(),
        revision: 1,
        values: vec![
            settings_entry(
                "mail_contacts_sync.account_id",
                Value::StringValue(MAIL_ACCOUNT_ID.to_owned()),
            ),
            settings_entry(
                "mail_contacts_sync.direction",
                Value::EnumValue("bidirectional".to_owned()),
            ),
            settings_entry("mail_contacts_sync.enabled", Value::BooleanValue(true)),
            settings_entry(
                "mail_contacts_sync.interval_seconds",
                Value::UnsignedIntegerValue(900),
            ),
            settings_entry(
                "mail_contacts_sync.remote_write_enabled",
                Value::BooleanValue(true),
            ),
        ],
    }
}

fn settings_entry(setting_id: &str, value: Value) -> SettingsValueEntryV1 {
    SettingsValueEntryV1 {
        setting_id: setting_id.to_owned(),
        value: Some(SettingValueV1 { value: Some(value) }),
    }
}

fn mail_contacts_sync_binary() -> PathBuf {
    binary("MAKOSH_MAIL_CONTACTS_SYNC_RUNTIME_BIN")
}
