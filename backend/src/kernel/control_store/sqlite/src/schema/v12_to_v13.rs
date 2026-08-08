use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_server_bootstrap_pairing (
            singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
            token_sha256 BLOB NOT NULL CHECK (length(token_sha256) = 32),
            certificate_sha256 BLOB NOT NULL CHECK (length(certificate_sha256) = 32),
            challenge BLOB NOT NULL CHECK (length(challenge) = 32),
            expires_at_unix_ms INTEGER NOT NULL CHECK (expires_at_unix_ms >= 1),
            status TEXT NOT NULL CHECK (status IN ('active', 'consumed', 'expired'))
        ) STRICT;
        UPDATE makosh_kernel_control_store_metadata SET schema_version = 13 WHERE singleton = 1;",
    )?;
    Ok(())
}
