//! Opaque Vault route contract validation.

use super::common::*;
use makosh_runtime_protocol::validation::vault::{
    VaultCiphertextRouteValidationError, validate_vault_ciphertext_route_v1,
};

#[test]
fn vault_ciphertext_route_requires_a_complete_current_transport_binding() {
    let mut route = VaultCiphertextRouteV1 {
        major: 1,
        registration_id: "registration-1".to_owned(),
        runtime_instance_id: "runtime-1".to_owned(),
        caller_runtime_generation: 1,
        vault_runtime_generation: 1,
        grant_epoch: 1,
        request_id: vec![1; 16],
        operation_digest_sha256: vec![2; 32],
        direction: VaultCiphertextRouteDirectionV1::ToVault as i32,
        hpke_encapped_key: vec![3; 32],
        ciphertext: vec![4],
        hpke_authentication_tag: vec![5; 16],
        response_recipient_hpke_public_key_x25519: vec![6; 32],
        kernel_instance_id: String::new(),
        kernel_authorization_signature_raw: Vec::new(),
        storage_role_epoch: 0,
        storage_credential_lease_revision: 0,
        storage_runtime_principal: String::new(),
        storage_owner_id: String::new(),
    };
    assert_eq!(validate_vault_ciphertext_route_v1(&route), Ok(()));
    route.storage_role_epoch = 1;
    assert_eq!(
        validate_vault_ciphertext_route_v1(&route),
        Err(VaultCiphertextRouteValidationError::InvalidFence)
    );
    route.storage_credential_lease_revision = 1;
    route.storage_runtime_principal = "storage_runtime_1".to_owned();
    route.storage_owner_id = "owner_1".to_owned();
    assert_eq!(validate_vault_ciphertext_route_v1(&route), Ok(()));
    route.response_recipient_hpke_public_key_x25519.pop();
    assert_eq!(
        validate_vault_ciphertext_route_v1(&route),
        Err(VaultCiphertextRouteValidationError::InvalidBinding)
    );
}

#[test]
fn vault_ciphertext_response_must_match_the_current_request_fence() {
    let request = VaultCiphertextRouteV1 {
        major: 1,
        registration_id: "registration-1".to_owned(),
        runtime_instance_id: "runtime-1".to_owned(),
        caller_runtime_generation: 3,
        vault_runtime_generation: 7,
        grant_epoch: 3,
        request_id: vec![1; 16],
        operation_digest_sha256: vec![2; 32],
        direction: VaultCiphertextRouteDirectionV1::ToVault as i32,
        hpke_encapped_key: vec![3; 32],
        ciphertext: vec![4],
        hpke_authentication_tag: vec![5; 16],
        response_recipient_hpke_public_key_x25519: vec![6; 32],
        kernel_instance_id: String::new(),
        kernel_authorization_signature_raw: Vec::new(),
        storage_role_epoch: 0,
        storage_credential_lease_revision: 0,
        storage_runtime_principal: String::new(),
        storage_owner_id: String::new(),
    };
    let response = makosh_runtime_protocol::v1::VaultCiphertextResponseV1 {
        major: 1,
        vault_runtime_generation: 7,
        caller_runtime_generation: 3,
        request_id: request.request_id.clone(),
        operation_digest_sha256: request.operation_digest_sha256.clone(),
        direction: VaultCiphertextRouteDirectionV1::FromVault as i32,
        hpke_encapped_key: vec![7; 32],
        ciphertext: vec![8],
        hpke_authentication_tag: vec![9; 16],
    };
    assert!(vault_ciphertext_route::validate_response(&request, response.clone()).is_ok());
    assert!(
        vault_ciphertext_route::validate_response(
            &request,
            makosh_runtime_protocol::v1::VaultCiphertextResponseV1 {
                vault_runtime_generation: 8,
                ..response.clone()
            }
        )
        .is_err()
    );
    assert!(
        vault_ciphertext_route::validate_response(
            &request,
            makosh_runtime_protocol::v1::VaultCiphertextResponseV1 {
                caller_runtime_generation: 4,
                ..response
            }
        )
        .is_err()
    );
}
