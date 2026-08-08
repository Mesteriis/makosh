//! Fences opaque Vault ciphertext before a managed-runtime relay accepts it.

use makosh_runtime_protocol::v1::{
    VaultCiphertextResponseV1, VaultCiphertextRouteDirectionV1, VaultCiphertextRouteV1,
};
use makosh_runtime_protocol::validation::vault::validate_vault_ciphertext_route_v1;
use prost::Message;

use crate::identity::device::signer::{DeviceSigner, FileDeviceSigner};

use crate::modules::capability::router::{
    AuthorizedExternalCapabilityRoute, ExternalCapabilityRouteRequest,
};

pub struct ValidatedVaultCiphertextRoute {
    route: VaultCiphertextRouteV1,
}

impl ValidatedVaultCiphertextRoute {
    #[must_use]
    pub fn route(&self) -> &VaultCiphertextRouteV1 {
        &self.route
    }

    #[must_use]
    pub fn into_route(self) -> VaultCiphertextRouteV1 {
        self.route
    }
}

pub fn validate_for_authorized_external_runtime(
    authorization: &AuthorizedExternalCapabilityRoute,
    request: &ExternalCapabilityRouteRequest<'_>,
    vault_runtime_generation: u64,
    route: VaultCiphertextRouteV1,
) -> Result<ValidatedVaultCiphertextRoute, String> {
    validate_vault_ciphertext_route_v1(&route)
        .map_err(|_| "Vault ciphertext route is invalid".to_owned())?;
    if route.registration_id != request.registration_id()
        || route.runtime_instance_id != request.runtime_id()
        || route.caller_runtime_generation != request.runtime_generation()
        || route.vault_runtime_generation != vault_runtime_generation
        || route.grant_epoch != authorization.grant_epoch()
    {
        return Err("Vault ciphertext route is stale or unauthorized".to_owned());
    }
    Ok(ValidatedVaultCiphertextRoute { route })
}

pub fn sign_for_kernel(
    data_dir: &std::path::Path,
    instance_id: &str,
    route: &mut VaultCiphertextRouteV1,
) -> Result<(), String> {
    let signer = FileDeviceSigner::open_for_instance(data_dir)?;
    route.kernel_instance_id = instance_id.to_owned();
    route.kernel_authorization_signature_raw.clear();
    let mut message = b"makosh.vault-route-authorization.v1\0".to_vec();
    message.extend_from_slice(&route.encode_to_vec());
    route.kernel_authorization_signature_raw = signer.sign(&message).to_vec();
    Ok(())
}

pub fn validate_response(
    request: &VaultCiphertextRouteV1,
    response: VaultCiphertextResponseV1,
) -> Result<VaultCiphertextResponseV1, String> {
    if response.major != 1
        || response.vault_runtime_generation != request.vault_runtime_generation
        || response.caller_runtime_generation != request.caller_runtime_generation
        || response.request_id != request.request_id
        || response.operation_digest_sha256 != request.operation_digest_sha256
        || VaultCiphertextRouteDirectionV1::try_from(response.direction).ok()
            != Some(VaultCiphertextRouteDirectionV1::FromVault)
        || response.hpke_encapped_key.len() != 32
        || response.hpke_authentication_tag.len() != 16
        || response.ciphertext.is_empty()
    {
        return Err("Vault ciphertext response is invalid".to_owned());
    }
    Ok(response)
}
