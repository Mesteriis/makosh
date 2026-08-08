//! Persists descriptor-declared Storage requests independently of grant approval.

use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_module_storage_request (
            registration_id TEXT NOT NULL,
            capability_id TEXT NOT NULL,
            owner_id TEXT NOT NULL,
            connection_budget INTEGER NOT NULL CHECK (connection_budget >= 1),
            statement_timeout_millis INTEGER NOT NULL CHECK (statement_timeout_millis >= 1),
            PRIMARY KEY (registration_id, capability_id)
        ) STRICT;
        UPDATE makosh_kernel_control_store_metadata SET schema_version = 18 WHERE singleton = 1;",
    )?;
    Ok(())
}
