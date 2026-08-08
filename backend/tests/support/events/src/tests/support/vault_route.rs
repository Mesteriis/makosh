//! Deterministic encrypted Vault route responses for Events tests.

use std::collections::VecDeque;
use std::future::{Future, ready};

use makosh_events_jetstream::{NatsVaultRouteFailureV1, NatsVaultRoutePortV1};
use makosh_runtime_protocol::v1::{
    VaultCiphertextResponseV1, VaultCiphertextRouteDirectionV1, VaultCiphertextRouteV1,
};
use makosh_vault_protocol::{
    LeaseAudienceV1, VaultTransportBindingV1, VaultTransportDirectionV1, VaultTransportPublicKey,
    seal,
};

pub(crate) struct ScriptedVaultRoute {
    responses: VecDeque<Result<Vec<u8>, NatsVaultRouteFailureV1>>,
    pub(crate) routes: Vec<VaultCiphertextRouteV1>,
}

impl ScriptedVaultRoute {
    pub(crate) fn new(responses: Vec<Result<Vec<u8>, NatsVaultRouteFailureV1>>) -> Self {
        Self {
            responses: responses.into(),
            routes: Vec::new(),
        }
    }
}

impl NatsVaultRoutePortV1 for ScriptedVaultRoute {
    fn route_vault_ciphertext(
        &mut self,
        route: VaultCiphertextRouteV1,
    ) -> impl Future<Output = Result<VaultCiphertextResponseV1, NatsVaultRouteFailureV1>> + Send
    {
        let response = self
            .responses
            .pop_front()
            .expect("expected Vault route response");
        self.routes.push(route.clone());
        ready(response.and_then(|payload| encrypted_response(&route, payload)))
    }
}

pub(crate) fn encrypted_response(
    route: &VaultCiphertextRouteV1,
    payload: Vec<u8>,
) -> Result<VaultCiphertextResponseV1, NatsVaultRouteFailureV1> {
    let audience = LeaseAudienceV1::new(
        route.registration_id.clone(),
        route.runtime_instance_id.clone(),
        route.caller_runtime_generation,
        route.grant_epoch,
    )
    .map_err(|_| NatsVaultRouteFailureV1::Rejected)?;
    let recipient = route
        .response_recipient_hpke_public_key_x25519
        .as_slice()
        .try_into()
        .map_err(|_| NatsVaultRouteFailureV1::Rejected)
        .and_then(|value| {
            VaultTransportPublicKey::from_bytes(value)
                .map_err(|_| NatsVaultRouteFailureV1::Rejected)
        })?;
    let binding = VaultTransportBindingV1::new(
        route.vault_runtime_generation,
        audience,
        route
            .request_id
            .as_slice()
            .try_into()
            .map_err(|_| NatsVaultRouteFailureV1::Rejected)?,
        route
            .operation_digest_sha256
            .as_slice()
            .try_into()
            .map_err(|_| NatsVaultRouteFailureV1::Rejected)?,
        VaultTransportDirectionV1::FromVault,
        *recipient.as_bytes(),
    )
    .map_err(|_| NatsVaultRouteFailureV1::Rejected)?;
    let frame =
        seal(&recipient, &binding, &payload).map_err(|_| NatsVaultRouteFailureV1::Rejected)?;
    Ok(VaultCiphertextResponseV1 {
        major: 1,
        vault_runtime_generation: route.vault_runtime_generation,
        request_id: route.request_id.clone(),
        operation_digest_sha256: route.operation_digest_sha256.clone(),
        direction: VaultCiphertextRouteDirectionV1::FromVault as i32,
        hpke_encapped_key: frame.encapped_key().to_vec(),
        ciphertext: frame.ciphertext().to_vec(),
        hpke_authentication_tag: frame.tag().to_vec(),
        caller_runtime_generation: route.caller_runtime_generation,
    })
}
