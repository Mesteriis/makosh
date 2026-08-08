//! Owner-derived projection-key access over the managed Vault control channel.

use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use makosh_runtime_protocol::v1::{
    ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    ManagedRuntimeOwnerDerivedKeyDeliveryV1, ManagedRuntimeOwnerDerivedKeyRequestV1,
    ManagedRuntimeVaultRouteRequestV1, VaultCiphertextRouteDirectionV1, VaultCiphertextRouteV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use makosh_vault_protocol::{
    LeaseAudienceV1, LeaseIdV1, SecretClassV1, VaultActionV1, VaultCiphertextFrameV1,
    VaultLeaseIssueRequestV1, VaultPurposeRequestV1, VaultResponseRecipientV1,
    VaultTransportCommandV1, VaultTransportDirectionV1, VaultTransportPublicKey, seal,
};
use prost::Message;
use zeroize::Zeroizing;

use crate::{
    ManagedProviderCredentialErrorV1, binding, next_request_id, read_frame, valid_response,
    write_frame,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedOwnerDerivedKeyContextV1 {
    pub vault_instance_id: String,
    pub vault_runtime_generation: u64,
    pub vault_public_key_x25519: [u8; 32],
    pub logical_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManagedOwnerDerivedKeyErrorV1 {
    InvalidContext,
    Rejected,
    Unavailable,
}

pub struct ManagedOwnerDerivedKeyClientV1 {
    channel: UnixStream,
}

impl ManagedOwnerDerivedKeyClientV1 {
    #[must_use]
    pub fn new(channel: UnixStream) -> Self {
        Self { channel }
    }

    pub fn ensure(
        &mut self,
        context: &ManagedOwnerDerivedKeyContextV1,
        capability_id: &str,
        purpose_id: &str,
        key_schema_revision: u32,
        ttl_seconds: u32,
    ) -> Result<Zeroizing<Vec<u8>>, ManagedOwnerDerivedKeyErrorV1> {
        let audience = audience(context)?;
        if !valid_identifier(capability_id)
            || !valid_identifier(purpose_id)
            || key_schema_revision == 0
            || !(1..=600).contains(&ttl_seconds)
        {
            return Err(ManagedOwnerDerivedKeyErrorV1::InvalidContext);
        }
        let lease_id = self.issue_lease(
            context,
            audience.clone(),
            capability_id,
            purpose_id,
            key_schema_revision,
            ttl_seconds,
        )?;
        let key = self.execute_command(
            context,
            audience,
            VaultTransportCommandV1::EnsureOwnerDerivedKey { lease_id },
        )?;
        (key.len() == 32)
            .then_some(key)
            .ok_or(ManagedOwnerDerivedKeyErrorV1::Rejected)
    }

    fn issue_lease(
        &mut self,
        context: &ManagedOwnerDerivedKeyContextV1,
        audience: LeaseAudienceV1,
        capability_id: &str,
        purpose_id: &str,
        key_schema_revision: u32,
        ttl_seconds: u32,
    ) -> Result<LeaseIdV1, ManagedOwnerDerivedKeyErrorV1> {
        let recipient = VaultResponseRecipientV1::generate();
        let request_id = next_request_id().map_err(map_transport_error)?;
        let delivery = self.request_owner_key_lease(
            request_id,
            capability_id,
            purpose_id,
            key_schema_revision,
            ttl_seconds,
            recipient.public_key().as_bytes(),
        )?;
        let purpose = VaultPurposeRequestV1::new(
            purpose_id.to_owned(),
            capability_id.to_owned(),
            vec![SecretClassV1::OwnerDerivedKey],
            vec![VaultActionV1::IssueOwnerDerivedKey],
            ttl_seconds,
        )
        .map_err(|_| ManagedOwnerDerivedKeyErrorV1::InvalidContext)?;
        let issue = VaultLeaseIssueRequestV1::new(
            context.vault_instance_id.clone(),
            context.vault_runtime_generation,
            u64::from(key_schema_revision),
            context.logical_owner_id.clone(),
            purpose,
            audience.clone(),
        )
        .map_err(|_| ManagedOwnerDerivedKeyErrorV1::InvalidContext)?;
        let command = VaultTransportCommandV1::IssueLease { request: issue };
        let binding = binding(
            &audience,
            context.vault_runtime_generation,
            request_id,
            &command,
            &recipient,
            VaultTransportDirectionV1::FromVault,
        )
        .map_err(map_transport_error)?;
        let frame = VaultCiphertextFrameV1::from_parts(
            delivery.encapped_key,
            delivery.ciphertext,
            delivery.tag,
        )
        .map_err(|_| ManagedOwnerDerivedKeyErrorV1::Rejected)?;
        let lease_id = recipient
            .open(&binding, &frame)
            .map_err(|_| ManagedOwnerDerivedKeyErrorV1::Rejected)?;
        String::from_utf8(lease_id.to_vec())
            .ok()
            .and_then(|value| LeaseIdV1::new(value).ok())
            .ok_or(ManagedOwnerDerivedKeyErrorV1::Rejected)
    }

    fn request_owner_key_lease(
        &mut self,
        request_id: [u8; 16],
        capability_id: &str,
        purpose_id: &str,
        key_schema_revision: u32,
        ttl_seconds: u32,
        recipient_public_key_x25519: &[u8; 32],
    ) -> Result<ManagedRuntimeOwnerDerivedKeyDeliveryV1, ManagedOwnerDerivedKeyErrorV1> {
        let request = owner_key_request(
            request_id,
            capability_id,
            purpose_id,
            key_schema_revision,
            ttl_seconds,
            recipient_public_key_x25519,
        );
        write_frame(
            &mut self.channel,
            &ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::IssueOwnerDerivedKey(request)),
            }
            .encode_to_vec(),
        )
        .map_err(map_transport_error)?;
        let response = ManagedRuntimeControlResponseV1::decode(
            read_frame(&mut self.channel)
                .map_err(map_transport_error)?
                .as_slice(),
        )
        .map_err(|_| ManagedOwnerDerivedKeyErrorV1::Rejected)?;
        match response.result {
            Some(ControlResult::OwnerDerivedKeyDelivery(delivery))
                if response.error_code.is_empty() =>
            {
                Ok(delivery)
            }
            _ => Err(ManagedOwnerDerivedKeyErrorV1::Rejected),
        }
    }

    fn execute_command(
        &mut self,
        context: &ManagedOwnerDerivedKeyContextV1,
        audience: LeaseAudienceV1,
        command: VaultTransportCommandV1,
    ) -> Result<Zeroizing<Vec<u8>>, ManagedOwnerDerivedKeyErrorV1> {
        let request_id = next_request_id().map_err(map_transport_error)?;
        let recipient = VaultResponseRecipientV1::generate();
        let request_binding = binding(
            &audience,
            context.vault_runtime_generation,
            request_id,
            &command,
            &recipient,
            VaultTransportDirectionV1::ToVault,
        )
        .map_err(map_transport_error)?;
        let response_binding = binding(
            &audience,
            context.vault_runtime_generation,
            request_id,
            &command,
            &recipient,
            VaultTransportDirectionV1::FromVault,
        )
        .map_err(map_transport_error)?;
        let vault_key = VaultTransportPublicKey::from_bytes(context.vault_public_key_x25519)
            .map_err(|_| ManagedOwnerDerivedKeyErrorV1::InvalidContext)?;
        let frame = seal(&vault_key, &request_binding, &command.encode())
            .map_err(|_| ManagedOwnerDerivedKeyErrorV1::Rejected)?;
        let route = VaultCiphertextRouteV1 {
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
        write_frame(
            &mut self.channel,
            &ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::RouteVaultCiphertext(
                    ManagedRuntimeVaultRouteRequestV1 {
                        route: Some(route.clone()),
                    },
                )),
            }
            .encode_to_vec(),
        )
        .map_err(map_transport_error)?;
        let response = ManagedRuntimeControlResponseV1::decode(
            read_frame(&mut self.channel)
                .map_err(map_transport_error)?
                .as_slice(),
        )
        .map_err(|_| ManagedOwnerDerivedKeyErrorV1::Rejected)?
        .result
        .and_then(|result| match result {
            ControlResult::VaultRoute(response) => Some(response),
            _ => None,
        })
        .ok_or(ManagedOwnerDerivedKeyErrorV1::Rejected)?;
        let response = response
            .response
            .filter(|_| response.error_code.is_empty())
            .ok_or(ManagedOwnerDerivedKeyErrorV1::Rejected)?;
        let frame = valid_response(&route, response).map_err(map_transport_error)?;
        recipient
            .open(&response_binding, &frame)
            .map_err(|_| ManagedOwnerDerivedKeyErrorV1::Rejected)
    }
}

/// Ensures the owner-derived projection key through the correlated inherited
/// control channel. It keeps the same HPKE lease and Vault-route checks as V1,
/// but never takes ownership of the domain runtime's channel.
pub fn ensure_managed_owner_derived_key_v2(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    context: &ManagedOwnerDerivedKeyContextV1,
    capability_id: &str,
    purpose_id: &str,
    key_schema_revision: u32,
    ttl_seconds: u32,
) -> Result<Zeroizing<Vec<u8>>, ManagedOwnerDerivedKeyErrorV1> {
    let audience = audience(context)?;
    if !valid_identifier(capability_id)
        || !valid_identifier(purpose_id)
        || key_schema_revision == 0
        || !(1..=600).contains(&ttl_seconds)
    {
        return Err(ManagedOwnerDerivedKeyErrorV1::InvalidContext);
    }
    let recipient = VaultResponseRecipientV1::generate();
    let request_id = next_request_id().map_err(map_transport_error)?;
    let delivery = request_owner_key_lease_v2(
        channel,
        dispatcher,
        owner_key_request(
            request_id,
            capability_id,
            purpose_id,
            key_schema_revision,
            ttl_seconds,
            recipient.public_key().as_bytes(),
        ),
    )?;
    let purpose = VaultPurposeRequestV1::new(
        purpose_id.to_owned(),
        capability_id.to_owned(),
        vec![SecretClassV1::OwnerDerivedKey],
        vec![VaultActionV1::IssueOwnerDerivedKey],
        ttl_seconds,
    )
    .map_err(|_| ManagedOwnerDerivedKeyErrorV1::InvalidContext)?;
    let issue = VaultLeaseIssueRequestV1::new(
        context.vault_instance_id.clone(),
        context.vault_runtime_generation,
        u64::from(key_schema_revision),
        context.logical_owner_id.clone(),
        purpose,
        audience.clone(),
    )
    .map_err(|_| ManagedOwnerDerivedKeyErrorV1::InvalidContext)?;
    let issue_command = VaultTransportCommandV1::IssueLease { request: issue };
    let binding = binding(
        &audience,
        context.vault_runtime_generation,
        request_id,
        &issue_command,
        &recipient,
        VaultTransportDirectionV1::FromVault,
    )
    .map_err(map_transport_error)?;
    let frame = VaultCiphertextFrameV1::from_parts(
        delivery.encapped_key,
        delivery.ciphertext,
        delivery.tag,
    )
    .map_err(|_| ManagedOwnerDerivedKeyErrorV1::Rejected)?;
    let lease_id = recipient
        .open(&binding, &frame)
        .map_err(|_| ManagedOwnerDerivedKeyErrorV1::Rejected)?;
    let lease_id = String::from_utf8(lease_id.to_vec())
        .ok()
        .and_then(|value| LeaseIdV1::new(value).ok())
        .ok_or(ManagedOwnerDerivedKeyErrorV1::Rejected)?;
    let key = execute_owner_key_command_v2(
        channel,
        dispatcher,
        context,
        audience,
        VaultTransportCommandV1::EnsureOwnerDerivedKey { lease_id },
    )?;
    (key.len() == 32)
        .then_some(key)
        .ok_or(ManagedOwnerDerivedKeyErrorV1::Rejected)
}

fn request_owner_key_lease_v2(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    request: ManagedRuntimeOwnerDerivedKeyRequestV1,
) -> Result<ManagedRuntimeOwnerDerivedKeyDeliveryV1, ManagedOwnerDerivedKeyErrorV1> {
    let response = channel
        .request_next_with_dispatch(
            ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::IssueOwnerDerivedKey(request)),
            },
            dispatcher,
        )
        .map_err(|_| ManagedOwnerDerivedKeyErrorV1::Unavailable)?;
    match response.result {
        Some(ControlResult::OwnerDerivedKeyDelivery(delivery))
            if response.error_code.is_empty() =>
        {
            Ok(delivery)
        }
        _ => Err(ManagedOwnerDerivedKeyErrorV1::Rejected),
    }
}

fn execute_owner_key_command_v2(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    context: &ManagedOwnerDerivedKeyContextV1,
    audience: LeaseAudienceV1,
    command: VaultTransportCommandV1,
) -> Result<Zeroizing<Vec<u8>>, ManagedOwnerDerivedKeyErrorV1> {
    let request_id = next_request_id().map_err(map_transport_error)?;
    let recipient = VaultResponseRecipientV1::generate();
    let request_binding = binding(
        &audience,
        context.vault_runtime_generation,
        request_id,
        &command,
        &recipient,
        VaultTransportDirectionV1::ToVault,
    )
    .map_err(map_transport_error)?;
    let response_binding = binding(
        &audience,
        context.vault_runtime_generation,
        request_id,
        &command,
        &recipient,
        VaultTransportDirectionV1::FromVault,
    )
    .map_err(map_transport_error)?;
    let vault_key = VaultTransportPublicKey::from_bytes(context.vault_public_key_x25519)
        .map_err(|_| ManagedOwnerDerivedKeyErrorV1::InvalidContext)?;
    let frame = seal(&vault_key, &request_binding, &command.encode())
        .map_err(|_| ManagedOwnerDerivedKeyErrorV1::Rejected)?;
    let route = VaultCiphertextRouteV1 {
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
    let response = channel
        .request_next_with_dispatch(
            ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::RouteVaultCiphertext(
                    ManagedRuntimeVaultRouteRequestV1 {
                        route: Some(route.clone()),
                    },
                )),
            },
            dispatcher,
        )
        .map_err(|_| ManagedOwnerDerivedKeyErrorV1::Unavailable)?;
    let response = response
        .result
        .and_then(|result| match result {
            ControlResult::VaultRoute(response) => Some(response),
            _ => None,
        })
        .ok_or(ManagedOwnerDerivedKeyErrorV1::Rejected)?;
    let response = response
        .response
        .filter(|_| response.error_code.is_empty())
        .ok_or(ManagedOwnerDerivedKeyErrorV1::Rejected)?;
    let frame = valid_response(&route, response).map_err(map_transport_error)?;
    recipient
        .open(&response_binding, &frame)
        .map_err(|_| ManagedOwnerDerivedKeyErrorV1::Rejected)
}

fn audience(
    context: &ManagedOwnerDerivedKeyContextV1,
) -> Result<LeaseAudienceV1, ManagedOwnerDerivedKeyErrorV1> {
    if context.vault_runtime_generation == 0
        || context.runtime_generation == 0
        || context.grant_epoch == 0
    {
        return Err(ManagedOwnerDerivedKeyErrorV1::InvalidContext);
    }
    LeaseAudienceV1::new(
        context.registration_id.clone(),
        context.runtime_instance_id.clone(),
        context.runtime_generation,
        context.grant_epoch,
    )
    .map_err(|_| ManagedOwnerDerivedKeyErrorV1::InvalidContext)
}

fn owner_key_request(
    request_id: [u8; 16],
    capability_id: &str,
    purpose_id: &str,
    key_schema_revision: u32,
    ttl_seconds: u32,
    recipient_public_key_x25519: &[u8; 32],
) -> ManagedRuntimeOwnerDerivedKeyRequestV1 {
    ManagedRuntimeOwnerDerivedKeyRequestV1 {
        request_id: request_id.to_vec(),
        purpose_id: purpose_id.to_owned(),
        key_schema_revision,
        ttl_seconds,
        recipient_public_key_x25519: recipient_public_key_x25519.to_vec(),
        capability_id: capability_id.to_owned(),
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

fn map_transport_error(value: ManagedProviderCredentialErrorV1) -> ManagedOwnerDerivedKeyErrorV1 {
    match value {
        ManagedProviderCredentialErrorV1::InvalidContext => {
            ManagedOwnerDerivedKeyErrorV1::InvalidContext
        }
        ManagedProviderCredentialErrorV1::Rejected => ManagedOwnerDerivedKeyErrorV1::Rejected,
        ManagedProviderCredentialErrorV1::Unavailable => ManagedOwnerDerivedKeyErrorV1::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::owner_key_request;

    #[test]
    fn lease_request_keeps_the_declared_capability_in_the_kernel_contract() {
        let request = owner_key_request(
            [1; 16],
            "search",
            "communications.search.index",
            1,
            60,
            &[2; 32],
        );
        assert_eq!(request.capability_id, "search");
        assert_eq!(request.purpose_id, "communications.search.index");
    }
}
