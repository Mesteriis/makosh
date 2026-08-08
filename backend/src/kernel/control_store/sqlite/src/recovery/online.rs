//! Online recovery-fence advancement through the Control Store actor.

use makosh_kernel_control_store::ControlStore;
use rusqlite::params;

use crate::{SqliteControlStore, StoreError};

impl SqliteControlStore {
    pub fn advance_recovery_fences(&self) -> Result<ControlStore, StoreError> {
        let next_generation = increment(self.snapshot().generation())?;
        let next_identity_epoch = increment(self.snapshot().identity_epoch())?;
        let next_grant_epoch = increment(self.snapshot().grant_epoch())?;
        let instance_id = self.snapshot().instance_id().to_owned();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            transaction.execute(
                "UPDATE makosh_kernel_control_store_metadata
                 SET generation = ?1, identity_epoch = ?2, grant_epoch = ?3 WHERE singleton = 1",
                params![
                    as_sql(next_generation)?,
                    as_sql(next_identity_epoch)?,
                    as_sql(next_grant_epoch)?
                ],
            )?;
            transaction.commit()?;
            Ok(ControlStore::with_recovery_fences(
                instance_id,
                next_generation,
                next_identity_epoch,
                next_grant_epoch,
            ))
        })
    }
}

fn increment(value: u64) -> Result<u64, StoreError> {
    value
        .checked_add(1)
        .ok_or(StoreError::RecoveryFenceOverflow)
}

fn as_sql(value: u64) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::RecoveryFenceOverflow)
}
