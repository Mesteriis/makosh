//! Account-signing seed storage scoped to the isolated Events authority.

use makosh_vault_protocol::{
    DEFAULT_LEASE_TTL_SECONDS, LeaseAudienceV1, LeaseIdV1, SecretClassV1, VaultActionV1,
    VaultLeaseIssueRequestV1, VaultPurposeRequestV1, VaultTransportCommandV1,
};
use zeroize::Zeroizing;

use super::{NatsVaultRouteContextV1, NatsVaultRouteFailureV1, NatsVaultRoutePortV1, session};

const ACCOUNT_SIGNER_OWNER: &str = "events";
const ACCOUNT_SIGNER_PURPOSE: &str = "events.nats.account_signer";

/// Fences the account signer to the one verified Events authority runtime.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NatsAccountSignerFenceV1 {
    registration_id: String,
    authority_instance_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
    credential_revision: u64,
}

impl NatsAccountSignerFenceV1 {
    pub fn new(
        registration_id: impl Into<String>,
        authority_instance_id: impl Into<String>,
        runtime_generation: u64,
        grant_epoch: u64,
        credential_revision: u64,
    ) -> Result<Self, NatsAccountSignerLeaseErrorV1> {
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
            .ok_or(NatsAccountSignerLeaseErrorV1::Rejected)
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

/// Stores and resolves the account signer only through the ciphertext Vault route.
pub struct NatsAccountSignerLeaseAdapterV1<T> {
    route_port: T,
    context: NatsVaultRouteContextV1,
}

impl<T> NatsAccountSignerLeaseAdapterV1<T>
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

    pub async fn enroll_once(
        &mut self,
        fence: &NatsAccountSignerFenceV1,
        account_signing_seed: Zeroizing<Vec<u8>>,
    ) -> Result<(), NatsAccountSignerLeaseErrorV1> {
        if account_signing_seed.is_empty() {
            return Err(NatsAccountSignerLeaseErrorV1::Rejected);
        }
        let lease_id = self.issue(fence, VaultActionV1::Create).await?;
        let record_id = self
            .execute(
                fence,
                VaultTransportCommandV1::StoreLease {
                    lease_id,
                    secret_class: SecretClassV1::PlatformCredential,
                    payload: account_signing_seed.to_vec(),
                },
            )
            .await?;
        (record_id.len() == 16 && record_id.iter().any(|byte| *byte != 0))
            .then_some(())
            .ok_or(NatsAccountSignerLeaseErrorV1::Rejected)
    }

    pub async fn resolve_once(
        &mut self,
        fence: &NatsAccountSignerFenceV1,
    ) -> Result<Zeroizing<Vec<u8>>, NatsAccountSignerLeaseErrorV1> {
        let lease_id = self.issue(fence, VaultActionV1::Resolve).await?;
        let credential = self
            .execute(
                fence,
                VaultTransportCommandV1::ResolveLease {
                    lease_id,
                    secret_class: SecretClassV1::PlatformCredential,
                },
            )
            .await?;
        (!credential.is_empty())
            .then_some(credential)
            .ok_or(NatsAccountSignerLeaseErrorV1::Rejected)
    }

    pub async fn revoke_leases(
        &mut self,
        fence: &NatsAccountSignerFenceV1,
    ) -> Result<(), NatsAccountSignerLeaseErrorV1> {
        let result = self
            .execute(fence, VaultTransportCommandV1::RevokeAudience)
            .await?;
        (result.as_slice() == [1])
            .then_some(())
            .ok_or(NatsAccountSignerLeaseErrorV1::Rejected)
    }

    async fn issue(
        &mut self,
        fence: &NatsAccountSignerFenceV1,
        action: VaultActionV1,
    ) -> Result<LeaseIdV1, NatsAccountSignerLeaseErrorV1> {
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
            .ok_or(NatsAccountSignerLeaseErrorV1::Rejected)
    }

    async fn execute(
        &mut self,
        fence: &NatsAccountSignerFenceV1,
        command: VaultTransportCommandV1,
    ) -> Result<Zeroizing<Vec<u8>>, NatsAccountSignerLeaseErrorV1> {
        let audience = LeaseAudienceV1::new(
            fence.registration_id.clone(),
            fence.authority_instance_id.clone(),
            fence.runtime_generation,
            fence.grant_epoch,
        )
        .map_err(|_| NatsAccountSignerLeaseErrorV1::Rejected)?;
        let prepared = session::prepare(audience, &self.context, &command)
            .map_err(|_| NatsAccountSignerLeaseErrorV1::Rejected)?;
        session::execute(&mut self.route_port, prepared)
            .await
            .map_err(map_route_error)
    }
}

fn issue_request(
    fence: &NatsAccountSignerFenceV1,
    context: &NatsVaultRouteContextV1,
    action: VaultActionV1,
) -> Result<VaultLeaseIssueRequestV1, NatsAccountSignerLeaseErrorV1> {
    let purpose = VaultPurposeRequestV1::new(
        ACCOUNT_SIGNER_PURPOSE.to_owned(),
        fence.authority_instance_id.clone(),
        vec![SecretClassV1::PlatformCredential],
        vec![action],
        DEFAULT_LEASE_TTL_SECONDS,
    )
    .map_err(|_| NatsAccountSignerLeaseErrorV1::Rejected)?;
    VaultLeaseIssueRequestV1::new(
        context.vault_instance_id().to_owned(),
        context.vault_runtime_generation(),
        fence.credential_revision,
        ACCOUNT_SIGNER_OWNER.to_owned(),
        purpose,
        LeaseAudienceV1::new(
            fence.registration_id.clone(),
            fence.authority_instance_id.clone(),
            fence.runtime_generation,
            fence.grant_epoch,
        )
        .map_err(|_| NatsAccountSignerLeaseErrorV1::Rejected)?,
    )
    .map_err(|_| NatsAccountSignerLeaseErrorV1::Rejected)
}

const fn map_route_error(error: NatsVaultRouteFailureV1) -> NatsAccountSignerLeaseErrorV1 {
    match error {
        NatsVaultRouteFailureV1::Rejected => NatsAccountSignerLeaseErrorV1::Rejected,
        NatsVaultRouteFailureV1::Unavailable => NatsAccountSignerLeaseErrorV1::Unavailable,
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
pub enum NatsAccountSignerLeaseErrorV1 {
    Rejected,
    Unavailable,
}
