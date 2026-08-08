//! Adds descriptor-declared authenticated client Blob routes.

use crate::StoreError;
use rusqlite::Transaction;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_module_client_blob_route_request (
            registration_id TEXT NOT NULL
                REFERENCES makosh_kernel_module_registration(registration_id),
            capability_id TEXT NOT NULL,
            contract_owner TEXT NOT NULL CHECK (length(contract_owner) BETWEEN 1 AND 128),
            contract_name TEXT NOT NULL CHECK (length(contract_name) BETWEEN 1 AND 128),
            contract_major INTEGER NOT NULL CHECK (contract_major > 0),
            contract_revision INTEGER NOT NULL CHECK (contract_revision > 0),
            contract_schema_sha256 BLOB NOT NULL CHECK (length(contract_schema_sha256) = 32),
            path TEXT NOT NULL UNIQUE CHECK (length(path) BETWEEN 1 AND 512),
            max_response_bytes INTEGER NOT NULL
                CHECK (max_response_bytes BETWEEN 1 AND 16777216),
            PRIMARY KEY (registration_id, capability_id, path),
            FOREIGN KEY (registration_id, capability_id)
                REFERENCES makosh_kernel_module_registration_capability(registration_id, capability_id)
         ) STRICT;
         UPDATE makosh_kernel_control_store_metadata
         SET schema_version = 42 WHERE singleton = 1;",
    )?;
    Ok(())
}
