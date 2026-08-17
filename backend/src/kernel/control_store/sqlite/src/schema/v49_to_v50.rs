//! Raises the client Blob object ceiling for authenticated range delivery.

use crate::StoreError;
use rusqlite::Transaction;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_module_client_blob_route_request_v50 (
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
                CHECK (max_response_bytes BETWEEN 1 AND 4294967296),
            PRIMARY KEY (registration_id, capability_id, path),
            FOREIGN KEY (registration_id, capability_id)
                REFERENCES makosh_kernel_module_registration_capability(registration_id, capability_id)
         ) STRICT;

         INSERT INTO makosh_kernel_module_client_blob_route_request_v50
         SELECT * FROM makosh_kernel_module_client_blob_route_request;

         DROP TABLE makosh_kernel_module_client_blob_route_request;
         ALTER TABLE makosh_kernel_module_client_blob_route_request_v50
            RENAME TO makosh_kernel_module_client_blob_route_request;

         UPDATE makosh_kernel_control_store_metadata
         SET schema_version = 50 WHERE singleton = 1;",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::apply;

    #[test]
    fn preserves_inline_routes_and_admits_the_range_object_ceiling() {
        let connection = Connection::open_in_memory().expect("connection");
        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON;
                 CREATE TABLE makosh_kernel_control_store_metadata (
                    singleton INTEGER PRIMARY KEY,
                    schema_version INTEGER NOT NULL
                 ) STRICT;
                 INSERT INTO makosh_kernel_control_store_metadata VALUES (1, 49);
                 CREATE TABLE makosh_kernel_module_registration (
                    registration_id TEXT PRIMARY KEY
                 ) STRICT;
                 INSERT INTO makosh_kernel_module_registration VALUES
                    ('existing'), ('range');
                 CREATE TABLE makosh_kernel_module_registration_capability (
                    registration_id TEXT NOT NULL,
                    capability_id TEXT NOT NULL,
                    PRIMARY KEY (registration_id, capability_id),
                    FOREIGN KEY (registration_id)
                        REFERENCES makosh_kernel_module_registration(registration_id)
                 ) STRICT;
                 INSERT INTO makosh_kernel_module_registration_capability VALUES
                    ('existing', 'blob'), ('range', 'blob');
                 CREATE TABLE makosh_kernel_module_client_blob_route_request (
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
                        CHECK (max_response_bytes BETWEEN 1 AND 33554432),
                    PRIMARY KEY (registration_id, capability_id, path),
                    FOREIGN KEY (registration_id, capability_id)
                        REFERENCES makosh_kernel_module_registration_capability(registration_id, capability_id)
                 ) STRICT;
                 INSERT INTO makosh_kernel_module_client_blob_route_request VALUES
                    ('existing', 'blob', 'owner', 'contract', 1, 1, zeroblob(32),
                     '/api/blobs/existing', 33554432);",
            )
            .expect("v49 fixture");

        let transaction = connection.unchecked_transaction().expect("transaction");
        apply(&transaction).expect("migration");
        transaction.commit().expect("commit");
        connection
            .execute(
                "INSERT INTO makosh_kernel_module_client_blob_route_request VALUES
                 ('range', 'blob', 'owner', 'contract', 1, 1, zeroblob(32),
                  '/api/blobs/range', 4294967296)",
                [],
            )
            .expect("4 GiB range object route");
    }
}
