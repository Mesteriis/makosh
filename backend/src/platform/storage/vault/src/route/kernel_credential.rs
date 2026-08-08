//! Generic managed-runtime lease resolution over the inherited Kernel channel.

use makosh_runtime_protocol::v1::{
    VaultCiphertextResponseV1, VaultCiphertextRouteDirectionV1, VaultCiphertextRouteV1,
};
use makosh_runtime_protocol::vault_request_id::next_vault_transport_request_id_v1;
use makosh_vault_protocol::{
    CredentialLeaseV1, LeaseAudienceV1, SecretClassV1, VaultCiphertextFrameV1,
    VaultResponseRecipientV1, VaultTransportBindingV1, VaultTransportCommandV1,
    VaultTransportDirectionV1, VaultTransportPublicKey, seal,
};
use zeroize::Zeroizing;

use super::InheritedKernelVaultRouteV1;

const REQUEST_ID_BYTES: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelVaultLeaseContextV1 {
    vault_runtime_generation: u64,
    vault_public_key: [u8; 32],
}

impl KernelVaultLeaseContextV1 {
    #[must_use]
    pub const fn new(vault_runtime_generation: u64, vault_public_key: [u8; 32]) -> Self {
        Self {
            vault_runtime_generation,
            vault_public_key,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KernelCredentialLeaseErrorV1 {
    InvalidContext,
    InvalidLease,
    Unavailable,
    Rejected,
}

pub fn resolve_kernel_credential_lease(
    route: &mut InheritedKernelVaultRouteV1,
    context: KernelVaultLeaseContextV1,
    lease: &CredentialLeaseV1,
) -> Result<Zeroizing<Vec<u8>>, KernelCredentialLeaseErrorV1> {
    if context.vault_runtime_generation == 0 {
        return Err(KernelCredentialLeaseErrorV1::InvalidContext);
    }
    let key = VaultTransportPublicKey::from_bytes(context.vault_public_key)
        .map_err(|_| KernelCredentialLeaseErrorV1::InvalidContext)?;
    let request = lease.request();
    if request.vault_runtime_generation() != context.vault_runtime_generation {
        return Err(KernelCredentialLeaseErrorV1::InvalidLease);
    }
    let audience = request.audience().clone();
    let command = VaultTransportCommandV1::ResolveLease {
        lease_id: lease.lease_id().clone(),
        secret_class: SecretClassV1::PlatformCredential,
    };
    let request_id = next_request_id().map_err(|_| KernelCredentialLeaseErrorV1::Rejected)?;
    let recipient = VaultResponseRecipientV1::generate();
    let request_binding = binding(
        &audience,
        context.vault_runtime_generation,
        request_id,
        &command,
        &recipient,
        VaultTransportDirectionV1::ToVault,
    )?;
    let response_binding = binding(
        &audience,
        context.vault_runtime_generation,
        request_id,
        &command,
        &recipient,
        VaultTransportDirectionV1::FromVault,
    )?;
    let frame = seal(&key, &request_binding, &command.encode())
        .map_err(|_| KernelCredentialLeaseErrorV1::Rejected)?;
    let request_route = VaultCiphertextRouteV1 {
        major: 1,
        registration_id: audience.module_registration_id().to_owned(),
        runtime_instance_id: audience.runtime_instance_id().to_owned(),
        vault_runtime_generation: context.vault_runtime_generation,
        grant_epoch: audience.grant_epoch(),
        request_id: request_id.to_vec(),
        operation_digest_sha256: command.operation_digest().to_vec(),
        direction: VaultCiphertextRouteDirectionV1::ToVault as i32,
        hpke_encapped_key: frame.encapped_key().to_vec(),
        ciphertext: frame.ciphertext().to_vec(),
        hpke_authentication_tag: frame.tag().to_vec(),
        response_recipient_hpke_public_key_x25519: recipient.public_key().as_bytes().to_vec(),
        kernel_instance_id: String::new(),
        kernel_authorization_signature_raw: Vec::new(),
        caller_runtime_generation: audience.runtime_generation(),
        storage_role_epoch: 0,
        storage_credential_lease_revision: 0,
        storage_runtime_principal: String::new(),
        storage_owner_id: String::new(),
    };
    let response = route
        .route(request_route.clone())
        .map_err(|_| KernelCredentialLeaseErrorV1::Unavailable)?;
    let frame = validated_response(&request_route, response)
        .ok_or(KernelCredentialLeaseErrorV1::Rejected)?;
    recipient
        .open(&response_binding, &frame)
        .map_err(|_| KernelCredentialLeaseErrorV1::Rejected)
}

fn binding(
    audience: &LeaseAudienceV1,
    vault_runtime_generation: u64,
    request_id: [u8; REQUEST_ID_BYTES],
    command: &VaultTransportCommandV1,
    recipient: &VaultResponseRecipientV1,
    direction: VaultTransportDirectionV1,
) -> Result<VaultTransportBindingV1, KernelCredentialLeaseErrorV1> {
    VaultTransportBindingV1::new(
        vault_runtime_generation,
        audience.clone(),
        request_id,
        command.operation_digest(),
        direction,
        *recipient.public_key().as_bytes(),
    )
    .map_err(|_| KernelCredentialLeaseErrorV1::Rejected)
}

fn validated_response(
    request: &VaultCiphertextRouteV1,
    response: VaultCiphertextResponseV1,
) -> Option<VaultCiphertextFrameV1> {
    if response.major != 1
        || response.vault_runtime_generation != request.vault_runtime_generation
        || response.caller_runtime_generation != request.caller_runtime_generation
        || response.request_id != request.request_id
        || response.operation_digest_sha256 != request.operation_digest_sha256
        || response.direction != VaultCiphertextRouteDirectionV1::FromVault as i32
    {
        return None;
    }
    VaultCiphertextFrameV1::from_parts(
        response.hpke_encapped_key,
        response.ciphertext,
        response.hpke_authentication_tag,
    )
    .ok()
}

fn next_request_id() -> Result<[u8; REQUEST_ID_BYTES], ()> {
    next_vault_transport_request_id_v1().ok_or(())
}
