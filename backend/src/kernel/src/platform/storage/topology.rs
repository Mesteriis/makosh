//! Conversion of durable Kernel topology policy into the Storage runtime contract.

use std::path::Path;

use makosh_kernel_control_store::{
    PlatformStorageBindingV1, PlatformStorageBundleV1, PlatformStorageTopology,
    StorageDeploymentProfileV1 as ControlStoreDeploymentProfile,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_storage_protocol::v1::{
    StorageBindingV1, StorageBundleV1, StorageDeploymentProfileV1 as RuntimeDeploymentProfile,
    StorageEffectiveBudgetsV1, StorageRuntimeConfigurationV1, StorageRuntimeTopologyV1,
};
use prost::Message;
use sha2::Digest;

pub fn current(store: &SqliteControlStore) -> Result<PlatformStorageTopology, String> {
    store
        .platform_storage_topology()
        .map_err(|_| "Storage topology is unavailable".to_owned())?
        .ok_or_else(|| "Storage topology is unavailable".to_owned())
}

pub struct ManagedStorageConfigurationInputV1<'a> {
    pub topology: &'a PlatformStorageTopology,
    pub bindings: &'a [PlatformStorageBindingV1],
    pub bundles: &'a [PlatformStorageBundleV1],
    pub pgbouncer_database_config_path: &'a Path,
    pub pgbouncer_auth_file_path: &'a Path,
    pub vault_instance_id: &'a str,
    pub vault_runtime_generation: u64,
    pub vault_hpke_public_key_x25519: &'a [u8; 32],
}

pub fn encoded_managed_macos(
    input: ManagedStorageConfigurationInputV1<'_>,
) -> Result<Vec<u8>, String> {
    let topology = to_runtime(input.topology)?;
    if topology.deployment_profile != RuntimeDeploymentProfile::MacosTauriEmbedded as i32 {
        return Err("Storage topology is not managed by this Kernel profile".to_owned());
    }
    let desired_bindings = input
        .bindings
        .iter()
        .map(|binding| to_runtime_binding(&topology, binding))
        .collect::<Result<Vec<_>, _>>()?;
    let desired_bundles = input
        .bundles
        .iter()
        .map(to_runtime_bundle)
        .collect::<Result<Vec<_>, _>>()?;
    verify_bound_bundles(&desired_bindings, &desired_bundles)?;
    let pgbouncer_database_config_path = configured_path(input.pgbouncer_database_config_path)?;
    let pgbouncer_auth_file_path = configured_path(input.pgbouncer_auth_file_path)?;
    let configuration = StorageRuntimeConfigurationV1 {
        topology: Some(topology),
        vault_instance_id: input.vault_instance_id.to_owned(),
        vault_runtime_generation: input.vault_runtime_generation,
        vault_hpke_public_key_x25519: input.vault_hpke_public_key_x25519.to_vec(),
        desired_bindings,
        pgbouncer_database_config_path,
        desired_bundles,
        pgbouncer_auth_file_path,
    };
    makosh_storage_protocol::validation::validate_storage_runtime_configuration(&configuration)
        .map_err(|_| "Storage runtime configuration is invalid".to_owned())?;
    Ok(configuration.encode_to_vec())
}

pub(crate) fn to_runtime_bundle(
    bundle: &PlatformStorageBundleV1,
) -> Result<StorageBundleV1, String> {
    let message = StorageBundleV1::decode(bundle.canonical_bytes())
        .map_err(|_| "Storage bundle is invalid".to_owned())?;
    (message.owner_id == bundle.owner_id() && u64::from(message.revision) == bundle.revision())
        .then_some(())
        .ok_or_else(|| "Storage bundle is invalid".to_owned())?;
    (message.encode_to_vec() == bundle.canonical_bytes())
        .then_some(message)
        .ok_or_else(|| "Storage bundle is not canonical".to_owned())
}

fn verify_bound_bundles(
    bindings: &[StorageBindingV1],
    bundles: &[StorageBundleV1],
) -> Result<(), String> {
    for binding in bindings {
        let present = bundles.iter().any(|bundle| {
            bundle.owner_id == binding.owner
                && u64::from(bundle.revision) == binding.storage_bundle_revision
                && sha2::Sha256::digest(bundle.encode_to_vec()).as_slice()
                    == binding.storage_bundle_digest.as_slice()
        });
        if !present {
            return Err("Storage bundle is unavailable".to_owned());
        }
    }
    Ok(())
}

pub(crate) fn to_runtime_binding(
    topology: &StorageRuntimeTopologyV1,
    binding: &PlatformStorageBindingV1,
) -> Result<StorageBindingV1, String> {
    if binding.topology_revision() != topology.topology_revision
        || binding.storage_generation() != topology.storage_generation
    {
        return Err("Storage binding is stale for the current topology".to_owned());
    }
    let pool_alias = makosh_storage_protocol::storage_runtime_pool_alias(
        binding.registration_id(),
        binding.runtime_generation(),
    );
    let message = StorageBindingV1 {
        storage_instance_id: topology.storage_instance_id.clone(),
        storage_generation: topology.storage_generation,
        database_id: topology.database_id.clone(),
        owner: binding.owner_id().to_owned(),
        registration_id: binding.registration_id().to_owned(),
        runtime_instance_id: binding.runtime_instance_id().to_owned(),
        runtime_generation: binding.runtime_generation(),
        grant_epoch: binding.grant_epoch(),
        role_epoch: binding.role_epoch(),
        runtime_principal: binding.runtime_principal().to_owned(),
        pool_alias,
        effective_budgets: Some(StorageEffectiveBudgetsV1 {
            max_connections: u32::from(binding.connection_budget()),
            statement_timeout_millis: binding.statement_timeout_millis(),
        }),
        credential_lease_revision: binding.credential_lease_revision(),
        storage_bundle_revision: binding.storage_bundle_revision(),
        storage_bundle_digest: binding.storage_bundle_digest().to_vec(),
    };
    makosh_storage_protocol::validation::validate_storage_binding_message(&message)
        .map_err(|_| "Storage binding is invalid".to_owned())?;
    Ok(message)
}

fn configured_path(path: &Path) -> Result<String, String> {
    path.is_absolute()
        .then_some(path.display().to_string())
        .ok_or_else(|| "Storage PgBouncer configuration path is invalid".to_owned())
}

pub fn to_runtime(topology: &PlatformStorageTopology) -> Result<StorageRuntimeTopologyV1, String> {
    let deployment_profile = match topology.deployment_profile() {
        ControlStoreDeploymentProfile::MacosTauriEmbedded => {
            RuntimeDeploymentProfile::MacosTauriEmbedded
        }
        ControlStoreDeploymentProfile::LinuxDockerServer => {
            RuntimeDeploymentProfile::LinuxDockerServer
        }
    };
    let runtime = StorageRuntimeTopologyV1 {
        topology_revision: topology.revision(),
        storage_generation: topology.storage_generation(),
        storage_instance_id: topology.storage_instance_id().to_owned(),
        database_id: topology.database_id().to_owned(),
        deployment_profile: deployment_profile as i32,
        postgres_artifact_sha256: topology.postgres_artifact_sha256().to_vec(),
        pgbouncer_artifact_sha256: topology.pgbouncer_artifact_sha256().to_vec(),
        postgres_host: topology.postgres_endpoint().host().to_owned(),
        postgres_port: u32::from(topology.postgres_endpoint().port()),
        pgbouncer_host: topology.pgbouncer_endpoint().host().to_owned(),
        pgbouncer_port: u32::from(topology.pgbouncer_endpoint().port()),
        pgbouncer_postgres_host: topology.pgbouncer_backend_endpoint().host().to_owned(),
        pgbouncer_postgres_port: u32::from(topology.pgbouncer_backend_endpoint().port()),
    };
    makosh_storage_protocol::validation::validate_storage_runtime_topology(&runtime)
        .map_err(|_| "Storage topology is invalid".to_owned())?;
    Ok(runtime)
}

pub fn to_managed_runtime_configuration(
    topology: &PlatformStorageTopology,
    binding: &PlatformStorageBindingV1,
    vault_instance_id: &str,
    vault_runtime_generation: u64,
    vault_hpke_public_key_x25519: &[u8; 32],
) -> Result<makosh_runtime_protocol::v1::ManagedStorageRuntimeConfigurationV1, String> {
    let topology_runtime = to_runtime(topology)?;
    let binding_message = to_runtime_binding(&topology_runtime, binding)?;
    let binding_runtime =
        makosh_storage_protocol::validation::storage_binding_from_message(&binding_message)
            .map_err(|_| "Storage binding is invalid".to_owned())?;
    let budgets = binding_runtime.access().effective_budgets();
    let configuration = makosh_runtime_protocol::v1::ManagedStorageRuntimeConfigurationV1 {
        database_id: binding_runtime.identity().database_id().to_owned(),
        pgbouncer_host: topology_runtime.pgbouncer_host,
        pgbouncer_port: topology_runtime.pgbouncer_port,
        runtime_principal: binding_runtime.access().runtime_principal().to_owned(),
        storage_generation: binding_runtime.fences().storage_generation(),
        credential_revision: binding_runtime.fences().credential_lease_revision(),
        storage_instance_id: binding_runtime.identity().storage_instance_id().to_owned(),
        owner: binding_runtime.identity().owner().to_owned(),
        role_epoch: binding_runtime.fences().role_epoch(),
        pool_alias: binding_runtime.access().pool_alias().to_owned(),
        max_connections: u32::from(budgets.max_connections()),
        statement_timeout_millis: budgets.statement_timeout_millis(),
        storage_bundle_revision: binding_runtime.fences().storage_bundle_revision(),
        storage_bundle_digest: binding_runtime.access().storage_bundle_digest().to_vec(),
        vault_instance_id: vault_instance_id.to_owned(),
        vault_runtime_generation,
        vault_hpke_public_key_x25519: vault_hpke_public_key_x25519.to_vec(),
        runtime_instance_id: binding_runtime.identity().runtime_instance_id().to_owned(),
        logical_owner_id: binding_runtime.identity().owner().to_owned(),
    };
    Ok(configuration)
}
