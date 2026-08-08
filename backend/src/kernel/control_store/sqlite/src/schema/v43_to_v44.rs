//! Adds configuration-instance-scoped Settings target state and snapshots.

use crate::StoreError;
use rusqlite::Transaction;

pub(super) fn apply(transaction: &Transaction<'_>) -> Result<(), StoreError> {
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_settings_configuration_target (
            registration_id TEXT NOT NULL
                REFERENCES makosh_kernel_settings_schema_binding(registration_id)
                ON DELETE CASCADE,
            configuration_instance_id TEXT NOT NULL,
            desired_revision INTEGER NOT NULL CHECK (desired_revision >= 0),
            effective_revision INTEGER NOT NULL CHECK (effective_revision >= 0),
            apply_state TEXT NOT NULL CHECK (
                apply_state IN (
                    'current',
                    'pending_validation',
                    'pending_apply',
                    'applying',
                    'awaiting_external_restart',
                    'blocked_config'
                )
            ),
            sanitized_reason_code TEXT,
            created_operation_id BLOB,
            PRIMARY KEY (registration_id, configuration_instance_id),
            UNIQUE (registration_id, created_operation_id),
            CHECK (
                created_operation_id IS NULL OR length(created_operation_id) = 16
            )
         ) STRICT;
         INSERT INTO makosh_kernel_settings_configuration_target (
            registration_id,
            configuration_instance_id,
            desired_revision,
            effective_revision,
            apply_state,
            sanitized_reason_code,
            created_operation_id
         )
         SELECT registration_id,
                registration_id,
                desired_revision,
                effective_revision,
                apply_state,
                sanitized_reason_code,
                NULL
         FROM makosh_kernel_settings_schema_binding;

         CREATE TABLE makosh_kernel_settings_desired_snapshot_v44 (
            registration_id TEXT NOT NULL,
            configuration_instance_id TEXT NOT NULL,
            revision INTEGER NOT NULL CHECK (revision >= 1),
            snapshot_bytes BLOB NOT NULL,
            PRIMARY KEY (registration_id, configuration_instance_id),
            FOREIGN KEY (registration_id, configuration_instance_id)
                REFERENCES makosh_kernel_settings_configuration_target(
                    registration_id,
                    configuration_instance_id
                )
                ON DELETE CASCADE
         ) STRICT;
         INSERT INTO makosh_kernel_settings_desired_snapshot_v44 (
            registration_id,
            configuration_instance_id,
            revision,
            snapshot_bytes
         )
         SELECT registration_id,
                registration_id,
                revision,
                snapshot_bytes
         FROM makosh_kernel_settings_desired_snapshot;
         DROP TABLE makosh_kernel_settings_desired_snapshot;
         ALTER TABLE makosh_kernel_settings_desired_snapshot_v44
            RENAME TO makosh_kernel_settings_desired_snapshot;

         UPDATE makosh_kernel_control_store_metadata
         SET schema_version = 44 WHERE singleton = 1;",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::apply;
    use rusqlite::{Connection, params};

    #[test]
    fn preserves_the_legacy_snapshot_as_one_explicit_target() {
        let connection = Connection::open_in_memory().expect("connection");
        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON;
                 CREATE TABLE makosh_kernel_control_store_metadata (
                    singleton INTEGER PRIMARY KEY,
                    schema_version INTEGER NOT NULL
                 ) STRICT;
                 INSERT INTO makosh_kernel_control_store_metadata VALUES (1, 43);
                 CREATE TABLE makosh_kernel_module_registration (
                    registration_id TEXT PRIMARY KEY
                 ) STRICT;
                 INSERT INTO makosh_kernel_module_registration VALUES ('mail-runtime');
                 CREATE TABLE makosh_kernel_settings_schema_binding (
                    registration_id TEXT PRIMARY KEY
                        REFERENCES makosh_kernel_module_registration(registration_id)
                        ON DELETE CASCADE,
                    schema_major INTEGER NOT NULL,
                    schema_revision INTEGER NOT NULL,
                    schema_sha256 BLOB NOT NULL,
                    desired_revision INTEGER NOT NULL,
                    effective_revision INTEGER NOT NULL,
                    apply_state TEXT NOT NULL,
                    sanitized_reason_code TEXT
                 ) STRICT;
                 INSERT INTO makosh_kernel_settings_schema_binding VALUES (
                    'mail-runtime',
                    2,
                    2,
                    zeroblob(32),
                    3,
                    3,
                    'current',
                    NULL
                 );
                 CREATE TABLE makosh_kernel_settings_desired_snapshot (
                    registration_id TEXT PRIMARY KEY
                        REFERENCES makosh_kernel_settings_schema_binding(registration_id)
                        ON DELETE CASCADE,
                    revision INTEGER NOT NULL,
                    snapshot_bytes BLOB NOT NULL
                 ) STRICT;
                 INSERT INTO makosh_kernel_settings_desired_snapshot VALUES (
                    'mail-runtime',
                    3,
                    x'010203'
                 );",
            )
            .expect("v43 settings schema");

        let transaction = connection.unchecked_transaction().expect("transaction");
        apply(&transaction).expect("migration");
        transaction.commit().expect("commit");

        let target: (String, i64, i64, String) = connection
            .query_row(
                "SELECT configuration_instance_id,
                        desired_revision,
                        effective_revision,
                        apply_state
                 FROM makosh_kernel_settings_configuration_target
                 WHERE registration_id = 'mail-runtime'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("target");
        assert_eq!(
            target,
            ("mail-runtime".to_owned(), 3, 3, "current".to_owned())
        );
        let snapshot: (String, i64, Vec<u8>) = connection
            .query_row(
                "SELECT configuration_instance_id, revision, snapshot_bytes
                 FROM makosh_kernel_settings_desired_snapshot
                 WHERE registration_id = ?1",
                params!["mail-runtime"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("snapshot");
        assert_eq!(snapshot, ("mail-runtime".to_owned(), 3, vec![1, 2, 3]));
    }

    #[test]
    fn enforces_target_local_operation_id_uniqueness() {
        let connection = Connection::open_in_memory().expect("connection");
        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON;
                 CREATE TABLE makosh_kernel_control_store_metadata (
                    singleton INTEGER PRIMARY KEY,
                    schema_version INTEGER NOT NULL
                 ) STRICT;
                 INSERT INTO makosh_kernel_control_store_metadata VALUES (1, 43);
                 CREATE TABLE makosh_kernel_module_registration (
                    registration_id TEXT PRIMARY KEY
                 ) STRICT;
                 INSERT INTO makosh_kernel_module_registration VALUES ('mail-runtime');
                 CREATE TABLE makosh_kernel_settings_schema_binding (
                    registration_id TEXT PRIMARY KEY,
                    schema_major INTEGER NOT NULL,
                    schema_revision INTEGER NOT NULL,
                    schema_sha256 BLOB NOT NULL,
                    desired_revision INTEGER NOT NULL,
                    effective_revision INTEGER NOT NULL,
                    apply_state TEXT NOT NULL,
                    sanitized_reason_code TEXT
                 ) STRICT;
                 INSERT INTO makosh_kernel_settings_schema_binding VALUES (
                    'mail-runtime', 2, 2, zeroblob(32), 0, 0, 'current', NULL
                 );
                 CREATE TABLE makosh_kernel_settings_desired_snapshot (
                    registration_id TEXT PRIMARY KEY,
                    revision INTEGER NOT NULL,
                    snapshot_bytes BLOB NOT NULL
                 ) STRICT;",
            )
            .expect("v43 settings schema");
        let transaction = connection.unchecked_transaction().expect("transaction");
        apply(&transaction).expect("migration");
        transaction.commit().expect("commit");

        let operation_id = [7_u8; 16];
        connection
            .execute(
                "INSERT INTO makosh_kernel_settings_configuration_target VALUES (
                    'mail-runtime', 'account-a', 1, 0, 'blocked_config',
                    'required_settings_missing', ?1
                 )",
                [operation_id.as_slice()],
            )
            .expect("first target");
        assert!(
            connection
                .execute(
                    "INSERT INTO makosh_kernel_settings_configuration_target VALUES (
                        'mail-runtime', 'account-b', 1, 0, 'blocked_config',
                        'required_settings_missing', ?1
                     )",
                    [operation_id.as_slice()],
                )
                .is_err()
        );
    }
}
