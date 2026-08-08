//! Initial owner identity persistence through the single-writer actor.

use makosh_kernel_control_store::InitialOwnerIdentity;
use rusqlite::{OptionalExtension, params};

use crate::{SqliteControlStore, StoreError, valid_identity_token};

impl SqliteControlStore {
    pub fn initial_owner_identity(&self) -> Result<Option<InitialOwnerIdentity>, StoreError> {
        self.with_connection(|connection| {
            connection
                .query_row(
                    "SELECT owner_id, device_id, public_key_sec1
                     FROM makosh_kernel_initial_owner_identity WHERE singleton = 1",
                    [],
                    |row| {
                        let key: Vec<u8> = row.get(2)?;
                        let public_key_sec1 = key
                            .try_into()
                            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(2, 65))?;
                        Ok(InitialOwnerIdentity::new(
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            public_key_sec1,
                        ))
                    },
                )
                .optional()
                .map_err(StoreError::from)
        })
    }

    pub fn claim_initial_owner(&self, identity: &InitialOwnerIdentity) -> Result<(), StoreError> {
        validate_identity(identity)?;
        let identity = identity.clone();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            let changed = transaction.execute(
                "INSERT OR IGNORE INTO makosh_kernel_initial_owner_identity
                 (singleton, owner_id, device_id, public_key_sec1) VALUES (1, ?1, ?2, ?3)",
                params![
                    identity.owner_id(),
                    identity.device_id(),
                    identity.public_key_sec1().as_slice()
                ],
            )?;
            if changed != 1 {
                return Err(StoreError::InitialOwnerAlreadyClaimed);
            }
            transaction.commit()?;
            Ok(())
        })
    }
}

fn validate_identity(identity: &InitialOwnerIdentity) -> Result<(), StoreError> {
    if valid_identity_token(identity.owner_id())
        && valid_identity_token(identity.device_id())
        && identity.public_key_sec1()[0] == 0x04
    {
        Ok(())
    } else {
        Err(StoreError::InvalidInitialOwnerIdentity)
    }
}
