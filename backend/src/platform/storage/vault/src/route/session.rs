//! Shared encrypted command session for the one Storage-to-Vault route.

use makosh_runtime_protocol::v1::{
    VaultCiphertextResponseV1, VaultCiphertextRouteDirectionV1, VaultCiphertextRouteV1,
};
use makosh_runtime_protocol::vault_request_id::next_vault_transport_request_id_v1;
use makosh_storage_protocol::StorageBindingV1;
use makosh_vault_protocol::{
    LeaseAudienceV1, VaultCiphertextFrameV1, VaultResponseRecipientV1, VaultTransportBindingV1,
    VaultTransportCommandV1, VaultTransportDirectionV1, seal,
};
use zeroize::Zeroizing;

use super::{StorageVaultRouteContextV1, StorageVaultRouteFailureV1, StorageVaultRoutePortV1};

pub struct StorageVaultLeaseAdapterV1<T> {
    pub(super) route_port: T,
    pub(super) context: StorageVaultRouteContextV1,
}

impl<T> StorageVaultLeaseAdapterV1<T> {
    #[must_use]
    pub fn new(route_port: T, context: StorageVaultRouteContextV1) -> Self {
        Self {
            route_port,
            context,
        }
    }

    pub fn into_route_port(self) -> T {
        self.route_port
    }

    /// Exposes only the typed route port so a managed bootstrap can inspect
    /// its inherited control transport without consuming protocol frames.
    pub fn route_port_mut(&mut self) -> &mut T {
        &mut self.route_port
    }
}

impl<T> StorageVaultLeaseAdapterV1<T>
where
    T: StorageVaultRoutePortV1 + Send,
{
    pub async fn revoke_runtime_credential(
        &mut self,
        binding: &StorageBindingV1,
    ) -> Result<(), StorageVaultRouteFailureV1> {
        let prepared = prepare_storage_credential(
            binding,
            &self.context,
            &VaultTransportCommandV1::RevokeAudience,
        )
        .map_err(|_| StorageVaultRouteFailureV1::Rejected)?;
        let outcome = execute(&mut self.route_port, prepared).await?;
        (outcome.as_slice() == [1])
            .then_some(())
            .ok_or(StorageVaultRouteFailureV1::Rejected)
    }
}

pub(super) struct PreparedVaultCommandV1 {
    route: VaultCiphertextRouteV1,
    recipient: VaultResponseRecipientV1,
    response_binding: VaultTransportBindingV1,
}

pub(super) async fn execute<T>(
    route_port: &mut T,
    prepared: PreparedVaultCommandV1,
) -> Result<Zeroizing<Vec<u8>>, StorageVaultRouteFailureV1>
where
    T: StorageVaultRoutePortV1 + Send,
{
    let response = route_port
        .route_vault_ciphertext(prepared.route.clone())
        .await?;
    prepared.confirm(response)
}

pub(super) fn prepare(
    audience: LeaseAudienceV1,
    context: &StorageVaultRouteContextV1,
    command: &VaultTransportCommandV1,
) -> Result<PreparedVaultCommandV1, ()> {
    let request_id = next_request_id()?;
    let recipient = VaultResponseRecipientV1::generate();
    let request_binding =
        transport_binding(&audience, context, request_id, command, &recipient, true)?;
    let response_binding =
        transport_binding(&audience, context, request_id, command, &recipient, false)?;
    let frame = seal(context.public_key(), &request_binding, &command.encode()).map_err(|_| ())?;
    Ok(PreparedVaultCommandV1 {
        route: route_from_audience(&audience, context, request_id, command, &recipient, frame),
        recipient,
        response_binding,
    })
}

pub(super) fn prepare_storage_credential(
    binding: &StorageBindingV1,
    context: &StorageVaultRouteContextV1,
    command: &VaultTransportCommandV1,
) -> Result<PreparedVaultCommandV1, ()> {
    let mut prepared = prepare(binding_audience(binding)?, context, command)?;
    prepared.route.storage_role_epoch = binding.fences().role_epoch();
    prepared.route.storage_credential_lease_revision = binding.fences().credential_lease_revision();
    prepared.route.storage_runtime_principal = binding.access().runtime_principal().to_owned();
    prepared.route.storage_owner_id = binding.identity().owner().to_owned();
    Ok(prepared)
}

impl PreparedVaultCommandV1 {
    fn confirm(
        self,
        response: VaultCiphertextResponseV1,
    ) -> Result<Zeroizing<Vec<u8>>, StorageVaultRouteFailureV1> {
        let frame = validated_response(&self.route, response)
            .ok_or(StorageVaultRouteFailureV1::Rejected)?;
        self.recipient
            .open(&self.response_binding, &frame)
            .map_err(|_| StorageVaultRouteFailureV1::Rejected)
    }
}

fn transport_binding(
    audience: &LeaseAudienceV1,
    context: &StorageVaultRouteContextV1,
    request_id: [u8; 16],
    command: &VaultTransportCommandV1,
    recipient: &VaultResponseRecipientV1,
    to_vault: bool,
) -> Result<VaultTransportBindingV1, ()> {
    let direction = if to_vault {
        VaultTransportDirectionV1::ToVault
    } else {
        VaultTransportDirectionV1::FromVault
    };
    VaultTransportBindingV1::new(
        context.vault_runtime_generation(),
        audience.clone(),
        request_id,
        command.operation_digest(),
        direction,
        *recipient.public_key().as_bytes(),
    )
    .map_err(|_| ())
}

fn route_from_audience(
    audience: &LeaseAudienceV1,
    context: &StorageVaultRouteContextV1,
    request_id: [u8; 16],
    command: &VaultTransportCommandV1,
    recipient: &VaultResponseRecipientV1,
    frame: VaultCiphertextFrameV1,
) -> VaultCiphertextRouteV1 {
    VaultCiphertextRouteV1 {
        major: 1,
        registration_id: audience.module_registration_id().to_owned(),
        runtime_instance_id: audience.runtime_instance_id().to_owned(),
        vault_runtime_generation: context.vault_runtime_generation(),
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
    }
}

pub(super) fn binding_audience(
    binding: &StorageBindingV1,
) -> Result<makosh_vault_protocol::LeaseAudienceV1, ()> {
    makosh_vault_protocol::LeaseAudienceV1::new(
        binding.identity().registration_id().to_owned(),
        binding.identity().runtime_instance_id().to_owned(),
        binding.fences().runtime_generation(),
        binding.fences().grant_epoch(),
    )
    .map_err(|_| ())
}

fn next_request_id() -> Result<[u8; 16], ()> {
    next_vault_transport_request_id_v1().ok_or(())
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
