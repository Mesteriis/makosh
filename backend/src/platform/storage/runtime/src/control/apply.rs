//! Applies one newly reserved Storage binding without restarting Storage Control.

use std::os::unix::net::UnixStream;

use sha2::Digest;

use makosh_storage_protocol::{
    v1::{StorageBindingV1, StorageBundleV1, StorageRuntimeConfigurationV1},
    validation::{storage_binding_from_message, validate_storage_runtime_configuration},
};

use super::{
    handshake::ManagedStorageRuntimeIdentityV1,
    runtime::{resolve_platform_credential, resolve_runtime_credentials},
};
use crate::admin::{
    apply_authorized_bindings, apply_authorized_migrations, reconcile_authorized_roles,
    verify_platform_admin, verify_platform_postgres,
};
use crate::vault::StoragePlatformCredentialPurposeV1;

pub(super) fn apply_active_binding(
    channel: &UnixStream,
    identity: &ManagedStorageRuntimeIdentityV1,
    configuration: &mut StorageRuntimeConfigurationV1,
    active_bindings: &mut Vec<StorageBindingV1>,
    binding: StorageBindingV1,
    bundle: StorageBundleV1,
) -> Result<StorageBindingV1, String> {
    if let Some(current) = active_bindings.iter().find(|current| **current == binding) {
        return Ok(current.clone());
    }
    validate_candidate(configuration, active_bindings, &binding, &bundle)?;
    let desired = configuration_for_apply(configuration, active_bindings, &binding, &bundle)?;
    provision(channel, identity, &desired)?;
    retain_applied_configuration(configuration, active_bindings, desired);
    Ok(binding)
}

fn retain_applied_configuration(
    configuration: &mut StorageRuntimeConfigurationV1,
    active_bindings: &mut Vec<StorageBindingV1>,
    desired: StorageRuntimeConfigurationV1,
) {
    *active_bindings = desired.desired_bindings.clone();
    *configuration = desired;
}

pub(super) fn error_code(error: &str) -> &'static str {
    match error {
        "Storage binding is stale or unavailable" | "Storage binding is invalid" => {
            "binding_not_admissible"
        }
        "Storage runtime configuration is invalid" => "binding_runtime_configuration_invalid",
        "Storage platform credential bootstrap is invalid"
        | "Storage platform credential bootstrap was rejected"
        | "Storage platform credential bootstrap is unavailable"
        | "Storage runtime credential is unavailable" => "binding_credential_unavailable",
        "Storage role specification is invalid"
        | "Storage role reconciliation is unavailable"
        | "Storage role credential is unavailable" => "binding_role_provisioning_failed",
        "Storage PostgreSQL runtime is unavailable"
        | "Storage PostgreSQL admin authentication is unavailable"
        | "Storage PostgreSQL bootstrap is unavailable"
        | "Storage PostgreSQL readiness is unavailable"
        | "Storage PostgreSQL identity is unavailable"
        | "Storage PostgreSQL admin endpoint is invalid" => "binding_postgres_unavailable",
        "Storage migration bundle is unavailable" => "binding_migration_bundle_failed",
        "Storage migration application is unavailable" => "binding_migration_failed",
        "Storage migration owner role is unavailable" => "binding_migration_owner_role_failed",
        "Storage migration statement is unavailable" => "binding_migration_statement_failed",
        "Storage migration ledger is unavailable" => "binding_migration_ledger_failed",
        "Storage migration role reset is unavailable" => "binding_migration_role_reset_failed",
        "Storage migration commit is unavailable" => "binding_migration_commit_failed",
        "Storage migration privilege reconciliation is unavailable" => {
            "binding_migration_privileges_failed"
        }
        "Storage PgBouncer database configuration is unavailable" => {
            "binding_pool_configuration_failed"
        }
        "Storage PgBouncer admin runtime is unavailable"
        | "Storage PgBouncer admin authentication is unavailable"
        | "Storage PgBouncer admin endpoint is invalid"
        | "Storage PgBouncer admin credential is invalid"
        | "Storage PgBouncer authentication configuration is unavailable" => {
            "binding_pool_authentication_failed"
        }
        "Storage PgBouncer configuration reload is unavailable" => "binding_pool_reload_failed",
        "Storage PgBouncer configuration is unavailable" => "binding_pool_verification_failed",
        "Storage PgBouncer catalog is unavailable" => "binding_pool_catalog_failed",
        "Storage runtime SCRAM verifier is unavailable" => "binding_pool_runtime_credential_failed",
        "Storage binding pool alias is invalid"
        | "Storage binding budget is invalid"
        | "Storage binding configuration is invalid" => "binding_pool_binding_invalid",
        _ => "binding_apply_failed",
    }
}

fn validate_candidate(
    configuration: &StorageRuntimeConfigurationV1,
    active_bindings: &[StorageBindingV1],
    binding: &StorageBindingV1,
    bundle: &StorageBundleV1,
) -> Result<(), String> {
    let topology = configuration
        .topology
        .as_ref()
        .ok_or_else(|| "Storage runtime configuration is invalid".to_owned())?;
    let model = storage_binding_from_message(binding)
        .map_err(|_| "Storage binding is invalid".to_owned())?;
    if binding.storage_instance_id != topology.storage_instance_id
        || binding.storage_generation != topology.storage_generation
        || binding.database_id != topology.database_id
        || bundle.owner_id != binding.owner
        || u64::from(bundle.revision) != binding.storage_bundle_revision
        || sha2::Sha256::digest(prost::Message::encode_to_vec(bundle)).as_slice()
            != binding.storage_bundle_digest.as_slice()
        || active_bindings.iter().any(|current| {
            current.runtime_principal == binding.runtime_principal
                || current.pool_alias == binding.pool_alias
                || (current.registration_id == binding.registration_id
                    && current.runtime_generation == binding.runtime_generation)
        })
        || model.identity().runtime_instance_id().is_empty()
    {
        return Err("Storage binding is stale or unavailable".to_owned());
    }
    Ok(())
}

fn configuration_for_apply(
    configuration: &StorageRuntimeConfigurationV1,
    active_bindings: &[StorageBindingV1],
    binding: &StorageBindingV1,
    bundle: &StorageBundleV1,
) -> Result<StorageRuntimeConfigurationV1, String> {
    let mut desired = configuration.clone();
    desired.desired_bindings = active_bindings.to_vec();
    desired.desired_bindings.push(binding.clone());
    if !desired
        .desired_bundles
        .iter()
        .any(|current| current.owner_id == bundle.owner_id && current.revision == bundle.revision)
    {
        desired.desired_bundles.push(bundle.clone());
    }
    validate_storage_runtime_configuration(&desired)
        .map_err(|_| "Storage binding is invalid".to_owned())?;
    Ok(desired)
}

fn provision(
    channel: &UnixStream,
    identity: &ManagedStorageRuntimeIdentityV1,
    configuration: &StorageRuntimeConfigurationV1,
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
    let runtime_credentials = resolve_runtime_credentials(channel, configuration)?;
    verify_platform_admin(topology, &pgbouncer_credential)?;
    verify_platform_postgres(topology, &postgres_credential)?;
    reconcile_authorized_roles(topology, &postgres_credential, &runtime_credentials)?;
    apply_authorized_migrations(
        topology,
        &postgres_credential,
        &configuration.desired_bindings,
        &configuration.desired_bundles,
    )?;
    apply_authorized_bindings(
        configuration,
        &pgbouncer_credential,
        &postgres_credential,
        &runtime_credentials,
    )
}

#[cfg(test)]
mod tests {
    use super::retain_applied_configuration;
    use makosh_storage_protocol::v1::{
        StorageBindingV1, StorageBundleV1, StorageRuntimeConfigurationV1,
    };

    #[test]
    fn retains_prior_bundle_when_a_second_owner_binding_is_applied() {
        let communications = bundle("communications");
        let telegram = bundle("telegram");
        let communications_binding = binding("communications");
        let telegram_binding = binding("telegram");
        let mut configuration = StorageRuntimeConfigurationV1 {
            desired_bindings: vec![communications_binding.clone()],
            desired_bundles: vec![communications.clone()],
            ..Default::default()
        };
        let mut active_bindings = vec![communications_binding.clone()];
        let desired = StorageRuntimeConfigurationV1 {
            desired_bindings: vec![communications_binding, telegram_binding.clone()],
            desired_bundles: vec![communications.clone(), telegram.clone()],
            ..Default::default()
        };

        retain_applied_configuration(&mut configuration, &mut active_bindings, desired);

        assert_eq!(
            configuration.desired_bundles,
            vec![communications, telegram]
        );
        assert_eq!(
            configuration.desired_bindings,
            vec![binding("communications"), telegram_binding]
        );
        assert_eq!(active_bindings, configuration.desired_bindings);
    }

    fn bundle(owner_id: &str) -> StorageBundleV1 {
        StorageBundleV1 {
            owner_id: owner_id.to_owned(),
            revision: 1,
            ..Default::default()
        }
    }

    fn binding(owner: &str) -> StorageBindingV1 {
        StorageBindingV1 {
            owner: owner.to_owned(),
            ..Default::default()
        }
    }
}
