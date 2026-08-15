//! Structural limits for opaque Vault ciphertext routing metadata.

use crate::v1::{
    VaultCiphertextRouteDirectionV1, VaultCiphertextRouteV1, VaultRuntimeStateV1,
    VaultRuntimeStatusV1,
};

pub const MAX_VAULT_CIPHERTEXT_BYTES: usize = 256 * 1024;
/// Expected negative result when a caller probes for a credential before
/// provisioning it. Callers may generate or provision the secret afterwards.
pub const VAULT_SECRET_UNAVAILABLE_ERROR_CODE: &str = "secret_unavailable";
/// Opaque SHA-256 digest of the only Storage Vault operation permitted while a
/// binding is already in the durable `revoking` state.
pub const STORAGE_REVOKE_AUDIENCE_OPERATION_DIGEST_V1: [u8; 32] = [
    0x55, 0xe3, 0x78, 0x7d, 0x6a, 0x94, 0x34, 0xa3, 0xec, 0xa4, 0x41, 0x86, 0x56, 0x5f, 0xfe, 0x1a,
    0x40, 0x54, 0x12, 0x7f, 0x0f, 0xbd, 0x35, 0x55, 0xdc, 0x15, 0x59, 0xd0, 0x77, 0x1a, 0xb9, 0x52,
];
const ID_BYTES: usize = 256;
const REQUEST_ID_BYTES: usize = 16;
const SHA256_BYTES: usize = 32;
const HPKE_TAG_BYTES: usize = 16;
const ES256_SIGNATURE_BYTES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultCiphertextRouteValidationError {
    InvalidVersion,
    InvalidIdentity,
    InvalidFence,
    InvalidDirection,
    InvalidBinding,
    InvalidCiphertext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultRuntimeStatusValidationError {
    InvalidState,
    InvalidGeneration,
    InvalidPublicKey,
    InvalidBlocker,
}

pub fn validate_vault_runtime_status_v1(
    status: &VaultRuntimeStatusV1,
) -> Result<(), VaultRuntimeStatusValidationError> {
    let state = VaultRuntimeStateV1::try_from(status.state)
        .map_err(|_| VaultRuntimeStatusValidationError::InvalidState)?;
    if state == VaultRuntimeStateV1::Unspecified || status.vault_runtime_generation == 0 {
        return Err(VaultRuntimeStatusValidationError::InvalidGeneration);
    }
    if state == VaultRuntimeStateV1::Ready {
        if status.hpke_public_key_x25519.len() != SHA256_BYTES || !status.blocker_code.is_empty() {
            return Err(VaultRuntimeStatusValidationError::InvalidPublicKey);
        }
        return Ok(());
    }
    if !status.hpke_public_key_x25519.is_empty() || !valid_blocker_code(&status.blocker_code) {
        return Err(VaultRuntimeStatusValidationError::InvalidBlocker);
    }
    Ok(())
}

pub fn validate_vault_ciphertext_route_v1(
    route: &VaultCiphertextRouteV1,
) -> Result<(), VaultCiphertextRouteValidationError> {
    if route.major != 1 {
        return Err(VaultCiphertextRouteValidationError::InvalidVersion);
    }
    if !valid_id(&route.registration_id) || !valid_id(&route.runtime_instance_id) {
        return Err(VaultCiphertextRouteValidationError::InvalidIdentity);
    }
    if route.vault_runtime_generation == 0
        || route.caller_runtime_generation == 0
        || route.grant_epoch == 0
    {
        return Err(VaultCiphertextRouteValidationError::InvalidFence);
    }
    let has_storage_fence = route.storage_role_epoch != 0
        || route.storage_credential_lease_revision != 0
        || !route.storage_runtime_principal.is_empty()
        || !route.storage_owner_id.is_empty();
    if has_storage_fence
        && (route.storage_role_epoch == 0
            || route.storage_credential_lease_revision == 0
            || !valid_id(&route.storage_runtime_principal)
            || !valid_id(&route.storage_owner_id))
    {
        return Err(VaultCiphertextRouteValidationError::InvalidFence);
    }
    if VaultCiphertextRouteDirectionV1::try_from(route.direction).ok()
        != Some(VaultCiphertextRouteDirectionV1::ToVault)
    {
        return Err(VaultCiphertextRouteValidationError::InvalidDirection);
    }
    if route.request_id.len() != REQUEST_ID_BYTES
        || route.operation_digest_sha256.len() != SHA256_BYTES
        || route.hpke_encapped_key.len() != SHA256_BYTES
        || route.hpke_authentication_tag.len() != HPKE_TAG_BYTES
        || route.response_recipient_hpke_public_key_x25519.len() != SHA256_BYTES
    {
        return Err(VaultCiphertextRouteValidationError::InvalidBinding);
    }
    if route.ciphertext.is_empty() || route.ciphertext.len() > MAX_VAULT_CIPHERTEXT_BYTES {
        return Err(VaultCiphertextRouteValidationError::InvalidCiphertext);
    }
    if !route.kernel_authorization_signature_raw.is_empty()
        && (!valid_id(&route.kernel_instance_id)
            || route.kernel_authorization_signature_raw.len() != ES256_SIGNATURE_BYTES)
    {
        return Err(VaultCiphertextRouteValidationError::InvalidBinding);
    }
    Ok(())
}

fn valid_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= ID_BYTES && value.is_ascii()
}

fn valid_blocker_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
