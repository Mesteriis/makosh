use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "
        ALTER TABLE makosh_kernel_control_store_metadata
            ADD COLUMN identity_epoch INTEGER NOT NULL DEFAULT 1 CHECK (identity_epoch >= 1);
        ALTER TABLE makosh_kernel_control_store_metadata
            ADD COLUMN grant_epoch INTEGER NOT NULL DEFAULT 1 CHECK (grant_epoch >= 1);
        UPDATE makosh_kernel_control_store_metadata SET schema_version = 2 WHERE singleton = 1;
        ",
    )?;
    Ok(())
}
