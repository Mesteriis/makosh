//! Adds the atomic idempotency receipt for verified bundled artifact proposals.

use crate::StoreError;
use rusqlite::Transaction;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_bundled_artifact_proposal (
            operation_id BLOB PRIMARY KEY NOT NULL CHECK (length(operation_id) = 16),
            request_digest BLOB NOT NULL CHECK (length(request_digest) = 32),
            registration_id TEXT NOT NULL UNIQUE
                REFERENCES makosh_kernel_module_registration(registration_id),
            distribution_id TEXT NOT NULL CHECK (length(distribution_id) BETWEEN 1 AND 128),
            distribution_generation INTEGER NOT NULL CHECK (distribution_generation > 0),
            artifact_id TEXT NOT NULL CHECK (length(artifact_id) BETWEEN 1 AND 128),
            descriptor_sha256 BLOB NOT NULL CHECK (length(descriptor_sha256) = 32)
         ) STRICT;
         UPDATE makosh_kernel_control_store_metadata
         SET schema_version = 41 WHERE singleton = 1;",
    )?;
    Ok(())
}
