//! One-time issue and resolution of a Storage runtime credential lease.

use makosh_storage_protocol::StorageBindingV1;
use makosh_vault_protocol::{
    DEFAULT_LEASE_TTL_SECONDS, LeaseAudienceV1, LeaseIdV1, SecretClassV1, VaultActionV1,
    VaultLeaseIssueRequestV1, VaultPurposeRequestV1, VaultTransportCommandV1,
};
use zeroize::Zeroizing;

use super::{
    StorageVaultLeaseAdapterV1, StorageVaultRouteFailureV1, StorageVaultRoutePortV1, session,
};

const STORAGE_CREDENTIAL_PURPOSE: &str = "storage.runtime.credential";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageCredentialLeaseErrorV1 {
    Rejected,
    Unavailable,
}

impl<T> StorageVaultLeaseAdapterV1<T>
where
    T: StorageVaultRoutePortV1 + Send,
{
    pub async fn issue_platform_credential_create(
        &mut self,
        binding: &StorageBindingV1,
    ) -> Result<LeaseIdV1, StorageCredentialLeaseErrorV1> {
        self.issue_credential(binding, VaultActionV1::Create).await
    }

    pub async fn issue_runtime_credential(
        &mut self,
        binding: &StorageBindingV1,
    ) -> Result<LeaseIdV1, StorageCredentialLeaseErrorV1> {
        self.issue_credential(binding, VaultActionV1::Resolve).await
    }

    pub async fn store_platform_credential(
        &mut self,
        binding: &StorageBindingV1,
        lease_id: LeaseIdV1,
        credential: &[u8],
    ) -> Result<(), StorageCredentialLeaseErrorV1> {
        let command = VaultTransportCommandV1::StoreLease {
            lease_id,
            secret_class: SecretClassV1::PlatformCredential,
            payload: credential.to_vec(),
        };
        let record_id = self.execute(binding, &command).await?;
        (record_id.len() == 16 && record_id.iter().any(|byte| *byte != 0))
            .then_some(())
            .ok_or(StorageCredentialLeaseErrorV1::Rejected)
    }

    async fn issue_credential(
        &mut self,
        binding: &StorageBindingV1,
        action: VaultActionV1,
    ) -> Result<LeaseIdV1, StorageCredentialLeaseErrorV1> {
        let request = issue_request(binding, &self.context, action)?;
        let prepared = session::prepare_storage_credential(
            binding,
            &self.context,
            &VaultTransportCommandV1::IssueLease { request },
        )
        .map_err(|_| StorageCredentialLeaseErrorV1::Rejected)?;
        let lease_id = session::execute(&mut self.route_port, prepared)
            .await
            .map_err(map_route_error)?;
        String::from_utf8(lease_id.to_vec())
            .ok()
            .and_then(|value| LeaseIdV1::new(value).ok())
            .ok_or(StorageCredentialLeaseErrorV1::Rejected)
    }

    pub async fn resolve_runtime_credential(
        &mut self,
        binding: &StorageBindingV1,
        lease_id: LeaseIdV1,
    ) -> Result<Zeroizing<Vec<u8>>, StorageCredentialLeaseErrorV1> {
        self.execute(
            binding,
            &VaultTransportCommandV1::ResolveLease {
                lease_id,
                secret_class: SecretClassV1::PlatformCredential,
            },
        )
        .await
        .and_then(|credential| {
            (!credential.is_empty())
                .then_some(credential)
                .ok_or(StorageCredentialLeaseErrorV1::Rejected)
        })
    }

    pub async fn ensure_runtime_credential(
        &mut self,
        binding: &StorageBindingV1,
    ) -> Result<Zeroizing<Vec<u8>>, StorageCredentialLeaseErrorV1> {
        let existing = match self.issue_runtime_credential(binding).await {
            Ok(lease_id) => self.resolve_runtime_credential(binding, lease_id).await,
            Err(error) => Err(error),
        };
        match existing {
            Ok(credential) => Ok(credential),
            Err(StorageCredentialLeaseErrorV1::Unavailable) => {
                self.rebuild_runtime_credential(binding).await
            }
            Err(StorageCredentialLeaseErrorV1::Rejected) => {
                self.rebuild_runtime_credential(binding).await
            }
        }
    }

    async fn rebuild_runtime_credential(
        &mut self,
        binding: &StorageBindingV1,
    ) -> Result<Zeroizing<Vec<u8>>, StorageCredentialLeaseErrorV1> {
        match self.create_runtime_credential(binding).await {
            Ok(credential) => Ok(credential),
            Err(StorageCredentialLeaseErrorV1::Unavailable) => {
                self.issue_runtime_credential_recreate(binding).await?;
                self.create_runtime_credential(binding).await
            }
            Err(StorageCredentialLeaseErrorV1::Rejected) => {
                Err(StorageCredentialLeaseErrorV1::Rejected)
            }
        }
    }

    async fn create_runtime_credential(
        &mut self,
        binding: &StorageBindingV1,
    ) -> Result<Zeroizing<Vec<u8>>, StorageCredentialLeaseErrorV1> {
        let lease_id = self.issue_platform_credential_create(binding).await?;
        let record_id = self
            .execute(
                binding,
                &VaultTransportCommandV1::GenerateOpaqueToken {
                    lease_id,
                    secret_class: SecretClassV1::PlatformCredential,
                },
            )
            .await?;
        valid_record_id(&record_id)?;
        let lease_id = self.issue_runtime_credential(binding).await?;
        self.resolve_runtime_credential(binding, lease_id).await
    }

    async fn issue_runtime_credential_recreate(
        &mut self,
        binding: &StorageBindingV1,
    ) -> Result<(), StorageCredentialLeaseErrorV1> {
        let lease_id = self
            .issue_credential(binding, VaultActionV1::Delete)
            .await?;
        self.execute(
            binding,
            &VaultTransportCommandV1::DeleteLease {
                lease_id,
                secret_class: SecretClassV1::PlatformCredential,
            },
        )
        .await
        .map(|_| ())
    }

    async fn execute(
        &mut self,
        binding: &StorageBindingV1,
        command: &VaultTransportCommandV1,
    ) -> Result<Zeroizing<Vec<u8>>, StorageCredentialLeaseErrorV1> {
        let prepared = session::prepare_storage_credential(binding, &self.context, command)
            .map_err(|_| StorageCredentialLeaseErrorV1::Rejected)?;
        session::execute(&mut self.route_port, prepared)
            .await
            .map_err(map_route_error)
    }
}

fn issue_request(
    binding: &StorageBindingV1,
    context: &super::StorageVaultRouteContextV1,
    action: VaultActionV1,
) -> Result<VaultLeaseIssueRequestV1, StorageCredentialLeaseErrorV1> {
    let purpose = VaultPurposeRequestV1::new(
        STORAGE_CREDENTIAL_PURPOSE.to_owned(),
        binding.access().runtime_principal().to_owned(),
        vec![SecretClassV1::PlatformCredential],
        vec![action],
        DEFAULT_LEASE_TTL_SECONDS,
    )
    .map_err(|_| StorageCredentialLeaseErrorV1::Rejected)?;
    let audience = LeaseAudienceV1::new(
        binding.identity().registration_id().to_owned(),
        binding.identity().runtime_instance_id().to_owned(),
        binding.fences().runtime_generation(),
        binding.fences().grant_epoch(),
    )
    .map_err(|_| StorageCredentialLeaseErrorV1::Rejected)?;
    VaultLeaseIssueRequestV1::new(
        context.vault_instance_id().to_owned(),
        context.vault_runtime_generation(),
        binding.fences().credential_lease_revision(),
        binding.identity().owner().to_owned(),
        purpose,
        audience,
    )
    .map_err(|_| StorageCredentialLeaseErrorV1::Rejected)
}

fn valid_record_id(record_id: &Zeroizing<Vec<u8>>) -> Result<(), StorageCredentialLeaseErrorV1> {
    (record_id.len() == 16 && record_id.iter().any(|byte| *byte != 0))
        .then_some(())
        .ok_or(StorageCredentialLeaseErrorV1::Rejected)
}

fn map_route_error(error: StorageVaultRouteFailureV1) -> StorageCredentialLeaseErrorV1 {
    match error {
        StorageVaultRouteFailureV1::Rejected => StorageCredentialLeaseErrorV1::Rejected,
        StorageVaultRouteFailureV1::Unavailable => StorageCredentialLeaseErrorV1::Unavailable,
    }
}
