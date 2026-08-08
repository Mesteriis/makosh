use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_settings_schema_binding (
            registration_id TEXT PRIMARY KEY REFERENCES makosh_kernel_module_registration(registration_id) ON DELETE CASCADE,
            schema_major INTEGER NOT NULL CHECK (schema_major >= 1),
            schema_revision INTEGER NOT NULL CHECK (schema_revision >= 1),
            schema_sha256 BLOB NOT NULL CHECK (length(schema_sha256) = 32),
            desired_revision INTEGER NOT NULL CHECK (desired_revision >= 0),
            effective_revision INTEGER NOT NULL CHECK (effective_revision >= 0)
        ) STRICT;
        UPDATE makosh_kernel_control_store_metadata SET schema_version = 7 WHERE singleton = 1;",
    )?;
    Ok(())
}
