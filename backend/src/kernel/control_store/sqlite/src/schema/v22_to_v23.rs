//! Adds persisted descriptor-declared Event Hub route requests.

use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_module_event_route_request (
             registration_id TEXT NOT NULL,
             capability_id TEXT NOT NULL,
             envelope_kind INTEGER NOT NULL CHECK (envelope_kind BETWEEN 1 AND 5),
             contract_owner TEXT NOT NULL,
             contract_name TEXT NOT NULL,
             contract_major INTEGER NOT NULL CHECK (contract_major > 0),
             contract_revision INTEGER NOT NULL CHECK (contract_revision > 0),
             contract_schema_sha256 BLOB NOT NULL CHECK (length(contract_schema_sha256) = 32),
             direction INTEGER NOT NULL CHECK (direction IN (1, 2)),
             max_in_flight INTEGER NOT NULL CHECK (max_in_flight BETWEEN 1 AND 4096),
             PRIMARY KEY (registration_id, capability_id, envelope_kind, contract_owner,
                          contract_name, contract_major, direction),
             FOREIGN KEY (registration_id, capability_id)
                 REFERENCES makosh_kernel_module_registration_capability(registration_id, capability_id)
                 ON DELETE CASCADE
         );
         UPDATE makosh_kernel_control_store_metadata SET schema_version = 23 WHERE singleton = 1;",
    )?;
    Ok(())
}
