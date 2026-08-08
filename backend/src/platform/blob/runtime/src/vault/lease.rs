//! Per-reference Blob key authority issued and resolved through Vault leases.

use makosh_blob_protocol::{BlobAccessFenceV1, BlobCustodyScopeV1, BlobRefV1};
use makosh_vault_protocol::{
    DEFAULT_LEASE_TTL_SECONDS, LeaseIdV1, SecretClassV1, VaultActionV1, VaultLeaseIssueRequestV1,
    VaultPurposeRequestV1, VaultTransportCommandV1,
};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::lease::BlobKeyLeaseV1;

use super::{BlobVaultRouteContextV1, BlobVaultRouteFailureV1, BlobVaultRoutePortV1, session};

const BLOB_CONTENT_KEY_PURPOSE: &str = "blob.content.key";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlobContentKeyFenceV1 {
    access: BlobAccessFenceV1,
    custody: BlobCustodyScopeV1,
    key_revision: u64,
}

impl BlobContentKeyFenceV1 {
    pub fn new(
        access: BlobAccessFenceV1,
        custody: BlobCustodyScopeV1,
        key_revision: u64,
    ) -> Result<Self, BlobContentKeyLeaseErrorV1> {
        (key_revision > 0 && access.owner_id() == custody.owner_id())
            .then_some(Self {
                access,
                custody,
                key_revision,
            })
            .ok_or(BlobContentKeyLeaseErrorV1::Rejected)
    }

    #[must_use]
    pub fn access(&self) -> &BlobAccessFenceV1 {
        &self.access
    }

    #[must_use]
    pub const fn key_revision(&self) -> u64 {
        self.key_revision
    }

    #[must_use]
    pub fn custody(&self) -> &BlobCustodyScopeV1 {
        &self.custody
    }
}

pub struct BlobVaultKeyLeaseAdapterV1<T> {
    route_port: T,
    context: BlobVaultRouteContextV1,
}

impl<T> BlobVaultKeyLeaseAdapterV1<T>
where
    T: BlobVaultRoutePortV1 + Send,
{
    #[must_use]
    pub fn new(route_port: T, context: BlobVaultRouteContextV1) -> Self {
        Self {
            route_port,
            context,
        }
    }

    #[must_use]
    pub fn into_route_port(self) -> T {
        self.route_port
    }

    pub async fn ensure_content_key(
        &mut self,
        reference: &BlobRefV1,
        fence: &BlobContentKeyFenceV1,
        now_unix_ms: u64,
    ) -> Result<BlobKeyLeaseV1, BlobContentKeyLeaseErrorV1> {
        let authority = match self.resolve_authority(reference, fence).await {
            Ok(value) => value,
            Err(BlobContentKeyLeaseErrorV1::Rejected) => {
                self.create_authority(reference, fence).await?
            }
            Err(BlobContentKeyLeaseErrorV1::Unavailable) => {
                return Err(BlobContentKeyLeaseErrorV1::Unavailable);
            }
        };
        lease_from_authority(reference, fence, now_unix_ms, authority)
    }

    pub async fn revoke_runtime_content_keys(&mut self) -> Result<(), BlobContentKeyLeaseErrorV1> {
        let result = self
            .execute(VaultTransportCommandV1::RevokeAudience)
            .await?;
        (result.as_slice() == [1])
            .then_some(())
            .ok_or(BlobContentKeyLeaseErrorV1::Rejected)
    }

    async fn resolve_authority(
        &mut self,
        reference: &BlobRefV1,
        fence: &BlobContentKeyFenceV1,
    ) -> Result<Zeroizing<Vec<u8>>, BlobContentKeyLeaseErrorV1> {
        let lease_id = self.issue(reference, fence, VaultActionV1::Resolve).await?;
        self.execute(VaultTransportCommandV1::ResolveLease {
            lease_id,
            secret_class: SecretClassV1::PlatformCredential,
        })
        .await
        .and_then(non_empty)
    }

    async fn create_authority(
        &mut self,
        reference: &BlobRefV1,
        fence: &BlobContentKeyFenceV1,
    ) -> Result<Zeroizing<Vec<u8>>, BlobContentKeyLeaseErrorV1> {
        let lease_id = self.issue(reference, fence, VaultActionV1::Create).await?;
        let record_id = self
            .execute(VaultTransportCommandV1::GenerateOpaqueToken {
                lease_id,
                secret_class: SecretClassV1::PlatformCredential,
            })
            .await?;
        valid_record_id(&record_id)?;
        self.resolve_authority(reference, fence).await
    }

    async fn issue(
        &mut self,
        reference: &BlobRefV1,
        fence: &BlobContentKeyFenceV1,
        action: VaultActionV1,
    ) -> Result<LeaseIdV1, BlobContentKeyLeaseErrorV1> {
        let command = VaultTransportCommandV1::IssueLease {
            request: issue_request(reference, fence, &self.context, action)?,
        };
        let response = self.execute(command).await?;
        String::from_utf8(response.to_vec())
            .ok()
            .and_then(|value| LeaseIdV1::new(value).ok())
            .ok_or(BlobContentKeyLeaseErrorV1::Rejected)
    }

    async fn execute(
        &mut self,
        command: VaultTransportCommandV1,
    ) -> Result<Zeroizing<Vec<u8>>, BlobContentKeyLeaseErrorV1> {
        let prepared = session::prepare(
            self.context.route_audience().clone(),
            &self.context,
            &command,
        )
        .map_err(|_| BlobContentKeyLeaseErrorV1::Rejected)?;
        session::execute(&mut self.route_port, prepared)
            .await
            .map_err(map_route_error)
    }
}

fn lease_from_authority(
    reference: &BlobRefV1,
    fence: &BlobContentKeyFenceV1,
    now_unix_ms: u64,
    authority: Zeroizing<Vec<u8>>,
) -> Result<BlobKeyLeaseV1, BlobContentKeyLeaseErrorV1> {
    let expiry = now_unix_ms
        .checked_add(u64::from(DEFAULT_LEASE_TTL_SECONDS) * 1_000)
        .ok_or(BlobContentKeyLeaseErrorV1::Rejected)?;
    BlobKeyLeaseV1::from_vault_response(
        reference,
        fence.access.clone(),
        fence.custody.clone(),
        fence.key_revision,
        expiry,
        now_unix_ms,
        authority,
    )
    .map_err(|_| BlobContentKeyLeaseErrorV1::Rejected)
}

fn issue_request(
    reference: &BlobRefV1,
    fence: &BlobContentKeyFenceV1,
    context: &BlobVaultRouteContextV1,
    action: VaultActionV1,
) -> Result<VaultLeaseIssueRequestV1, BlobContentKeyLeaseErrorV1> {
    let purpose = VaultPurposeRequestV1::new(
        BLOB_CONTENT_KEY_PURPOSE.to_owned(),
        opaque_reference_scope(reference, fence.custody()),
        vec![SecretClassV1::PlatformCredential],
        vec![action],
        DEFAULT_LEASE_TTL_SECONDS,
    )
    .map_err(|_| BlobContentKeyLeaseErrorV1::Rejected)?;
    VaultLeaseIssueRequestV1::new(
        context.vault_instance_id().to_owned(),
        context.vault_runtime_generation(),
        fence.key_revision,
        fence.access.owner_id().to_owned(),
        purpose,
        context.route_audience().clone(),
    )
    .map_err(|_| BlobContentKeyLeaseErrorV1::Rejected)
}

fn opaque_reference_scope(reference: &BlobRefV1, custody: &BlobCustodyScopeV1) -> String {
    let custody_digest = Sha256::digest(custody.custody_scope_id().as_bytes());
    let mut scope = String::from("blob2-");
    for byte in custody_digest {
        use std::fmt::Write as _;
        write!(&mut scope, "{byte:02x}").expect("writing to String cannot fail");
    }
    scope.push('-');
    for byte in reference.reference_id() {
        use std::fmt::Write as _;
        write!(&mut scope, "{byte:02x}").expect("writing to String cannot fail");
    }
    scope
}

fn valid_record_id(value: &Zeroizing<Vec<u8>>) -> Result<(), BlobContentKeyLeaseErrorV1> {
    (value.len() == 16 && value.iter().any(|byte| *byte != 0))
        .then_some(())
        .ok_or(BlobContentKeyLeaseErrorV1::Rejected)
}

fn non_empty(value: Zeroizing<Vec<u8>>) -> Result<Zeroizing<Vec<u8>>, BlobContentKeyLeaseErrorV1> {
    (!value.is_empty())
        .then_some(value)
        .ok_or(BlobContentKeyLeaseErrorV1::Rejected)
}

const fn map_route_error(error: BlobVaultRouteFailureV1) -> BlobContentKeyLeaseErrorV1 {
    match error {
        BlobVaultRouteFailureV1::Rejected => BlobContentKeyLeaseErrorV1::Rejected,
        BlobVaultRouteFailureV1::Unavailable => BlobContentKeyLeaseErrorV1::Unavailable,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlobContentKeyLeaseErrorV1 {
    Rejected,
    Unavailable,
}
