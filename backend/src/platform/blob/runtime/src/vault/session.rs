//! One authenticated encrypted request/response transaction through Kernel.

use makosh_runtime_protocol::v1::{
    VaultCiphertextResponseV1, VaultCiphertextRouteDirectionV1, VaultCiphertextRouteV1,
};
use makosh_runtime_protocol::vault_request_id::next_vault_transport_request_id_v1;
use makosh_vault_protocol::{
    LeaseAudienceV1, VaultCiphertextFrameV1, VaultResponseRecipientV1, VaultTransportBindingV1,
    VaultTransportCommandV1, VaultTransportDirectionV1, seal,
};
use zeroize::Zeroizing;

use super::{BlobVaultRouteContextV1, BlobVaultRouteFailureV1, BlobVaultRoutePortV1};

pub(super) struct PreparedVaultCommandV1 {
    route: VaultCiphertextRouteV1,
    recipient: VaultResponseRecipientV1,
    response_binding: VaultTransportBindingV1,
}

pub(super) fn prepare(
    audience: LeaseAudienceV1,
    context: &BlobVaultRouteContextV1,
    command: &VaultTransportCommandV1,
) -> Result<PreparedVaultCommandV1, ()> {
    let request_id = request_id()?;
    let recipient = VaultResponseRecipientV1::generate();
    let request_binding = binding(&audience, context, request_id, command, &recipient, true)?;
    let response_binding = binding(&audience, context, request_id, command, &recipient, false)?;
    let frame = seal(context.public_key(), &request_binding, &command.encode()).map_err(|_| ())?;
    Ok(PreparedVaultCommandV1 {
        route: route(&audience, context, request_id, command, &recipient, frame),
        recipient,
        response_binding,
    })
}

pub(super) async fn execute<T>(
    route_port: &mut T,
    prepared: PreparedVaultCommandV1,
) -> Result<Zeroizing<Vec<u8>>, BlobVaultRouteFailureV1>
where
    T: BlobVaultRoutePortV1 + Send,
{
    let response = route_port
        .route_vault_ciphertext(prepared.route.clone())
        .await?;
    prepared.confirm(response)
}

impl PreparedVaultCommandV1 {
    fn confirm(
        self,
        response: VaultCiphertextResponseV1,
    ) -> Result<Zeroizing<Vec<u8>>, BlobVaultRouteFailureV1> {
        let frame =
            valid_response(&self.route, response).ok_or(BlobVaultRouteFailureV1::Rejected)?;
        self.recipient
            .open(&self.response_binding, &frame)
            .map_err(|_| BlobVaultRouteFailureV1::Rejected)
    }
}

fn binding(
    audience: &LeaseAudienceV1,
    context: &BlobVaultRouteContextV1,
    request_id: [u8; 16],
    command: &VaultTransportCommandV1,
    recipient: &VaultResponseRecipientV1,
    to_vault: bool,
) -> Result<VaultTransportBindingV1, ()> {
    VaultTransportBindingV1::new(
        context.vault_runtime_generation(),
        audience.clone(),
        request_id,
        command.operation_digest(),
        if to_vault {
            VaultTransportDirectionV1::ToVault
        } else {
            VaultTransportDirectionV1::FromVault
        },
        *recipient.public_key().as_bytes(),
    )
    .map_err(|_| ())
}

fn route(
    audience: &LeaseAudienceV1,
    context: &BlobVaultRouteContextV1,
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

fn request_id() -> Result<[u8; 16], ()> {
    next_vault_transport_request_id_v1().ok_or(())
}

fn valid_response(
    request: &VaultCiphertextRouteV1,
    response: VaultCiphertextResponseV1,
) -> Option<VaultCiphertextFrameV1> {
    (response.major == 1
        && response.vault_runtime_generation == request.vault_runtime_generation
        && response.caller_runtime_generation == request.caller_runtime_generation
        && response.request_id == request.request_id
        && response.operation_digest_sha256 == request.operation_digest_sha256
        && response.direction == VaultCiphertextRouteDirectionV1::FromVault as i32)
        .then_some(())?;
    VaultCiphertextFrameV1::from_parts(
        response.hpke_encapped_key,
        response.ciphertext,
        response.hpke_authentication_tag,
    )
    .ok()
}
