//! Extends the persisted Blob operation mask with explicit custody release.

use crate::StoreError;
use rusqlite::Transaction;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_module_blob_quota_request_v48 (
             registration_id TEXT NOT NULL,
             capability_id TEXT NOT NULL,
             owner_id TEXT NOT NULL,
             max_bytes INTEGER NOT NULL CHECK (max_bytes BETWEEN 1 AND 1099511627776),
             custody_scope_id TEXT NOT NULL DEFAULT '',
             allowed_operations INTEGER NOT NULL DEFAULT 0
                 CHECK (allowed_operations BETWEEN 0 AND 15),
             PRIMARY KEY (registration_id, capability_id),
             FOREIGN KEY (registration_id, capability_id)
                 REFERENCES makosh_kernel_module_registration_capability(registration_id, capability_id)
                 ON DELETE CASCADE
         );

         INSERT INTO makosh_kernel_module_blob_quota_request_v48 (
             registration_id,
             capability_id,
             owner_id,
             max_bytes,
             custody_scope_id,
             allowed_operations
         )
         SELECT
             registration_id,
             capability_id,
             owner_id,
             max_bytes,
             custody_scope_id,
             allowed_operations
         FROM makosh_kernel_module_blob_quota_request;

         DROP TABLE makosh_kernel_module_blob_quota_request;
         ALTER TABLE makosh_kernel_module_blob_quota_request_v48
             RENAME TO makosh_kernel_module_blob_quota_request;

         UPDATE makosh_kernel_control_store_metadata
         SET schema_version = 48 WHERE singleton = 1;",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::apply;
    use rusqlite::Connection;

    #[test]
    fn admits_custody_release_without_losing_existing_blob_requests() {
        let connection = Connection::open_in_memory().expect("connection");
        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON;
                 CREATE TABLE makosh_kernel_control_store_metadata (
                    singleton INTEGER PRIMARY KEY,
                    schema_version INTEGER NOT NULL
                 );
                 INSERT INTO makosh_kernel_control_store_metadata VALUES (1, 47);
                 CREATE TABLE makosh_kernel_module_registration_capability (
                    registration_id TEXT NOT NULL,
                    capability_id TEXT NOT NULL,
                    PRIMARY KEY (registration_id, capability_id)
                 );
                 INSERT INTO makosh_kernel_module_registration_capability
                 VALUES ('delayed', 'blob');
                 CREATE TABLE makosh_kernel_module_blob_quota_request (
                    registration_id TEXT NOT NULL,
                    capability_id TEXT NOT NULL,
                    owner_id TEXT NOT NULL,
                    max_bytes INTEGER NOT NULL CHECK (max_bytes BETWEEN 1 AND 1099511627776),
                    custody_scope_id TEXT NOT NULL DEFAULT '',
                    allowed_operations INTEGER NOT NULL DEFAULT 0
                        CHECK (allowed_operations BETWEEN 0 AND 7),
                    PRIMARY KEY (registration_id, capability_id),
                    FOREIGN KEY (registration_id, capability_id)
                        REFERENCES makosh_kernel_module_registration_capability(
                            registration_id, capability_id
                        )
                        ON DELETE CASCADE
                 );
                 INSERT INTO makosh_kernel_module_blob_quota_request
                 VALUES ('delayed', 'blob', 'communication_delayed_delivery', 4096, 'delayed', 7);",
            )
            .expect("v47 fixture");

        let transaction = connection.unchecked_transaction().expect("transaction");
        apply(&transaction).expect("migration");
        transaction.commit().expect("commit");

        connection
            .execute(
                "UPDATE makosh_kernel_module_blob_quota_request
                 SET allowed_operations = 15
                 WHERE registration_id = 'delayed' AND capability_id = 'blob'",
                [],
            )
            .expect("custody release operation");
        assert!(
            connection
                .execute(
                    "UPDATE makosh_kernel_module_blob_quota_request
                     SET allowed_operations = 16
                     WHERE registration_id = 'delayed' AND capability_id = 'blob'",
                    [],
                )
                .is_err()
        );
        let (version, operations): (i64, i64) = connection
            .query_row(
                "SELECT
                    (SELECT schema_version FROM makosh_kernel_control_store_metadata),
                    allowed_operations
                 FROM makosh_kernel_module_blob_quota_request
                 WHERE registration_id = 'delayed' AND capability_id = 'blob'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("migrated request");
        assert_eq!((version, operations), (48, 15));
    }
}
