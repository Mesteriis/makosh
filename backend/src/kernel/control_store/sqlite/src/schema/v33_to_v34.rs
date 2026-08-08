//! Adds the small Kernel-owned operator settings singleton.

use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_operator_settings (
            singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
            developer_mode_enabled INTEGER NOT NULL CHECK (developer_mode_enabled IN (0, 1))
         ) STRICT;
         INSERT INTO makosh_kernel_operator_settings (singleton, developer_mode_enabled)
             VALUES (1, 0);
         UPDATE makosh_kernel_control_store_metadata SET schema_version = 34 WHERE singleton = 1;",
    )?;
    Ok(())
}
