use std::os::unix::fs::PermissionsExt;

use makosh_vault_key_provider::WrappingKeyProvider;
use makosh_vault_key_provider_file::FileWrappingKeyProvider;
use makosh_vault_protocol::{SecretClassV1, VaultActionV1, VaultPurposeRequestV1};
use makosh_vault_store_sqlcipher::{SecretRecordScope, VaultStore};
use tempfile::TempDir;

#[test]
fn replacement_advances_one_revision_and_removes_the_prior_record() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let store = initialize_store(&temporary);
    let purpose = credential_purpose();
    let first_scope = scope(&purpose, 1);
    let first_record = store
        .store_secret(&first_scope, b"credential-revision-one")
        .expect("first credential");
    let second_scope = scope(&purpose, 2);

    let second_record = store
        .replace_secret(
            &first_record,
            &first_scope,
            &second_scope,
            b"credential-revision-two",
        )
        .expect("atomic replacement");
    assert_eq!(
        store
            .resolve_scoped_secret(&second_record, &second_scope)
            .expect("replacement credential")
            .as_slice(),
        b"credential-revision-two"
    );
    assert!(
        store
            .resolve_scoped_secret(&first_record, &first_scope)
            .is_err()
    );
}

#[test]
fn replacement_rejects_a_non_sequential_revision_without_destroying_the_prior_record() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let store = initialize_store(&temporary);
    let purpose = credential_purpose();
    let first_scope = scope(&purpose, 1);
    let first_record = store
        .store_secret(&first_scope, b"credential-revision-one")
        .expect("first credential");

    assert!(
        store
            .replace_secret(
                &first_record,
                &first_scope,
                &scope(&purpose, 3),
                b"credential-revision-three",
            )
            .is_err()
    );
    assert_eq!(
        store
            .resolve_scoped_secret(&first_record, &first_scope)
            .expect("prior credential remains")
            .as_slice(),
        b"credential-revision-one"
    );
}

#[test]
fn one_scope_revision_has_exactly_one_active_secret_record() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let store = initialize_store(&temporary);
    let scope = scope(&credential_purpose(), 1);

    store
        .store_secret(&scope, b"credential-revision-one")
        .expect("first credential");
    assert!(store.store_secret(&scope, b"duplicate-credential").is_err());
}

#[test]
fn retirement_persists_a_tombstone_and_denies_resolution_or_recreation() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let store = initialize_store(&temporary);
    let scope = scope(&credential_purpose(), 1);
    let record_id = store
        .store_secret(&scope, b"credential-revision-one")
        .expect("active credential");

    store.retire_secret(&scope, 100).expect("retire credential");

    assert!(store.resolve_current_secret(&scope).is_err());
    assert!(store.resolve_scoped_secret(&record_id, &scope).is_err());
    assert!(
        store
            .store_secret(&scope, b"credential-revision-one-recreated")
            .is_err()
    );
    store
        .retire_secret(&scope, 101)
        .expect("idempotent explicit retire retry");
}

#[test]
fn delete_promotes_a_retired_tombstone_and_directly_deletes_an_active_revision() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let store = initialize_store(&temporary);
    let purpose = credential_purpose();
    let retired = scope(&purpose, 1);
    store
        .store_secret(&retired, b"credential-revision-one")
        .expect("retired credential");
    store.retire_secret(&retired, 100).expect("retire");
    store.delete_secret(&retired, 101).expect("delete retired");
    store
        .delete_secret(&retired, 102)
        .expect("idempotent explicit delete retry");
    assert!(
        store
            .store_secret(&retired, b"credential-revision-one-recreated")
            .is_err()
    );

    let direct = scope(&purpose, 2);
    store
        .store_secret(&direct, b"credential-revision-two")
        .expect("active credential");
    store.delete_secret(&direct, 103).expect("direct delete");
    assert!(store.resolve_current_secret(&direct).is_err());
    assert!(
        store
            .store_secret(&direct, b"credential-revision-two-recreated")
            .is_err()
    );
}

#[test]
fn retirement_tombstone_survives_store_restart() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let provider = FileWrappingKeyProvider::new(&temporary.path().join("wrapping-key.bin"));
    let key = provider.load_or_create().expect("file wrapping key");
    let database = temporary.path().join("vault.db");
    let anchor = temporary.path().join("vault.anchor");
    let scope = scope(&credential_purpose(), 1);
    let store =
        VaultStore::initialize(&database, &anchor, "vault-instance", &key).expect("Vault store");
    store
        .store_secret(&scope, b"credential-revision-one")
        .expect("active credential");
    store.retire_secret(&scope, 100).expect("retire credential");
    drop(store);

    let reopened = VaultStore::open(&database, &anchor, &key).expect("reopened Vault store");
    assert!(reopened.resolve_current_secret(&scope).is_err());
    assert!(
        reopened
            .store_secret(&scope, b"credential-revision-one-recreated")
            .is_err()
    );
    reopened
        .delete_secret(&scope, 101)
        .expect("delete retired credential after restart");
}

fn initialize_store(temporary: &TempDir) -> VaultStore {
    let provider = FileWrappingKeyProvider::new(&temporary.path().join("wrapping-key.bin"));
    let key = provider.load_or_create().expect("file wrapping key");
    VaultStore::initialize(
        &temporary.path().join("vault.db"),
        &temporary.path().join("vault.anchor"),
        "vault-instance",
        &key,
    )
    .expect("Vault store")
}

fn credential_purpose() -> VaultPurposeRequestV1 {
    VaultPurposeRequestV1::new(
        "mail.credential".to_owned(),
        "account-a".to_owned(),
        vec![SecretClassV1::ProviderCredential],
        vec![VaultActionV1::Resolve, VaultActionV1::Create],
        60,
    )
    .expect("typed purpose")
}

fn scope(purpose: &VaultPurposeRequestV1, revision: u64) -> SecretRecordScope {
    SecretRecordScope::new(
        "mail".to_owned(),
        purpose,
        SecretClassV1::ProviderCredential,
        revision,
    )
    .expect("record scope")
}
