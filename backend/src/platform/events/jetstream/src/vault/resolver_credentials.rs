//! System resolver credentials resolved only by the Events authority.

use makosh_vault_protocol::{
    DEFAULT_LEASE_TTL_SECONDS, LeaseAudienceV1, LeaseIdV1, SecretClassV1, VaultActionV1,
    VaultLeaseIssueRequestV1, VaultPurposeRequestV1, VaultTransportCommandV1,
};

use crate::resolver::NatsResolverSystemCredentialsV1;

use super::{NatsVaultRouteContextV1, NatsVaultRouteFailureV1, NatsVaultRoutePortV1, session};

const RESOLVER_CREDENTIAL_OWNER: &str = "events";
const RESOLVER_CREDENTIAL_PURPOSE: &str = "events.nats.resolver.system_credential";

/// Fences a System Account credential to one verified Events authority runtime.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NatsResolverCredentialFenceV1 {
    registration_id: String,
    authority_instance_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
    credential_revision: u64,
}

impl NatsResolverCredentialFenceV1 {
    pub fn new(
        registration_id: impl Into<String>,
        authority_instance_id: impl Into<String>,
        runtime_generation: u64,
        grant_epoch: u64,
        credential_revision: u64,
    ) -> Result<Self, NatsResolverCredentialLeaseErrorV1> {
        let registration_id = registration_id.into();
        let authority_instance_id = authority_instance_id.into();
        (valid_id(&registration_id)
            && valid_id(&authority_instance_id)
            && runtime_generation > 0
            && grant_epoch > 0
            && credential_revision > 0)
            .then_some(Self {
                registration_id,
                authority_instance_id,
                runtime_generation,
                grant_epoch,
                credential_revision,
            })
            .ok_or(NatsResolverCredentialLeaseErrorV1::Rejected)
    }

    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }

    #[must_use]
    pub fn authority_instance_id(&self) -> &str {
        &self.authority_instance_id
    }

    #[must_use]
    pub const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }

    #[must_use]
    pub const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }

    #[must_use]
    pub const fn credential_revision(&self) -> u64 {
        self.credential_revision
    }
}

/// Resolves a bounded System Account `.creds` document through the Vault route.
pub struct NatsResolverCredentialLeaseAdapterV1<T> {
    route_port: T,
    context: NatsVaultRouteContextV1,
}

impl<T> NatsResolverCredentialLeaseAdapterV1<T>
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

    pub async fn resolve_system_credentials(
        &mut self,
        fence: &NatsResolverCredentialFenceV1,
    ) -> Result<NatsResolverSystemCredentialsV1, NatsResolverCredentialLeaseErrorV1> {
        let lease_id = self.issue(fence).await?;
        let document = self
            .execute(
                fence,
                VaultTransportCommandV1::ResolveLease {
                    lease_id,
                    secret_class: SecretClassV1::PlatformCredential,
                },
            )
            .await?;
        let document = String::from_utf8(document.to_vec())
            .map_err(|_| NatsResolverCredentialLeaseErrorV1::Rejected)?;
        NatsResolverSystemCredentialsV1::new(document)
            .map_err(|_| NatsResolverCredentialLeaseErrorV1::Rejected)
    }

    async fn issue(
        &mut self,
        fence: &NatsResolverCredentialFenceV1,
    ) -> Result<LeaseIdV1, NatsResolverCredentialLeaseErrorV1> {
        let response = self
            .execute(
                fence,
                VaultTransportCommandV1::IssueLease {
                    request: issue_request(fence, &self.context)?,
                },
            )
            .await?;
        String::from_utf8(response.to_vec())
            .ok()
            .and_then(|value| LeaseIdV1::new(value).ok())
            .ok_or(NatsResolverCredentialLeaseErrorV1::Rejected)
    }

    async fn execute(
        &mut self,
        fence: &NatsResolverCredentialFenceV1,
        command: VaultTransportCommandV1,
    ) -> Result<zeroize::Zeroizing<Vec<u8>>, NatsResolverCredentialLeaseErrorV1> {
        let prepared = session::prepare(audience(fence)?, &self.context, &command)
            .map_err(|_| NatsResolverCredentialLeaseErrorV1::Rejected)?;
        session::execute(&mut self.route_port, prepared)
            .await
            .map_err(map_route_error)
    }
}

fn issue_request(
    fence: &NatsResolverCredentialFenceV1,
    context: &NatsVaultRouteContextV1,
) -> Result<VaultLeaseIssueRequestV1, NatsResolverCredentialLeaseErrorV1> {
    let purpose = VaultPurposeRequestV1::new(
        RESOLVER_CREDENTIAL_PURPOSE.to_owned(),
        fence.authority_instance_id.clone(),
        vec![SecretClassV1::PlatformCredential],
        vec![VaultActionV1::Resolve],
        DEFAULT_LEASE_TTL_SECONDS,
    )
    .map_err(|_| NatsResolverCredentialLeaseErrorV1::Rejected)?;
    VaultLeaseIssueRequestV1::new(
        context.vault_instance_id().to_owned(),
        context.vault_runtime_generation(),
        fence.credential_revision,
        RESOLVER_CREDENTIAL_OWNER.to_owned(),
        purpose,
        audience(fence)?,
    )
    .map_err(|_| NatsResolverCredentialLeaseErrorV1::Rejected)
}

fn audience(
    fence: &NatsResolverCredentialFenceV1,
) -> Result<LeaseAudienceV1, NatsResolverCredentialLeaseErrorV1> {
    LeaseAudienceV1::new(
        fence.registration_id.clone(),
        fence.authority_instance_id.clone(),
        fence.runtime_generation,
        fence.grant_epoch,
    )
    .map_err(|_| NatsResolverCredentialLeaseErrorV1::Rejected)
}

const fn map_route_error(error: NatsVaultRouteFailureV1) -> NatsResolverCredentialLeaseErrorV1 {
    match error {
        NatsVaultRouteFailureV1::Rejected => NatsResolverCredentialLeaseErrorV1::Rejected,
        NatsVaultRouteFailureV1::Unavailable => NatsResolverCredentialLeaseErrorV1::Unavailable,
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
pub enum NatsResolverCredentialLeaseErrorV1 {
    Rejected,
    Unavailable,
}
