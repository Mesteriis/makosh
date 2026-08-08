//! Adds an immutable managed runtime instance identity to each launch record.

use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "ALTER TABLE makosh_kernel_managed_launch_record
         ADD COLUMN runtime_instance_id TEXT NOT NULL DEFAULT '';
         UPDATE makosh_kernel_managed_launch_record
         SET runtime_instance_id = registration_id || '_legacy'
         WHERE runtime_instance_id = '';
         UPDATE makosh_kernel_control_store_metadata SET schema_version = 19 WHERE singleton = 1;",
    )?;
    Ok(())
}
