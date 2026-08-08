//! Live external-runtime Storage credential conformance.

mod fixture;
mod route;

use crate::vault::{
    StorageCredentialLeaseErrorV1, StorageVaultLeaseAdapterV1, complete_immediately,
};
use fixture::ExternalStorageVaultFixture;

#[test]
fn external_runtime_receives_rotated_credential_only_while_its_attestation_is_current() {
    let mut fixture = ExternalStorageVaultFixture::new();
    let initial_binding = fixture.binding(1, 1, "runtime_principal_1");
    let mut adapter = fixture.take_adapter();

    let initial = resolve(&mut adapter, &initial_binding);
    fixture.rotate_binding();
    assert_eq!(
        complete_immediately(adapter.ensure_runtime_credential(&initial_binding)),
        Ok(Err(StorageCredentialLeaseErrorV1::Rejected)),
        "the prior durable binding cannot recover its credential after rotation"
    );
    let rotated_binding = fixture.binding(2, 2, "runtime_principal_2");
    let rotated = resolve(&mut adapter, &rotated_binding);

    assert_eq!(
        initial.len(),
        64,
        "Vault creates an opaque 256-bit credential"
    );
    assert_ne!(
        initial, rotated,
        "a new role lease revision rotates the credential"
    );

    fixture.suspend_runtime();
    assert_eq!(
        complete_immediately(adapter.ensure_runtime_credential(&rotated_binding)),
        Ok(Err(StorageCredentialLeaseErrorV1::Rejected)),
        "a suspended external runtime cannot obtain another Vault lease"
    );
}

fn resolve<T>(
    adapter: &mut StorageVaultLeaseAdapterV1<T>,
    binding: &makosh_storage_protocol::StorageBindingV1,
) -> Vec<u8>
where
    T: crate::vault::StorageVaultRoutePortV1 + Send,
{
    complete_immediately(adapter.ensure_runtime_credential(binding))
        .expect("synchronous external Vault route")
        .expect("authorized external credential")
        .to_vec()
}
