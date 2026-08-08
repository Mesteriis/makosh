use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_owner_pinned_artifact_binding (
            registration_id TEXT PRIMARY KEY REFERENCES makosh_kernel_module_registration(registration_id) ON DELETE CASCADE,
            binding_revision INTEGER NOT NULL CHECK (binding_revision >= 1),
            canonical_artifact_path TEXT NOT NULL,
            artifact_sha256 BLOB NOT NULL CHECK (length(artifact_sha256) = 32),
            artifact_size INTEGER NOT NULL CHECK (artifact_size >= 1),
            artifact_device INTEGER NOT NULL CHECK (artifact_device >= 0),
            artifact_inode INTEGER NOT NULL CHECK (artifact_inode >= 1),
            owner_signature_raw BLOB NOT NULL CHECK (length(owner_signature_raw) = 64)
        ) STRICT;
        UPDATE makosh_kernel_control_store_metadata SET schema_version = 10 WHERE singleton = 1;",
    )?;
    Ok(())
}
