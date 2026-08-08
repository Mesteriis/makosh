use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;

use makosh_vault_key_provider::WrappingKeyProvider;
use makosh_vault_key_provider_file::FileWrappingKeyProvider;
use makosh_vault_protocol::{SecretClassV1, VaultActionV1, VaultPurposeRequestV1};
use makosh_vault_store_sqlcipher::{SecretRecordScope, VaultStore};
use tempfile::TempDir;

#[test]
fn concurrent_record_requests_are_serialized_by_the_bounded_vault_actor() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private Vault directory");
    let provider = FileWrappingKeyProvider::new(&temporary.path().join("wrapping-key.bin"));
    let key = provider.load_or_create().expect("wrapping key");
    let store = Arc::new(
        VaultStore::initialize(
            &temporary.path().join("vault.db"),
            &temporary.path().join("vault.anchor"),
            "vault-instance",
            &key,
        )
        .expect("Vault store"),
    );
    let workers = (1_u64..=32)
        .map(|revision| {
            let store = Arc::clone(&store);
            std::thread::spawn(move || store_one_record(&store, revision))
        })
        .collect::<Vec<_>>();
    let records = workers
        .into_iter()
        .map(|worker| worker.join().expect("record worker"))
        .collect::<Vec<_>>();

    for (revision, record_id) in (1_u64..).zip(records) {
        let scope = scope_for_revision(revision);
        let payload = store
            .resolve_scoped_secret(&record_id, &scope)
            .expect("actor resolves stored credential");
        assert_eq!(
            payload.as_slice(),
            format!("credential-{revision}").as_bytes()
        );
    }
    drop(store);
    assert_reopen_after_actor_shutdown(&temporary, &key);
}

fn assert_reopen_after_actor_shutdown(
    temporary: &TempDir,
    key: &makosh_vault_key_provider::WrappingKey,
) {
    let reopened = VaultStore::open(
        &temporary.path().join("vault.db"),
        &temporary.path().join("vault.anchor"),
        key,
    )
    .expect("store reopens after actor shutdown");
    assert_eq!(reopened.instance_id(), "vault-instance");
}

fn store_one_record(
    store: &VaultStore,
    revision: u64,
) -> makosh_vault_store_sqlcipher::SecretRecordId {
    let scope = scope_for_revision(revision);
    store
        .store_secret(&scope, format!("credential-{revision}").as_bytes())
        .expect("actor stores credential")
}

fn scope_for_revision(revision: u64) -> SecretRecordScope {
    let purpose = VaultPurposeRequestV1::new(
        "mail.credential".to_owned(),
        "account-a".to_owned(),
        vec![SecretClassV1::ProviderCredential],
        vec![VaultActionV1::Resolve, VaultActionV1::Create],
        60,
    )
    .expect("typed purpose");
    SecretRecordScope::new(
        "mail".to_owned(),
        &purpose,
        SecretClassV1::ProviderCredential,
        revision,
    )
    .expect("typed scope")
}
