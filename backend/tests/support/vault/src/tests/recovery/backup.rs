use std::os::unix::fs::PermissionsExt;

use makosh_vault_key_provider::WrappingKeyProvider;
use makosh_vault_key_provider_file::FileWrappingKeyProvider;
use makosh_vault_protocol::{SecretClassV1, VaultActionV1, VaultPurposeRequestV1};
use makosh_vault_store_sqlcipher::{
    SecretRecordScope, VaultBackupClassV1, VaultRecoveryKeyV1, VaultStore,
};
use tempfile::TempDir;

#[test]
fn offline_backup_is_an_authenticated_encrypted_snapshot_without_recovery_phrase() {
    let temporary = private_directory();
    let database = temporary.path().join("vault.db");
    let anchor = temporary.path().join("vault.anchor");
    let backup = temporary.path().join("backup");
    let provider = FileWrappingKeyProvider::new(&temporary.path().join("wrapping-key.bin"));
    let wrapping_key = provider.load_or_create().expect("wrapping key");
    let store = VaultStore::initialize(&database, &anchor, "vault-instance", &wrapping_key)
        .expect("initialize Vault");
    let recovery_phrase = VaultRecoveryKeyV1::generate()
        .expect("recovery entropy")
        .into_mnemonic()
        .expect("recovery phrase");
    let recovery_key = VaultRecoveryKeyV1::from_mnemonic(&recovery_phrase).expect("re-import");
    VaultStore::add_recovery_slot(&anchor, &wrapping_key, &recovery_key).expect("recovery slot");
    store
        .store_secret(&scope(), b"backup-credential-marker")
        .expect("encrypted credential");
    drop(store);

    let manifest = VaultStore::export_backup_offline(&database, &anchor, &backup, &wrapping_key)
        .expect("snapshot export");
    assert_eq!(manifest.class(), VaultBackupClassV1::EncryptedVaultState);
    assert_eq!(manifest.instance_id(), "vault-instance");
    assert_eq!(
        std::fs::read_dir(&backup)
            .expect("backup directory")
            .map(|entry| entry.expect("directory entry").file_name())
            .collect::<std::collections::BTreeSet<_>>(),
        ["vault.anchor", "vault.db", "vault.manifest"]
            .into_iter()
            .map(std::ffi::OsString::from)
            .collect()
    );
    assert_backup_excludes_plaintext(&backup, &recovery_phrase);
    assert_eq!(
        VaultStore::verify_backup_offline(&backup, &wrapping_key).expect("snapshot verification"),
        manifest
    );

    let mut manifest_bytes = std::fs::read(backup.join("vault.manifest")).expect("manifest");
    *manifest_bytes.last_mut().expect("manifest byte") ^= 1;
    std::fs::write(backup.join("vault.manifest"), manifest_bytes).expect("tamper manifest");
    assert!(VaultStore::verify_backup_offline(&backup, &wrapping_key).is_err());
    assert!(VaultStore::open(&database, &anchor, &wrapping_key).is_ok());
}

#[test]
fn recovery_restore_requires_the_recovery_key_and_rebinds_a_new_platform_slot() {
    let source = private_directory();
    let database = source.path().join("vault.db");
    let anchor = source.path().join("vault.anchor");
    let backup = source.path().join("backup");
    let source_provider = FileWrappingKeyProvider::new(&source.path().join("wrapping-key.bin"));
    let source_wrapping = source_provider
        .load_or_create()
        .expect("source wrapping key");
    let store = VaultStore::initialize(&database, &anchor, "vault-instance", &source_wrapping)
        .expect("source Vault");
    let recovery_key = VaultRecoveryKeyV1::generate().expect("recovery key");
    VaultStore::add_recovery_slot(&anchor, &source_wrapping, &recovery_key).expect("recovery slot");
    let record_id = store
        .store_secret(&scope(), b"restored-credential-marker")
        .expect("credential");
    drop(store);
    VaultStore::export_backup_offline(&database, &anchor, &backup, &source_wrapping)
        .expect("backup");

    let target = private_directory();
    let target_database = target.path().join("vault.db");
    let target_anchor = target.path().join("vault.anchor");
    let target_provider = FileWrappingKeyProvider::new(&target.path().join("wrapping-key.bin"));
    let target_wrapping = target_provider
        .load_or_create()
        .expect("target wrapping key");
    assert_wrong_recovery_key_preserves_empty_target(
        &backup,
        &target_database,
        &target_anchor,
        &target_wrapping,
    );
    VaultStore::restore_backup_offline(
        &backup,
        &target_database,
        &target_anchor,
        &recovery_key,
        &target_wrapping,
    )
    .expect("recovery restore");
    assert_rebound_vault(
        &target_database,
        &target_anchor,
        &target_wrapping,
        &source_wrapping,
        &recovery_key,
        &record_id,
    );
    assert!(
        VaultStore::restore_backup_offline(
            &backup,
            &target_database,
            &target_anchor,
            &recovery_key,
            &target_wrapping,
        )
        .is_err()
    );
}

fn assert_wrong_recovery_key_preserves_empty_target(
    backup: &std::path::Path,
    database: &std::path::Path,
    anchor: &std::path::Path,
    wrapping_key: &makosh_vault_key_provider::WrappingKey,
) {
    let wrong = VaultRecoveryKeyV1::generate().expect("wrong recovery key");
    assert!(
        VaultStore::restore_backup_offline(backup, database, anchor, &wrong, wrapping_key).is_err()
    );
    assert!(!database.exists());
    assert!(!anchor.exists());
}

fn assert_rebound_vault(
    database: &std::path::Path,
    anchor: &std::path::Path,
    target_key: &makosh_vault_key_provider::WrappingKey,
    source_key: &makosh_vault_key_provider::WrappingKey,
    recovery_key: &VaultRecoveryKeyV1,
    record_id: &makosh_vault_store_sqlcipher::SecretRecordId,
) {
    let restored = VaultStore::open(database, anchor, target_key).expect("new platform slot opens");
    assert_eq!(
        restored
            .resolve_scoped_secret(record_id, &scope())
            .expect("restored credential")
            .as_slice(),
        b"restored-credential-marker"
    );
    assert!(VaultStore::open(database, anchor, source_key).is_err());
    assert!(VaultStore::open_with_recovery(database, anchor, recovery_key).is_ok());
}

fn private_directory() -> TempDir {
    let temporary = TempDir::new().expect("temporary directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private directory");
    temporary
}

fn scope() -> SecretRecordScope {
    let purpose = VaultPurposeRequestV1::new(
        "mail.credential".to_owned(),
        "account-a".to_owned(),
        vec![SecretClassV1::ProviderCredential],
        vec![VaultActionV1::Create],
        60,
    )
    .expect("purpose");
    SecretRecordScope::new(
        "mail".to_owned(),
        &purpose,
        SecretClassV1::ProviderCredential,
        1,
    )
    .expect("scope")
}

fn assert_backup_excludes_plaintext(backup: &std::path::Path, recovery_phrase: &str) {
    let bytes = ["vault.db", "vault.anchor", "vault.manifest"]
        .into_iter()
        .flat_map(|name| std::fs::read(backup.join(name)).expect("backup bytes"))
        .collect::<Vec<_>>();
    for marker in ["backup-credential-marker", recovery_phrase] {
        assert!(
            !bytes
                .windows(marker.len())
                .any(|window| window == marker.as_bytes())
        );
    }
}
