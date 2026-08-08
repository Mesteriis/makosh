//! SQLite persistence for immutable canonical owner-local Storage bundles.

use makosh_kernel_control_store::PlatformStorageBundleV1;
use rusqlite::{OptionalExtension, params};

use crate::{SqliteControlStore, StoreError};

impl SqliteControlStore {
    pub fn record_platform_storage_bundle(
        &self,
        bundle: &PlatformStorageBundleV1,
    ) -> Result<(), StoreError> {
        let bundle = bundle.clone();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            let changed = transaction.execute(
                "INSERT INTO makosh_kernel_platform_storage_bundle
                 (owner_id, revision, sha256, canonical_bytes)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(owner_id, revision) DO NOTHING",
                params![
                    bundle.owner_id(),
                    as_sql(bundle.revision())?,
                    bundle.digest().as_slice(),
                    bundle.canonical_bytes(),
                ],
            )?;
            if changed == 0 {
                let existing = transaction.query_row(
                    "SELECT sha256, canonical_bytes
                     FROM makosh_kernel_platform_storage_bundle
                     WHERE owner_id = ?1 AND revision = ?2",
                    params![bundle.owner_id(), as_sql(bundle.revision())?],
                    |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?)),
                )?;
                if existing.0.as_slice() != bundle.digest()
                    || existing.1.as_slice() != bundle.canonical_bytes()
                {
                    return Err(StoreError::PlatformStorageBundleRevisionConflict);
                }
            }
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn platform_storage_bundle(
        &self,
        owner_id: &str,
        revision: u64,
    ) -> Result<Option<PlatformStorageBundleV1>, StoreError> {
        let owner_id = owner_id.to_owned();
        self.with_connection(move |connection| {
            connection
                .query_row(
                    "SELECT sha256, canonical_bytes
                     FROM makosh_kernel_platform_storage_bundle
                     WHERE owner_id = ?1 AND revision = ?2",
                    params![owner_id, as_sql(revision)?],
                    |row| decode_bundle(row, &owner_id, revision),
                )
                .optional()
                .map_err(StoreError::from)
        })
    }
}

fn decode_bundle(
    row: &rusqlite::Row<'_>,
    owner_id: &str,
    revision: u64,
) -> Result<PlatformStorageBundleV1, rusqlite::Error> {
    let digest: Vec<u8> = row.get(0)?;
    PlatformStorageBundleV1::new(
        owner_id,
        revision,
        digest
            .try_into()
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(0, 32))?,
        row.get(1)?,
    )
    .map_err(|_| rusqlite::Error::InvalidQuery)
}

fn as_sql(value: u64) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::InvalidPlatformStorageBundle)
}
