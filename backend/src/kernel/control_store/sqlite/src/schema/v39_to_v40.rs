use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "ALTER TABLE makosh_kernel_module_blob_quota_request
             ADD COLUMN custody_scope_id TEXT NOT NULL DEFAULT '';
         ALTER TABLE makosh_kernel_module_blob_quota_request
             ADD COLUMN allowed_operations INTEGER NOT NULL DEFAULT 0
             CHECK (allowed_operations BETWEEN 0 AND 7);
         UPDATE makosh_kernel_control_store_metadata SET schema_version = 40 WHERE singleton = 1;",
    )?;
    Ok(())
}
