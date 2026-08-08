use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_module_registration (
            registration_id TEXT PRIMARY KEY,
            module_id TEXT NOT NULL,
            owner_id TEXT NOT NULL,
            descriptor_sha256 BLOB NOT NULL CHECK (length(descriptor_sha256) = 32),
            state TEXT NOT NULL CHECK (state IN ('pending', 'approved', 'suspended', 'revoked', 'blocked_incompatible')),
            grant_epoch INTEGER NOT NULL CHECK (grant_epoch >= 1)
        ) STRICT;
        UPDATE makosh_kernel_control_store_metadata SET schema_version = 4 WHERE singleton = 1;",
    )?;
    Ok(())
}
