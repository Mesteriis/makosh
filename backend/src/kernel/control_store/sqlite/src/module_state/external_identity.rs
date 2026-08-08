//! Owner-pinned external runtime public identities.

use makosh_kernel_control_store::{
    ExternalRuntimeIdentity, ModuleRegistration, ModuleRegistrationState,
};
use rusqlite::{OptionalExtension, params};

use crate::module_state::registry::read_required_registration;
use crate::{SqliteControlStore, StoreError, valid_identity_token};

impl SqliteControlStore {
    pub fn bind_external_runtime_identity(
        &self,
        identity: &ExternalRuntimeIdentity,
    ) -> Result<ModuleRegistration, StoreError> {
        validate_identity(identity)?;
        let identity = identity.clone();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            let current = read_required_registration(&transaction, identity.registration_id())?;
            if current.state() == ModuleRegistrationState::Revoked {
                return Err(StoreError::InvalidExternalRuntimeIdentity);
            }
            let existing = transaction
                .query_row(
                    "SELECT public_key_sec1 FROM makosh_kernel_external_runtime_identity
                     WHERE registration_id = ?1",
                    [identity.registration_id()],
                    |row| row.get::<_, Vec<u8>>(0),
                )
                .optional()?;
            if existing.as_deref() == Some(identity.public_key_sec1().as_slice()) {
                return Ok(current);
            }
            let rebound = bind_identity(&transaction, &identity);
            if let Err(error) = rebound {
                return map_binding_error(error);
            }
            let next_epoch = current
                .grant_epoch()
                .checked_add(1)
                .ok_or(StoreError::RecoveryFenceOverflow)?;
            update_registration_epoch(&transaction, &current, next_epoch)?;
            transaction.commit()?;
            Ok(ModuleRegistration::new(
                current.registration_id(),
                current.module_id(),
                current.owner_id(),
                *current.descriptor_sha256(),
                current.state(),
                next_epoch,
            ))
        })
    }

    pub fn external_runtime_identity(
        &self,
        registration_id: &str,
    ) -> Result<Option<ExternalRuntimeIdentity>, StoreError> {
        let registration_id = registration_id.to_owned();
        self.with_connection(move |connection| {
            connection
                .query_row(
                    "SELECT public_key_sec1 FROM makosh_kernel_external_runtime_identity
                     WHERE registration_id = ?1",
                    [&registration_id],
                    |row| {
                        let public_key: Vec<u8> = row.get(0)?;
                        let public_key_sec1 = public_key
                            .try_into()
                            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(0, 65))?;
                        Ok(ExternalRuntimeIdentity::new(
                            &registration_id,
                            public_key_sec1,
                        ))
                    },
                )
                .optional()
                .map_err(StoreError::from)
        })
    }
}

fn validate_identity(identity: &ExternalRuntimeIdentity) -> Result<(), StoreError> {
    if valid_identity_token(identity.registration_id()) && identity.public_key_sec1()[0] == 0x04 {
        Ok(())
    } else {
        Err(StoreError::InvalidExternalRuntimeIdentity)
    }
}

fn bind_identity(
    connection: &rusqlite::Connection,
    identity: &ExternalRuntimeIdentity,
) -> Result<usize, rusqlite::Error> {
    connection.execute(
        "INSERT INTO makosh_kernel_external_runtime_identity (registration_id, public_key_sec1)
         VALUES (?1, ?2) ON CONFLICT(registration_id)
         DO UPDATE SET public_key_sec1 = excluded.public_key_sec1",
        params![
            identity.registration_id(),
            identity.public_key_sec1().as_slice()
        ],
    )
}

fn map_binding_error(error: rusqlite::Error) -> Result<ModuleRegistration, StoreError> {
    if error.sqlite_error_code() == Some(rusqlite::ErrorCode::ConstraintViolation) {
        Err(StoreError::ExternalRuntimeIdentityAlreadyBound)
    } else {
        Err(StoreError::Sqlite(error))
    }
}

fn update_registration_epoch(
    connection: &rusqlite::Connection,
    current: &ModuleRegistration,
    next_epoch: u64,
) -> Result<(), StoreError> {
    let changed = connection.execute(
        "UPDATE makosh_kernel_module_registration SET grant_epoch = ?1
         WHERE registration_id = ?2 AND grant_epoch = ?3",
        params![
            i64::try_from(next_epoch).map_err(|_| StoreError::RecoveryFenceOverflow)?,
            current.registration_id(),
            i64::try_from(current.grant_epoch()).map_err(|_| StoreError::RecoveryFenceOverflow)?
        ],
    )?;
    if changed == 1 {
        Ok(())
    } else {
        Err(StoreError::InvalidExternalRuntimeIdentity)
    }
}
