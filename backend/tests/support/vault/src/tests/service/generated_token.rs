use std::os::unix::fs::PermissionsExt;

use makosh_vault_key_provider::WrappingKeyProvider;
use makosh_vault_key_provider_file::FileWrappingKeyProvider;
use makosh_vault_protocol::{
    LeaseAudienceV1, SecretClassV1, VaultActionV1, VaultLeaseIssueRequestV1, VaultPurposeRequestV1,
    VaultTransportCommandV1,
};
use makosh_vault_store_sqlcipher::VaultStore;
use tempfile::TempDir;

use crate::service::runtime::VaultService;

#[test]
fn generated_token_never_leaves_vault_until_a_distinct_resolve_lease() {
    let temporary = private_temporary_directory();
    let audience = storage_audience();
    let mut service = VaultService::new(initialize_store(&temporary), 3).expect("Vault service");
    let create = issue(&mut service, &audience, VaultActionV1::Create, 100);
    let record_id = service
        .execute_command_once(
            &VaultTransportCommandV1::GenerateOpaqueToken {
                lease_id: create,
                secret_class: SecretClassV1::PlatformCredential,
            },
            &audience,
            101,
        )
        .expect("generated token record identity");
    assert_eq!(record_id.len(), 16);

    let resolve = issue(&mut service, &audience, VaultActionV1::Resolve, 102);
    let token = service
        .execute_command_once(
            &VaultTransportCommandV1::ResolveLease {
                lease_id: resolve,
                secret_class: SecretClassV1::PlatformCredential,
            },
            &audience,
            103,
        )
        .expect("resolved token");
    assert_eq!(token.len(), 64);
    assert!(token.iter().all(u8::is_ascii_hexdigit));
}

#[test]
fn owner_derived_key_is_generated_inside_vault_and_resolves_as_32_raw_bytes() {
    let temporary = private_temporary_directory();
    let audience = storage_audience();
    let mut service = VaultService::new(initialize_store(&temporary), 3).expect("Vault service");
    let issue = issue_for(
        &mut service,
        &audience,
        SecretClassV1::OwnerDerivedKey,
        VaultActionV1::IssueOwnerDerivedKey,
        100,
    );
    let record_id = service
        .execute_command_once(
            &VaultTransportCommandV1::GenerateOpaqueToken {
                lease_id: issue,
                secret_class: SecretClassV1::OwnerDerivedKey,
            },
            &audience,
            101,
        )
        .expect("owner-derived key record identity");
    assert_eq!(record_id.len(), 16);

    let resolve = issue_for(
        &mut service,
        &audience,
        SecretClassV1::OwnerDerivedKey,
        VaultActionV1::Resolve,
        102,
    );
    let key = service
        .execute_command_once(
            &VaultTransportCommandV1::ResolveLease {
                lease_id: resolve,
                secret_class: SecretClassV1::OwnerDerivedKey,
            },
            &audience,
            103,
        )
        .expect("resolved owner-derived key");
    assert_eq!(key.len(), 32);
}

#[test]
fn ensure_owner_derived_key_returns_the_same_vault_generated_key() {
    let temporary = private_temporary_directory();
    let audience = storage_audience();
    let mut service = VaultService::new(initialize_store(&temporary), 3).expect("Vault service");
    let first = ensure_owner_derived_key(&mut service, &audience, 100);
    let second = ensure_owner_derived_key(&mut service, &audience, 102);
    assert_eq!(first.len(), 32);
    assert_eq!(first, second);
}

fn ensure_owner_derived_key(
    service: &mut VaultService,
    audience: &LeaseAudienceV1,
    now: u64,
) -> zeroize::Zeroizing<Vec<u8>> {
    let lease = issue_for(
        service,
        audience,
        SecretClassV1::OwnerDerivedKey,
        VaultActionV1::IssueOwnerDerivedKey,
        now,
    );
    service
        .execute_command_once(
            &VaultTransportCommandV1::EnsureOwnerDerivedKey { lease_id: lease },
            audience,
            now + 1,
        )
        .expect("ensured owner-derived key")
}

fn private_temporary_directory() -> TempDir {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    temporary
}

fn storage_audience() -> LeaseAudienceV1 {
    LeaseAudienceV1::new(
        "registration-storage".to_owned(),
        "runtime-storage-1".to_owned(),
        1,
        7,
    )
    .expect("typed audience")
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

fn issue(
    service: &mut VaultService,
    audience: &LeaseAudienceV1,
    action: VaultActionV1,
    now: u64,
) -> makosh_vault_protocol::LeaseIdV1 {
    issue_for(
        service,
        audience,
        SecretClassV1::PlatformCredential,
        action,
        now,
    )
}

fn issue_for(
    service: &mut VaultService,
    audience: &LeaseAudienceV1,
    secret_class: SecretClassV1,
    action: VaultActionV1,
    now: u64,
) -> makosh_vault_protocol::LeaseIdV1 {
    let purpose = VaultPurposeRequestV1::new(
        "storage.control.pgbouncer.admin".to_owned(),
        "storage-main".to_owned(),
        vec![secret_class],
        vec![action],
        60,
    )
    .expect("typed platform credential purpose");
    let request = VaultLeaseIssueRequestV1::new(
        "vault-instance".to_owned(),
        3,
        1,
        "storage".to_owned(),
        purpose,
        audience.clone(),
    )
    .expect("typed lease request");
    service
        .issue_lease(request, now)
        .expect("platform credential lease")
        .lease_id()
        .clone()
}
