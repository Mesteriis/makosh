use std::os::unix::fs::PermissionsExt;

use makosh_vault_key_provider::WrappingKeyProvider;
use makosh_vault_key_provider_file::FileWrappingKeyProvider;
use makosh_vault_protocol::{
    LeaseAudienceV1, SecretClassV1, VaultActionV1, VaultLeaseIssueRequestV1,
    VaultProvisioningReceiptV1, VaultProvisioningStateV1, VaultPurposeRequestV1,
    VaultTransportCommandV1,
};
use makosh_vault_store_sqlcipher::VaultStore;
use tempfile::TempDir;

use crate::service::runtime::{VaultService, VaultServiceError};

#[test]
fn owner_provisioning_is_revision_only_idempotent_and_restart_safe() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let key_path = temporary.path().join("wrapping-key.bin");
    let database_path = temporary.path().join("vault.db");
    let anchor_path = temporary.path().join("vault.anchor");
    let provider = FileWrappingKeyProvider::new(&key_path);
    let key = provider.load_or_create().expect("file wrapping key");
    let store = VaultStore::initialize(&database_path, &anchor_path, "vault-instance", &key)
        .expect("Vault store");
    let audience = audience();
    let operation_id = [11; 16];
    let mut service = VaultService::new(store, 3).expect("Vault service");

    let create_receipt = provision(
        &mut service,
        &audience,
        operation_id,
        VaultActionV1::Create,
        1,
        b"credential-v1",
        100,
    )
    .expect("create credential");
    assert_receipt(
        &create_receipt,
        operation_id,
        VaultActionV1::Create,
        1,
        VaultProvisioningStateV1::Active,
    );
    assert!(
        !create_receipt
            .windows(b"credential-v1".len())
            .any(|window| window == b"credential-v1"),
        "sanitized receipt must not carry credential bytes",
    );
    assert_resolves(&mut service, &audience, 1, b"credential-v1", 110);
    drop(service);

    let reopened = VaultStore::open(&database_path, &anchor_path, &key).expect("reopen Vault");
    let mut service = VaultService::new(reopened, 3).expect("restarted Vault service");
    let replay = provision(
        &mut service,
        &audience,
        operation_id,
        VaultActionV1::Create,
        1,
        b"credential-v1",
        120,
    )
    .expect("idempotent replay");
    assert_eq!(replay, create_receipt);

    assert_eq!(
        provision(
            &mut service,
            &audience,
            operation_id,
            VaultActionV1::Create,
            1,
            b"different-credential",
            130,
        ),
        Err(VaultServiceError::SecretUnavailable),
        "operation ID reuse with a different payload must fail closed",
    );

    let replace_receipt = provision(
        &mut service,
        &audience,
        [12; 16],
        VaultActionV1::ReplaceCas,
        2,
        b"credential-v2",
        140,
    )
    .expect("replace credential");
    assert_receipt(
        &replace_receipt,
        [12; 16],
        VaultActionV1::ReplaceCas,
        2,
        VaultProvisioningStateV1::Active,
    );
    assert_resolves(&mut service, &audience, 2, b"credential-v2", 150);

    assert_eq!(
        provision(
            &mut service,
            &audience,
            [13; 16],
            VaultActionV1::ReplaceCas,
            4,
            b"skipped-revision",
            160,
        ),
        Err(VaultServiceError::SecretUnavailable),
        "replace must CAS the exact prior revision",
    );

    let retire_receipt = provision(
        &mut service,
        &audience,
        [14; 16],
        VaultActionV1::Retire,
        2,
        b"",
        170,
    )
    .expect("retire credential");
    assert_receipt(
        &retire_receipt,
        [14; 16],
        VaultActionV1::Retire,
        2,
        VaultProvisioningStateV1::Retired,
    );

    let delete_receipt = provision(
        &mut service,
        &audience,
        [15; 16],
        VaultActionV1::Delete,
        2,
        b"",
        180,
    )
    .expect("delete credential tombstone");
    assert_receipt(
        &delete_receipt,
        [15; 16],
        VaultActionV1::Delete,
        2,
        VaultProvisioningStateV1::Deleted,
    );
}

fn provision(
    service: &mut VaultService,
    audience: &LeaseAudienceV1,
    operation_id: [u8; 16],
    action: VaultActionV1,
    revision: u64,
    payload: &[u8],
    now: u64,
) -> Result<Vec<u8>, VaultServiceError> {
    let lease = service.issue_lease(
        lease_request(purpose(action), audience.clone(), revision),
        now,
    )?;
    service
        .execute_command_once(
            &VaultTransportCommandV1::ProvisionLease {
                lease_id: lease.lease_id().clone(),
                operation_id,
                action,
                secret_class: SecretClassV1::ProviderCredential,
                payload: payload.to_vec(),
            },
            audience,
            now + 1,
        )
        .map(|receipt| receipt.to_vec())
}

fn assert_resolves(
    service: &mut VaultService,
    audience: &LeaseAudienceV1,
    revision: u64,
    expected: &[u8],
    now: u64,
) {
    let purpose = purpose(VaultActionV1::Resolve);
    let lease = service
        .issue_lease(
            lease_request(purpose.clone(), audience.clone(), revision),
            now,
        )
        .expect("resolve lease");
    let resolved = service
        .execute_command_once(
            &VaultTransportCommandV1::ResolveLease {
                lease_id: lease.lease_id().clone(),
                secret_class: SecretClassV1::ProviderCredential,
            },
            audience,
            now + 1,
        )
        .expect("resolve credential");
    assert_eq!(resolved.as_slice(), expected);
}

fn assert_receipt(
    bytes: &[u8],
    operation_id: [u8; 16],
    action: VaultActionV1,
    revision: u64,
    state: VaultProvisioningStateV1,
) {
    let receipt = VaultProvisioningReceiptV1::decode(bytes).expect("sanitized receipt");
    assert_eq!(receipt.operation_id(), &operation_id);
    assert_eq!(receipt.action(), action);
    assert_eq!(receipt.secret_revision(), revision);
    assert_eq!(receipt.state(), state);
}

fn audience() -> LeaseAudienceV1 {
    LeaseAudienceV1::new(
        "registration-mail".to_owned(),
        "owner-device-session".to_owned(),
        1,
        7,
    )
    .expect("typed audience")
}

fn purpose(action: VaultActionV1) -> VaultPurposeRequestV1 {
    VaultPurposeRequestV1::new(
        "mail.credential".to_owned(),
        "account-a".to_owned(),
        vec![SecretClassV1::ProviderCredential],
        vec![action],
        60,
    )
    .expect("typed purpose")
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
