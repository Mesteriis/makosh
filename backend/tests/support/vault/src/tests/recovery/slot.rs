use std::os::unix::fs::PermissionsExt;

use makosh_vault_key_provider::WrappingKeyProvider;
use makosh_vault_key_provider_file::FileWrappingKeyProvider;
use makosh_vault_protocol::{SecretClassV1, VaultActionV1, VaultPurposeRequestV1};
use makosh_vault_store_sqlcipher::{SecretRecordScope, VaultRecoveryKeyV1, VaultStore};
use tempfile::TempDir;

#[test]
fn recovery_slot_reopens_the_same_vault_and_rejects_replacement_or_wrong_keys() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private Vault directory");
    let database = temporary.path().join("vault.db");
    let anchor = temporary.path().join("vault.anchor");
    let provider = FileWrappingKeyProvider::new(&temporary.path().join("wrapping-key.bin"));
    let wrapping_key = provider.load_or_create().expect("wrapping key");
    let store = VaultStore::initialize(&database, &anchor, "vault-instance", &wrapping_key)
        .expect("Vault store");
    let recovery_key = VaultRecoveryKeyV1::generate().expect("recovery key");

    VaultStore::add_recovery_slot(&anchor, &wrapping_key, &recovery_key)
        .expect("add recovery slot");
    assert!(VaultStore::add_recovery_slot(&anchor, &wrapping_key, &recovery_key).is_err());
    assert_eq!(
        VaultStore::open(&database, &anchor, &wrapping_key)
            .expect("platform slot still opens")
            .instance_id(),
        "vault-instance"
    );
    assert_eq!(
        VaultStore::open_with_recovery(&database, &anchor, &recovery_key)
            .expect("recovery slot opens")
            .instance_id(),
        "vault-instance"
    );
    let wrong_key = VaultRecoveryKeyV1::generate().expect("wrong recovery key");
    assert!(VaultStore::open_with_recovery(&database, &anchor, &wrong_key).is_err());
    drop(store);
}

#[test]
fn recovery_key_ceremony_uses_only_a_checked_english_24_word_mnemonic() {
    let temporary = private_vault_directory();
    let database = temporary.path().join("vault.db");
    let anchor = temporary.path().join("vault.anchor");
    let provider = FileWrappingKeyProvider::new(&temporary.path().join("wrapping-key.bin"));
    let wrapping_key = provider.load_or_create().expect("wrapping key");
    VaultStore::initialize(&database, &anchor, "vault-instance", &wrapping_key)
        .expect("Vault store");

    let phrase = VaultRecoveryKeyV1::generate()
        .expect("recovery entropy")
        .into_mnemonic()
        .expect("English BIP-39 mnemonic");
    assert_eq!(phrase.split_whitespace().count(), 24);
    let imported = VaultRecoveryKeyV1::from_mnemonic(&phrase).expect("checked mnemonic import");
    VaultStore::add_recovery_slot(&anchor, &wrapping_key, &imported).expect("recovery slot");
    assert!(VaultStore::open_with_recovery(&database, &anchor, &imported).is_ok());

    let known = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    assert_eq!(
        VaultRecoveryKeyV1::from_mnemonic(known)
            .expect("published BIP-39 vector")
            .into_mnemonic()
            .expect("canonical display"),
        known
    );
    assert!(VaultRecoveryKeyV1::from_mnemonic(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon zoo"
    )
    .is_err());
}

#[test]
fn recovery_slot_rotation_replaces_only_the_recovery_wrapping_key() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private Vault directory");
    let database = temporary.path().join("vault.db");
    let anchor = temporary.path().join("vault.anchor");
    let provider = FileWrappingKeyProvider::new(&temporary.path().join("wrapping-key.bin"));
    let wrapping_key = provider.load_or_create().expect("wrapping key");
    VaultStore::initialize(&database, &anchor, "vault-instance", &wrapping_key)
        .expect("Vault store");
    let current = VaultRecoveryKeyV1::generate().expect("current recovery key");
    let next = VaultRecoveryKeyV1::generate().expect("next recovery key");
    VaultStore::add_recovery_slot(&anchor, &wrapping_key, &current).expect("recovery slot");
    let before = std::fs::read(&anchor).expect("anchor bytes");
    let wrong = VaultRecoveryKeyV1::generate().expect("wrong recovery key");
    assert!(VaultStore::rotate_recovery_slot(&anchor, &wrapping_key, &wrong, &next).is_err());
    assert_eq!(std::fs::read(&anchor).expect("unchanged anchor"), before);
    VaultStore::rotate_recovery_slot(&anchor, &wrapping_key, &current, &next)
        .expect("rotate recovery slot");
    assert!(VaultStore::open_with_recovery(&database, &anchor, &current).is_err());
    assert!(VaultStore::open_with_recovery(&database, &anchor, &next).is_ok());
    assert!(VaultStore::open(&database, &anchor, &wrapping_key).is_ok());
}

#[test]
fn offline_root_rotation_preserves_credential_records_and_recovery_access() {
    let temporary = private_vault_directory();
    let database = temporary.path().join("vault.db");
    let anchor = temporary.path().join("vault.anchor");
    let provider = FileWrappingKeyProvider::new(&temporary.path().join("wrapping-key.bin"));
    let wrapping_key = provider.load_or_create().expect("wrapping key");
    let store = VaultStore::initialize(&database, &anchor, "vault-instance", &wrapping_key)
        .expect("Vault store");
    let recovery_key = VaultRecoveryKeyV1::generate().expect("recovery key");
    VaultStore::add_recovery_slot(&anchor, &wrapping_key, &recovery_key).expect("recovery slot");
    let scope = credential_scope();
    let record_id = store
        .store_secret(&scope, b"root-rotation-credential")
        .expect("credential");
    drop(store);

    VaultStore::rotate_root_offline(&database, &anchor, &wrapping_key, Some(&recovery_key))
        .expect("offline root rotation");
    assert_record_survives(
        VaultStore::open(&database, &anchor, &wrapping_key).expect("platform open"),
        &record_id,
        &scope,
    );
    assert_record_survives(
        VaultStore::open_with_recovery(&database, &anchor, &recovery_key).expect("recovery open"),
        &record_id,
        &scope,
    );
}

#[test]
fn offline_root_rotation_refuses_to_replace_a_recovery_slot_without_its_key() {
    let temporary = private_vault_directory();
    let database = temporary.path().join("vault.db");
    let anchor = temporary.path().join("vault.anchor");
    let provider = FileWrappingKeyProvider::new(&temporary.path().join("wrapping-key.bin"));
    let wrapping_key = provider.load_or_create().expect("wrapping key");
    VaultStore::initialize(&database, &anchor, "vault-instance", &wrapping_key)
        .expect("Vault store");
    let recovery_key = VaultRecoveryKeyV1::generate().expect("recovery key");
    VaultStore::add_recovery_slot(&anchor, &wrapping_key, &recovery_key).expect("recovery slot");
    let before_database = std::fs::read(&database).expect("database bytes");
    let before_anchor = std::fs::read(&anchor).expect("anchor bytes");

    assert!(VaultStore::rotate_root_offline(&database, &anchor, &wrapping_key, None).is_err());
    assert_eq!(
        std::fs::read(&database).expect("database unchanged"),
        before_database
    );
    assert_eq!(
        std::fs::read(&anchor).expect("anchor unchanged"),
        before_anchor
    );
}

#[test]
fn offline_root_rotation_rejects_a_symlinked_recovery_reservation() {
    let temporary = private_vault_directory();
    let database = temporary.path().join("vault.db");
    let anchor = temporary.path().join("vault.anchor");
    let provider = FileWrappingKeyProvider::new(&temporary.path().join("wrapping-key.bin"));
    let wrapping_key = provider.load_or_create().expect("wrapping key");
    VaultStore::initialize(&database, &anchor, "vault-instance", &wrapping_key)
        .expect("Vault store");
    let sentinel = temporary.path().join("sentinel");
    std::fs::write(&sentinel, b"do not touch").expect("sentinel");
    std::os::unix::fs::symlink(
        &sentinel,
        temporary.path().join(".makosh-vault-root-rotation-v1"),
    )
    .expect("symlinked reservation");

    assert!(VaultStore::rotate_root_offline(&database, &anchor, &wrapping_key, None).is_err());
    assert_eq!(
        std::fs::read(&sentinel).expect("sentinel remains"),
        b"do not touch"
    );
    assert!(
        temporary
            .path()
            .join(".makosh-vault-root-rotation-v1")
            .is_symlink()
    );
}

fn private_vault_directory() -> TempDir {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private Vault directory");
    temporary
}

fn credential_scope() -> SecretRecordScope {
    let purpose = VaultPurposeRequestV1::new(
        "mail.credential".to_owned(),
        "account-a".to_owned(),
        vec![SecretClassV1::ProviderCredential],
        vec![VaultActionV1::Resolve],
        60,
    )
    .expect("typed purpose");
    SecretRecordScope::new(
        "mail".to_owned(),
        &purpose,
        SecretClassV1::ProviderCredential,
        1,
    )
    .expect("typed scope")
}

fn assert_record_survives(
    store: VaultStore,
    record_id: &makosh_vault_store_sqlcipher::SecretRecordId,
    scope: &SecretRecordScope,
) {
    assert_eq!(
        store
            .resolve_scoped_secret(record_id, scope)
            .expect("rotated credential")
            .as_slice(),
        b"root-rotation-credential"
    );
}
