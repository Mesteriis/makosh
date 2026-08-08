//! Adds the exact idempotency journal for owner-control mutations.

use crate::StoreError;
use rusqlite::Transaction;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_operation_journal (
            operation_id BLOB PRIMARY KEY NOT NULL CHECK (length(operation_id) = 16),
            request_digest BLOB NOT NULL CHECK (length(request_digest) = 32),
            deadline_unix_millis INTEGER NOT NULL CHECK (deadline_unix_millis >= 0),
            terminal_kind TEXT NULL CHECK (terminal_kind IN ('succeeded', 'rejected', 'failed')),
            terminal_payload BLOB NULL,
            CHECK ((terminal_kind IS NULL AND terminal_payload IS NULL) OR (terminal_kind IS NOT NULL AND terminal_payload IS NOT NULL))
         ) STRICT;
         UPDATE makosh_kernel_control_store_metadata SET schema_version = 35 WHERE singleton = 1;",
    )?;
    Ok(())
}
