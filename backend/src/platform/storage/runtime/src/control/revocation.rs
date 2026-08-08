//! Live multi-boundary fencing for one Kernel-reserved Storage binding.

use std::os::unix::net::UnixStream;

use makosh_storage_control::{StorageLifecycleV1, StorageRevocationErrorV1, StorageRevokerV1};
use makosh_storage_pgbouncer::{
    PgBouncerPoolFenceAdapterV1, PoolAliasV1, TokioPostgresPgBouncerAdminPortV1,
    database_is_configured,
};
use makosh_storage_postgres::{PostgresRuntimeFenceAdapterV1, StorageRoleSpecV1};
use makosh_storage_protocol::{
    v1::{StorageBindingV1, StorageRuntimeConfigurationV1},
    validation::storage_binding_from_message,
};

use super::{
    handshake::ManagedStorageRuntimeIdentityV1, runtime::resolve_platform_credential,
    vault_route::InheritedVaultRoutePortV1,
};
use crate::{
    admin::{admin_credential, admin_endpoint, connect_platform},
    vault::{
        StoragePlatformCredentialPurposeV1, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
    },
};

pub(super) fn revoke_reserved_binding(
    channel: &UnixStream,
    identity: &ManagedStorageRuntimeIdentityV1,
    configuration: &StorageRuntimeConfigurationV1,
    active_bindings: &mut Vec<StorageBindingV1>,
    requested: StorageBindingV1,
) -> Result<StorageBindingV1, String> {
    // The durable Kernel reservation is the revocation authority. A Storage
    // runtime restarted after that reservation intentionally excludes the
    // revoking binding from its desired active set, but still has to finish
    // the physical Vault, PgBouncer and PostgreSQL fences.
    fence(channel, identity, configuration, &requested)?;
    active_bindings.retain(|candidate| *candidate != requested);
    Ok(requested)
}

fn fence(
    channel: &UnixStream,
    identity: &ManagedStorageRuntimeIdentityV1,
    configuration: &StorageRuntimeConfigurationV1,
    binding: &StorageBindingV1,
) -> Result<(), String> {
    let topology = configuration
        .topology
        .as_ref()
        .ok_or_else(|| "Storage runtime configuration is invalid".to_owned())?;
    let pgbouncer_credential = resolve_platform_credential(
        channel,
        identity,
        configuration,
        StoragePlatformCredentialPurposeV1::PgBouncerAdmin,
    )?;
    let postgres_credential = resolve_platform_credential(
        channel,
        identity,
        configuration,
        StoragePlatformCredentialPurposeV1::PostgresAdmin,
    )?;
    let context = vault_context(configuration)?;
    let route = InheritedVaultRoutePortV1::new(
        channel
            .try_clone()
            .map_err(|_| "Storage inherited control channel is unavailable".to_owned())?,
    );
    let mut vault = StorageVaultLeaseAdapterV1::new(route, context);
    let endpoint = admin_endpoint(topology)?;
    let credential = admin_credential(&pgbouncer_credential)?;
    let model_binding = storage_binding_from_message(binding)
        .map_err(|_| "Storage binding is invalid".to_owned())?;
    let role = StorageRoleSpecV1::platform_binding(model_binding.clone())
        .map_err(|_| "Storage role specification is invalid".to_owned())?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "Storage revocation runtime is unavailable".to_owned())?;
    runtime.block_on(async {
        let admin = TokioPostgresPgBouncerAdminPortV1::connect(&endpoint, &credential)
            .await
            .map_err(|_| "Storage PgBouncer admin authentication is unavailable".to_owned())?;
        let alias = PoolAliasV1::from_binding(&model_binding)
            .map_err(|_| "Storage binding is invalid".to_owned())?;
        let pool_is_configured = database_is_configured(&endpoint, &credential, &alias)
            .await
            .map_err(|_| "Storage PgBouncer admin authentication is unavailable".to_owned())?;
        let connector = connect_platform(topology, &postgres_credential).await?;
        let mut lifecycle = StorageLifecycleV1::default();
        lifecycle
            .activate(model_binding)
            .map_err(|_| "Storage binding is not revocable".to_owned())?;
        let mut pool =
            PgBouncerPoolFenceAdapterV1::for_configuration_state(admin, pool_is_configured);
        let mut postgres = PostgresRuntimeFenceAdapterV1::new(&connector, &role);
        StorageRevokerV1
            .revoke(&mut lifecycle, &mut vault, &mut pool, &mut postgres)
            .await
            .map(|_| ())
            .map_err(revocation_error)
    })
}

fn revocation_error(error: StorageRevocationErrorV1) -> String {
    match error {
        StorageRevocationErrorV1::Lifecycle(_) => {
            "Storage binding revocation lifecycle is invalid".to_owned()
        }
        StorageRevocationErrorV1::Incomplete(report) if !report.vault_lease_invalidated() => {
            "Storage binding Vault lease revocation is incomplete".to_owned()
        }
        StorageRevocationErrorV1::Incomplete(report) if !report.pool_paused() => {
            "Storage binding PgBouncer pause is incomplete".to_owned()
        }
        StorageRevocationErrorV1::Incomplete(report) if !report.pool_disabled() => {
            "Storage binding PgBouncer disable is incomplete".to_owned()
        }
        StorageRevocationErrorV1::Incomplete(report) if !report.pool_killed() => {
            "Storage binding PgBouncer kill is incomplete".to_owned()
        }
        StorageRevocationErrorV1::Incomplete(report) if !report.postgres_role_fenced() => {
            "Storage binding PostgreSQL role fence is incomplete".to_owned()
        }
        StorageRevocationErrorV1::Incomplete(_) => {
            "Storage binding revocation is incomplete".to_owned()
        }
    }
}

fn vault_context(
    configuration: &StorageRuntimeConfigurationV1,
) -> Result<StorageVaultRouteContextV1, String> {
    let public_key = configuration
        .vault_hpke_public_key_x25519
        .as_slice()
        .try_into()
        .map_err(|_| "Storage Vault route context is invalid".to_owned())?;
    StorageVaultRouteContextV1::new(
        configuration.vault_instance_id.clone(),
        configuration.vault_runtime_generation,
        public_key,
    )
    .map_err(|_| "Storage Vault route context is invalid".to_owned())
}
