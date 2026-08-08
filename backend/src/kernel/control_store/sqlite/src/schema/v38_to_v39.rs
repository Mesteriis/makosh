use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_module_client_rpc_route_request (
             registration_id TEXT NOT NULL,
             capability_id TEXT NOT NULL,
             contract_owner TEXT NOT NULL,
             contract_name TEXT NOT NULL,
             contract_major INTEGER NOT NULL CHECK (contract_major > 0),
             contract_revision INTEGER NOT NULL CHECK (contract_revision > 0),
             contract_schema_sha256 BLOB NOT NULL CHECK (length(contract_schema_sha256) = 32),
             path TEXT NOT NULL,
             PRIMARY KEY (registration_id, capability_id, path),
             UNIQUE (registration_id, capability_id, contract_owner, contract_name, contract_major),
             FOREIGN KEY (registration_id, capability_id)
                 REFERENCES makosh_kernel_module_registration_capability(registration_id, capability_id)
                 ON DELETE CASCADE
         ) STRICT;
         UPDATE makosh_kernel_control_store_metadata SET schema_version = 39 WHERE singleton = 1;",
    )?;
    Ok(())
}
