//! Exact admission, storage, Vault and release assembly for managed Telegram conformance.

use super::*;

use makosh_telegram_api::{
    TelegramAccount, TelegramAccountState, TelegramCredentialBinding, TelegramCredentialPurpose,
    TelegramRuntimeState,
    client_contract::{TELEGRAM_MODULE_ID, TELEGRAM_OWNER_ID},
};
use makosh_telegram_assembly::{
    TELEGRAM_STORAGE_BUNDLE_REVISION_V10, telegram_storage_bundle_with_owner_rls_v10,
};
use makosh_telegram_core::credential_lease_purpose_for_purpose;
use makosh_telegram_persistence::{TelegramDurablePersistence, TelegramPersistenceConformanceV1};
use makosh_telegram_runtime::{
    admission::{
        TELEGRAM_STORAGE_CAPABILITY_ID, TELEGRAM_TDJSON_ARTIFACT_ID, TELEGRAM_TGCALLS_ARTIFACT_ID,
        telegram_module_descriptor_v1,
    },
    settings::telegram_settings_schema_bytes_v1,
};
use makosh_vault_key_provider::WrappingKeyProvider;
use makosh_vault_key_provider_file::FileWrappingKeyProvider;
use makosh_vault_protocol::SecretClassV1;
use makosh_vault_store_sqlcipher::{SecretRecordScope, VaultStore};
use zeroize::Zeroizing;

const TELEGRAM_RELEASE_ARTIFACT_ID: &str = "integration.telegram";
pub(super) const TELEGRAM_ACCOUNT_ID: &str = "telegram-account-1";

pub(super) struct AdmittedTelegramRuntime {
    registration_id: String,
    capability_ids: Vec<String>,
    api_id: i64,
}

#[derive(Clone)]
pub(super) struct StartedTelegramRuntime {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    pub(super) grant_epoch: u64,
    capability_ids: Vec<String>,
    api_id: i64,
}

#[derive(Clone, Copy)]
pub(super) enum TelegramBootstrapOverrideV1 {
    None,
    MissingSettings,
    InvalidSettingsValue,
    MissingStorage,
    StaleStorageFence,
    StaleVaultFence,
    StaleEventFence,
    MissingNativeArtifacts,
}

pub(super) fn installed_communications_telegram_release_with_native_v1(
    root: &Path,
    tdjson: &Path,
    tgcalls: &Path,
) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(
        SignedRuntimeArtifact::new(
            TELEGRAM_RELEASE_ARTIFACT_ID,
            telegram_binary(),
            telegram_module_descriptor_v1("managed-telegram-live").encode_to_vec(),
        )
        .with_settings_schema(telegram_settings_schema_bytes_v1()),
    );
    InstalledSignedBundle::install_with_native_dependencies(
        root,
        &artifacts,
        &[
            SignedNativeDependency::new(
                TELEGRAM_TDJSON_ARTIFACT_ID,
                tdjson.to_path_buf(),
                TELEGRAM_MODULE_ID,
            ),
            SignedNativeDependency::new(
                TELEGRAM_TGCALLS_ARTIFACT_ID,
                tgcalls.to_path_buf(),
                TELEGRAM_MODULE_ID,
            ),
        ],
    )
    .expect("install signed Communications and Telegram release")
}

pub(super) fn seed_telegram_vault_with_secrets_v1(
    vault_dir: &Path,
    api_hash: &[u8],
    session_encryption_key: &[u8],
) {
    let key = FileWrappingKeyProvider::new(&vault_dir.join("platform-wrapping-key.bin"))
        .load_or_create()
        .expect("open Vault wrapping key");
    let store = VaultStore::open(
        &vault_dir.join("vault.db"),
        &vault_dir.join("vault.anchor"),
        &key,
    )
    .expect("open initialized Vault");
    store_telegram_secret(
        &store,
        TelegramCredentialPurpose::ApiHash,
        SecretClassV1::ProviderCredential,
        api_hash,
    );
    store_telegram_secret(
        &store,
        TelegramCredentialPurpose::SessionEncryptionKey,
        SecretClassV1::SessionStoreKey,
        session_encryption_key,
    );
}

fn store_telegram_secret(
    store: &VaultStore,
    purpose: TelegramCredentialPurpose,
    secret_class: SecretClassV1,
    payload: &[u8],
) {
    let request = credential_lease_purpose_for_purpose(TELEGRAM_ACCOUNT_ID, purpose)
        .expect("Telegram credential purpose");
    let scope = SecretRecordScope::new(TELEGRAM_OWNER_ID.to_owned(), &request, secret_class, 1)
        .expect("Telegram secret scope");
    store
        .store_secret(&scope, payload)
        .expect("store Telegram test credential");
}

pub(super) fn admit_telegram_runtime_with_api_id_v1(
    store: &SqliteControlStore,
    excluded_capability_id: Option<&str>,
    api_id: i64,
) -> AdmittedTelegramRuntime {
    assert!(api_id > 0, "Telegram API ID must be positive");
    let descriptor = telegram_module_descriptor_v1("managed-telegram-live");
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Telegram descriptor");
    let capability_ids = descriptor
        .capabilities
        .iter()
        .map(|capability| capability.capability_id.clone())
        .filter(|capability_id| excluded_capability_id != Some(capability_id.as_str()))
        .collect::<Vec<_>>();
    crate::modules::registration::registry::approve_after_owner_authorization(
        store,
        registration.registration_id(),
        &capability_ids,
    )
    .expect("approve exact Telegram capabilities");
    let schema = telegram_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            TELEGRAM_RELEASE_ARTIFACT_ID,
            Sha256::digest(
                std::fs::read(telegram_binary()).expect("Telegram runtime binary bytes"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&schema).into()),
        ))
        .expect("record Telegram release binding");
    let bundle = telegram_storage_bundle_with_owner_rls_v10().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                TELEGRAM_OWNER_ID,
                u64::from(TELEGRAM_STORAGE_BUNDLE_REVISION_V10),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("record Telegram Storage bundle"),
        )
        .expect("persist Telegram Storage bundle");
    AdmittedTelegramRuntime {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
        api_id,
    }
}

pub(super) fn prepare_telegram_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedTelegramRuntime,
) -> AdmittedTelegramRuntime {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Telegram managed launch");
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let bundle = store
        .platform_storage_bundle(
            TELEGRAM_OWNER_ID,
            u64::from(TELEGRAM_STORAGE_BUNDLE_REVISION_V10),
        )
        .expect("read Telegram Storage bundle")
        .expect("Telegram Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        &runtime_instance_id,
        runtime_generation,
        TELEGRAM_STORAGE_CAPABILITY_ID,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(TELEGRAM_STORAGE_BUNDLE_REVISION_V10),
            *bundle.digest(),
        )
        .expect("Telegram Storage binding issue"),
    )
    .expect("issue Telegram Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Telegram Storage binding");
    seed_telegram_account();
    admitted
}

pub(super) fn start_telegram_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedTelegramRuntime,
) -> StartedTelegramRuntime {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Telegram managed launch reservation");
    launch_reserved_telegram_runtime_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        reservation,
        admitted,
        TelegramBootstrapOverrideV1::None,
        true,
        None,
    )
}

pub(super) fn start_telegram_runtime_with_capture_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedTelegramRuntime,
    test_stdio_capture_directory: &Path,
) -> StartedTelegramRuntime {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Telegram managed launch reservation");
    launch_reserved_telegram_runtime_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        reservation,
        admitted,
        TelegramBootstrapOverrideV1::None,
        true,
        Some(test_stdio_capture_directory),
    )
}

pub(super) fn launch_telegram_runtime_without_ready_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    admitted: AdmittedTelegramRuntime,
    bootstrap_override: TelegramBootstrapOverrideV1,
    test_stdio_capture_directory: &Path,
) -> StartedTelegramRuntime {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Telegram managed launch reservation");
    launch_reserved_telegram_runtime_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        reservation,
        admitted,
        bootstrap_override,
        false,
        Some(test_stdio_capture_directory),
    )
}

pub(super) fn launch_telegram_successor_without_ready_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    predecessor: StartedTelegramRuntime,
    bootstrap_override: TelegramBootstrapOverrideV1,
    test_stdio_capture_directory: &Path,
) -> StartedTelegramRuntime {
    let predecessor_binding = store
        .platform_storage_binding(&predecessor.registration_id, TELEGRAM_STORAGE_CAPABILITY_ID)
        .expect("read predecessor Telegram Storage binding")
        .expect("predecessor Telegram Storage binding");
    let issue = storage_successor::issue_after(&predecessor_binding)
        .expect("derive Telegram successor storage fences");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        TELEGRAM_STORAGE_CAPABILITY_ID,
        issue,
    )
    .expect("reserve successor Telegram launch and Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision successor Telegram Storage binding");
    launch_reserved_telegram_runtime_v1(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        reservation,
        AdmittedTelegramRuntime {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
            api_id: predecessor.api_id,
        },
        bootstrap_override,
        false,
        Some(test_stdio_capture_directory),
    )
}

#[allow(clippy::too_many_arguments)]
fn launch_reserved_telegram_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedTelegramRuntime,
    bootstrap_override: TelegramBootstrapOverrideV1,
    wait_until_ready: bool,
    test_stdio_capture_directory: Option<&Path>,
) -> StartedTelegramRuntime {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = store
        .platform_storage_binding(&admitted.registration_id, TELEGRAM_STORAGE_CAPABILITY_ID)
        .expect("read Telegram Storage binding")
        .expect("Telegram Storage binding");
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
    .expect("build Telegram Storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let mut settings_snapshot = telegram_settings_snapshot(admitted.api_id);
    let include_storage = match bootstrap_override {
        TelegramBootstrapOverrideV1::None
        | TelegramBootstrapOverrideV1::MissingSettings
        | TelegramBootstrapOverrideV1::InvalidSettingsValue
        | TelegramBootstrapOverrideV1::StaleEventFence
        | TelegramBootstrapOverrideV1::MissingNativeArtifacts => true,
        TelegramBootstrapOverrideV1::MissingStorage => false,
        TelegramBootstrapOverrideV1::StaleStorageFence => {
            storage.credential_revision = storage.credential_revision.saturating_add(1);
            true
        }
        TelegramBootstrapOverrideV1::StaleVaultFence => {
            storage.vault_runtime_generation = storage.vault_runtime_generation.saturating_add(1);
            true
        }
    };
    if matches!(
        bootstrap_override,
        TelegramBootstrapOverrideV1::InvalidSettingsValue
    ) {
        let api_id = settings_snapshot
            .values
            .iter_mut()
            .find(|entry| entry.setting_id == "telegram.api_id")
            .and_then(|entry| entry.value.as_mut())
            .expect("Telegram API ID setting");
        api_id.value =
            Some(makosh_runtime_protocol::v1::setting_value_v1::Value::SignedIntegerValue(0));
    }
    let event_credential_revision = if matches!(
        bootstrap_override,
        TelegramBootstrapOverrideV1::StaleEventFence
    ) {
        events.credential_revision().saturating_add(1)
    } else {
        events.credential_revision()
    };
    let configuration = makosh_runtime_protocol::v1::ManagedIntegrationRuntimeConfigurationV1 {
        major: 1,
        logical_owner_id: TELEGRAM_OWNER_ID.to_owned(),
        registration_id: admitted.registration_id.clone(),
        runtime_instance_id: runtime_instance_id.clone(),
        runtime_generation,
        grant_epoch,
        storage: include_storage.then_some(storage),
        event_hub_endpoint: events.nats_endpoint().to_owned(),
        event_credential_revision,
        configuration_instance_id: TELEGRAM_ACCOUNT_ID.to_owned(),
        runtime_artifacts: Vec::new(),
        integration_state_root: None,
        configuration_instances: Vec::new(),
        logical_human_owner_id: "owner-1".to_owned(),
    };
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
                TelegramBootstrapOverrideV1::MissingSettings
            ) {
                Vec::new()
            } else {
                settings_snapshot.encode_to_vec()
            },
            granted_capability_ids: &admitted.capability_ids,
        },
    );
    if matches!(
        bootstrap_override,
        TelegramBootstrapOverrideV1::MissingSettings
            | TelegramBootstrapOverrideV1::MissingStorage
            | TelegramBootstrapOverrideV1::MissingNativeArtifacts
    ) {
        assert!(
            started.is_err(),
            "Kernel must deny incomplete Telegram bootstrap"
        );
    } else {
        started.expect("start managed Telegram integration");
    }
    if wait_until_ready {
        supervisor
            .wait_until_ready(&admitted.registration_id)
            .unwrap_or_else(|error| {
                panic!(
                    "Telegram readiness: {error}; last_failure={:?}",
                    supervisor.last_failure(&admitted.registration_id)
                )
            });
    }
    StartedTelegramRuntime {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        grant_epoch,
        capability_ids: admitted.capability_ids,
        api_id: admitted.api_id,
    }
}

pub(super) fn restart_telegram_runtime(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    kernel_data: &Path,
    runtime_dir: &Path,
    predecessor: StartedTelegramRuntime,
) -> StartedTelegramRuntime {
    let predecessor_generation = predecessor.runtime_generation;
    let predecessor_binding = store
        .platform_storage_binding(&predecessor.registration_id, TELEGRAM_STORAGE_CAPABILITY_ID)
        .expect("read predecessor Telegram Storage binding")
        .expect("predecessor Telegram Storage binding");
    let issue = storage_successor::issue_after(&predecessor_binding)
        .expect("derive Telegram successor storage fences");
    let (_, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        TELEGRAM_STORAGE_CAPABILITY_ID,
        issue,
    )
    .expect("reserve successor Telegram launch and Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision successor Telegram Storage binding");
    let successor = start_telegram_runtime(
        supervisor,
        store,
        kernel_data,
        runtime_dir,
        AdmittedTelegramRuntime {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
            api_id: predecessor.api_id,
        },
    );
    assert_eq!(
        successor.runtime_generation,
        predecessor_generation + 1,
        "Telegram restart must use the next managed runtime generation",
    );
    successor
}

fn telegram_settings_snapshot(api_id: i64) -> makosh_runtime_protocol::v1::SettingsSnapshotV1 {
    use makosh_runtime_protocol::v1::{
        SettingValueV1, SettingsValueEntryV1, setting_value_v1::Value,
    };

    makosh_runtime_protocol::v1::SettingsSnapshotV1 {
        target_id: TELEGRAM_ACCOUNT_ID.to_owned(),
        revision: 1,
        values: vec![
            SettingsValueEntryV1 {
                setting_id: "telegram.account_id".to_owned(),
                value: Some(SettingValueV1 {
                    value: Some(Value::StringValue(TELEGRAM_ACCOUNT_ID.to_owned())),
                }),
            },
            SettingsValueEntryV1 {
                setting_id: "telegram.api_id".to_owned(),
                value: Some(SettingValueV1 {
                    value: Some(Value::SignedIntegerValue(api_id)),
                }),
            },
        ],
    }
}

fn seed_telegram_account() {
    tokio::runtime::Runtime::new()
        .expect("Telegram seed runtime")
        .block_on(async {
            let durable = telegram_admin_persistence().await;
            durable
                .initialize()
                .await
                .expect("initialize Telegram persistence");
            durable
                .upsert_account(
                    &TelegramAccount {
                        account_id: TELEGRAM_ACCOUNT_ID.to_owned(),
                        display_name: "Managed Telegram".to_owned(),
                        external_account_id: "telegram-owner-account".to_owned(),
                        state: TelegramAccountState::Ready,
                        runtime_state: TelegramRuntimeState::Stopped,
                        runtime_epoch: 1,
                    },
                    &[
                        TelegramCredentialBinding {
                            purpose: TelegramCredentialPurpose::ApiHash,
                            revision: 1,
                        },
                        TelegramCredentialBinding {
                            purpose: TelegramCredentialPurpose::SessionEncryptionKey,
                            revision: 1,
                        },
                    ],
                )
                .await
                .expect("seed Telegram account");
        });
}

pub(super) fn seed_telegram_legacy_call_frame() {
    tokio::runtime::Runtime::new()
        .expect("Telegram legacy call seed runtime")
        .block_on(async {
            let pool = telegram_admin_pool().await;
            let mut transaction = pool.begin().await.expect("begin legacy call seed");
            sqlx::query(
                "INSERT INTO makosh_data.telegram_call_sessions (\
                 call_session_id, account_id, runtime_generation, tdlib_call_id, \
                 provider_call_unique_id, provider_user_id, direction, provider_state, \
                 pending_created, pending_received, discard_reason, failure_category, revision, \
                 created_at_unix_seconds, updated_at_unix_seconds, ended_at_unix_seconds\
                 ) VALUES (\
                 'legacy-call-before-v4', $1, 1, 941, 4001, '41', 'incoming', 'discarded', \
                 FALSE, FALSE, 'missed', NULL, 1, 10, 10, 10\
                 )",
            )
            .bind(TELEGRAM_ACCOUNT_ID)
            .execute(&mut *transaction)
            .await
            .expect("seed legacy call projection");
            sqlx::query(
                "INSERT INTO makosh_data.telegram_call_state_history (\
                 call_session_id, revision, provider_state, pending_created, pending_received, \
                 discard_reason, failure_category, observed_at_unix_seconds\
                 ) VALUES (\
                 'legacy-call-before-v4', 1, 'discarded', FALSE, FALSE, 'missed', NULL, 10\
                 )",
            )
            .execute(&mut *transaction)
            .await
            .expect("seed legacy call history");
            sqlx::query(
                "INSERT INTO makosh_data.telegram_call_realtime_frames (\
                 account_id, call_session_id, call_revision, provider_state, pending_created, \
                 pending_received, discard_reason, failure_category, observed_at_unix_seconds\
                 ) VALUES (\
                 $1, 'legacy-call-before-v4', 1, 'discarded', FALSE, FALSE, 'missed', NULL, 10\
                 )",
            )
            .bind(TELEGRAM_ACCOUNT_ID)
            .execute(&mut *transaction)
            .await
            .expect("seed legacy realtime frame");
            transaction.commit().await.expect("commit legacy call seed");
        });
}

pub(super) fn telegram_calls_backfill_state() -> (String, i64, i64) {
    tokio::runtime::Runtime::new()
        .expect("Telegram Calls backfill query runtime")
        .block_on(async {
            sqlx::query_as::<_, (String, i64, i64)>(
                "SELECT execution_state, processed_frame_count, backfilled_frame_count \
                 FROM makosh_data.telegram_call_realtime_backfill_jobs",
            )
            .fetch_one(&telegram_admin_pool().await)
            .await
            .expect("read Telegram Calls backfill execution")
        })
}

async fn telegram_admin_persistence() -> TelegramDurablePersistence {
    let password = Zeroizing::new(
        std::fs::read_to_string(required(
            "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE",
        ))
        .expect("read disposable PostgreSQL credential")
        .trim()
        .to_owned(),
    );
    TelegramPersistenceConformanceV1::connect(
        &required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST"),
        required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT")
            .parse()
            .expect("valid PostgreSQL port"),
        "makosh_postgres_admin",
        password.as_str(),
        "makosh_storage_authenticated",
    )
    .await
    .expect("connect Telegram conformance persistence")
}

pub(super) async fn telegram_admin_pool() -> sqlx::PgPool {
    let password = Zeroizing::new(
        std::fs::read_to_string(required(
            "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE",
        ))
        .expect("read disposable PostgreSQL credential")
        .trim()
        .to_owned(),
    );
    let options = sqlx::postgres::PgConnectOptions::new()
        .host(&required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST"))
        .port(
            required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT")
                .parse()
                .expect("valid PostgreSQL port"),
        )
        .username("makosh_postgres_admin")
        .password(password.as_str())
        .database("makosh_storage_authenticated")
        .ssl_mode(sqlx::postgres::PgSslMode::Disable);
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("connect Telegram conformance database")
}

pub(super) fn telegram_call_media_state() -> String {
    let password = Zeroizing::new(
        std::fs::read_to_string(required(
            "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE",
        ))
        .expect("read disposable PostgreSQL credential")
        .trim()
        .to_owned(),
    );
    tokio::runtime::Runtime::new()
        .expect("Telegram media projection runtime")
        .block_on(async {
            let options = sqlx::postgres::PgConnectOptions::new()
                .host(&required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST"))
                .port(
                    required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT")
                        .parse()
                        .expect("valid PostgreSQL port"),
                )
                .username("makosh_postgres_admin")
                .password(password.as_str())
                .database("makosh_storage_authenticated")
                .ssl_mode(sqlx::postgres::PgSslMode::Disable);
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await
                .expect("connect Telegram media projection database");
            sqlx::query_scalar::<_, String>(
                "SELECT media_state FROM makosh_data.telegram_call_media_projection \
                 WHERE account_id = $1",
            )
            .bind(TELEGRAM_ACCOUNT_ID)
            .fetch_one(&pool)
            .await
            .expect("read durable Telegram media projection")
        })
}

pub(super) fn telegram_pending_call_evidence_count() -> i64 {
    tokio::runtime::Runtime::new()
        .expect("Telegram call evidence outbox runtime")
        .block_on(async {
            let pool = telegram_admin_pool().await;
            let count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM makosh_data.telegram_call_evidence_outbox \
                 WHERE published_at_unix_seconds IS NULL",
            )
            .fetch_one(&pool)
            .await
            .expect("read Telegram call evidence outbox");
            pool.close().await;
            count
        })
}

fn telegram_binary() -> PathBuf {
    binary("MAKOSH_TELEGRAM_RUNTIME_BIN")
}

pub(super) fn telegram_tdjson_fixture() -> PathBuf {
    binary("MAKOSH_TELEGRAM_TDJSON_FIXTURE")
}

pub(super) fn telegram_tgcalls_fixture() -> PathBuf {
    binary("MAKOSH_TELEGRAM_TGCALLS_FIXTURE")
}
