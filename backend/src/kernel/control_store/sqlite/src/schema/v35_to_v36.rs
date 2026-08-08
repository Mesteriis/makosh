//! Removes the persistent LAN-development authorization bypass.

use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "DROP TABLE makosh_kernel_operator_settings;
         UPDATE makosh_kernel_control_store_metadata SET schema_version = 36 WHERE singleton = 1;",
    )?;
    Ok(())
}
