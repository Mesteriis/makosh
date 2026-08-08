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
        migrate_schema(&connection)?;
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
