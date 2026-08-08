//! Per-runtime broker credential creation and resolution through Vault leases.

use makosh_vault_protocol::{
    DEFAULT_LEASE_TTL_SECONDS, LeaseAudienceV1, LeaseIdV1, SecretClassV1, VaultActionV1,
    VaultLeaseIssueRequestV1, VaultPurposeRequestV1, VaultTransportCommandV1,
};
use zeroize::Zeroizing;

use super::{NatsVaultRouteContextV1, NatsVaultRouteFailureV1, NatsVaultRoutePortV1, session};

const NATS_CREDENTIAL_PURPOSE: &str = "events.nats.runtime.credential";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NatsRuntimeCredentialFenceV1 {
    logical_owner_id: String,
    registration_id: String,
    runtime_instance_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
    credential_revision: u64,
}

impl NatsRuntimeCredentialFenceV1 {
    pub fn new(
        logical_owner_id: impl Into<String>,
        registration_id: impl Into<String>,
        runtime_instance_id: impl Into<String>,
        runtime_generation: u64,
        grant_epoch: u64,
        credential_revision: u64,
    ) -> Result<Self, NatsCredentialLeaseErrorV1> {
        let logical_owner_id = logical_owner_id.into();
        let registration_id = registration_id.into();
        let runtime_instance_id = runtime_instance_id.into();
        (valid_id(&logical_owner_id)
            && valid_id(&registration_id)
            && valid_id(&runtime_instance_id)
            && runtime_generation > 0
            && grant_epoch > 0
            && credential_revision > 0)
            .then_some(Self {
                logical_owner_id,
                registration_id,
                runtime_instance_id,
                runtime_generation,
                grant_epoch,
                credential_revision,
            })
            .ok_or(NatsCredentialLeaseErrorV1::Rejected)
    }

    #[must_use]
    pub fn logical_owner_id(&self) -> &str {
        &self.logical_owner_id
    }

    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }

    #[must_use]
    pub fn runtime_instance_id(&self) -> &str {
        &self.runtime_instance_id
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

pub struct NatsCredentialLeaseAdapterV1<T> {
    route_port: T,
    context: NatsVaultRouteContextV1,
}

impl<T> NatsCredentialLeaseAdapterV1<T>
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

    pub async fn ensure_runtime_credential(
        &mut self,
        fence: &NatsRuntimeCredentialFenceV1,
    ) -> Result<Zeroizing<Vec<u8>>, NatsCredentialLeaseErrorV1> {
        match self.resolve(fence).await {
            Ok(credential) => Ok(credential),
            Err(NatsCredentialLeaseErrorV1::Rejected) => self.create_and_resolve(fence).await,
            Err(NatsCredentialLeaseErrorV1::Unavailable) => {
                Err(NatsCredentialLeaseErrorV1::Unavailable)
            }
        }
    }

    pub async fn revoke_runtime_leases(
        &mut self,
        fence: &NatsRuntimeCredentialFenceV1,
    ) -> Result<(), NatsCredentialLeaseErrorV1> {
        let result = self
            .execute(fence, VaultTransportCommandV1::RevokeAudience)
            .await?;
        (result.as_slice() == [1])
            .then_some(())
            .ok_or(NatsCredentialLeaseErrorV1::Rejected)
    }

    async fn create_and_resolve(
        &mut self,
        fence: &NatsRuntimeCredentialFenceV1,
    ) -> Result<Zeroizing<Vec<u8>>, NatsCredentialLeaseErrorV1> {
        let lease_id = self.issue(fence, VaultActionV1::Create).await?;
        let record_id = self
            .execute(
                fence,
                VaultTransportCommandV1::GenerateOpaqueToken {
                    lease_id,
                    secret_class: SecretClassV1::PlatformCredential,
                },
            )
            .await?;
        (record_id.len() == 16 && record_id.iter().any(|byte| *byte != 0))
            .then_some(())
            .ok_or(NatsCredentialLeaseErrorV1::Rejected)?;
        self.resolve(fence).await
    }

    async fn resolve(
        &mut self,
        fence: &NatsRuntimeCredentialFenceV1,
    ) -> Result<Zeroizing<Vec<u8>>, NatsCredentialLeaseErrorV1> {
        let lease_id = self.issue(fence, VaultActionV1::Resolve).await?;
        self.execute(
            fence,
            VaultTransportCommandV1::ResolveLease {
                lease_id,
                secret_class: SecretClassV1::PlatformCredential,
            },
        )
        .await
        .and_then(non_empty)
    }

    async fn issue(
        &mut self,
        fence: &NatsRuntimeCredentialFenceV1,
        action: VaultActionV1,
    ) -> Result<LeaseIdV1, NatsCredentialLeaseErrorV1> {
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
            .ok_or(NatsCredentialLeaseErrorV1::Rejected)
    }

    async fn execute(
        &mut self,
        fence: &NatsRuntimeCredentialFenceV1,
        command: VaultTransportCommandV1,
    ) -> Result<Zeroizing<Vec<u8>>, NatsCredentialLeaseErrorV1> {
        let prepared = session::prepare(audience_from_fence(fence)?, &self.context, &command)
            .map_err(|_| NatsCredentialLeaseErrorV1::Rejected)?;
        session::execute(&mut self.route_port, prepared)
            .await
            .map_err(map_route_error)
    }
}

fn issue_request(
    fence: &NatsRuntimeCredentialFenceV1,
    context: &NatsVaultRouteContextV1,
    action: VaultActionV1,
) -> Result<VaultLeaseIssueRequestV1, NatsCredentialLeaseErrorV1> {
    let purpose = VaultPurposeRequestV1::new(
        NATS_CREDENTIAL_PURPOSE.to_owned(),
        fence.runtime_instance_id.clone(),
        vec![SecretClassV1::PlatformCredential],
        vec![action],
        DEFAULT_LEASE_TTL_SECONDS,
    )
    .map_err(|_| NatsCredentialLeaseErrorV1::Rejected)?;
    VaultLeaseIssueRequestV1::new(
        context.vault_instance_id().to_owned(),
        context.vault_runtime_generation(),
        fence.credential_revision,
        fence.logical_owner_id.clone(),
        purpose,
        audience_from_fence(fence)?,
    )
    .map_err(|_| NatsCredentialLeaseErrorV1::Rejected)
}

fn audience_from_fence(
    fence: &NatsRuntimeCredentialFenceV1,
) -> Result<LeaseAudienceV1, NatsCredentialLeaseErrorV1> {
    LeaseAudienceV1::new(
        fence.registration_id.clone(),
        fence.runtime_instance_id.clone(),
        fence.runtime_generation,
        fence.grant_epoch,
    )
    .map_err(|_| NatsCredentialLeaseErrorV1::Rejected)
}

fn non_empty(
    credential: Zeroizing<Vec<u8>>,
) -> Result<Zeroizing<Vec<u8>>, NatsCredentialLeaseErrorV1> {
    (!credential.is_empty())
        .then_some(credential)
        .ok_or(NatsCredentialLeaseErrorV1::Rejected)
}

const fn map_route_error(error: NatsVaultRouteFailureV1) -> NatsCredentialLeaseErrorV1 {
    match error {
        NatsVaultRouteFailureV1::Rejected => NatsCredentialLeaseErrorV1::Rejected,
        NatsVaultRouteFailureV1::Unavailable => NatsCredentialLeaseErrorV1::Unavailable,
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
pub enum NatsCredentialLeaseErrorV1 {
    Rejected,
    Unavailable,
}
