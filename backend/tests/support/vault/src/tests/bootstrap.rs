use std::os::unix::fs::PermissionsExt;

use makosh_vault_key_provider::WrappingKeyProvider;
use makosh_vault_key_provider_file::FileWrappingKeyProvider;
use makosh_vault_protocol::{SecretClassV1, VaultActionV1, VaultPurposeRequestV1};
use makosh_vault_store_sqlcipher::{SecretRecordScope, VaultStore};
use tempfile::TempDir;
use zeroize::Zeroizing;

use crate::bootstrap;

#[test]
fn file_platform_credential_source_imports_only_exact_storage_scopes() {
    let temporary = private_temporary_directory();
    let source = private_directory(temporary.path().join("credentials"));
    write_private(
        &source.join("pgbouncer-admin-password"),
        b"pgbouncer-secret\n",
    );
    write_private(
        &source.join("postgres-admin-password"),
        b"postgres-secret\n",
    );
    let store = initialize_store(temporary.path());

    bootstrap::import_platform_credentials(&store, Some(&source)).expect("import credentials");

    assert_eq!(
        store
            .resolve_current_secret(&platform_scope("storage.control.pgbouncer.admin"))
            .expect("PgBouncer credential")
            .as_slice(),
        b"pgbouncer-secret"
    );
    assert_eq!(
        store
            .resolve_current_secret(&platform_scope("storage.control.postgres.admin"))
            .expect("PostgreSQL credential")
            .as_slice(),
        b"postgres-secret"
    );
}

#[test]
fn file_platform_credential_source_imports_the_optional_event_hub_scope() {
    let temporary = private_temporary_directory();
    let source = private_directory(temporary.path().join("credentials"));
    write_private(
        &source.join("pgbouncer-admin-password"),
        b"pgbouncer-secret\n",
    );
    write_private(
        &source.join("postgres-admin-password"),
        b"postgres-secret\n",
    );
    write_private(
        &source.join("nats-event-hub-password"),
        b"event-hub-secret\n",
    );
    let store = initialize_store(temporary.path());

    bootstrap::import_platform_credentials(&store, Some(&source)).expect("import credentials");

    assert_eq!(
        store
            .resolve_current_secret(&event_hub_scope())
            .expect("Event Hub credential")
            .as_slice(),
        b"event-hub-secret"
    );
}

#[test]
fn file_platform_credential_source_imports_the_optional_account_signer_scope() {
    let temporary = private_temporary_directory();
    let source = private_directory(temporary.path().join("credentials"));
    write_private(
        &source.join("pgbouncer-admin-password"),
        b"pgbouncer-secret\n",
    );
    write_private(
        &source.join("postgres-admin-password"),
        b"postgres-secret\n",
    );
    write_private(
        &source.join("nats-account-signer-seed"),
        b"account-signer-seed\n",
    );
    let store = initialize_store(temporary.path());

    bootstrap::import_platform_credentials(&store, Some(&source)).expect("import credentials");

    assert_eq!(
        store
            .resolve_current_secret(&event_account_signer_scope())
            .expect("account signer credential")
            .as_slice(),
        b"account-signer-seed"
    );
}

#[test]
fn file_platform_credential_source_rejects_a_symlinked_secret() {
    let temporary = private_temporary_directory();
    let source = private_directory(temporary.path().join("credentials"));
    let target = temporary.path().join("target");
    write_private(&target, b"secret");
    std::os::unix::fs::symlink(&target, source.join("pgbouncer-admin-password"))
        .expect("credential symlink");
    write_private(&source.join("postgres-admin-password"), b"postgres-secret");
    let store = initialize_store(temporary.path());

    assert!(bootstrap::import_platform_credentials(&store, Some(&source)).is_err());
}

#[test]
fn platform_credential_import_is_atomic_when_a_scope_conflicts() {
    let temporary = private_temporary_directory();
    let store = initialize_store(temporary.path());
    let scope = platform_scope("storage.control.pgbouncer.admin");

    assert!(
        store
            .store_secrets_atomically(vec![
                (scope.clone(), Zeroizing::new(b"first-secret".to_vec())),
                (scope.clone(), Zeroizing::new(b"second-secret".to_vec())),
            ])
            .is_err()
    );
    assert!(store.resolve_current_secret(&scope).is_err());
}

fn private_temporary_directory() -> TempDir {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("temporary directory mode");
    temporary
}

fn private_directory(path: std::path::PathBuf) -> std::path::PathBuf {
    std::fs::create_dir(&path).expect("credential source directory");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
        .expect("credential source directory mode");
    path
}

fn write_private(path: &std::path::Path, value: &[u8]) {
    std::fs::write(path, value).expect("credential source value");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .expect("credential source mode");
}

fn initialize_store(root: &std::path::Path) -> VaultStore {
    let provider = FileWrappingKeyProvider::new(&root.join("platform-wrapping-key.bin"));
    let key = provider.load_or_create().expect("wrapping key");
    VaultStore::initialize(
        &root.join("vault.db"),
        &root.join("vault.anchor"),
        "vault-main",
        &key,
    )
    .expect("Vault store")
}

fn platform_scope(purpose_id: &str) -> SecretRecordScope {
    let purpose = VaultPurposeRequestV1::new(
        purpose_id.to_owned(),
        "storage_main".to_owned(),
        vec![SecretClassV1::PlatformCredential],
        vec![VaultActionV1::Resolve],
        600,
    )
    .expect("platform purpose");
    SecretRecordScope::new(
        "storage".to_owned(),
        &purpose,
        SecretClassV1::PlatformCredential,
        1,
    )
    .expect("platform scope")
}

fn event_hub_scope() -> SecretRecordScope {
    let purpose = VaultPurposeRequestV1::new(
        "events.nats.event_hub.credential".to_owned(),
        "event_hub_main".to_owned(),
        vec![SecretClassV1::PlatformCredential],
        vec![VaultActionV1::Resolve],
        600,
    )
    .expect("Event Hub purpose");
    SecretRecordScope::new(
        "kernel".to_owned(),
        &purpose,
        SecretClassV1::PlatformCredential,
        1,
    )
    .expect("Event Hub scope")
}

fn event_account_signer_scope() -> SecretRecordScope {
    let purpose = VaultPurposeRequestV1::new(
        "events.nats.account_signer".to_owned(),
        "events_authority_runtime".to_owned(),
        vec![SecretClassV1::PlatformCredential],
        vec![VaultActionV1::Resolve],
        600,
    )
    .expect("account signer purpose");
    SecretRecordScope::new(
        "events".to_owned(),
        &purpose,
        SecretClassV1::PlatformCredential,
        1,
    )
    .expect("account signer scope")
}
