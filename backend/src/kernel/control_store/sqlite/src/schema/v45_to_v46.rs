//! Persists exact managed client-safe realtime publisher contracts.

use crate::StoreError;
use rusqlite::Transaction;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_module_client_realtime_route_request (
            registration_id TEXT NOT NULL,
            capability_id TEXT NOT NULL,
            contract_owner TEXT NOT NULL,
            contract_name TEXT NOT NULL,
            contract_major INTEGER NOT NULL CHECK (contract_major >= 1),
            contract_revision INTEGER NOT NULL CHECK (contract_revision >= 1),
            contract_schema_sha256 BLOB NOT NULL CHECK (length(contract_schema_sha256) = 32),
            PRIMARY KEY (
                registration_id, capability_id, contract_owner, contract_name,
                contract_major, contract_revision, contract_schema_sha256
            ),
            FOREIGN KEY (registration_id, capability_id)
                REFERENCES makosh_kernel_module_registration_capability(
                    registration_id, capability_id
                )
                ON DELETE CASCADE
         ) STRICT;

         UPDATE makosh_kernel_control_store_metadata
         SET schema_version = 46 WHERE singleton = 1;",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::apply;
    use rusqlite::Connection;

    #[test]
    fn creates_exact_client_realtime_contract_catalog() {
        let connection = Connection::open_in_memory().expect("connection");
        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON;
                 CREATE TABLE makosh_kernel_control_store_metadata (
                    singleton INTEGER PRIMARY KEY,
                    schema_version INTEGER NOT NULL
                 ) STRICT;
                 INSERT INTO makosh_kernel_control_store_metadata VALUES (1, 45);
                 CREATE TABLE makosh_kernel_module_registration_capability (
                    registration_id TEXT NOT NULL,
                    capability_id TEXT NOT NULL,
                    PRIMARY KEY (registration_id, capability_id)
                 ) STRICT;
                 INSERT INTO makosh_kernel_module_registration_capability
                 VALUES ('delivery', 'status');",
            )
            .expect("v45 fixture");
        let transaction = connection.unchecked_transaction().expect("transaction");
        apply(&transaction).expect("migration");
        transaction.commit().expect("commit");

        connection
            .execute(
                "INSERT INTO makosh_kernel_module_client_realtime_route_request VALUES (
                    'delivery', 'status', 'communication_delivery_intent',
                    'status_changed', 1, 1, zeroblob(32)
                 )",
                [],
            )
            .expect("exact realtime route");
        let version: i64 = connection
            .query_row(
                "SELECT schema_version FROM makosh_kernel_control_store_metadata",
                [],
                |row| row.get(0),
            )
            .expect("schema version");
        assert_eq!(version, 46);
    }
}
