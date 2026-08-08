//! Persists WebAuthn backup flags required to reconstruct a credential exactly.

use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "ALTER TABLE makosh_kernel_browser_device_identity
             ADD COLUMN backup_eligible INTEGER NOT NULL DEFAULT 0 CHECK (backup_eligible IN (0, 1));
         ALTER TABLE makosh_kernel_browser_device_identity
             ADD COLUMN backup_state INTEGER NOT NULL DEFAULT 0 CHECK (backup_state IN (0, 1));
         UPDATE makosh_kernel_control_store_metadata SET schema_version = 33 WHERE singleton = 1;",
    )?;
    Ok(())
}
