//! Adds explicit delivery policy for Event consumers without inventing legacy defaults.

use rusqlite::Transaction;

use crate::StoreError;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_module_event_delivery_policy (
             registration_id TEXT NOT NULL,
             capability_id TEXT NOT NULL,
             envelope_kind INTEGER NOT NULL CHECK (envelope_kind BETWEEN 1 AND 5),
             contract_owner TEXT NOT NULL,
             contract_name TEXT NOT NULL,
             contract_major INTEGER NOT NULL CHECK (contract_major > 0),
             direction INTEGER NOT NULL CHECK (direction = 2),
             subscription_requirement INTEGER NOT NULL CHECK (subscription_requirement IN (1, 2)),
             max_deliver INTEGER NOT NULL CHECK (max_deliver BETWEEN 1 AND 32),
             ack_wait_millis INTEGER NOT NULL CHECK (ack_wait_millis BETWEEN 1 AND 600000),
             PRIMARY KEY (registration_id, capability_id, envelope_kind, contract_owner,
                          contract_name, contract_major, direction),
             FOREIGN KEY (registration_id, capability_id, envelope_kind, contract_owner,
                          contract_name, contract_major, direction)
                 REFERENCES makosh_kernel_module_event_route_request(registration_id, capability_id,
                     envelope_kind, contract_owner, contract_name, contract_major, direction)
                 ON DELETE CASCADE
         );
         UPDATE makosh_kernel_control_store_metadata SET schema_version = 25 WHERE singleton = 1;",
    )?;
    Ok(())
}
