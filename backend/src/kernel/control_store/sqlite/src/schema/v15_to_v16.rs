use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_platform_storage_topology (
            singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
            revision INTEGER NOT NULL CHECK (revision >= 1),
            storage_generation INTEGER NOT NULL CHECK (storage_generation >= 1),
            storage_instance_id TEXT NOT NULL,
            database_id TEXT NOT NULL,
            deployment_profile TEXT NOT NULL CHECK (deployment_profile IN ('macos_tauri_embedded', 'linux_docker_server')),
            postgres_artifact_sha256 BLOB NOT NULL CHECK (length(postgres_artifact_sha256) = 32),
            pgbouncer_artifact_sha256 BLOB NOT NULL CHECK (length(pgbouncer_artifact_sha256) = 32)
        ) STRICT;
        UPDATE makosh_kernel_control_store_metadata SET schema_version = 16 WHERE singleton = 1;",
    )?;
    Ok(())
}
