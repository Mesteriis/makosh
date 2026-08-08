//! Vault-private encryption of command results for the requesting runtime.

use makosh_runtime_protocol::v1::{VaultCiphertextResponseV1, VaultCiphertextRouteDirectionV1};
use makosh_vault_protocol::{
    VaultTransportBindingV1, VaultTransportDirectionV1, VaultTransportPublicKey, seal,
};

pub fn encrypt_result(
    request: &VaultTransportBindingV1,
    plaintext: &[u8],
) -> Result<VaultCiphertextResponseV1, String> {
    let recipient = VaultTransportPublicKey::from_bytes(*request.response_recipient_public_key())
        .map_err(|_| "Vault response recipient is invalid".to_owned())?;
    let binding = VaultTransportBindingV1::new(
        request.vault_runtime_generation(),
        request.audience().clone(),
        *request.request_id(),
        *request.operation_digest(),
        VaultTransportDirectionV1::FromVault,
        *request.response_recipient_public_key(),
    )
    .map_err(|_| "Vault response binding is invalid".to_owned())?;
    let frame = seal(&recipient, &binding, plaintext)
        .map_err(|_| "Vault response encryption failed".to_owned())?;
    Ok(VaultCiphertextResponseV1 {
        major: 1,
        vault_runtime_generation: binding.vault_runtime_generation(),
        request_id: binding.request_id().to_vec(),
        operation_digest_sha256: binding.operation_digest().to_vec(),
        direction: VaultCiphertextRouteDirectionV1::FromVault as i32,
        hpke_encapped_key: frame.encapped_key().to_vec(),
        ciphertext: frame.ciphertext().to_vec(),
        hpke_authentication_tag: frame.tag().to_vec(),
        caller_runtime_generation: request.audience().runtime_generation(),
    })
}
