use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_external_runtime_attestation (
            registration_id TEXT PRIMARY KEY REFERENCES makosh_kernel_module_registration(registration_id) ON DELETE CASCADE,
            runtime_id TEXT NOT NULL,
            runtime_generation INTEGER NOT NULL CHECK (runtime_generation >= 1),
            grant_epoch INTEGER NOT NULL CHECK (grant_epoch >= 1),
            distribution_sha256 BLOB NOT NULL CHECK (length(distribution_sha256) = 32)
        ) STRICT;
        UPDATE makosh_kernel_control_store_metadata SET schema_version = 6 WHERE singleton = 1;",
    )?;
    Ok(())
}
