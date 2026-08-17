//! Control Store construction and immutable installation metadata.

use std::path::{Path, PathBuf};

use makosh_kernel_control_store::ControlStore;
use rusqlite::{Connection, OptionalExtension, params};

use crate::StoreError;
use crate::actor::handle::ControlStoreHandle;
use crate::database::connection::{configure_writable, validate_quick_check};
use crate::schema::{SCHEMA_VERSION, migrate_schema};

pub struct SqliteControlStore {
    pub(crate) handle: ControlStoreHandle,
    pub(crate) path: PathBuf,
    snapshot: ControlStore,
}

impl SqliteControlStore {
    pub fn create(path: &Path, instance_id: &str, generation: u64) -> Result<Self, StoreError> {
        let generation_sql =
            i64::try_from(generation).map_err(|_| StoreError::InvalidGeneration)?;
        let connection = Connection::open(path)?;
        configure_writable(&connection)?;
        create_version_one(&connection, instance_id, generation_sql)?;
        migrate_schema(&connection)?;
        Self::from_connection(path, connection)
    }

    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let connection = Connection::open(path)?;
        configure_writable(&connection)?;
        validate_quick_check(&connection)?;
        migrate_legacy_table_namespace(&connection)?;
        migrate_legacy_product_identifiers(&connection)?;
        migrate_schema(&connection)?;
        migrate_legacy_module_identities(&connection)?;
        Self::from_connection(path, connection)
    }

    #[must_use]
    pub fn snapshot(&self) -> &ControlStore {
        &self.snapshot
    }

    pub(crate) fn with_connection<T, F>(&self, operation: F) -> Result<T, StoreError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, StoreError> + Send + 'static,
    {
        self.handle.call(operation)
    }

    pub(crate) fn with_maintenance_connection<T, F>(&self, operation: F) -> Result<T, StoreError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, StoreError> + Send + 'static,
    {
        self.handle.maintenance(operation)
    }

    fn from_connection(path: &Path, connection: Connection) -> Result<Self, StoreError> {
        let metadata = read_metadata(&connection)?;
        if metadata.0 != SCHEMA_VERSION {
            return Err(StoreError::UnsupportedSchema(metadata.0));
        }
        let snapshot = ControlStore::with_recovery_fences(
            metadata.1,
            as_fence(metadata.2)?,
            as_fence(metadata.3)?,
            as_fence(metadata.4)?,
        );
        Ok(Self {
            handle: ControlStoreHandle::spawn(connection)?,
            path: path.to_owned(),
            snapshot,
        })
    }
}

fn create_version_one(
    connection: &Connection,
    instance_id: &str,
    generation: i64,
) -> Result<(), StoreError> {
    let transaction = connection.unchecked_transaction()?;
    transaction.execute_batch(
        "CREATE TABLE makosh_kernel_control_store_metadata (
            singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
            schema_version INTEGER NOT NULL,
            instance_id TEXT NOT NULL,
            generation INTEGER NOT NULL CHECK (generation >= 1)
        ) STRICT;",
    )?;
    transaction.execute(
        "INSERT INTO makosh_kernel_control_store_metadata
         (singleton, schema_version, instance_id, generation) VALUES (1, 1, ?1, ?2)",
        params![instance_id, generation],
    )?;
    transaction.commit()?;
    Ok(())
}

fn read_metadata(connection: &Connection) -> Result<(i64, String, i64, i64, i64), StoreError> {
    connection
        .query_row(
            "SELECT schema_version, instance_id, generation, identity_epoch, grant_epoch
             FROM makosh_kernel_control_store_metadata WHERE singleton = 1",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .optional()?
        .ok_or(StoreError::MissingMetadata)
}

fn as_fence(value: i64) -> Result<u64, StoreError> {
    u64::try_from(value).map_err(|_| StoreError::InvalidGeneration)
}

fn migrate_legacy_table_namespace(connection: &Connection) -> Result<(), StoreError> {
    const LEGACY_PREFIX: &str = "hermes_kernel_";
    const CURRENT_PREFIX: &str = "makosh_kernel_";
    const LEGACY_METADATA: &str = "hermes_kernel_control_store_metadata";

    let legacy_tables = {
        let mut statement = connection.prepare(
            "SELECT name FROM sqlite_master
             WHERE type = 'table' AND name GLOB 'hermes_kernel_*'
             ORDER BY name",
        )?;
        statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?
    };
    if legacy_tables.is_empty() {
        return Ok(());
    }
    if !legacy_tables.iter().any(|name| name == LEGACY_METADATA)
        || legacy_tables.iter().any(|name| {
            !name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        })
    {
        return Err(StoreError::LegacyNamespaceConflict);
    }

    let current_table_count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM sqlite_master
         WHERE type = 'table' AND name GLOB 'makosh_kernel_*'",
        [],
        |row| row.get(0),
    )?;
    if current_table_count != 0 {
        return Err(StoreError::LegacyNamespaceConflict);
    }

    let transaction = connection.unchecked_transaction()?;
    for legacy_name in legacy_tables {
        let suffix = legacy_name
            .strip_prefix(LEGACY_PREFIX)
            .ok_or(StoreError::LegacyNamespaceConflict)?;
        let current_name = format!("{CURRENT_PREFIX}{suffix}");
        transaction.execute_batch(&format!(
            "ALTER TABLE \"{legacy_name}\" RENAME TO \"{current_name}\";"
        ))?;
    }
    transaction.commit()?;
    Ok(())
}

fn migrate_legacy_product_identifiers(connection: &Connection) -> Result<(), StoreError> {
    let topology_table_exists: i64 = connection.query_row(
        "SELECT COUNT(*) FROM sqlite_master
         WHERE type = 'table' AND name = 'makosh_kernel_platform_storage_topology'",
        [],
        |row| row.get(0),
    )?;
    if topology_table_exists == 0 {
        return Ok(());
    }
    connection.execute(
        "UPDATE makosh_kernel_platform_storage_topology
         SET database_id = 'makosh_storage_authenticated'
         WHERE singleton = 1 AND database_id = 'hermes_storage_authenticated'",
        [],
    )?;
    Ok(())
}

fn migrate_legacy_module_identities(connection: &Connection) -> Result<(), StoreError> {
    const SUCCESSORS: [(&str, &str, &str, &str); 16] = [
        (
            "hermes-communications-runtime",
            "communications",
            "makosh-communications-runtime",
            "communications",
        ),
        (
            "hermes-communications-export-runtime",
            "communications_export",
            "makosh-communications-export-runtime",
            "communications_export",
        ),
        (
            "hermes-communication-delivery-intent-runtime",
            "communication_delivery_intent",
            "makosh-communication-delivery-intent-runtime",
            "communication_delivery_intent",
        ),
        (
            "hermes-communication-bulk-action-runtime",
            "communication_bulk_action",
            "makosh-communication-bulk-action-runtime",
            "communication_bulk_action",
        ),
        (
            "hermes-communication-delayed-delivery-runtime",
            "communication_delayed_delivery",
            "makosh-communication-delayed-delivery-runtime",
            "communication_delayed_delivery",
        ),
        (
            "hermes-attachment-security-runtime",
            "attachment_security",
            "makosh-attachment-security-runtime",
            "attachment_security",
        ),
        (
            "hermes-attachment-text-extraction-runtime",
            "attachment_text_extraction",
            "makosh-attachment-text-extraction-runtime",
            "attachment_text_extraction",
        ),
        (
            "hermes-attachment-preview-runtime",
            "attachment_preview",
            "makosh-attachment-preview-runtime",
            "attachment_preview",
        ),
        (
            "hermes-attachment-preview-evidence-replay-runtime",
            "attachment_preview_evidence_replay",
            "makosh-attachment-preview-evidence-replay-runtime",
            "attachment_preview_evidence_replay",
        ),
        (
            "hermes-attachment-translation-runtime",
            "attachment_translation",
            "makosh-attachment-translation-runtime",
            "attachment_translation",
        ),
        ("hermes-mail-runtime", "mail", "makosh-mail-runtime", "mail"),
        (
            "hermes-telegram-runtime",
            "telegram",
            "makosh-telegram-runtime",
            "telegram",
        ),
        (
            "hermes-whatsapp-runtime",
            "whatsapp",
            "makosh-whatsapp-runtime",
            "whatsapp",
        ),
        (
            "hermes-zulip-runtime",
            "zulip",
            "makosh-zulip-runtime",
            "zulip",
        ),
        (
            "hermes-contacts-runtime",
            "contacts",
            "makosh-persons-runtime",
            "persons",
        ),
        (
            "hermes-mail-contacts-sync-runtime",
            "mail_contacts_sync",
            "makosh-mail-persons-sync-runtime",
            "mail_persons_sync",
        ),
    ];

    let table_exists: i64 = connection.query_row(
        "SELECT COUNT(*) FROM sqlite_master
         WHERE type = 'table' AND name = 'makosh_kernel_module_registration'",
        [],
        |row| row.get(0),
    )?;
    if table_exists == 0 {
        return Ok(());
    }

    let transaction = connection.unchecked_transaction()?;
    for (legacy_module, legacy_owner, successor_module, successor_owner) in SUCCESSORS {
        let unexpected_owner_count: i64 = transaction.query_row(
            "SELECT COUNT(*) FROM makosh_kernel_module_registration
             WHERE module_id = ?1 AND owner_id <> ?2",
            params![legacy_module, legacy_owner],
            |row| row.get(0),
        )?;
        if unexpected_owner_count != 0 {
            return Err(StoreError::LegacyNamespaceConflict);
        }
        transaction.execute(
            "UPDATE makosh_kernel_module_registration
             SET module_id = ?1, owner_id = ?2
             WHERE module_id = ?3 AND owner_id = ?4",
            params![
                successor_module,
                successor_owner,
                legacy_module,
                legacy_owner
            ],
        )?;
    }
    transaction.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_complete_legacy_table_namespace_atomically() {
        let connection = Connection::open_in_memory().expect("open sqlite");
        connection
            .execute_batch(
                "CREATE TABLE hermes_kernel_control_store_metadata (value TEXT) STRICT;
                 INSERT INTO hermes_kernel_control_store_metadata VALUES ('preserved');
                 CREATE TABLE hermes_kernel_module_registration (value INTEGER) STRICT;
                 INSERT INTO hermes_kernel_module_registration VALUES (7);",
            )
            .expect("create legacy namespace");

        migrate_legacy_table_namespace(&connection).expect("migrate namespace");

        let metadata: String = connection
            .query_row(
                "SELECT value FROM makosh_kernel_control_store_metadata",
                [],
                |row| row.get(0),
            )
            .expect("read migrated metadata");
        let registration: i64 = connection
            .query_row(
                "SELECT value FROM makosh_kernel_module_registration",
                [],
                |row| row.get(0),
            )
            .expect("read migrated registration");
        assert_eq!(metadata, "preserved");
        assert_eq!(registration, 7);
    }

    #[test]
    fn rejects_mixed_legacy_and_current_table_namespaces() {
        let connection = Connection::open_in_memory().expect("open sqlite");
        connection
            .execute_batch(
                "CREATE TABLE hermes_kernel_control_store_metadata (value TEXT) STRICT;
                 CREATE TABLE makosh_kernel_module_registration (value INTEGER) STRICT;",
            )
            .expect("create mixed namespaces");

        let error = migrate_legacy_table_namespace(&connection)
            .expect_err("mixed namespaces must fail closed");

        assert!(matches!(error, StoreError::LegacyNamespaceConflict));
    }

    #[test]
    fn migrates_exact_legacy_storage_database_identity() {
        let connection = Connection::open_in_memory().expect("open sqlite");
        connection
            .execute_batch(
                "CREATE TABLE makosh_kernel_platform_storage_topology (
                    singleton INTEGER PRIMARY KEY,
                    database_id TEXT NOT NULL
                 ) STRICT;
                 INSERT INTO makosh_kernel_platform_storage_topology
                    VALUES (1, 'hermes_storage_authenticated');",
            )
            .expect("create legacy topology");

        migrate_legacy_product_identifiers(&connection).expect("migrate database identity");

        let database_id: String = connection
            .query_row(
                "SELECT database_id FROM makosh_kernel_platform_storage_topology
                 WHERE singleton = 1",
                [],
                |row| row.get(0),
            )
            .expect("read database identity");
        assert_eq!(database_id, "makosh_storage_authenticated");
    }

    #[test]
    fn migrates_only_exact_legacy_module_identity_successors() {
        let connection = Connection::open_in_memory().expect("open sqlite");
        connection
            .execute_batch(
                "CREATE TABLE makosh_kernel_module_registration (
                    registration_id TEXT PRIMARY KEY,
                    module_id TEXT NOT NULL,
                    owner_id TEXT NOT NULL
                 ) STRICT;
                 INSERT INTO makosh_kernel_module_registration VALUES
                    ('communications', 'hermes-communications-runtime', 'communications'),
                    ('persons', 'hermes-contacts-runtime', 'contacts'),
                    ('current', 'makosh-mail-runtime', 'mail');",
            )
            .expect("create registrations");

        migrate_legacy_module_identities(&connection).expect("migrate identities");

        let identities = connection
            .prepare(
                "SELECT registration_id, module_id, owner_id
                 FROM makosh_kernel_module_registration ORDER BY registration_id",
            )
            .expect("prepare identities")
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .expect("query identities")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect identities");
        assert_eq!(
            identities,
            vec![
                (
                    "communications".to_owned(),
                    "makosh-communications-runtime".to_owned(),
                    "communications".to_owned(),
                ),
                (
                    "current".to_owned(),
                    "makosh-mail-runtime".to_owned(),
                    "mail".to_owned(),
                ),
                (
                    "persons".to_owned(),
                    "makosh-persons-runtime".to_owned(),
                    "persons".to_owned(),
                ),
            ],
        );
    }

    #[test]
    fn rejects_legacy_module_identity_with_an_unexpected_owner() {
        let connection = Connection::open_in_memory().expect("open sqlite");
        connection
            .execute_batch(
                "CREATE TABLE makosh_kernel_module_registration (
                    registration_id TEXT PRIMARY KEY,
                    module_id TEXT NOT NULL,
                    owner_id TEXT NOT NULL
                 ) STRICT;
                 INSERT INTO makosh_kernel_module_registration VALUES
                    ('unexpected', 'hermes-telegram-runtime', 'mail');",
            )
            .expect("create registration");

        assert!(matches!(
            migrate_legacy_module_identities(&connection),
            Err(StoreError::LegacyNamespaceConflict)
        ));
    }
}
