//! Provider-neutral encrypted credential access over a Kernel-inherited FD.

pub mod owner_derived_key;

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use makosh_runtime_protocol::v1::{
    ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    ManagedRuntimeProviderCredentialRequestV1, ManagedRuntimeVaultRouteRequestV1,
    VaultCiphertextResponseV1, VaultCiphertextRouteDirectionV1, VaultCiphertextRouteV1,
    managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use makosh_runtime_protocol::vault_request_id::next_vault_transport_request_id_v1;
use makosh_vault_protocol::{
    LeaseAudienceV1, LeaseIdV1, SecretClassV1, VaultActionV1, VaultCiphertextFrameV1,
    VaultLeaseIssueRequestV1, VaultPurposeRequestV1, VaultResponseRecipientV1,
    VaultTransportBindingV1, VaultTransportCommandV1, VaultTransportDirectionV1,
    VaultTransportPublicKey, seal,
};
use prost::Message;
use zeroize::Zeroizing;

pub(crate) const MAX_FRAME_BYTES: usize = 512 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedProviderCredentialContextV1 {
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
pub enum ManagedProviderCredentialErrorV1 {
    InvalidContext,
    Rejected,
    Unavailable,
}

pub struct ManagedProviderCredentialRequestV1<'a> {
    pub configuration_instance_id: &'a str,
    pub purpose_id: &'a str,
    pub credential_revision: u64,
    pub ttl_seconds: u32,
    pub secret_class: SecretClassV1,
}

pub struct ManagedProviderCredentialClientV1 {
    channel: UnixStream,
}

/// Provider-credential access over the correlated managed-control transport.
///
/// The caller retains the single control-channel frame pump and supplies the
/// owner-local dispatcher used while a Vault operation is awaiting its
/// correlated response.
pub struct ManagedProviderCredentialClientV2<'a> {
    channel: &'a mut ManagedControlChannelV2<UnixStream>,
}

impl ManagedProviderCredentialClientV1 {
    #[must_use]
    pub fn new(channel: UnixStream) -> Self {
        Self { channel }
    }

    pub fn resolve(
        &mut self,
        context: &ManagedProviderCredentialContextV1,
        configuration_instance_id: &str,
        purpose_id: &str,
        credential_revision: u64,
        ttl_seconds: u32,
        secret_class: SecretClassV1,
    ) -> Result<Zeroizing<Vec<u8>>, ManagedProviderCredentialErrorV1> {
        let audience = audience(context)?;
        let request = ManagedProviderCredentialRequestV1 {
            configuration_instance_id,
            purpose_id,
            credential_revision,
            ttl_seconds,
            secret_class,
        };
        let lease_id =
            self.issue_action_lease(context, audience.clone(), &request, VaultActionV1::Resolve)?;
        self.resolve_lease(context, audience, lease_id, request.secret_class)
    }

    pub fn store_once(
        &mut self,
        context: &ManagedProviderCredentialContextV1,
        request: ManagedProviderCredentialRequestV1<'_>,
        payload: &[u8],
    ) -> Result<[u8; 16], ManagedProviderCredentialErrorV1> {
        let audience = audience(context)?;
        if !valid_mutation_payload(payload) {
            return Err(ManagedProviderCredentialErrorV1::InvalidContext);
        }
        let lease_id =
            self.issue_action_lease(context, audience.clone(), &request, VaultActionV1::Create)?;
        let response = self.execute_command(
            context,
            audience,
            VaultTransportCommandV1::StoreLease {
                lease_id,
                secret_class: request.secret_class,
                payload: payload.to_vec(),
            },
        )?;
        response
            .as_slice()
            .try_into()
            .map_err(|_| ManagedProviderCredentialErrorV1::Rejected)
    }

    pub fn replace_once(
        &mut self,
        context: &ManagedProviderCredentialContextV1,
        request: ManagedProviderCredentialRequestV1<'_>,
        prior_record_id: [u8; 16],
        payload: &[u8],
    ) -> Result<[u8; 16], ManagedProviderCredentialErrorV1> {
        let audience = audience(context)?;
        if !valid_mutation_payload(payload) {
            return Err(ManagedProviderCredentialErrorV1::InvalidContext);
        }
        let lease_id = self.issue_action_lease(
            context,
            audience.clone(),
            &request,
            VaultActionV1::ReplaceCas,
        )?;
        let response = self.execute_command(
            context,
            audience,
            VaultTransportCommandV1::ReplaceLease {
                lease_id,
                secret_class: request.secret_class,
                prior_record_id,
                payload: payload.to_vec(),
            },
        )?;
        response
            .as_slice()
            .try_into()
            .map_err(|_| ManagedProviderCredentialErrorV1::Rejected)
    }

    pub fn retire_once(
        &mut self,
        context: &ManagedProviderCredentialContextV1,
        request: ManagedProviderCredentialRequestV1<'_>,
    ) -> Result<(), ManagedProviderCredentialErrorV1> {
        let audience = audience(context)?;
        let lease_id =
            self.issue_action_lease(context, audience.clone(), &request, VaultActionV1::Retire)?;
        let response = self.execute_command(
            context,
            audience,
            VaultTransportCommandV1::RetireLease {
                lease_id,
                secret_class: request.secret_class,
            },
        )?;
        (response.as_slice() == [1])
            .then_some(())
            .ok_or(ManagedProviderCredentialErrorV1::Rejected)
    }

    pub fn delete_once(
        &mut self,
        context: &ManagedProviderCredentialContextV1,
        request: ManagedProviderCredentialRequestV1<'_>,
    ) -> Result<(), ManagedProviderCredentialErrorV1> {
        let audience = audience(context)?;
        let lease_id =
            self.issue_action_lease(context, audience.clone(), &request, VaultActionV1::Delete)?;
        let response = self.execute_command(
            context,
            audience,
            VaultTransportCommandV1::DeleteLease {
                lease_id,
                secret_class: request.secret_class,
            },
        )?;
        (response.as_slice() == [1])
            .then_some(())
            .ok_or(ManagedProviderCredentialErrorV1::Rejected)
    }

    fn issue_action_lease(
        &mut self,
        context: &ManagedProviderCredentialContextV1,
        audience: LeaseAudienceV1,
        request: &ManagedProviderCredentialRequestV1<'_>,
        action: VaultActionV1,
    ) -> Result<LeaseIdV1, ManagedProviderCredentialErrorV1> {
        if request.credential_revision == 0 || request.ttl_seconds == 0 || request.ttl_seconds > 600
        {
            return Err(ManagedProviderCredentialErrorV1::InvalidContext);
        }
        let recipient = VaultResponseRecipientV1::generate();
        let request_id = next_request_id()?;
        let delivery = self.issue_lease(
            request_id,
            request,
            action,
            recipient.public_key().as_bytes(),
        )?;
        let issue = issue_request(context, audience.clone(), request, action)?;
        let command = VaultTransportCommandV1::IssueLease { request: issue };
        let binding = binding(
            &audience,
            context.vault_runtime_generation,
            request_id,
            &command,
            &recipient,
            VaultTransportDirectionV1::FromVault,
        )?;
        let frame = VaultCiphertextFrameV1::from_parts(
            delivery.encapped_key,
            delivery.ciphertext,
            delivery.tag,
        )
        .map_err(|_| ManagedProviderCredentialErrorV1::Rejected)?;
        let lease_id = recipient
            .open(&binding, &frame)
            .map_err(|_| ManagedProviderCredentialErrorV1::Rejected)?;
        String::from_utf8(lease_id.to_vec())
            .ok()
            .and_then(|value| LeaseIdV1::new(value).ok())
            .ok_or(ManagedProviderCredentialErrorV1::Rejected)
    }

    fn issue_lease(
        &mut self,
        request_id: [u8; 16],
        request: &ManagedProviderCredentialRequestV1<'_>,
        action: VaultActionV1,
        recipient_public_key_x25519: &[u8; 32],
    ) -> Result<
        makosh_runtime_protocol::v1::ManagedRuntimeProviderCredentialDeliveryV1,
        ManagedProviderCredentialErrorV1,
    > {
        write_frame(
            &mut self.channel,
            &ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::IssueProviderCredential(
                    ManagedRuntimeProviderCredentialRequestV1 {
                        request_id: request_id.to_vec(),
                        purpose_id: request.purpose_id.to_owned(),
                        credential_revision: request.credential_revision,
                        ttl_seconds: request.ttl_seconds,
                        secret_class: request.secret_class.code() as u32,
                        recipient_public_key_x25519: recipient_public_key_x25519.to_vec(),
                        configuration_instance_id: request.configuration_instance_id.to_owned(),
                        action: action.code() as u32,
                    },
                )),
            }
            .encode_to_vec(),
        )?;
        let response =
            ManagedRuntimeControlResponseV1::decode(read_frame(&mut self.channel)?.as_slice())
                .map_err(|_| ManagedProviderCredentialErrorV1::Rejected)?;
        match response.result {
            Some(ControlResult::ProviderCredentialDelivery(delivery))
                if response.error_code.is_empty() =>
            {
                Ok(delivery)
            }
            _ => Err(ManagedProviderCredentialErrorV1::Rejected),
        }
    }

    fn resolve_lease(
        &mut self,
        context: &ManagedProviderCredentialContextV1,
        audience: LeaseAudienceV1,
        lease_id: LeaseIdV1,
        secret_class: SecretClassV1,
    ) -> Result<Zeroizing<Vec<u8>>, ManagedProviderCredentialErrorV1> {
        self.execute_command(
            context,
            audience,
            VaultTransportCommandV1::ResolveLease {
                lease_id,
                secret_class,
            },
        )
    }

    fn execute_command(
        &mut self,
        context: &ManagedProviderCredentialContextV1,
        audience: LeaseAudienceV1,
        command: VaultTransportCommandV1,
    ) -> Result<Zeroizing<Vec<u8>>, ManagedProviderCredentialErrorV1> {
        let request_id = next_request_id()?;
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
        let key = VaultTransportPublicKey::from_bytes(context.vault_public_key_x25519)
            .map_err(|_| ManagedProviderCredentialErrorV1::InvalidContext)?;
        let frame = seal(&key, &request_binding, &command.encode())
            .map_err(|_| ManagedProviderCredentialErrorV1::Rejected)?;
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
        let response = self.route(route.clone())?;
        let frame = valid_response(&route, response)?;
        recipient
            .open(&response_binding, &frame)
            .map_err(|_| ManagedProviderCredentialErrorV1::Rejected)
    }

    fn route(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> Result<VaultCiphertextResponseV1, ManagedProviderCredentialErrorV1> {
        write_frame(
            &mut self.channel,
            &ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::RouteVaultCiphertext(
                    ManagedRuntimeVaultRouteRequestV1 { route: Some(route) },
                )),
            }
            .encode_to_vec(),
        )?;
        let response =
            ManagedRuntimeControlResponseV1::decode(read_frame(&mut self.channel)?.as_slice())
                .map_err(|_| ManagedProviderCredentialErrorV1::Rejected)?
                .result
                .and_then(|result| match result {
                    ControlResult::VaultRoute(response) => Some(response),
                    _ => None,
                })
                .ok_or(ManagedProviderCredentialErrorV1::Rejected)?;
        response
            .response
            .filter(|_| response.error_code.is_empty())
            .ok_or(ManagedProviderCredentialErrorV1::Rejected)
    }
}

impl<'a> ManagedProviderCredentialClientV2<'a> {
    #[must_use]
    pub fn new(channel: &'a mut ManagedControlChannelV2<UnixStream>) -> Self {
        Self { channel }
    }

    pub fn resolve(
        &mut self,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        context: &ManagedProviderCredentialContextV1,
        request: ManagedProviderCredentialRequestV1<'_>,
    ) -> Result<Zeroizing<Vec<u8>>, ManagedProviderCredentialErrorV1> {
        let audience = audience(context)?;
        let lease_id = self.issue_action_lease(
            dispatcher,
            context,
            audience.clone(),
            &request,
            VaultActionV1::Resolve,
        )?;
        self.execute_command(
            dispatcher,
            context,
            audience,
            VaultTransportCommandV1::ResolveLease {
                lease_id,
                secret_class: request.secret_class,
            },
        )
    }

    pub fn store_once(
        &mut self,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        context: &ManagedProviderCredentialContextV1,
        request: ManagedProviderCredentialRequestV1<'_>,
        payload: &[u8],
    ) -> Result<[u8; 16], ManagedProviderCredentialErrorV1> {
        let audience = audience(context)?;
        if !valid_mutation_payload(payload) {
            return Err(ManagedProviderCredentialErrorV1::InvalidContext);
        }
        let lease_id = self.issue_action_lease(
            dispatcher,
            context,
            audience.clone(),
            &request,
            VaultActionV1::Create,
        )?;
        let response = self.execute_command(
            dispatcher,
            context,
            audience,
            VaultTransportCommandV1::StoreLease {
                lease_id,
                secret_class: request.secret_class,
                payload: payload.to_vec(),
            },
        )?;
        response
            .as_slice()
            .try_into()
            .map_err(|_| ManagedProviderCredentialErrorV1::Rejected)
    }

    pub fn replace_once(
        &mut self,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        context: &ManagedProviderCredentialContextV1,
        request: ManagedProviderCredentialRequestV1<'_>,
        prior_record_id: [u8; 16],
        payload: &[u8],
    ) -> Result<[u8; 16], ManagedProviderCredentialErrorV1> {
        let audience = audience(context)?;
        if !valid_mutation_payload(payload) {
            return Err(ManagedProviderCredentialErrorV1::InvalidContext);
        }
        let lease_id = self.issue_action_lease(
            dispatcher,
            context,
            audience.clone(),
            &request,
            VaultActionV1::ReplaceCas,
        )?;
        let response = self.execute_command(
            dispatcher,
            context,
            audience,
            VaultTransportCommandV1::ReplaceLease {
                lease_id,
                secret_class: request.secret_class,
                prior_record_id,
                payload: payload.to_vec(),
            },
        )?;
        response
            .as_slice()
            .try_into()
            .map_err(|_| ManagedProviderCredentialErrorV1::Rejected)
    }

    pub fn retire_once(
        &mut self,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        context: &ManagedProviderCredentialContextV1,
        request: ManagedProviderCredentialRequestV1<'_>,
    ) -> Result<(), ManagedProviderCredentialErrorV1> {
        let audience = audience(context)?;
        let lease_id = self.issue_action_lease(
            dispatcher,
            context,
            audience.clone(),
            &request,
            VaultActionV1::Retire,
        )?;
        let response = self.execute_command(
            dispatcher,
            context,
            audience,
            VaultTransportCommandV1::RetireLease {
                lease_id,
                secret_class: request.secret_class,
            },
        )?;
        (response.as_slice() == [1])
            .then_some(())
            .ok_or(ManagedProviderCredentialErrorV1::Rejected)
    }

    pub fn delete_once(
        &mut self,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        context: &ManagedProviderCredentialContextV1,
        request: ManagedProviderCredentialRequestV1<'_>,
    ) -> Result<(), ManagedProviderCredentialErrorV1> {
        let audience = audience(context)?;
        let lease_id = self.issue_action_lease(
            dispatcher,
            context,
            audience.clone(),
            &request,
            VaultActionV1::Delete,
        )?;
        let response = self.execute_command(
            dispatcher,
            context,
            audience,
            VaultTransportCommandV1::DeleteLease {
                lease_id,
                secret_class: request.secret_class,
            },
        )?;
        (response.as_slice() == [1])
            .then_some(())
            .ok_or(ManagedProviderCredentialErrorV1::Rejected)
    }

    fn issue_action_lease(
        &mut self,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        context: &ManagedProviderCredentialContextV1,
        audience: LeaseAudienceV1,
        request: &ManagedProviderCredentialRequestV1<'_>,
        action: VaultActionV1,
    ) -> Result<LeaseIdV1, ManagedProviderCredentialErrorV1> {
        if request.credential_revision == 0 || request.ttl_seconds == 0 || request.ttl_seconds > 600
        {
            return Err(ManagedProviderCredentialErrorV1::InvalidContext);
        }
        let recipient = VaultResponseRecipientV1::generate();
        let request_id = next_request_id()?;
        let delivery = self.issue_lease(
            dispatcher,
            request_id,
            request,
            action,
            recipient.public_key().as_bytes(),
        )?;
        let issue = issue_request(context, audience.clone(), request, action)?;
        let command = VaultTransportCommandV1::IssueLease { request: issue };
        let binding = binding(
            &audience,
            context.vault_runtime_generation,
            request_id,
            &command,
            &recipient,
            VaultTransportDirectionV1::FromVault,
        )?;
        let frame = VaultCiphertextFrameV1::from_parts(
            delivery.encapped_key,
            delivery.ciphertext,
            delivery.tag,
        )
        .map_err(|_| ManagedProviderCredentialErrorV1::Rejected)?;
        let lease_id = recipient
            .open(&binding, &frame)
            .map_err(|_| ManagedProviderCredentialErrorV1::Rejected)?;
        String::from_utf8(lease_id.to_vec())
            .ok()
            .and_then(|value| LeaseIdV1::new(value).ok())
            .ok_or(ManagedProviderCredentialErrorV1::Rejected)
    }

    fn issue_lease(
        &mut self,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        request_id: [u8; 16],
        request: &ManagedProviderCredentialRequestV1<'_>,
        action: VaultActionV1,
        recipient_public_key_x25519: &[u8; 32],
    ) -> Result<
        makosh_runtime_protocol::v1::ManagedRuntimeProviderCredentialDeliveryV1,
        ManagedProviderCredentialErrorV1,
    > {
        let response = self
            .channel
            .request_next_with_dispatch(
                ManagedRuntimeControlRequestV1 {
                    operation: Some(Operation::IssueProviderCredential(
                        ManagedRuntimeProviderCredentialRequestV1 {
                            request_id: request_id.to_vec(),
                            purpose_id: request.purpose_id.to_owned(),
                            credential_revision: request.credential_revision,
                            ttl_seconds: request.ttl_seconds,
                            secret_class: request.secret_class.code() as u32,
                            recipient_public_key_x25519: recipient_public_key_x25519.to_vec(),
                            configuration_instance_id: request.configuration_instance_id.to_owned(),
                            action: action.code() as u32,
                        },
                    )),
                },
                dispatcher,
            )
            .map_err(|_| ManagedProviderCredentialErrorV1::Unavailable)?;
        match response.result {
            Some(ControlResult::ProviderCredentialDelivery(delivery))
                if response.error_code.is_empty() =>
            {
                Ok(delivery)
            }
            _ => Err(ManagedProviderCredentialErrorV1::Rejected),
        }
    }

    fn execute_command(
        &mut self,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        context: &ManagedProviderCredentialContextV1,
        audience: LeaseAudienceV1,
        command: VaultTransportCommandV1,
    ) -> Result<Zeroizing<Vec<u8>>, ManagedProviderCredentialErrorV1> {
        let request_id = next_request_id()?;
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
        let key = VaultTransportPublicKey::from_bytes(context.vault_public_key_x25519)
            .map_err(|_| ManagedProviderCredentialErrorV1::InvalidContext)?;
        let frame = seal(&key, &request_binding, &command.encode())
            .map_err(|_| ManagedProviderCredentialErrorV1::Rejected)?;
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
        let response = self.route(dispatcher, route.clone())?;
        let frame = valid_response(&route, response)?;
        recipient
            .open(&response_binding, &frame)
            .map_err(|_| ManagedProviderCredentialErrorV1::Rejected)
    }

    fn route(
        &mut self,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
        route: VaultCiphertextRouteV1,
    ) -> Result<VaultCiphertextResponseV1, ManagedProviderCredentialErrorV1> {
        let response = self
            .channel
            .request_next_with_dispatch(
                ManagedRuntimeControlRequestV1 {
                    operation: Some(Operation::RouteVaultCiphertext(
                        ManagedRuntimeVaultRouteRequestV1 { route: Some(route) },
                    )),
                },
                dispatcher,
            )
            .map_err(|_| ManagedProviderCredentialErrorV1::Unavailable)?
            .result
            .and_then(|result| match result {
                ControlResult::VaultRoute(response) => Some(response),
                _ => None,
            })
            .ok_or(ManagedProviderCredentialErrorV1::Rejected)?;
        response
            .response
            .filter(|_| response.error_code.is_empty())
            .ok_or(ManagedProviderCredentialErrorV1::Rejected)
    }
}

fn audience(
    context: &ManagedProviderCredentialContextV1,
) -> Result<LeaseAudienceV1, ManagedProviderCredentialErrorV1> {
    if context.vault_runtime_generation == 0
        || context.runtime_generation == 0
        || context.grant_epoch == 0
    {
        return Err(ManagedProviderCredentialErrorV1::InvalidContext);
    }
    LeaseAudienceV1::new(
        context.registration_id.clone(),
        context.runtime_instance_id.clone(),
        context.runtime_generation,
        context.grant_epoch,
    )
    .map_err(|_| ManagedProviderCredentialErrorV1::InvalidContext)
}

fn valid_mutation_payload(payload: &[u8]) -> bool {
    !payload.is_empty() && payload.len() <= MAX_FRAME_BYTES
}

fn issue_request(
    context: &ManagedProviderCredentialContextV1,
    audience: LeaseAudienceV1,
    request: &ManagedProviderCredentialRequestV1<'_>,
    action: VaultActionV1,
) -> Result<VaultLeaseIssueRequestV1, ManagedProviderCredentialErrorV1> {
    let purpose = VaultPurposeRequestV1::new(
        request.purpose_id.to_owned(),
        request.configuration_instance_id.to_owned(),
        vec![request.secret_class],
        vec![action],
        request.ttl_seconds,
    )
    .map_err(|_| ManagedProviderCredentialErrorV1::InvalidContext)?;
    VaultLeaseIssueRequestV1::new(
        context.vault_instance_id.clone(),
        context.vault_runtime_generation,
        request.credential_revision,
        context.logical_owner_id.clone(),
        purpose,
        audience,
    )
    .map_err(|_| ManagedProviderCredentialErrorV1::InvalidContext)
}

pub(crate) fn binding(
    audience: &LeaseAudienceV1,
    vault_runtime_generation: u64,
    request_id: [u8; 16],
    command: &VaultTransportCommandV1,
    recipient: &VaultResponseRecipientV1,
    direction: VaultTransportDirectionV1,
) -> Result<VaultTransportBindingV1, ManagedProviderCredentialErrorV1> {
    VaultTransportBindingV1::new(
        vault_runtime_generation,
        audience.clone(),
        request_id,
        command.operation_digest(),
        direction,
        *recipient.public_key().as_bytes(),
    )
    .map_err(|_| ManagedProviderCredentialErrorV1::Rejected)
}

pub(crate) fn valid_response(
    route: &VaultCiphertextRouteV1,
    response: VaultCiphertextResponseV1,
) -> Result<VaultCiphertextFrameV1, ManagedProviderCredentialErrorV1> {
    if response.major != 1
        || response.vault_runtime_generation != route.vault_runtime_generation
        || response.caller_runtime_generation != route.caller_runtime_generation
        || response.request_id != route.request_id
        || response.operation_digest_sha256 != route.operation_digest_sha256
        || response.direction != VaultCiphertextRouteDirectionV1::FromVault as i32
    {
        return Err(ManagedProviderCredentialErrorV1::Rejected);
    }
    VaultCiphertextFrameV1::from_parts(
        response.hpke_encapped_key,
        response.ciphertext,
        response.hpke_authentication_tag,
    )
    .map_err(|_| ManagedProviderCredentialErrorV1::Rejected)
}

pub(crate) fn write_frame(
    channel: &mut UnixStream,
    bytes: &[u8],
) -> Result<(), ManagedProviderCredentialErrorV1> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES {
        return Err(ManagedProviderCredentialErrorV1::Rejected);
    }
    let mut value =
        u32::try_from(bytes.len()).map_err(|_| ManagedProviderCredentialErrorV1::Rejected)?;
    let mut prefix = Vec::with_capacity(5);
    while value >= 0x80 {
        prefix.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    prefix.push(value as u8);
    channel
        .write_all(&prefix)
        .and_then(|_| channel.write_all(bytes))
        .and_then(|_| channel.flush())
        .map_err(|_| ManagedProviderCredentialErrorV1::Unavailable)
}

pub(crate) fn read_frame(
    channel: &mut UnixStream,
) -> Result<Vec<u8>, ManagedProviderCredentialErrorV1> {
    let mut value = 0_u64;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        channel
            .read_exact(&mut byte)
            .map_err(|_| ManagedProviderCredentialErrorV1::Unavailable)?;
        value |= u64::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 {
            let length =
                usize::try_from(value).map_err(|_| ManagedProviderCredentialErrorV1::Rejected)?;
            if length == 0 || length > MAX_FRAME_BYTES {
                return Err(ManagedProviderCredentialErrorV1::Rejected);
            }
            let mut frame = vec![0_u8; length];
            channel
                .read_exact(&mut frame)
                .map_err(|_| ManagedProviderCredentialErrorV1::Unavailable)?;
            return Ok(frame);
        }
    }
    Err(ManagedProviderCredentialErrorV1::Rejected)
}

pub(crate) fn next_request_id() -> Result<[u8; 16], ManagedProviderCredentialErrorV1> {
    next_vault_transport_request_id_v1().ok_or(ManagedProviderCredentialErrorV1::Unavailable)
}

#[cfg(test)]
mod tests {
    use std::os::unix::net::UnixStream;
    use std::thread;

    use makosh_runtime_protocol::managed_control::{
        ManagedControlChannelV2, RejectManagedControlRequestsV2,
    };
    use makosh_runtime_protocol::v1::{
        ManagedRuntimeControlResponseV1, ManagedRuntimeProviderCredentialDeliveryV1,
        ManagedRuntimeReadyRequestV1, managed_runtime_control_frame_v2::Frame,
        managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    };
    use makosh_runtime_protocol::validation::managed_control::{
        MANAGED_CONTROL_CORRELATION_ID_BYTES, MANAGED_CONTROL_TRANSPORT_MAJOR_V2,
    };
    use makosh_vault_protocol::{SecretClassV1, VaultActionV1};

    use super::{
        MAX_FRAME_BYTES, ManagedProviderCredentialClientV2, ManagedProviderCredentialErrorV1,
        ManagedProviderCredentialRequestV1, valid_mutation_payload,
    };

    #[test]
    fn correlated_provider_credential_actions_dispatch_an_opposite_direction_request() {
        for action in [
            VaultActionV1::Resolve,
            VaultActionV1::Create,
            VaultActionV1::ReplaceCas,
            VaultActionV1::Retire,
            VaultActionV1::Delete,
        ] {
            assert_correlated_provider_credential_action(action);
        }
    }

    fn assert_correlated_provider_credential_action(action: VaultActionV1) {
        let (client, server) = UnixStream::pair().expect("control pair");
        let server = thread::spawn(move || {
            let mut channel = ManagedControlChannelV2::new(server);
            let (provider_request_id, provider_request) =
                channel.receive_request().expect("provider request");
            assert!(matches!(
                provider_request.operation,
                Some(Operation::IssueProviderCredential(request))
                    if request.action == action.code() as u32
            ));
            channel
                .write_request(
                    [9; MANAGED_CONTROL_CORRELATION_ID_BYTES],
                    makosh_runtime_protocol::v1::ManagedRuntimeControlRequestV1 {
                        operation: Some(Operation::Ready(ManagedRuntimeReadyRequestV1::default())),
                    },
                )
                .expect("opposite request");
            let response = channel.read_frame().expect("opposite response");
            assert_eq!(response.transport_major, MANAGED_CONTROL_TRANSPORT_MAJOR_V2);
            assert_eq!(
                response.correlation_id,
                vec![9; MANAGED_CONTROL_CORRELATION_ID_BYTES]
            );
            assert!(matches!(response.frame, Some(Frame::Response(_))));
            channel
                .write_response(
                    provider_request_id,
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::ProviderCredentialDelivery(
                            ManagedRuntimeProviderCredentialDeliveryV1::default(),
                        )),
                        error_code: String::new(),
                    },
                )
                .expect("provider response");
        });

        let mut channel = ManagedControlChannelV2::new(client);
        let mut dispatcher = RejectManagedControlRequestsV2;
        let mut client = ManagedProviderCredentialClientV2::new(&mut channel);
        client
            .issue_lease(
                &mut dispatcher,
                [1; 16],
                &ManagedProviderCredentialRequestV1 {
                    configuration_instance_id: "telegram-account",
                    purpose_id: "telegram.api_hash",
                    credential_revision: 1,
                    ttl_seconds: 60,
                    secret_class: SecretClassV1::ProviderCredential,
                },
                action,
                &[7; 32],
            )
            .expect("correlated provider delivery");
        server.join().expect("server join");
    }

    #[test]
    fn correlated_credential_mutations_reject_unbounded_payloads_before_control_io() {
        assert!(!valid_mutation_payload(&[]));
        assert!(valid_mutation_payload(&[1]));
        assert!(valid_mutation_payload(&vec![1; MAX_FRAME_BYTES]));
        assert!(!valid_mutation_payload(&vec![1; MAX_FRAME_BYTES + 1]));

        let (client, _server) = UnixStream::pair().expect("control pair");
        let mut channel = ManagedControlChannelV2::new(client);
        let mut dispatcher = RejectManagedControlRequestsV2;
        let mut client = ManagedProviderCredentialClientV2::new(&mut channel);
        let context = super::ManagedProviderCredentialContextV1 {
            vault_instance_id: "vault-runtime".to_owned(),
            vault_runtime_generation: 1,
            vault_public_key_x25519: [7; 32],
            logical_owner_id: "owner-1".to_owned(),
            registration_id: "mail-registration".to_owned(),
            runtime_instance_id: "mail-runtime".to_owned(),
            runtime_generation: 1,
            grant_epoch: 1,
        };
        let request = || ManagedProviderCredentialRequestV1 {
            configuration_instance_id: "mail-account",
            purpose_id: "mail.oauth",
            credential_revision: 1,
            ttl_seconds: 60,
            secret_class: SecretClassV1::ProviderCredential,
        };

        assert_eq!(
            client.store_once(&mut dispatcher, &context, request(), &[]),
            Err(ManagedProviderCredentialErrorV1::InvalidContext)
        );
        assert_eq!(
            client.replace_once(
                &mut dispatcher,
                &context,
                request(),
                [1; 16],
                &vec![1; MAX_FRAME_BYTES + 1],
            ),
            Err(ManagedProviderCredentialErrorV1::InvalidContext)
        );
    }
}
