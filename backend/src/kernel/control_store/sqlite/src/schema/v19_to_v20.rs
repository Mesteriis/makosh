//! Adds durable owner-authorized Storage binding fences.

use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_platform_storage_binding (
            registration_id TEXT NOT NULL,
            capability_id TEXT NOT NULL,
            owner_id TEXT NOT NULL,
            binding_revision INTEGER NOT NULL CHECK (binding_revision >= 1),
            topology_revision INTEGER NOT NULL CHECK (topology_revision >= 1),
            storage_generation INTEGER NOT NULL CHECK (storage_generation >= 1),
            runtime_instance_id TEXT NOT NULL,
            runtime_generation INTEGER NOT NULL CHECK (runtime_generation >= 1),
            grant_epoch INTEGER NOT NULL CHECK (grant_epoch >= 1),
            role_epoch INTEGER NOT NULL CHECK (role_epoch >= 1),
            runtime_principal TEXT NOT NULL,
            connection_budget INTEGER NOT NULL CHECK (connection_budget >= 1),
            statement_timeout_millis INTEGER NOT NULL CHECK (statement_timeout_millis >= 1),
            credential_lease_revision INTEGER NOT NULL CHECK (credential_lease_revision >= 1),
            storage_bundle_revision INTEGER NOT NULL CHECK (storage_bundle_revision >= 1),
            storage_bundle_digest BLOB NOT NULL CHECK (length(storage_bundle_digest) = 32),
            PRIMARY KEY (registration_id, capability_id)
        ) STRICT;
        UPDATE makosh_kernel_control_store_metadata SET schema_version = 20 WHERE singleton = 1;",
    )?;
    Ok(())
}
