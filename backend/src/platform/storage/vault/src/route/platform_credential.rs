//! Idempotent bootstrap of the Storage Control administrative credential.

use std::future::Future;
use std::task::{Context, Poll, Waker};

use makosh_vault_protocol::{
    DEFAULT_LEASE_TTL_SECONDS, LeaseAudienceV1, LeaseIdV1, SecretClassV1, VaultActionV1,
    VaultLeaseIssueRequestV1, VaultPurposeRequestV1, VaultTransportCommandV1,
};

use super::{
    StorageVaultRouteContextV1, StorageVaultRouteFailureV1, StorageVaultRoutePortV1, session,
};

const PLATFORM_LOGICAL_OWNER: &str = "storage";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoragePlatformCredentialPurposeV1 {
    PgBouncerAdmin,
    PostgresAdmin,
}

impl StoragePlatformCredentialPurposeV1 {
    const fn identifier(self) -> &'static str {
        match self {
            Self::PgBouncerAdmin => "storage.control.pgbouncer.admin",
            Self::PostgresAdmin => "storage.control.postgres.admin",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoragePlatformCredentialStateV1 {
    Existing,
    Generated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoragePlatformCredentialErrorV1 {
    Rejected,
    Unavailable,
}

pub fn complete_immediately<T>(
    future: impl Future<Output = T>,
) -> Result<T, StoragePlatformCredentialErrorV1> {
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    let mut future = std::pin::pin!(future);
    match future.as_mut().poll(&mut context) {
        Poll::Ready(value) => Ok(value),
        Poll::Pending => Err(StoragePlatformCredentialErrorV1::Unavailable),
    }
}

pub struct StoragePlatformCredentialBootstrapV1<T> {
    route_port: T,
    context: StorageVaultRouteContextV1,
    audience: LeaseAudienceV1,
    purpose: StoragePlatformCredentialPurposeV1,
    storage_instance_id: String,
    storage_generation: u64,
}

impl<T> StoragePlatformCredentialBootstrapV1<T>
where
    T: StorageVaultRoutePortV1 + Send,
{
    pub fn new(
        route_port: T,
        context: StorageVaultRouteContextV1,
        audience: LeaseAudienceV1,
        purpose: StoragePlatformCredentialPurposeV1,
        storage_instance_id: String,
        storage_generation: u64,
    ) -> Result<Self, StoragePlatformCredentialErrorV1> {
        if !valid_storage_instance_id(&storage_instance_id) || storage_generation == 0 {
            return Err(StoragePlatformCredentialErrorV1::Rejected);
        }
        Ok(Self {
            route_port,
            context,
            audience,
            purpose,
            storage_instance_id,
            storage_generation,
        })
    }

    pub async fn ensure(
        &mut self,
    ) -> Result<StoragePlatformCredentialStateV1, StoragePlatformCredentialErrorV1> {
        self.ensure_credential().await.map(|(state, _)| state)
    }

    pub async fn ensure_and_resolve(
        &mut self,
    ) -> Result<zeroize::Zeroizing<Vec<u8>>, StoragePlatformCredentialErrorV1> {
        self.ensure_credential()
            .await
            .map(|(_, credential)| credential)
    }

    async fn ensure_credential(
        &mut self,
    ) -> Result<
        (
            StoragePlatformCredentialStateV1,
            zeroize::Zeroizing<Vec<u8>>,
        ),
        StoragePlatformCredentialErrorV1,
    > {
        match self.resolve_existing().await {
            Ok(credential) => Ok((StoragePlatformCredentialStateV1::Existing, credential)),
            Err(StoragePlatformCredentialErrorV1::Rejected) => self.generate_and_resolve().await,
            Err(StoragePlatformCredentialErrorV1::Unavailable) => {
                Err(StoragePlatformCredentialErrorV1::Unavailable)
            }
        }
    }

    async fn resolve_existing(
        &mut self,
    ) -> Result<zeroize::Zeroizing<Vec<u8>>, StoragePlatformCredentialErrorV1> {
        let lease_id = self.issue(VaultActionV1::Resolve).await?;
        let credential = self
            .execute(VaultTransportCommandV1::ResolveLease {
                lease_id,
                secret_class: SecretClassV1::PlatformCredential,
            })
            .await?;
        (!credential.is_empty())
            .then_some(credential)
            .ok_or(StoragePlatformCredentialErrorV1::Rejected)
    }

    async fn generate_and_resolve(
        &mut self,
    ) -> Result<
        (
            StoragePlatformCredentialStateV1,
            zeroize::Zeroizing<Vec<u8>>,
        ),
        StoragePlatformCredentialErrorV1,
    > {
        let lease_id = self.issue(VaultActionV1::Create).await?;
        let record_id = self
            .execute(VaultTransportCommandV1::GenerateOpaqueToken {
                lease_id,
                secret_class: SecretClassV1::PlatformCredential,
            })
            .await?;
        if record_id.len() != 16 || record_id.iter().all(|byte| *byte == 0) {
            return Err(StoragePlatformCredentialErrorV1::Rejected);
        }
        self.resolve_existing()
            .await
            .map(|credential| (StoragePlatformCredentialStateV1::Generated, credential))
    }

    async fn issue(
        &mut self,
        action: VaultActionV1,
    ) -> Result<LeaseIdV1, StoragePlatformCredentialErrorV1> {
        let request = self.issue_request(action)?;
        let response = self
            .execute(VaultTransportCommandV1::IssueLease { request })
            .await?;
        String::from_utf8(response.to_vec())
            .ok()
            .and_then(|value| LeaseIdV1::new(value).ok())
            .ok_or(StoragePlatformCredentialErrorV1::Rejected)
    }

    fn issue_request(
        &self,
        action: VaultActionV1,
    ) -> Result<VaultLeaseIssueRequestV1, StoragePlatformCredentialErrorV1> {
        let purpose = VaultPurposeRequestV1::new(
            self.purpose.identifier().to_owned(),
            self.storage_instance_id.clone(),
            vec![SecretClassV1::PlatformCredential],
            vec![action],
            DEFAULT_LEASE_TTL_SECONDS,
        )
        .map_err(|_| StoragePlatformCredentialErrorV1::Rejected)?;
        VaultLeaseIssueRequestV1::new(
            self.context.vault_instance_id().to_owned(),
            self.context.vault_runtime_generation(),
            self.storage_generation,
            PLATFORM_LOGICAL_OWNER.to_owned(),
            purpose,
            self.audience.clone(),
        )
        .map_err(|_| StoragePlatformCredentialErrorV1::Rejected)
    }

    async fn execute(
        &mut self,
        command: VaultTransportCommandV1,
    ) -> Result<zeroize::Zeroizing<Vec<u8>>, StoragePlatformCredentialErrorV1> {
        let prepared = session::prepare(self.audience.clone(), &self.context, &command)
            .map_err(|_| StoragePlatformCredentialErrorV1::Rejected)?;
        session::execute(&mut self.route_port, prepared)
            .await
            .map_err(map_route_error)
    }
}

fn valid_storage_instance_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

const fn map_route_error(error: StorageVaultRouteFailureV1) -> StoragePlatformCredentialErrorV1 {
    match error {
        StorageVaultRouteFailureV1::Rejected => StoragePlatformCredentialErrorV1::Rejected,
        StorageVaultRouteFailureV1::Unavailable => StoragePlatformCredentialErrorV1::Unavailable,
    }
}
