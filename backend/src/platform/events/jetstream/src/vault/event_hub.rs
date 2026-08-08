//! Vault-issued Event Hub administration identity for JetStream reconciliation.

use makosh_vault_protocol::{
    DEFAULT_LEASE_TTL_SECONDS, LeaseAudienceV1, LeaseIdV1, SecretClassV1, VaultActionV1,
    VaultLeaseIssueRequestV1, VaultPurposeRequestV1, VaultTransportCommandV1,
};
use zeroize::Zeroizing;

use crate::NatsPasswordCredentialV1;

use super::{NatsVaultRouteContextV1, NatsVaultRouteFailureV1, NatsVaultRoutePortV1, session};

const EVENT_HUB_OWNER: &str = "kernel";
const EVENT_HUB_CREDENTIAL_PURPOSE: &str = "events.nats.event_hub.credential";

/// Reserved Kernel platform principal; it is not a module registration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventHubCredentialFenceV1 {
    registration_id: String,
    runtime_instance_id: String,
    event_hub_instance_id: String,
    runtime_generation: u64,
    control_epoch: u64,
    credential_revision: u64,
    nats_username: String,
}

impl EventHubCredentialFenceV1 {
    pub fn new(
        registration_id: impl Into<String>,
        runtime_instance_id: impl Into<String>,
        event_hub_instance_id: impl Into<String>,
        runtime_generation: u64,
        control_epoch: u64,
        credential_revision: u64,
        nats_username: impl Into<String>,
    ) -> Result<Self, EventHubCredentialLeaseErrorV1> {
        let registration_id = registration_id.into();
        let runtime_instance_id = runtime_instance_id.into();
        let event_hub_instance_id = event_hub_instance_id.into();
        let nats_username = nats_username.into();
        (valid_id(&registration_id)
            && valid_id(&runtime_instance_id)
            && valid_id(&event_hub_instance_id)
            && valid_id(&nats_username)
            && runtime_generation > 0
            && control_epoch > 0
            && credential_revision > 0)
            .then_some(Self {
                registration_id,
                runtime_instance_id,
                event_hub_instance_id,
                runtime_generation,
                control_epoch,
                credential_revision,
                nats_username,
            })
            .ok_or(EventHubCredentialLeaseErrorV1::Rejected)
    }
}

/// Resolves the pre-seeded Event Hub credential only through the encrypted Vault route.
pub struct EventHubCredentialLeaseAdapterV1<T> {
    route_port: T,
    context: NatsVaultRouteContextV1,
}

impl<T> EventHubCredentialLeaseAdapterV1<T>
where
    T: NatsVaultRoutePortV1 + Send,
{
    #[must_use]
    pub fn new(route_port: T, context: NatsVaultRouteContextV1) -> Self {
        Self {
            route_port,
            context,
        }
    }

    #[must_use]
    pub fn into_route_port(self) -> T {
        self.route_port
    }

    pub async fn resolve_event_hub_identity(
        &mut self,
        fence: &EventHubCredentialFenceV1,
    ) -> Result<NatsPasswordCredentialV1, EventHubCredentialLeaseErrorV1> {
        let lease_id = self.issue(fence, VaultActionV1::Resolve).await?;
        let password = self
            .execute(
                fence,
                VaultTransportCommandV1::ResolveLease {
                    lease_id,
                    secret_class: SecretClassV1::PlatformCredential,
                },
            )
            .await?;
        identity(fence, password)
    }

    pub async fn revoke_event_hub_leases(
        &mut self,
        fence: &EventHubCredentialFenceV1,
    ) -> Result<(), EventHubCredentialLeaseErrorV1> {
        let result = self
            .execute(fence, VaultTransportCommandV1::RevokeAudience)
            .await?;
        (result.as_slice() == [1])
            .then_some(())
            .ok_or(EventHubCredentialLeaseErrorV1::Rejected)
    }

    async fn issue(
        &mut self,
        fence: &EventHubCredentialFenceV1,
        action: VaultActionV1,
    ) -> Result<LeaseIdV1, EventHubCredentialLeaseErrorV1> {
        let response = self
            .execute(
                fence,
                VaultTransportCommandV1::IssueLease {
                    request: issue_request(fence, &self.context, action)?,
                },
            )
            .await?;
        String::from_utf8(response.to_vec())
            .ok()
            .and_then(|value| LeaseIdV1::new(value).ok())
            .ok_or(EventHubCredentialLeaseErrorV1::Rejected)
    }

    async fn execute(
        &mut self,
        fence: &EventHubCredentialFenceV1,
        command: VaultTransportCommandV1,
    ) -> Result<Zeroizing<Vec<u8>>, EventHubCredentialLeaseErrorV1> {
        let prepared = session::prepare(audience_from_fence(fence)?, &self.context, &command)
            .map_err(|_| EventHubCredentialLeaseErrorV1::Rejected)?;
        session::execute(&mut self.route_port, prepared)
            .await
            .map_err(map_route_error)
    }
}

fn issue_request(
    fence: &EventHubCredentialFenceV1,
    context: &NatsVaultRouteContextV1,
    action: VaultActionV1,
) -> Result<VaultLeaseIssueRequestV1, EventHubCredentialLeaseErrorV1> {
    let purpose = VaultPurposeRequestV1::new(
        EVENT_HUB_CREDENTIAL_PURPOSE.to_owned(),
        fence.event_hub_instance_id.clone(),
        vec![SecretClassV1::PlatformCredential],
        vec![action],
        DEFAULT_LEASE_TTL_SECONDS,
    )
    .map_err(|_| EventHubCredentialLeaseErrorV1::Rejected)?;
    VaultLeaseIssueRequestV1::new(
        context.vault_instance_id().to_owned(),
        context.vault_runtime_generation(),
        fence.credential_revision,
        EVENT_HUB_OWNER.to_owned(),
        purpose,
        audience_from_fence(fence)?,
    )
    .map_err(|_| EventHubCredentialLeaseErrorV1::Rejected)
}

fn audience_from_fence(
    fence: &EventHubCredentialFenceV1,
) -> Result<LeaseAudienceV1, EventHubCredentialLeaseErrorV1> {
    LeaseAudienceV1::new(
        fence.registration_id.clone(),
        fence.runtime_instance_id.clone(),
        fence.runtime_generation,
        fence.control_epoch,
    )
    .map_err(|_| EventHubCredentialLeaseErrorV1::Rejected)
}

fn identity(
    fence: &EventHubCredentialFenceV1,
    password: Zeroizing<Vec<u8>>,
) -> Result<NatsPasswordCredentialV1, EventHubCredentialLeaseErrorV1> {
    String::from_utf8(password.to_vec())
        .ok()
        .and_then(|password| {
            NatsPasswordCredentialV1::new(fence.nats_username.clone(), password).ok()
        })
        .ok_or(EventHubCredentialLeaseErrorV1::Rejected)
}

const fn map_route_error(error: NatsVaultRouteFailureV1) -> EventHubCredentialLeaseErrorV1 {
    match error {
        NatsVaultRouteFailureV1::Rejected => EventHubCredentialLeaseErrorV1::Rejected,
        NatsVaultRouteFailureV1::Unavailable => EventHubCredentialLeaseErrorV1::Unavailable,
    }
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventHubCredentialLeaseErrorV1 {
    Rejected,
    Unavailable,
}
