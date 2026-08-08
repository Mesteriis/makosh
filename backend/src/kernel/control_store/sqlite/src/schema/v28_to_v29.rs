//! Adds public browser identities without storing assertions or session secrets.

use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_browser_device_identity (
            device_id TEXT PRIMARY KEY,
            owner_id TEXT NOT NULL,
            credential_id BLOB NOT NULL UNIQUE CHECK (length(credential_id) BETWEEN 1 AND 1024),
            cose_public_key BLOB NOT NULL CHECK (length(cose_public_key) BETWEEN 16 AND 1024),
            rp_id TEXT NOT NULL CHECK (length(rp_id) BETWEEN 3 AND 253),
            sign_count INTEGER NOT NULL CHECK (sign_count >= 0),
            state TEXT NOT NULL CHECK (state IN ('active', 'revoked')),
            identity_epoch INTEGER NOT NULL CHECK (identity_epoch >= 1)
        ) STRICT;
        UPDATE makosh_kernel_control_store_metadata SET schema_version = 29 WHERE singleton = 1;",
    )?;
    Ok(())
}
