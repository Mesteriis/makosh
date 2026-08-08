//! Adds a durable revocation reservation to each Storage binding.

use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "ALTER TABLE makosh_kernel_platform_storage_binding
         ADD COLUMN state INTEGER NOT NULL DEFAULT 1 CHECK (state IN (1, 2));
         UPDATE makosh_kernel_control_store_metadata SET schema_version = 22 WHERE singleton = 1;",
    )?;
    Ok(())
}
