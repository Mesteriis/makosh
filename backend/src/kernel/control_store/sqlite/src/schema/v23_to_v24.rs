//! Adds persisted descriptor-declared Blob quota requests.

use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_module_blob_quota_request (
             registration_id TEXT NOT NULL,
             capability_id TEXT NOT NULL,
             owner_id TEXT NOT NULL,
             max_bytes INTEGER NOT NULL CHECK (max_bytes BETWEEN 1 AND 1099511627776),
             PRIMARY KEY (registration_id, capability_id),
             FOREIGN KEY (registration_id, capability_id)
                 REFERENCES makosh_kernel_module_registration_capability(registration_id, capability_id)
                 ON DELETE CASCADE
         );
         UPDATE makosh_kernel_control_store_metadata SET schema_version = 24 WHERE singleton = 1;",
    )?;
    Ok(())
}
