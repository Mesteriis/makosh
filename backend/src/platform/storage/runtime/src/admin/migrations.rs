//! Applies only canonical bundles whose exact digest is bound into a staged binding.

use makosh_storage_postgres::{
    PLATFORM_ADMIN_USERNAME, PostgresAdapterErrorV1, PostgresAdminConnectorV1, StorageRoleSpecV1,
    apply_storage_bundle,
};
use makosh_storage_protocol::{
    v1::{StorageBindingV1, StorageBundleV1, StorageRuntimeTopologyV1},
    validation::storage_binding_from_message,
};
use prost::Message;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

pub(crate) fn apply_authorized_migrations(
    topology: &StorageRuntimeTopologyV1,
    credential: &Zeroizing<Vec<u8>>,
    bindings: &[StorageBindingV1],
    bundles: &[StorageBundleV1],
) -> Result<(), String> {
    if bindings.is_empty() {
        return Ok(());
    }
    let port = u16::try_from(topology.postgres_port)
        .map_err(|_| "Storage PostgreSQL admin endpoint is invalid".to_owned())?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "Storage PostgreSQL runtime is unavailable".to_owned())?;
    runtime.block_on(apply_all(topology, port, credential, bindings, bundles))
}

async fn apply_all(
    topology: &StorageRuntimeTopologyV1,
    port: u16,
    credential: &Zeroizing<Vec<u8>>,
    bindings: &[StorageBindingV1],
    bundles: &[StorageBundleV1],
) -> Result<(), String> {
    let connector = PostgresAdminConnectorV1::connect_with_password(
        &topology.postgres_host,
        port,
        &topology.database_id,
        PLATFORM_ADMIN_USERNAME,
        credential,
    )
    .await
    .map_err(|_| "Storage PostgreSQL admin authentication is unavailable".to_owned())?;
    for binding in bindings {
        let binding = storage_binding_from_message(binding)
            .map_err(|_| "Storage binding is invalid".to_owned())?;
        let spec = StorageRoleSpecV1::platform_binding(binding)
            .map_err(|_| "Storage role specification is invalid".to_owned())?;
        let bundle = matching_bundle(&spec, bundles)?;
        apply_storage_bundle(&connector, &spec, bundle)
            .await
            .map_err(migration_error)?;
    }
    Ok(())
}

fn migration_error(error: PostgresAdapterErrorV1) -> String {
    match error {
        PostgresAdapterErrorV1::MigrationOwnerRole => {
            "Storage migration owner role is unavailable".to_owned()
        }
        PostgresAdapterErrorV1::MigrationStatement => {
            "Storage migration statement is unavailable".to_owned()
        }
        PostgresAdapterErrorV1::MigrationLedgerRead
        | PostgresAdapterErrorV1::MigrationLedgerWrite => {
            "Storage migration ledger is unavailable".to_owned()
        }
        PostgresAdapterErrorV1::MigrationResetRole => {
            "Storage migration role reset is unavailable".to_owned()
        }
        PostgresAdapterErrorV1::MigrationCommit => {
            "Storage migration commit is unavailable".to_owned()
        }
        PostgresAdapterErrorV1::MigrationPrivileges => {
            "Storage migration privilege reconciliation is unavailable".to_owned()
        }
        _ => "Storage migration application is unavailable".to_owned(),
    }
}

fn matching_bundle<'a>(
    spec: &StorageRoleSpecV1,
    bundles: &'a [StorageBundleV1],
) -> Result<&'a StorageBundleV1, String> {
    let fences = spec.binding().fences();
    bundles
        .iter()
        .find(|bundle| {
            bundle.owner_id == spec.owner_id()
                && u64::from(bundle.revision) == fences.storage_bundle_revision()
                && Sha256::digest(bundle.encode_to_vec()).as_slice()
                    == spec.binding().access().storage_bundle_digest().as_slice()
        })
        .ok_or_else(|| "Storage migration bundle is unavailable".to_owned())
}
