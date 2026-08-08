use makosh_backend_testkit::context::TestContext;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use tempfile::tempdir;

use makosh_hub_backend::platform::secrets::database_vault::DatabaseEncryptedSecretVault;
use makosh_hub_backend::platform::secrets::errors::SecretResolutionError;
use makosh_hub_backend::platform::secrets::file_vault::EncryptedSecretVault;
use makosh_hub_backend::platform::secrets::models::{
    NewSecretReference, ResolvedSecret, SecretKind, SecretReference, SecretStoreKind,
};
use makosh_hub_backend::platform::secrets::resolver::SecretResolver;
use makosh_hub_backend::platform::secrets::store::SecretReferenceStore;
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::vault::HostVault;
use makosh_hub_backend::vault::models::{
    EntropyEvent, HostVaultConfig, SecretEntryContext, VaultMode,
};

#[tokio::test]
async fn encrypted_vault_persists_secrets_without_plaintext_leakage() {
    let directory = tempdir().expect("tempdir");
    let vault_path = directory.path().join("makosh-secrets.vault.json");
    let vault = EncryptedSecretVault::new(
        &vault_path,
        ResolvedSecret::new("correct horse battery staple").expect("vault key"),
    );

    vault
        .store_secret("secret:test:oauth", "gmail-refresh-token")
        .expect("store secret");

    let file_contents = fs::read_to_string(&vault_path).expect("vault file");
    assert!(!file_contents.contains("gmail-refresh-token"));

    let resolved = vault
        .resolve(&secret_reference("secret:test:oauth"))
        .await
        .expect("resolve secret");
    assert_eq!(resolved.expose_for_runtime(), "gmail-refresh-token");
    assert_eq!(
        format!("{resolved:?}"),
        "ResolvedSecret { value: \"<redacted>\" }"
    );
}

#[tokio::test]
async fn encrypted_vault_rejects_wrong_master_key() {
    let directory = tempdir().expect("tempdir");
    let vault_path = directory.path().join("makosh-secrets.vault.json");
    let vault = EncryptedSecretVault::new(
        &vault_path,
        ResolvedSecret::new("correct horse battery staple").expect("vault key"),
    );
    vault
        .store_secret("secret:test:password", "imap-app-password")
        .expect("store secret");

    let wrong_key_vault = EncryptedSecretVault::new(
        &vault_path,
        ResolvedSecret::new("wrong master key").expect("wrong vault key"),
    );
    let error = wrong_key_vault
        .resolve(&secret_reference("secret:test:password"))
        .await
        .expect_err("wrong master key must fail");

    assert!(matches!(error, SecretResolutionError::StoreFailure { .. }));
}

#[tokio::test]
async fn database_encrypted_vault_persists_ciphertext_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let secret_store = SecretReferenceStore::new(pool.clone());
    let vault = DatabaseEncryptedSecretVault::new(
        pool.clone(),
        ResolvedSecret::new("database vault key").expect("vault key"),
    );
    let secret_ref = format!("secret:test:database-vault:{}", unique_suffix());

    let reference = secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &secret_ref,
            SecretKind::Password,
            SecretStoreKind::DatabaseEncryptedVault,
            "Database encrypted test secret",
        ))
        .await
        .expect("store database vault secret reference");
    vault
        .store_secret(&secret_ref, "database-vault-secret")
        .await
        .expect("store database vault secret");

    let ciphertext: String = sqlx::query_scalar(
        r#"
        SELECT ciphertext
        FROM encrypted_secret_vault_entries
        WHERE secret_ref = $1
        "#,
    )
    .bind(&secret_ref)
    .fetch_one(&pool)
    .await
    .expect("load stored ciphertext");
    assert!(!ciphertext.contains("database-vault-secret"));

    let resolved = vault
        .resolve(&reference)
        .await
        .expect("resolve database vault secret");
    assert_eq!(resolved.expose_for_runtime(), "database-vault-secret");
}

#[tokio::test]
async fn database_encrypted_vault_rejects_wrong_master_key_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let secret_store = SecretReferenceStore::new(pool.clone());
    let vault = DatabaseEncryptedSecretVault::new(
        pool.clone(),
        ResolvedSecret::new("database vault key").expect("vault key"),
    );
    let wrong_key_vault = DatabaseEncryptedSecretVault::new(
        pool,
        ResolvedSecret::new("wrong database vault key").expect("wrong vault key"),
    );
    let secret_ref = format!("secret:test:database-vault-wrong-key:{}", unique_suffix());

    let reference = secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &secret_ref,
            SecretKind::Password,
            SecretStoreKind::DatabaseEncryptedVault,
            "Database encrypted wrong key test secret",
        ))
        .await
        .expect("store database vault secret reference");
    vault
        .store_secret(&secret_ref, "database-vault-secret")
        .await
        .expect("store database vault secret");

    let error = wrong_key_vault
        .resolve(&reference)
        .await
        .expect_err("wrong database vault key must fail");

    assert!(matches!(error, SecretResolutionError::StoreFailure { .. }));
}

#[test]
fn host_vault_requires_entropy_threshold_before_create() {
    let directory = tempdir().expect("tempdir");
    let vault = test_host_vault(directory.path());

    vault
        .collect_entropy(entropy_events(1_999))
        .expect("collect entropy");
    let error = vault.create().expect_err("insufficient entropy must fail");

    assert!(error.to_string().contains("insufficient vault entropy"));
}

#[tokio::test]
async fn host_vault_create_unlock_store_and_resolve_secret() {
    let directory = tempdir().expect("tempdir");
    let vault = test_host_vault(directory.path());
    vault
        .collect_entropy(entropy_events(2_000))
        .expect("collect entropy");
    let status = vault.create().expect("create vault");
    assert_eq!(status.state, VaultMode::Unlocked);

    let metadata = serde_json::json!({
        "provider": "imap",
        "account_id": "acct-host-vault"
    });
    vault
        .store_secret(
            "secret:provider-account:acct-host-vault:imap_password",
            "host-vault-password",
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id: "acct-host-vault",
                purpose: "imap_password",
                secret_kind: "password",
                label: "IMAP password",
                metadata: &metadata,
            },
        )
        .expect("store host vault secret");

    let database =
        fs::read_to_string(directory.path().join("vault").join("vault.db")).unwrap_or_default();
    assert!(!database.contains("host-vault-password"));

    let resolved = vault
        .resolve(&host_vault_secret_reference(
            "secret:provider-account:acct-host-vault:imap_password",
            SecretKind::Password,
        ))
        .await
        .expect("resolve host vault secret");
    assert_eq!(resolved.expose_for_runtime(), "host-vault-password");

    vault.lock().expect("lock vault");
    assert_eq!(vault.status().expect("status").state, VaultMode::Locked);
    vault.unlock().expect("unlock vault");
    assert_eq!(
        vault
            .read_secret("secret:provider-account:acct-host-vault:imap_password")
            .expect("read after unlock"),
        "host-vault-password"
    );
}

#[tokio::test]
async fn host_vault_unlock_existing_reopens_session_after_runtime_restart() {
    let directory = tempdir().expect("tempdir");
    let vault_home = directory.path().join("vault");
    let dev_key_path = directory.path().join("dev").join("master.key");
    let metadata = serde_json::json!({
        "provider": "imap",
        "account_id": "acct-host-vault-restart"
    });
    let secret_ref = "secret:provider-account:acct-host-vault-restart:imap_password";

    let vault = HostVault::new(HostVaultConfig {
        home: vault_home.clone(),
        dev_mode: true,
        dev_key_path: dev_key_path.clone(),
    })
    .expect("host vault");
    vault
        .collect_entropy(entropy_events(2_000))
        .expect("collect entropy");
    vault.create().expect("create vault");
    vault
        .store_secret(
            secret_ref,
            "restart-secret",
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id: "acct-host-vault-restart",
                purpose: "imap_password",
                secret_kind: "password",
                label: "IMAP password",
                metadata: &metadata,
            },
        )
        .expect("store host vault secret");

    let restarted = HostVault::new(HostVaultConfig {
        home: vault_home,
        dev_mode: true,
        dev_key_path,
    })
    .expect("restarted host vault");
    assert_eq!(restarted.status().expect("status").state, VaultMode::Locked);

    let status = restarted.unlock_existing().expect("unlock existing vault");
    assert_eq!(status.state, VaultMode::Unlocked);
    assert_eq!(
        restarted
            .read_secret(secret_ref)
            .expect("read restarted secret"),
        "restart-secret"
    );
}

#[test]
fn host_vault_delete_removes_secret_and_manifest() {
    let directory = tempdir().expect("tempdir");
    let vault = test_host_vault(directory.path());
    vault
        .collect_entropy(entropy_events(2_000))
        .expect("collect entropy");
    vault.create().expect("create vault");
    let metadata = serde_json::json!({
        "provider": "whatsapp_web",
        "account_id": "acct-whatsapp-delete"
    });
    let secret_ref = "secret:provider-account:acct-whatsapp-delete:whatsapp_web_session_key";
    vault
        .store_secret(
            secret_ref,
            "session-material",
            SecretEntryContext {
                entry_kind: "provider_session",
                account_id: "acct-whatsapp-delete",
                purpose: "whatsapp_web_session_key",
                secret_kind: "other",
                label: "WhatsApp session credential",
                metadata: &metadata,
            },
        )
        .expect("store host vault secret");

    assert_eq!(vault.account_secret_manifest().expect("manifest").len(), 1);
    assert!(vault.delete_secret(secret_ref).expect("delete secret"));
    assert!(!vault.delete_secret(secret_ref).expect("idempotent delete"));
    assert!(
        vault
            .account_secret_manifest()
            .expect("manifest")
            .is_empty()
    );
    let error = vault
        .read_secret(secret_ref)
        .expect_err("secret should be removed");
    assert!(
        error
            .to_string()
            .contains("secret was not found in host vault")
    );
}

#[tokio::test]
async fn host_vault_rejects_tampered_ciphertext() {
    let directory = tempdir().expect("tempdir");
    let vault = test_host_vault(directory.path());
    vault
        .collect_entropy(entropy_events(2_000))
        .expect("collect entropy");
    vault.create().expect("create vault");
    let metadata = serde_json::json!({});
    let secret_ref = "secret:provider-account:tampered:imap_password";
    vault
        .store_secret(
            secret_ref,
            "tamper-secret",
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id: "tampered",
                purpose: "imap_password",
                secret_kind: "password",
                label: "IMAP password",
                metadata: &metadata,
            },
        )
        .expect("store host vault secret");

    {
        let connection =
            rusqlite::Connection::open(directory.path().join("vault").join("vault.db"))
                .expect("open vault db");
        connection
            .execute(
                "UPDATE vault_entries SET aad = 'tampered aad' WHERE secret_ref = ?1",
                rusqlite::params![secret_ref],
            )
            .expect("tamper aad");
    }

    let error = vault
        .resolve(&host_vault_secret_reference(
            secret_ref,
            SecretKind::Password,
        ))
        .await
        .expect_err("tampered AAD must fail");
    assert!(matches!(error, SecretResolutionError::StoreFailure { .. }));
}

#[test]
fn host_vault_recovery_phrase_restores_existing_secret_access() {
    let directory = tempdir().expect("tempdir");
    let vault = test_host_vault(directory.path());
    vault
        .collect_entropy(entropy_events(2_000))
        .expect("collect entropy");
    vault.create().expect("create vault");
    let metadata = serde_json::json!({});
    let secret_ref = "secret:provider-account:recoverable:imap_password";
    vault
        .store_secret(
            secret_ref,
            "recoverable-secret",
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id: "recoverable",
                purpose: "imap_password",
                secret_kind: "password",
                label: "IMAP password",
                metadata: &metadata,
            },
        )
        .expect("store host vault secret");
    let recovery = vault.export_recovery().expect("export recovery");

    let restored = HostVault::new(HostVaultConfig {
        home: directory.path().join("vault"),
        dev_mode: true,
        dev_key_path: directory.path().join("restored-dev").join("master.key"),
    })
    .expect("restored host vault");
    restored
        .import_recovery(&recovery.recovery_phrase)
        .expect("import recovery");

    assert_eq!(
        restored
            .read_secret(secret_ref)
            .expect("read restored secret"),
        "recoverable-secret"
    );
}

fn secret_reference(secret_ref: &str) -> SecretReference {
    let now = Utc::now();

    SecretReference {
        secret_ref: secret_ref.to_owned(),
        secret_kind: SecretKind::OauthToken,
        store_kind: SecretStoreKind::EncryptedVault,
        label: "test secret".to_owned(),
        metadata: serde_json::json!({}),
        created_at: now,
        updated_at: now,
    }
}

fn host_vault_secret_reference(secret_ref: &str, secret_kind: SecretKind) -> SecretReference {
    let now = Utc::now();

    SecretReference {
        secret_ref: secret_ref.to_owned(),
        secret_kind,
        store_kind: SecretStoreKind::HostVault,
        label: "host vault secret".to_owned(),
        metadata: serde_json::json!({}),
        created_at: now,
        updated_at: now,
    }
}

fn test_host_vault(root: &std::path::Path) -> HostVault {
    HostVault::new(HostVaultConfig {
        home: root.join("vault"),
        dev_mode: true,
        dev_key_path: root.join("dev").join("master.key"),
    })
    .expect("host vault")
}

fn entropy_events(count: usize) -> Vec<EntropyEvent> {
    (0..count)
        .map(|index| EntropyEvent {
            x: (index % 977) as f64,
            y: (index % 541) as f64,
            dx: ((index % 13) as f64) - 6.0,
            dy: ((index % 17) as f64) - 8.0,
            timestamp_ms: index as f64 * 7.0,
            velocity: (index % 29) as f64 / 10.0,
            acceleration: (index % 31) as f64 / 100.0,
            interval_ms: 7.0,
        })
        .collect()
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
