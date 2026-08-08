use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "DELETE FROM makosh_kernel_platform_storage_topology;
        ALTER TABLE makosh_kernel_platform_storage_topology ADD COLUMN postgres_host TEXT NOT NULL DEFAULT '';
        ALTER TABLE makosh_kernel_platform_storage_topology ADD COLUMN postgres_port INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE makosh_kernel_platform_storage_topology ADD COLUMN pgbouncer_host TEXT NOT NULL DEFAULT '';
        ALTER TABLE makosh_kernel_platform_storage_topology ADD COLUMN pgbouncer_port INTEGER NOT NULL DEFAULT 0;
        UPDATE makosh_kernel_control_store_metadata SET schema_version = 17 WHERE singleton = 1;",
    )?;
    Ok(())
}
