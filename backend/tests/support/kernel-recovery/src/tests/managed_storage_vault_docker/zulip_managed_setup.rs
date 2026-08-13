//! Exact admission, storage, Vault and release binding for managed Zulip conformance.

use super::*;

use makosh_kernel_control_store::SettingsInitialSnapshot;
use makosh_vault_key_provider::WrappingKeyProvider;
use makosh_vault_key_provider_file::FileWrappingKeyProvider;
use makosh_vault_protocol::SecretClassV1;
use makosh_vault_store_sqlcipher::{SecretRecordId, SecretRecordScope, VaultStore};
use makosh_zulip_api::client_contract::{ZULIP_MODULE_ID, ZULIP_OWNER_ID, ZulipClientContractV1};
use makosh_zulip_api::{
    ZulipClientRequestV1, ZulipClientResponseV1,
    account::{
        ZulipAccountLifecycleCommandV1, ZulipAccountLifecycleReceiptV1,
        ZulipCredentialBindingStateV1,
    },
};
use makosh_zulip_assembly::{
    ZULIP_STORAGE_BUNDLE_REVISION_V7, zulip_storage_bundle_with_owner_rls_v7,
};
use makosh_zulip_core::credential_lease_purpose;
use makosh_zulip_delivery_intent_contract::ZULIP_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1;
use makosh_zulip_runtime::client_port::{decode_module_response, encode_module_request};
use makosh_zulip_runtime::{
    admission::{
        ZULIP_BLOB_CAPABILITY_ID, ZULIP_CREDENTIALS_CAPABILITY_ID, ZULIP_EVENTS_CAPABILITY_ID,
        ZULIP_STORAGE_CAPABILITY_ID, zulip_module_descriptor_v1,
    },
    settings::zulip_settings_schema_bytes_v3,
};

use crate::modules::capability::router::{
    ManagedCapabilityRouteRequest, route_managed_client_request,
};

const ZULIP_RELEASE_ARTIFACT_ID: &str = "integration.zulip";
pub(super) const ZULIP_ACCOUNT_ID: &str = "zulip-account-1";

pub(super) struct SeededZulipCredential {
    record_id: SecretRecordId,
}

pub(super) struct AdmittedZulipRuntime {
    registration_id: String,
    capability_ids: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum ZulipGrantProfileV1 {
    QueryOnly,
    CommandAndQuery,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum ZulipBootstrapOverrideV1 {
    None,
    MissingSettings,
    InvalidSettings,
    MissingStorage,
    MissingEventCapability,
    MissingBlobCapability,
    StaleStorageFence,
    StaleVaultFence,
    StaleEventFence,
}

#[derive(Clone)]
pub(super) struct StartedZulipRuntime {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    pub(super) capability_ids: Vec<String>,
}

pub(super) fn installed_communications_zulip_release(root: &Path) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(
        SignedRuntimeArtifact::new(
            ZULIP_RELEASE_ARTIFACT_ID,
            zulip_binary(),
            zulip_module_descriptor_v1("managed-zulip-live").encode_to_vec(),
        )
        .with_settings_schema(zulip_settings_schema_bytes_v3()),
    );
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed Communications and Zulip release")
}

pub(super) fn seed_zulip_vault(vault_dir: &Path) -> SeededZulipCredential {
    let key = FileWrappingKeyProvider::new(&vault_dir.join("platform-wrapping-key.bin"))
        .load_or_create()
        .expect("open Vault wrapping key");
    let store = VaultStore::open(
        &vault_dir.join("vault.db"),
        &vault_dir.join("vault.anchor"),
        &key,
    )
    .expect("open initialized Vault");
    let request = credential_lease_purpose(ZULIP_ACCOUNT_ID, ZULIP_ACCOUNT_ID)
        .expect("Zulip API key purpose");
    let scope = SecretRecordScope::new(
        ZULIP_OWNER_ID.to_owned(),
        &request,
        SecretClassV1::ProviderCredential,
        1,
    )
    .expect("Zulip API key scope");
    let record_id = store
        .store_secret(&scope, b"managed-zulip-api-key")
        .expect("store Zulip test credential");
    SeededZulipCredential { record_id }
}

pub(super) fn rotate_zulip_vault(
    vault_dir: &Path,
    seeded: &SeededZulipCredential,
) -> SeededZulipCredential {
    let key = FileWrappingKeyProvider::new(&vault_dir.join("platform-wrapping-key.bin"))
        .load_or_create()
        .expect("open Vault wrapping key");
    let store = VaultStore::open(
        &vault_dir.join("vault.db"),
        &vault_dir.join("vault.anchor"),
        &key,
    )
    .expect("open initialized Vault");
    let request = credential_lease_purpose(ZULIP_ACCOUNT_ID, ZULIP_ACCOUNT_ID)
        .expect("Zulip API key purpose");
    let prior_scope = SecretRecordScope::new(
        ZULIP_OWNER_ID.to_owned(),
        &request,
        SecretClassV1::ProviderCredential,
        1,
    )
    .expect("prior Zulip API key scope");
    let next_scope = SecretRecordScope::new(
        ZULIP_OWNER_ID.to_owned(),
        &request,
        SecretClassV1::ProviderCredential,
        2,
    )
    .expect("next Zulip API key scope");
    let record_id = store
        .replace_secret(
            &seeded.record_id,
            &prior_scope,
            &next_scope,
            b"managed-zulip-api-key-v2",
        )
        .expect("rotate Zulip test credential");
    SeededZulipCredential { record_id }
}

pub(super) fn admit_zulip_runtime(
    store: &SqliteControlStore,
    grant_profile: ZulipGrantProfileV1,
) -> AdmittedZulipRuntime {
    let descriptor = zulip_module_descriptor_v1("managed-zulip-live");
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Zulip descriptor");
    let capability_ids = granted_capability_ids(grant_profile);
    crate::modules::registration::registry::approve_after_owner_authorization(
        store,
        registration.registration_id(),
        &capability_ids,
    )
    .expect("approve exact Zulip query capabilities");
    let schema = zulip_settings_schema_bytes_v3();
    crate::modules::settings::schema::admit(
        store,
        registration.registration_id(),
        &descriptor_bytes,
        &schema,
    )
    .expect("admit exact Zulip Settings schema");
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            ZULIP_RELEASE_ARTIFACT_ID,
            Sha256::digest(std::fs::read(zulip_binary()).expect("Zulip runtime binary bytes"))
                .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&schema).into()),
        ))
        .expect("record Zulip release binding");
    let bundle = zulip_storage_bundle_with_owner_rls_v7().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                ZULIP_OWNER_ID,
                u64::from(ZULIP_STORAGE_BUNDLE_REVISION_V7),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("record Zulip Storage bundle"),
        )
        .expect("persist Zulip Storage bundle");
    AdmittedZulipRuntime {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

fn granted_capability_ids(grant_profile: ZulipGrantProfileV1) -> Vec<String> {
    let mut capability_ids = vec![
        ZulipClientContractV1::AccountLifecycle
            .capability_id()
            .to_owned(),
        ZULIP_BLOB_CAPABILITY_ID.to_owned(),
    ];
    if matches!(grant_profile, ZulipGrantProfileV1::CommandAndQuery) {
        capability_ids.extend([
            ZulipClientContractV1::Command.capability_id().to_owned(),
            ZulipClientContractV1::OperationalQuery
                .capability_id()
                .to_owned(),
            ZulipClientContractV1::OperationalRealtime
                .capability_id()
                .to_owned(),
        ]);
    }
    capability_ids.extend([
        ZULIP_CREDENTIALS_CAPABILITY_ID.to_owned(),
        ZULIP_DELIVERY_INTENT_TARGET_CAPABILITY_ID_V1.to_owned(),
        ZULIP_EVENTS_CAPABILITY_ID.to_owned(),
        ZulipClientContractV1::Query.capability_id().to_owned(),
        ZULIP_STORAGE_CAPABILITY_ID.to_owned(),
    ]);
    capability_ids.sort();
    capability_ids
}

pub(super) fn bind_zulip_credential(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    runtime: &StartedZulipRuntime,
    expected_binding_revision: u64,
    credential_revision: u64,
) -> ZulipAccountLifecycleReceiptV1 {
    let request =
        ZulipClientRequestV1::AccountLifecycle(ZulipAccountLifecycleCommandV1::BindCredential {
            account_id: ZULIP_ACCOUNT_ID.to_owned(),
            expected_binding_revision,
            credential_revision,
        });
    let encoded = encode_module_request(41, &request).expect("encode Zulip credential binding");
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    let response = loop {
        let route = ManagedCapabilityRouteRequest::new(
            &runtime.registration_id,
            &runtime.runtime_instance_id,
            runtime.runtime_generation,
            runtime.grant_epoch,
            ZulipClientContractV1::AccountLifecycle.capability_id(),
            &encoded,
        );
        let response = match route_managed_client_request(store, &supervisor.relay_port(), &route) {
            Ok(response) => response,
            Err(error) => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "Zulip credential binding route remained unavailable: {error}; active={:?}; failure={:?}",
                    supervisor.is_active(&runtime.registration_id),
                    supervisor.last_failure(&runtime.registration_id),
                );
                std::thread::sleep(Duration::from_millis(25));
                continue;
            }
        };
        let envelope =
            makosh_runtime_protocol::v1::ModuleClientResponseV1::decode(response.as_slice())
                .expect("decode Zulip credential binding envelope");
        if envelope.error_code.is_empty() {
            break response;
        }
        assert!(
            envelope.error_code == "RUNTIME_UNAVAILABLE" && std::time::Instant::now() < deadline,
            "Zulip credential binding failed: {}",
            envelope.error_code,
        );
        std::thread::sleep(Duration::from_millis(25));
    };
    let (request_id, response) =
        decode_module_response(ZulipClientContractV1::AccountLifecycle, &response)
            .expect("decode Zulip credential binding");
    assert_eq!(request_id, 41);
    let ZulipClientResponseV1::AccountLifecycle(receipt) = response else {
        panic!("Zulip account lifecycle returned the wrong response")
    };
    assert_eq!(receipt.state, ZulipCredentialBindingStateV1::PendingRestart);
    receipt
}

pub(super) fn prepare_zulip_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedZulipRuntime,
) -> AdmittedZulipRuntime {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Zulip managed launch");
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let bundle = store
        .platform_storage_bundle(ZULIP_OWNER_ID, u64::from(ZULIP_STORAGE_BUNDLE_REVISION_V7))
        .expect("read Zulip Storage bundle")
        .expect("Zulip Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        &runtime_instance_id,
        runtime_generation,
        ZULIP_STORAGE_CAPABILITY_ID,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(ZULIP_STORAGE_BUNDLE_REVISION_V7),
            *bundle.digest(),
        )
        .expect("Zulip Storage binding issue"),
    )
    .expect("issue Zulip Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Zulip Storage binding");
    admitted
}

pub(super) fn start_zulip_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedZulipRuntime,
    realm_url: &str,
    test_stdio_capture_directory: Option<&Path>,
) -> StartedZulipRuntime {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Zulip managed launch reservation");
    launch_reserved_zulip_runtime_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        reservation,
        admitted,
        realm_url,
        ZulipBootstrapOverrideV1::None,
        test_stdio_capture_directory,
    )
}

#[allow(clippy::too_many_arguments)]
fn launch_reserved_zulip_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedZulipRuntime,
    realm_url: &str,
    bootstrap_override: ZulipBootstrapOverrideV1,
    test_stdio_capture_directory: Option<&Path>,
) -> StartedZulipRuntime {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = store
        .platform_storage_binding(&admitted.registration_id, ZULIP_STORAGE_CAPABILITY_ID)
        .expect("read Zulip Storage binding")
        .expect("Zulip Storage binding");
    let topology =
        crate::platform::storage::topology::current(store).expect("read Storage topology");
    let vault = vault_status::read_current(store, &supervisor.relay_port())
        .expect("read live Vault status");
    let mut storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("build Zulip Storage configuration");
    let include_storage = match bootstrap_override {
        ZulipBootstrapOverrideV1::MissingStorage => false,
        ZulipBootstrapOverrideV1::StaleStorageFence => {
            storage.credential_revision = storage.credential_revision.saturating_add(1);
            true
        }
        ZulipBootstrapOverrideV1::StaleVaultFence => {
            storage.vault_runtime_generation = storage.vault_runtime_generation.saturating_add(1);
            true
        }
        _ => true,
    };
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let event_credential_revision = if matches!(
        bootstrap_override,
        ZulipBootstrapOverrideV1::StaleEventFence
    ) {
        events.credential_revision().saturating_add(1)
    } else {
        events.credential_revision()
    };
    let configuration = makosh_runtime_protocol::v1::ManagedIntegrationRuntimeConfigurationV1 {
        major: 1,
        logical_owner_id: ZULIP_OWNER_ID.to_owned(),
        registration_id: admitted.registration_id.clone(),
        runtime_instance_id: runtime_instance_id.clone(),
        runtime_generation,
        grant_epoch,
        storage: include_storage.then_some(storage),
        event_hub_endpoint: events.nats_endpoint().to_owned(),
        event_credential_revision,
        configuration_instance_id: ZULIP_ACCOUNT_ID.to_owned(),
        runtime_artifacts: Vec::new(),
        integration_state_root: None,
        configuration_instances: Vec::new(),
        logical_human_owner_id: "owner-1".to_owned(),
    };
    let mut settings_snapshot_bytes =
        current_zulip_settings_snapshot(store, &admitted.registration_id, realm_url);
    if matches!(
        bootstrap_override,
        ZulipBootstrapOverrideV1::InvalidSettings
    ) {
        let mut settings_snapshot = makosh_runtime_protocol::v1::SettingsSnapshotV1::decode(
            settings_snapshot_bytes.as_slice(),
        )
        .expect("decode current Zulip Settings snapshot");
        settings_snapshot.values[0]
            .value
            .as_mut()
            .expect("Zulip email setting")
            .value =
            Some(makosh_runtime_protocol::v1::setting_value_v1::Value::StringValue(" ".to_owned()));
        settings_snapshot_bytes = settings_snapshot.encode_to_vec();
    }
    let mut granted_capability_ids = admitted.capability_ids.clone();
    if matches!(
        bootstrap_override,
        ZulipBootstrapOverrideV1::MissingEventCapability
    ) {
        granted_capability_ids.retain(|capability| capability != ZULIP_EVENTS_CAPABILITY_ID);
    }
    if matches!(
        bootstrap_override,
        ZulipBootstrapOverrideV1::MissingBlobCapability
    ) {
        granted_capability_ids.retain(|capability| capability != ZULIP_BLOB_CAPABILITY_ID);
    }
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
                ZulipBootstrapOverrideV1::MissingSettings
            ) {
                Vec::new()
            } else {
                settings_snapshot_bytes
            },
            granted_capability_ids: &granted_capability_ids,
        },
    );
    if let Some(directory) = test_stdio_capture_directory {
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while std::fs::read_dir(directory)
            .map(|entries| entries.flatten().count())
            .unwrap_or(0)
            < 2
            && std::time::Instant::now() < deadline
        {
            std::thread::sleep(Duration::from_millis(10));
        }
        unsafe {
            std::env::remove_var(
                crate::runtime::managed::execution::MANAGED_CHILD_TEST_STDIO_CAPTURE_DIRECTORY_ENV,
            );
        }
    }
    if matches!(
        bootstrap_override,
        ZulipBootstrapOverrideV1::MissingSettings
            | ZulipBootstrapOverrideV1::MissingStorage
            | ZulipBootstrapOverrideV1::MissingEventCapability
            | ZulipBootstrapOverrideV1::MissingBlobCapability
    ) {
        assert!(
            started.is_err(),
            "Kernel must deny incomplete Zulip bootstrap: {bootstrap_override:?}"
        );
    } else {
        started.expect("start managed Zulip integration");
    }
    StartedZulipRuntime {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids: admitted.capability_ids,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn launch_zulip_successor_without_ready_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    predecessor: &StartedZulipRuntime,
    realm_url: &str,
    bootstrap_override: ZulipBootstrapOverrideV1,
    test_stdio_capture_directory: &Path,
) -> StartedZulipRuntime {
    let mut approved_capability_ids = predecessor.capability_ids.clone();
    if matches!(
        bootstrap_override,
        ZulipBootstrapOverrideV1::MissingEventCapability
    ) {
        approved_capability_ids.retain(|capability| capability != ZULIP_EVENTS_CAPABILITY_ID);
    }
    if matches!(
        bootstrap_override,
        ZulipBootstrapOverrideV1::MissingBlobCapability
    ) {
        approved_capability_ids.retain(|capability| capability != ZULIP_BLOB_CAPABILITY_ID);
    }
    crate::modules::registration::registry::transition_after_owner_authorization(
        store,
        &predecessor.registration_id,
        makosh_kernel_control_store::ModuleRegistrationState::Suspended,
    )
    .expect("suspend Zulip grants before successor");
    crate::modules::registration::registry::approve_after_owner_authorization(
        store,
        &predecessor.registration_id,
        &approved_capability_ids,
    )
    .expect("approve exact Zulip successor grants");
    let predecessor_binding = store
        .platform_storage_binding(&predecessor.registration_id, ZULIP_STORAGE_CAPABILITY_ID)
        .expect("read predecessor Zulip Storage binding")
        .expect("predecessor Zulip Storage binding");
    let issue = storage_successor::issue_after(&predecessor_binding)
        .expect("derive Zulip successor Storage fences");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        ZULIP_STORAGE_CAPABILITY_ID,
        issue,
    )
    .expect("reserve successor Zulip launch and Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision successor Zulip Storage binding");
    launch_reserved_zulip_runtime_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        reservation,
        AdmittedZulipRuntime {
            registration_id: predecessor.registration_id.clone(),
            capability_ids: predecessor.capability_ids.clone(),
        },
        realm_url,
        bootstrap_override,
        Some(test_stdio_capture_directory),
    )
}

pub(super) fn restart_zulip_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    predecessor: &StartedZulipRuntime,
    realm_url: &str,
    test_stdio_capture_directory: Option<&Path>,
) -> StartedZulipRuntime {
    let predecessor_generation = predecessor.runtime_generation;
    let predecessor_binding = store
        .platform_storage_binding(&predecessor.registration_id, ZULIP_STORAGE_CAPABILITY_ID)
        .expect("read predecessor Zulip Storage binding")
        .expect("predecessor Zulip Storage binding");
    let issue = storage_successor::issue_after(&predecessor_binding)
        .expect("derive Zulip successor storage fences");
    let (_, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        ZULIP_STORAGE_CAPABILITY_ID,
        issue,
    )
    .expect("reserve successor Zulip launch and Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision successor Zulip Storage binding");
    let successor = start_zulip_runtime(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        AdmittedZulipRuntime {
            registration_id: predecessor.registration_id.clone(),
            capability_ids: predecessor.capability_ids.clone(),
        },
        realm_url,
        test_stdio_capture_directory,
    );
    assert_eq!(
        successor.runtime_generation,
        predecessor_generation + 1,
        "Zulip restart must use the next managed runtime generation",
    );
    successor
}

pub(super) fn zulip_settings_snapshot(
    configuration_instance_id: &str,
    revision: u64,
    realm_url: &str,
) -> makosh_runtime_protocol::v1::SettingsSnapshotV1 {
    use makosh_runtime_protocol::v1::{
        SettingValueV1, SettingsValueEntryV1, setting_value_v1::Value,
    };

    fn entry(setting_id: &str, value: Value) -> SettingsValueEntryV1 {
        SettingsValueEntryV1 {
            setting_id: setting_id.to_owned(),
            value: Some(SettingValueV1 { value: Some(value) }),
        }
    }

    makosh_runtime_protocol::v1::SettingsSnapshotV1 {
        target_id: configuration_instance_id.to_owned(),
        revision,
        values: vec![
            entry(
                "zulip.account_email",
                Value::StringValue("managed-account@example.test".to_owned()),
            ),
            entry(
                "zulip.account_id",
                Value::StringValue(ZULIP_ACCOUNT_ID.to_owned()),
            ),
            entry("zulip.realm_url", Value::StringValue(realm_url.to_owned())),
        ],
    }
}

fn current_zulip_settings_snapshot(
    store: &SqliteControlStore,
    registration_id: &str,
    realm_url: &str,
) -> Vec<u8> {
    let target = store
        .settings_configuration_target(registration_id, ZULIP_ACCOUNT_ID)
        .expect("read Zulip Settings target");
    if target.is_none() {
        let snapshot = zulip_settings_snapshot(ZULIP_ACCOUNT_ID, 1, realm_url).encode_to_vec();
        store
            .materialize_initial_settings_snapshot(&SettingsInitialSnapshot {
                registration_id: registration_id.to_owned(),
                configuration_instance_id: ZULIP_ACCOUNT_ID.to_owned(),
                created_operation_id: Some([0x7a; 16]),
                snapshot_bytes: snapshot.clone(),
                complete: true,
            })
            .expect("materialize initial Zulip Settings target");
        for acknowledgement in [
            crate::modules::settings::application::ApplyAcknowledgement::ValidationAccepted,
            crate::modules::settings::application::ApplyAcknowledgement::ApplyStarted,
            crate::modules::settings::application::ApplyAcknowledgement::RuntimeApplied,
        ] {
            crate::modules::settings::application::acknowledge_target(
                store,
                registration_id,
                ZULIP_ACCOUNT_ID,
                1,
                acknowledgement,
            )
            .expect("admit initial Zulip Settings state");
        }
        return snapshot;
    }
    let target = target.expect("existing Zulip Settings target");
    let (revision, snapshot) = store
        .desired_settings_snapshot_for_target(registration_id, ZULIP_ACCOUNT_ID)
        .expect("read desired Zulip Settings target")
        .expect("desired Zulip Settings");
    assert_eq!(revision, target.effective_revision());
    snapshot
}

fn zulip_binary() -> PathBuf {
    binary("MAKOSH_ZULIP_RUNTIME_BIN")
}
