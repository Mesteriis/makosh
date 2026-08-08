use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_settings_schema_artifact (
            registration_id TEXT PRIMARY KEY REFERENCES makosh_kernel_settings_schema_binding(registration_id) ON DELETE CASCADE,
            schema_bytes BLOB NOT NULL
        ) STRICT;
        UPDATE makosh_kernel_control_store_metadata SET schema_version = 11 WHERE singleton = 1;",
    )?;
    Ok(())
}
