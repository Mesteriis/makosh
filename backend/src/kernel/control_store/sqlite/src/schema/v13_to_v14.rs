use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_bundled_managed_launch_binding (
            registration_id TEXT PRIMARY KEY REFERENCES makosh_kernel_module_registration(registration_id) ON DELETE CASCADE,
            binding_revision INTEGER NOT NULL CHECK (binding_revision >= 1),
            distribution_id TEXT NOT NULL,
            artifact_id TEXT NOT NULL,
            executable_sha256 BLOB NOT NULL CHECK (length(executable_sha256) = 32),
            descriptor_sha256 BLOB NOT NULL CHECK (length(descriptor_sha256) = 32),
            settings_schema_sha256 BLOB CHECK (settings_schema_sha256 IS NULL OR length(settings_schema_sha256) = 32)
        ) STRICT;
        CREATE TABLE makosh_kernel_managed_launch_record (
            registration_id TEXT PRIMARY KEY REFERENCES makosh_kernel_bundled_managed_launch_binding(registration_id) ON DELETE CASCADE,
            binding_revision INTEGER NOT NULL CHECK (binding_revision >= 1),
            kernel_generation INTEGER NOT NULL CHECK (kernel_generation >= 1),
            runtime_generation INTEGER NOT NULL CHECK (runtime_generation >= 1),
            grant_epoch INTEGER NOT NULL CHECK (grant_epoch >= 1)
        ) STRICT;
        UPDATE makosh_kernel_control_store_metadata SET schema_version = 14 WHERE singleton = 1;",
    )?;
    Ok(())
}
