//! Dormant exact Mail-to-Person workflow admission used only by conformance.

use super::*;

use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_mail_persons_sync_api::MAIL_PERSONS_SYNC_OWNER_V1;
use makosh_mail_persons_sync_persistence::{
    MAIL_PERSONS_SYNC_STORAGE_BUNDLE_REVISION_V1, mail_persons_sync_storage_bundle_v1,
};
use makosh_mail_persons_sync_runtime::{
    MAIL_PERSONS_SYNC_MODULE_ID_V1, MAIL_PERSONS_SYNC_STORAGE_CAPABILITY_ID_V1,
    mail_persons_sync_module_descriptor_v1, mail_persons_sync_settings_schema_bytes_v1,
};
use makosh_runtime_protocol::v1::{
    ManagedWorkflowConfigurationInstanceV1, ManagedWorkflowRuntimeConfigurationV1,
    SettingsSnapshotV1,
};

const RELEASE_ARTIFACT_ID_V1: &str = "mail_persons_sync.runtime.v1";
const BUILD_ID_V1: &str = "managed-mail-persons-sync-task5b";
const CONFIGURATION_ID_V1: &str = "dormant-public-account-binding";
pub(super) const MAIL_PERSONS_SYNC_LOGICAL_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedMailPersonsSyncRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedMailPersonsSyncRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    capability_ids: Vec<String>,
}

pub(super) enum MailPersonsSyncBootstrapOverrideV1 {
    None,
    MissingStorage,
    ExtraCapability,
    StopVaultAfterConfiguration,
    StaleCredentialFence,
    UnavailableStoragePort(u16),
    UnavailableEventEndpoint(String),
}

pub(super) fn installed_mail_persons_sync_release_v1(root: &Path) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(persons_release_artifact_v1());
    artifacts.push(mail_persons_sync_release_artifact_v1());
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed dormant Mail Persons Sync release")
}

pub(super) fn mail_persons_sync_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        RELEASE_ARTIFACT_ID_V1,
        binary("MAKOSH_MAIL_PERSONS_SYNC_RUNTIME_BIN"),
        mail_persons_sync_module_descriptor_v1(BUILD_ID_V1).encode_to_vec(),
    )
    .with_settings_schema(mail_persons_sync_settings_schema_bytes_v1())
}

pub(super) fn admit_mail_persons_sync_runtime_v1(
    store: &SqliteControlStore,
) -> AdmittedMailPersonsSyncRuntimeV1 {
    let descriptor = mail_persons_sync_module_descriptor_v1(BUILD_ID_V1);
    assert_eq!(descriptor.module_id, MAIL_PERSONS_SYNC_MODULE_ID_V1);
    assert_eq!(descriptor.owner_id, MAIL_PERSONS_SYNC_OWNER_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register dormant Mail Persons Sync descriptor");
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
    .expect("approve dormant Mail Persons Sync capabilities for conformance");
    let settings = mail_persons_sync_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(
                std::fs::read(binary("MAKOSH_MAIL_PERSONS_SYNC_RUNTIME_BIN"))
                    .expect("runtime binary"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record dormant Mail Persons Sync release binding");
    let bundle = mail_persons_sync_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                MAIL_PERSONS_SYNC_OWNER_V1,
                u64::from(MAIL_PERSONS_SYNC_STORAGE_BUNDLE_REVISION_V1),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Mail Persons Sync storage bundle"),
        )
        .expect("persist Mail Persons Sync bundle");
    AdmittedMailPersonsSyncRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_mail_persons_sync_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedMailPersonsSyncRuntimeV1,
) -> AdmittedMailPersonsSyncRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Mail Persons Sync launch");
    let bundle = store
        .platform_storage_bundle(
            MAIL_PERSONS_SYNC_OWNER_V1,
            u64::from(MAIL_PERSONS_SYNC_STORAGE_BUNDLE_REVISION_V1),
        )
        .expect("read bundle")
        .expect("bundle exists");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        MAIL_PERSONS_SYNC_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(MAIL_PERSONS_SYNC_STORAGE_BUNDLE_REVISION_V1),
            *bundle.digest(),
        )
        .expect("storage binding issue"),
    )
    .expect("issue Mail Persons Sync storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Mail Persons Sync binding");
    admitted
}

pub(super) fn start_mail_persons_sync_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedMailPersonsSyncRuntimeV1,
) -> StartedMailPersonsSyncRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Mail Persons Sync reservation");
    start_reserved_mail_persons_sync_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        admitted,
        MailPersonsSyncBootstrapOverrideV1::None,
        true,
    )
}

pub(super) fn launch_mail_persons_sync_runtime_without_ready_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedMailPersonsSyncRuntimeV1,
    bootstrap_override: MailPersonsSyncBootstrapOverrideV1,
) -> StartedMailPersonsSyncRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Mail Persons Sync reservation");
    start_reserved_mail_persons_sync_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        admitted,
        bootstrap_override,
        false,
    )
}

pub(super) fn restart_mail_persons_sync_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedMailPersonsSyncRuntimeV1,
) -> StartedMailPersonsSyncRuntimeV1 {
    let previous_generation = predecessor.runtime_generation;
    let binding = store
        .platform_storage_binding(
            &predecessor.registration_id,
            MAIL_PERSONS_SYNC_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read predecessor Mail Persons Sync binding")
        .expect("predecessor Mail Persons Sync binding");
    let issue = storage_successor::issue_after(&binding)
        .expect("derive Mail Persons Sync storage successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        MAIL_PERSONS_SYNC_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Mail Persons Sync successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Mail Persons Sync successor");
    let successor = start_reserved_mail_persons_sync_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        AdmittedMailPersonsSyncRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
        MailPersonsSyncBootstrapOverrideV1::None,
        true,
    );
    assert_eq!(successor.runtime_generation, previous_generation + 1);
    successor
}

pub(super) fn launch_mail_persons_sync_successor_without_ready_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedMailPersonsSyncRuntimeV1,
    bootstrap_override: MailPersonsSyncBootstrapOverrideV1,
) -> StartedMailPersonsSyncRuntimeV1 {
    let binding = store
        .platform_storage_binding(
            &predecessor.registration_id,
            MAIL_PERSONS_SYNC_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read predecessor Mail Persons Sync binding")
        .expect("predecessor Mail Persons Sync binding");
    let issue = storage_successor::issue_after(&binding)
        .expect("derive Mail Persons Sync storage successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        MAIL_PERSONS_SYNC_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Mail Persons Sync successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Mail Persons Sync successor");
    start_reserved_mail_persons_sync_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        AdmittedMailPersonsSyncRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
        bootstrap_override,
        false,
    )
}

pub(super) fn reject_mail_persons_sync_missing_storage_successor_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedMailPersonsSyncRuntimeV1,
) -> StartedMailPersonsSyncRuntimeV1 {
    let binding = store
        .platform_storage_binding(
            &predecessor.registration_id,
            MAIL_PERSONS_SYNC_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read predecessor Mail Persons Sync binding")
        .expect("predecessor Mail Persons Sync binding");
    let issue = storage_successor::issue_after(&binding)
        .expect("derive Mail Persons Sync missing-storage successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        MAIL_PERSONS_SYNC_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Mail Persons Sync missing-storage successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Mail Persons Sync missing-storage successor");
    let token = StartedMailPersonsSyncRuntimeV1 {
        registration_id: predecessor.registration_id.clone(),
        runtime_instance_id: reservation.runtime_instance_id().to_owned(),
        runtime_generation: reservation.runtime_generation(),
        grant_epoch: reservation.grant_epoch(),
        capability_ids: predecessor.capability_ids.clone(),
    };
    let error = match try_start_reserved_mail_persons_sync_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        AdmittedMailPersonsSyncRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
        MailPersonsSyncBootstrapOverrideV1::MissingStorage,
        false,
    ) {
        Err(error) => error,
        Ok(_) => panic!("missing storage must be rejected before launch"),
    };
    assert_eq!(error, "managed workflow runtime configuration is invalid");
    token
}

pub(super) fn reject_mail_persons_sync_extra_capability_successor_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedMailPersonsSyncRuntimeV1,
) -> StartedMailPersonsSyncRuntimeV1 {
    let binding = store
        .platform_storage_binding(
            &predecessor.registration_id,
            MAIL_PERSONS_SYNC_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read predecessor Mail Persons Sync binding")
        .expect("predecessor Mail Persons Sync binding");
    let issue = storage_successor::issue_after(&binding)
        .expect("derive Mail Persons Sync extra-capability successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        MAIL_PERSONS_SYNC_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Mail Persons Sync extra-capability successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Mail Persons Sync extra-capability successor");
    let token = StartedMailPersonsSyncRuntimeV1 {
        registration_id: predecessor.registration_id.clone(),
        runtime_instance_id: reservation.runtime_instance_id().to_owned(),
        runtime_generation: reservation.runtime_generation(),
        grant_epoch: reservation.grant_epoch(),
        capability_ids: predecessor.capability_ids.clone(),
    };
    assert!(
        try_start_reserved_mail_persons_sync_runtime_v1(
            supervisor,
            store,
            runtime_dir,
            reservation,
            AdmittedMailPersonsSyncRuntimeV1 {
                registration_id: predecessor.registration_id,
                capability_ids: predecessor.capability_ids,
            },
            MailPersonsSyncBootstrapOverrideV1::ExtraCapability,
            false,
        )
        .is_err(),
        "undeclared capability must be rejected before child launch"
    );
    token
}

fn start_reserved_mail_persons_sync_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedMailPersonsSyncRuntimeV1,
    bootstrap_override: MailPersonsSyncBootstrapOverrideV1,
    wait_until_ready: bool,
) -> StartedMailPersonsSyncRuntimeV1 {
    try_start_reserved_mail_persons_sync_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        admitted,
        bootstrap_override,
        wait_until_ready,
    )
    .expect("start dormant Mail Persons Sync runtime")
}

#[allow(clippy::too_many_arguments)]
fn try_start_reserved_mail_persons_sync_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedMailPersonsSyncRuntimeV1,
    bootstrap_override: MailPersonsSyncBootstrapOverrideV1,
    wait_until_ready: bool,
) -> Result<StartedMailPersonsSyncRuntimeV1, String> {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = store
        .platform_storage_binding(
            &admitted.registration_id,
            MAIL_PERSONS_SYNC_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active binding");
    let topology = crate::platform::storage::topology::current(store).expect("storage topology");
    let vault = vault_status::read_current(store, &supervisor.relay_port()).expect("Vault status");
    let mut storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("runtime storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("event topology")
        .expect("event topology exists");
    let mut event_hub_endpoint = events.nats_endpoint().to_owned();
    let mut include_storage = true;
    let mut launch_capability_ids = admitted.capability_ids.clone();
    match bootstrap_override {
        MailPersonsSyncBootstrapOverrideV1::None => {}
        MailPersonsSyncBootstrapOverrideV1::MissingStorage => include_storage = false,
        MailPersonsSyncBootstrapOverrideV1::ExtraCapability => {
            launch_capability_ids.push("mail_persons_sync.undeclared.v1".to_owned());
        }
        MailPersonsSyncBootstrapOverrideV1::StopVaultAfterConfiguration => {
            supervisor
                .stop(vault_binding::VAULT_PROCESS_ID)
                .expect("stop Vault after Mail Persons Sync configuration");
        }
        MailPersonsSyncBootstrapOverrideV1::StaleCredentialFence => {
            storage.credential_revision = storage.credential_revision.saturating_add(1);
        }
        MailPersonsSyncBootstrapOverrideV1::UnavailableStoragePort(port) => {
            storage.pgbouncer_port = u32::from(port);
        }
        MailPersonsSyncBootstrapOverrideV1::UnavailableEventEndpoint(endpoint) => {
            event_hub_endpoint = endpoint;
        }
    }
    let settings = SettingsSnapshotV1 {
        target_id: CONFIGURATION_ID_V1.to_owned(),
        revision: 1,
        values: Vec::new(),
    };
    let settings_bytes = settings.encode_to_vec();
    managed_launch::start_reserved_workflow_with_settings(
        supervisor,
        runtime_dir,
        reservation,
        ManagedWorkflowRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: MAIL_PERSONS_SYNC_LOGICAL_OWNER_ID_V1.to_owned(),
            registration_id: admitted.registration_id.clone(),
            runtime_instance_id: runtime_instance_id.clone(),
            runtime_generation,
            grant_epoch,
            storage: include_storage.then_some(storage),
            event_hub_endpoint,
            event_credential_revision: events.credential_revision(),
            runtime_artifacts: Vec::new(),
            configuration_instance_id: CONFIGURATION_ID_V1.to_owned(),
            settings_revision: 1,
            configuration_instances: vec![ManagedWorkflowConfigurationInstanceV1 {
                configuration_instance_id: CONFIGURATION_ID_V1.to_owned(),
                settings_snapshot_bytes: settings_bytes.clone(),
            }],
        },
        settings_bytes,
        &launch_capability_ids,
    )?;
    if wait_until_ready {
        supervisor
            .wait_until_ready(&admitted.registration_id)
            .map_err(|error| {
                format!(
                    "Mail Persons Sync readiness: {error}; last_failure={:?}",
                    supervisor.last_failure(&admitted.registration_id)
                )
            })?;
    }
    Ok(StartedMailPersonsSyncRuntimeV1 {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids: admitted.capability_ids,
    })
}
