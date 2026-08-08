//! Persists the PostgreSQL endpoint as resolved from the PgBouncer namespace.

use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "ALTER TABLE makosh_kernel_platform_storage_topology ADD COLUMN pgbouncer_backend_host TEXT NOT NULL DEFAULT '';
         ALTER TABLE makosh_kernel_platform_storage_topology ADD COLUMN pgbouncer_backend_port INTEGER NOT NULL DEFAULT 0;
         UPDATE makosh_kernel_platform_storage_topology
            SET pgbouncer_backend_host = postgres_host,
                pgbouncer_backend_port = postgres_port;
         UPDATE makosh_kernel_control_store_metadata SET schema_version = 31 WHERE singleton = 1;",
    )?;
    Ok(())
}
