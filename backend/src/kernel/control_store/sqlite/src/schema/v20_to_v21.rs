//! Adds immutable canonical Storage bundle records for owner-authorized bindings.

use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_platform_storage_bundle (
            owner_id TEXT NOT NULL,
            revision INTEGER NOT NULL CHECK (revision >= 1),
            sha256 BLOB NOT NULL CHECK (length(sha256) = 32),
            canonical_bytes BLOB NOT NULL CHECK (length(canonical_bytes) BETWEEN 1 AND 4194304),
            PRIMARY KEY (owner_id, revision),
            UNIQUE (owner_id, sha256)
        ) STRICT;
        UPDATE makosh_kernel_control_store_metadata SET schema_version = 21 WHERE singleton = 1;",
    )?;
    Ok(())
}
