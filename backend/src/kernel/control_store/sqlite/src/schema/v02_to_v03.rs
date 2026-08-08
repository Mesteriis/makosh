use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_initial_owner_identity (
            singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
            owner_id TEXT NOT NULL,
            device_id TEXT NOT NULL,
            public_key_sec1 BLOB NOT NULL CHECK (length(public_key_sec1) = 65)
        ) STRICT;
        UPDATE makosh_kernel_control_store_metadata SET schema_version = 3 WHERE singleton = 1;",
    )?;
    Ok(())
}
