//! Exact admission, storage, Vault and release binding for managed Zulip conformance.

use super::*;

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
use makosh_zulip_core::credential_lease_purpose;
use makosh_zulip_persistence::{ZULIP_STORAGE_BUNDLE_REVISION_V3, zulip_storage_bundle_v1};
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

#[derive(Clone, Copy)]
pub(super) enum ZulipGrantProfileV1 {
    QueryOnly,
    CommandAndQuery,
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
    let bundle = zulip_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                ZULIP_OWNER_ID,
                u64::from(ZULIP_STORAGE_BUNDLE_REVISION_V3),
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
        .platform_storage_bundle(ZULIP_OWNER_ID, u64::from(ZULIP_STORAGE_BUNDLE_REVISION_V3))
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
            u64::from(ZULIP_STORAGE_BUNDLE_REVISION_V3),
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
) -> StartedZulipRuntime {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Zulip managed launch reservation");
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
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("build Zulip Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let configuration = makosh_runtime_protocol::v1::ManagedIntegrationRuntimeConfigurationV1 {
        major: 1,
        logical_owner_id: ZULIP_OWNER_ID.to_owned(),
        registration_id: admitted.registration_id.clone(),
        runtime_instance_id: runtime_instance_id.clone(),
        runtime_generation,
        grant_epoch,
        storage: Some(storage),
        event_hub_endpoint: events.nats_endpoint().to_owned(),
        event_credential_revision: events.credential_revision(),
        configuration_instance_id: ZULIP_ACCOUNT_ID.to_owned(),
        runtime_artifacts: Vec::new(),
        integration_state_root: None,
        configuration_instances: Vec::new(),
        logical_human_owner_id: "owner-1".to_owned(),
    };
    managed_launch::start_reserved_integration(
        supervisor,
        kernel_data,
        runtime_dir,
        reservation,
        managed_launch::ManagedIntegrationLaunchConfiguration {
            runtime: configuration,
            settings_snapshot_bytes: current_zulip_settings_snapshot(
                store,
                &admitted.registration_id,
                realm_url,
            ),
            granted_capability_ids: &admitted.capability_ids,
        },
    )
    .expect("start managed Zulip integration");
    StartedZulipRuntime {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids: admitted.capability_ids,
    }
}

pub(super) fn restart_zulip_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    predecessor: &StartedZulipRuntime,
    realm_url: &str,
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
    );
    assert_eq!(
        successor.runtime_generation,
        predecessor_generation + 1,
        "Zulip restart must use the next managed runtime generation",
    );
    successor
}

pub(super) fn zulip_settings_snapshot(
    registration_id: &str,
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
        target_id: registration_id.to_owned(),
        revision,
        values: vec![
            entry(
                "zulip.account_id",
                Value::StringValue(ZULIP_ACCOUNT_ID.to_owned()),
            ),
            entry(
                "zulip.account_email",
                Value::StringValue("managed-account@example.test".to_owned()),
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
    let binding = store
        .settings_schema_binding(registration_id)
        .expect("read Zulip Settings binding")
        .expect("Zulip Settings binding");
    if binding.desired_revision() == 0 {
        let snapshot = zulip_settings_snapshot(registration_id, 1, realm_url).encode_to_vec();
        crate::modules::settings::mutation::commit_after_owner_authorization(
            store,
            registration_id,
            0,
            &snapshot,
        )
        .expect("commit initial Zulip Settings");
        for acknowledgement in [
            crate::modules::settings::application::ApplyAcknowledgement::ValidationAccepted,
            crate::modules::settings::application::ApplyAcknowledgement::ApplyStarted,
            crate::modules::settings::application::ApplyAcknowledgement::RuntimeApplied,
        ] {
            crate::modules::settings::application::acknowledge(
                store,
                registration_id,
                1,
                acknowledgement,
            )
            .expect("admit initial Zulip Settings state");
        }
        return snapshot;
    }
    let (revision, snapshot) = store
        .desired_settings_snapshot(registration_id)
        .expect("read desired Zulip Settings")
        .expect("desired Zulip Settings");
    assert_eq!(revision, binding.effective_revision());
    snapshot
}

fn zulip_binary() -> PathBuf {
    binary("MAKOSH_ZULIP_RUNTIME_BIN")
}
