use std::os::unix::fs::PermissionsExt;

use makosh_vault_key_provider::WrappingKeyProvider;
use makosh_vault_key_provider_file::FileWrappingKeyProvider;
use makosh_vault_protocol::{
    LeaseAudienceV1, SecretClassV1, VaultActionV1, VaultLeaseIssueRequestV1, VaultPurposeRequestV1,
    VaultTransportCommandV1,
};
use makosh_vault_store_sqlcipher::{SecretRecordScope, VaultStore};
use tempfile::TempDir;

use crate::service::runtime::{VaultSecretReplaceRequestV1, VaultService, VaultServiceError};

#[test]
fn resolves_only_a_lease_bound_to_the_exact_secret_scope() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let store = initialize_store(&temporary);
    let purpose = credential_purpose();
    let scope = SecretRecordScope::new(
        "mail".to_owned(),
        &purpose,
        SecretClassV1::ProviderCredential,
        1,
    )
    .expect("record scope");
    let record_id = store
        .store_secret(&scope, b"service-credential-marker")
        .expect("store secret");
    let audience = LeaseAudienceV1::new(
        "registration-mail".to_owned(),
        "runtime-mail-1".to_owned(),
        1,
        7,
    )
    .expect("typed audience");
    let mut service = VaultService::new(store, 3).expect("Vault service");

    let matching = service
        .issue_lease(lease_request(purpose.clone(), audience.clone(), 1), 100)
        .expect("matching lease");
    assert_eq!(
        service
            .resolve_once(matching.lease_id(), &audience, &record_id, &scope, 101)
            .expect("matching scope resolves")
            .as_slice(),
        b"service-credential-marker"
    );

    let mismatched = service
        .issue_lease(lease_request(purpose, audience.clone(), 2), 110)
        .expect("new lease");
    assert_eq!(
        service.resolve_once(mismatched.lease_id(), &audience, &record_id, &scope, 111),
        Err(VaultServiceError::LeaseScopeMismatch)
    );
}

#[test]
fn revoke_and_generation_advance_invalidate_unresolved_service_leases() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let store = initialize_store(&temporary);
    let purpose = credential_purpose();
    let scope = SecretRecordScope::new(
        "mail".to_owned(),
        &purpose,
        SecretClassV1::ProviderCredential,
        1,
    )
    .expect("record scope");
    let record_id = store
        .store_secret(&scope, b"service-credential-marker")
        .expect("store secret");
    let audience = LeaseAudienceV1::new(
        "registration-mail".to_owned(),
        "runtime-mail-1".to_owned(),
        1,
        7,
    )
    .expect("typed audience");
    let mut service = VaultService::new(store, 3).expect("Vault service");

    let revoked = service
        .issue_lease(lease_request(purpose.clone(), audience.clone(), 1), 100)
        .expect("lease before revoke");
    assert_eq!(
        service
            .execute_command_once(&VaultTransportCommandV1::RevokeAudience, &audience, 101)
            .expect("revoke command")
            .as_slice(),
        [1]
    );
    assert_lease_is_invalidated(
        &mut service,
        revoked.lease_id(),
        &audience,
        &record_id,
        &scope,
    );

    let stale_generation = service
        .issue_lease(lease_request(purpose, audience.clone(), 1), 110)
        .expect("lease before generation advance");
    service
        .advance_runtime_generation(4)
        .expect("next runtime generation");
    assert_eq!(service.runtime_generation(), 4);
    assert_lease_is_invalidated(
        &mut service,
        stale_generation.lease_id(),
        &audience,
        &record_id,
        &scope,
    );
}

#[test]
fn resolve_rejects_a_lease_without_the_resolve_action() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let store = initialize_store(&temporary);
    let purpose = VaultPurposeRequestV1::new(
        "mail.credential".to_owned(),
        "account-a".to_owned(),
        vec![SecretClassV1::ProviderCredential],
        vec![VaultActionV1::Create],
        60,
    )
    .expect("create-only purpose");
    let scope = SecretRecordScope::new(
        "mail".to_owned(),
        &purpose,
        SecretClassV1::ProviderCredential,
        1,
    )
    .expect("record scope");
    let record_id = store
        .store_secret(&scope, b"service-credential-marker")
        .expect("store secret");
    let audience = LeaseAudienceV1::new(
        "registration-mail".to_owned(),
        "runtime-mail-1".to_owned(),
        1,
        7,
    )
    .expect("typed audience");
    let mut service = VaultService::new(store, 3).expect("Vault service");
    let lease = service
        .issue_lease(lease_request(purpose, audience.clone(), 1), 100)
        .expect("create-only lease");

    assert_eq!(
        service.resolve_once(lease.lease_id(), &audience, &record_id, &scope, 101),
        Err(VaultServiceError::LeaseActionDenied)
    );
}

#[test]
fn lease_issuance_rejects_declared_actions_without_an_executable_lifecycle() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let store = initialize_store(&temporary);
    let purpose = VaultPurposeRequestV1::new(
        "mail.credential".to_owned(),
        "account-a".to_owned(),
        vec![SecretClassV1::ProviderCredential],
        vec![VaultActionV1::IssueSessionStoreKey],
        60,
    )
    .expect("typed unsupported purpose");
    let audience = LeaseAudienceV1::new(
        "registration-mail".to_owned(),
        "runtime-mail-1".to_owned(),
        1,
        7,
    )
    .expect("typed audience");
    let mut service = VaultService::new(store, 3).expect("Vault service");

    assert_eq!(
        service.issue_lease(lease_request(purpose, audience, 1), 100),
        Err(VaultServiceError::UnsupportedLeaseAction)
    );
}

#[test]
fn transport_retire_and_delete_use_exact_one_time_action_leases() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let store = initialize_store(&temporary);
    let scope = SecretRecordScope::new(
        "mail".to_owned(),
        &credential_purpose(),
        SecretClassV1::ProviderCredential,
        1,
    )
    .expect("record scope");
    store
        .store_secret(&scope, b"service-credential-marker")
        .expect("store secret");
    let audience = LeaseAudienceV1::new(
        "registration-mail".to_owned(),
        "runtime-mail-1".to_owned(),
        1,
        7,
    )
    .expect("typed audience");
    let mut service = VaultService::new(store, 3).expect("Vault service");

    let retire = service
        .issue_lease(
            lease_request(
                lifecycle_purpose(VaultActionV1::Retire),
                audience.clone(),
                1,
            ),
            100,
        )
        .expect("retire lease");
    assert_eq!(
        service
            .execute_command_once(
                &VaultTransportCommandV1::RetireLease {
                    lease_id: retire.lease_id().clone(),
                    secret_class: SecretClassV1::ProviderCredential,
                },
                &audience,
                101,
            )
            .expect("retire")
            .as_slice(),
        [1]
    );
    let resolve = service
        .issue_lease(
            lease_request(credential_purpose(), audience.clone(), 1),
            102,
        )
        .expect("resolve lease");
    assert_eq!(
        service.execute_command_once(
            &VaultTransportCommandV1::ResolveLease {
                lease_id: resolve.lease_id().clone(),
                secret_class: SecretClassV1::ProviderCredential,
            },
            &audience,
            103,
        ),
        Err(VaultServiceError::SecretUnavailable)
    );

    let delete = service
        .issue_lease(
            lease_request(
                lifecycle_purpose(VaultActionV1::Delete),
                audience.clone(),
                1,
            ),
            104,
        )
        .expect("delete lease");
    assert_eq!(
        service
            .execute_command_once(
                &VaultTransportCommandV1::DeleteLease {
                    lease_id: delete.lease_id().clone(),
                    secret_class: SecretClassV1::ProviderCredential,
                },
                &audience,
                105,
            )
            .expect("delete")
            .as_slice(),
        [1]
    );
}

#[test]
fn replace_cas_requires_a_one_time_replace_lease_for_the_next_revision() {
    let mut fixture = replacement_fixture();
    let lease = fixture
        .service
        .issue_lease(
            lease_request(replace_purpose(), fixture.audience.clone(), 2),
            100,
        )
        .expect("replace lease");

    let replacement = fixture
        .service
        .replace_once(VaultSecretReplaceRequestV1 {
            lease_id: lease.lease_id(),
            audience: &fixture.audience,
            prior_record_id: &fixture.prior_record,
            prior_scope: &fixture.prior_scope,
            next_scope: &fixture.next_scope,
            payload: b"credential-revision-two",
            now_unix_seconds: 101,
        })
        .expect("replacement");
    let resolve_lease = fixture
        .service
        .issue_lease(
            lease_request(credential_purpose(), fixture.audience.clone(), 2),
            102,
        )
        .expect("resolve lease");
    assert_eq!(
        fixture
            .service
            .resolve_once(
                resolve_lease.lease_id(),
                &fixture.audience,
                &replacement,
                &fixture.next_scope,
                103,
            )
            .expect("replacement resolution")
            .as_slice(),
        b"credential-revision-two"
    );
}

#[test]
fn resolve_command_uses_the_lease_scope_without_a_vault_record_identifier() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let store = initialize_store(&temporary);
    let purpose = credential_purpose();
    let scope = SecretRecordScope::new(
        "mail".to_owned(),
        &purpose,
        SecretClassV1::ProviderCredential,
        1,
    )
    .expect("record scope");
    store
        .store_secret(&scope, b"service-credential-marker")
        .expect("store secret");
    let audience = LeaseAudienceV1::new(
        "registration-mail".to_owned(),
        "runtime-mail-1".to_owned(),
        1,
        7,
    )
    .expect("typed audience");
    let mut service = VaultService::new(store, 3).expect("Vault service");
    let lease = service
        .issue_lease(lease_request(purpose, audience.clone(), 1), 100)
        .expect("resolve lease");
    let command = VaultTransportCommandV1::ResolveLease {
        lease_id: lease.lease_id().clone(),
        secret_class: SecretClassV1::ProviderCredential,
    };

    assert_eq!(
        service
            .execute_command_once(&command, &audience, 101)
            .expect("resolve command")
            .as_slice(),
        b"service-credential-marker"
    );
}

#[test]
fn transport_issue_lease_requires_the_exact_encrypted_route_audience() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let store = initialize_store(&temporary);
    let purpose = credential_purpose();
    let scope = SecretRecordScope::new(
        "mail".to_owned(),
        &purpose,
        SecretClassV1::ProviderCredential,
        1,
    )
    .expect("record scope");
    store
        .store_secret(&scope, b"transport-issued-credential")
        .expect("store credential");
    let audience = LeaseAudienceV1::new(
        "registration-mail".to_owned(),
        "runtime-mail-1".to_owned(),
        1,
        7,
    )
    .expect("typed audience");
    let request = lease_request(purpose, audience.clone(), 1);
    let command = VaultTransportCommandV1::IssueLease { request };
    let mut service = VaultService::new(store, 3).expect("Vault service");

    let issued = service
        .execute_command_once(&command, &audience, 100)
        .expect("issue through transport");
    let lease_id = makosh_vault_protocol::LeaseIdV1::new(
        String::from_utf8(issued.to_vec()).expect("lease identifier text"),
    )
    .expect("typed lease identifier");
    let resolve = VaultTransportCommandV1::ResolveLease {
        lease_id,
        secret_class: SecretClassV1::ProviderCredential,
    };
    assert_eq!(
        service
            .execute_command_once(&resolve, &audience, 101)
            .expect("resolve issued lease")
            .as_slice(),
        b"transport-issued-credential"
    );

    let wrong_audience = LeaseAudienceV1::new(
        "registration-other".to_owned(),
        "runtime-mail-1".to_owned(),
        1,
        7,
    )
    .expect("different audience");
    assert_eq!(
        service.execute_command_once(&command, &wrong_audience, 102),
        Err(VaultServiceError::LeaseScopeMismatch)
    );
}

struct ReplacementFixture {
    _temporary: TempDir,
    service: VaultService,
    audience: LeaseAudienceV1,
    prior_record: makosh_vault_store_sqlcipher::SecretRecordId,
    prior_scope: SecretRecordScope,
    next_scope: SecretRecordScope,
}

fn replacement_fixture() -> ReplacementFixture {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let store = initialize_store(&temporary);
    let purpose = replace_purpose();
    let prior_scope = SecretRecordScope::new(
        "mail".to_owned(),
        &purpose,
        SecretClassV1::ProviderCredential,
        1,
    )
    .expect("prior scope");
    let prior_record = store
        .store_secret(&prior_scope, b"credential-revision-one")
        .expect("prior credential");
    let next_scope = SecretRecordScope::new(
        "mail".to_owned(),
        &purpose,
        SecretClassV1::ProviderCredential,
        2,
    )
    .expect("next scope");
    let audience = LeaseAudienceV1::new(
        "registration-mail".to_owned(),
        "runtime-mail-1".to_owned(),
        1,
        7,
    )
    .expect("typed audience");
    ReplacementFixture {
        _temporary: temporary,
        service: VaultService::new(store, 3).expect("Vault service"),
        audience,
        prior_record,
        prior_scope,
        next_scope,
    }
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

fn replace_purpose() -> VaultPurposeRequestV1 {
    VaultPurposeRequestV1::new(
        "mail.credential".to_owned(),
        "account-a".to_owned(),
        vec![SecretClassV1::ProviderCredential],
        vec![VaultActionV1::ReplaceCas],
        60,
    )
    .expect("replace purpose")
}

fn lifecycle_purpose(action: VaultActionV1) -> VaultPurposeRequestV1 {
    VaultPurposeRequestV1::new(
        "mail.credential".to_owned(),
        "account-a".to_owned(),
        vec![SecretClassV1::ProviderCredential],
        vec![action],
        60,
    )
    .expect("credential lifecycle purpose")
}

fn lease_request(
    purpose: VaultPurposeRequestV1,
    audience: LeaseAudienceV1,
    secret_revision: u64,
) -> VaultLeaseIssueRequestV1 {
    VaultLeaseIssueRequestV1::new(
        "vault-instance".to_owned(),
        3,
        secret_revision,
        "mail".to_owned(),
        purpose,
        audience,
    )
    .expect("typed lease request")
}

fn assert_lease_is_invalidated(
    service: &mut VaultService,
    lease_id: &makosh_vault_protocol::LeaseIdV1,
    audience: &LeaseAudienceV1,
    record_id: &makosh_vault_store_sqlcipher::SecretRecordId,
    scope: &SecretRecordScope,
) {
    assert!(matches!(
        service.resolve_once(lease_id, audience, record_id, scope, 120),
        Err(VaultServiceError::Lease(_))
    ));
}
