//! Telegram admission glue over the shared ciphertext-only Vault route.

use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::managed_control::ManagedControlChannelV2;
use makosh_storage_protocol::StorageBindingV1;
use makosh_storage_vault::{
    InheritedKernelVaultRouteV2, StorageCredentialLeaseErrorV1, StorageVaultLeaseAdapterV1,
    StorageVaultRouteContextV1,
};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramCredentialRouteError {
    InvalidContext,
    InvalidLease,
    Unavailable,
    Rejected,
}

pub async fn resolve_storage_credential_v2(
    channel: ManagedControlChannelV2<UnixStream>,
    binding: &StorageBindingV1,
    context: StorageVaultRouteContextV1,
) -> Result<(Zeroizing<Vec<u8>>, ManagedControlChannelV2<UnixStream>), StorageCredentialLeaseErrorV1>
{
    let mut leases =
        StorageVaultLeaseAdapterV1::new(InheritedKernelVaultRouteV2::new(channel), context);
    let lease_id = leases.issue_runtime_credential(binding).await?;
    let credential = leases.resolve_runtime_credential(binding, lease_id).await?;
    Ok((credential, leases.into_route_port().into_channel()))
}
