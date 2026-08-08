//! Browser device identity persistence through the single-writer actor.

use makosh_kernel_control_store::{
    BrowserDeviceEnrollmentInputV1, BrowserDeviceEnrollmentV1, BrowserDeviceIdentityV1,
    BrowserDeviceStateV1, ControlStore,
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::{SqliteControlStore, StoreError, valid_identity_token};

impl SqliteControlStore {
    pub fn current_identity_epoch(&self) -> Result<u64, StoreError> {
        self.with_connection(read_connection_identity_epoch)
    }

    pub fn browser_device_identity(
        &self,
        device_id: &str,
    ) -> Result<Option<BrowserDeviceIdentityV1>, StoreError> {
        if !valid_identity_token(device_id) {
            return Err(StoreError::InvalidBrowserDeviceIdentity);
        }
        let device_id = device_id.to_owned();
        self.with_connection(move |connection| load_browser_device(connection, &device_id))
    }

    pub fn browser_device_identity_by_credential_id(
        &self,
        credential_id: &[u8],
    ) -> Result<Option<BrowserDeviceIdentityV1>, StoreError> {
        if credential_id.is_empty() || credential_id.len() > 1024 {
            return Err(StoreError::InvalidBrowserDeviceIdentity);
        }
        let credential_id = credential_id.to_vec();
        self.with_connection(move |connection| {
            load_browser_device_by_credential_id(connection, &credential_id)
        })
    }

    pub fn admit_browser_device(
        &self,
        enrollment: &BrowserDeviceEnrollmentV1,
        expected_identity_epoch: u64,
    ) -> Result<BrowserDeviceIdentityV1, StoreError> {
        let enrollment = enrollment.clone();
        self.with_connection(move |connection| {
            admit_browser_device(connection, &enrollment, expected_identity_epoch)
        })
    }

    pub fn record_verified_browser_assertion(
        &self,
        credential_id: &[u8],
        observed_sign_count: u32,
        observed_backup_eligible: bool,
        observed_backup_state: bool,
        expected_identity_epoch: u64,
    ) -> Result<BrowserDeviceIdentityV1, StoreError> {
        if credential_id.is_empty() || credential_id.len() > 1024 {
            return Err(StoreError::InvalidBrowserDeviceIdentity);
        }
        let credential_id = credential_id.to_vec();
        self.with_connection(move |connection| {
            record_verified_browser_assertion(
                connection,
                &credential_id,
                observed_sign_count,
                observed_backup_eligible,
                observed_backup_state,
                expected_identity_epoch,
            )
        })
    }

    pub fn revoke_browser_device(
        &self,
        device_id: &str,
        expected_identity_epoch: u64,
    ) -> Result<ControlStore, StoreError> {
        if !valid_identity_token(device_id) {
            return Err(StoreError::InvalidBrowserDeviceIdentity);
        }
        let device_id = device_id.to_owned();
        self.with_connection(move |connection| {
            revoke_browser_device(connection, &device_id, expected_identity_epoch)
        })
    }
}

fn read_connection_identity_epoch(connection: &mut Connection) -> Result<u64, StoreError> {
    connection
        .query_row(
            "SELECT identity_epoch FROM makosh_kernel_control_store_metadata WHERE singleton = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(StoreError::from)
        .and_then(as_u64)
}

fn admit_browser_device(
    connection: &mut Connection,
    enrollment: &BrowserDeviceEnrollmentV1,
    expected_identity_epoch: u64,
) -> Result<BrowserDeviceIdentityV1, StoreError> {
    let transaction = connection.transaction()?;
    let current_epoch = current_identity_epoch(&transaction)?;
    ensure_admission_owner(&transaction, enrollment)?;
    ensure_expected_epoch(current_epoch, expected_identity_epoch)?;
    ensure_browser_device_absent(&transaction, enrollment)?;
    insert_browser_device(&transaction, enrollment, current_epoch)?;
    transaction.commit()?;
    BrowserDeviceIdentityV1::new(
        enrollment.clone(),
        BrowserDeviceStateV1::Active,
        current_epoch,
    )
    .map_err(|_| StoreError::InvalidBrowserDeviceIdentity)
}

fn revoke_browser_device(
    connection: &mut Connection,
    device_id: &str,
    expected_identity_epoch: u64,
) -> Result<ControlStore, StoreError> {
    let transaction = connection.transaction()?;
    let metadata = read_metadata(&transaction)?;
    ensure_expected_epoch(metadata.identity_epoch, expected_identity_epoch)?;
    let next_epoch = metadata
        .identity_epoch
        .checked_add(1)
        .ok_or(StoreError::RecoveryFenceOverflow)?;
    mark_browser_device_revoked(&transaction, device_id, next_epoch)?;
    transaction.execute(
        "UPDATE makosh_kernel_control_store_metadata SET identity_epoch = ?1 WHERE singleton = 1",
        [as_sqlite_integer(next_epoch)?],
    )?;
    transaction.commit()?;
    Ok(ControlStore::with_recovery_fences(
        metadata.instance_id,
        metadata.generation,
        next_epoch,
        metadata.grant_epoch,
    ))
}

fn record_verified_browser_assertion(
    connection: &mut Connection,
    credential_id: &[u8],
    observed_sign_count: u32,
    observed_backup_eligible: bool,
    observed_backup_state: bool,
    expected_identity_epoch: u64,
) -> Result<BrowserDeviceIdentityV1, StoreError> {
    if observed_backup_state && !observed_backup_eligible {
        return Err(StoreError::InvalidBrowserDeviceIdentity);
    }
    let transaction = connection.transaction()?;
    ensure_expected_epoch(
        current_identity_epoch(&transaction)?,
        expected_identity_epoch,
    )?;
    let mut record =
        load_browser_device_by_credential_id_in_transaction(&transaction, credential_id)?
            .ok_or(StoreError::BrowserDeviceMissing)?;
    let device = record.clone().decode()?;
    (device.state() == BrowserDeviceStateV1::Active)
        .then_some(())
        .ok_or(StoreError::BrowserDeviceStateConflict)?;
    counter_progresses(device.enrollment().sign_count(), observed_sign_count)
        .then_some(())
        .ok_or(StoreError::BrowserDeviceCounterConflict)?;
    let changed = transaction.execute(
        "UPDATE makosh_kernel_browser_device_identity
         SET sign_count = ?1, backup_eligible = ?2, backup_state = ?3
         WHERE credential_id = ?4 AND state = 'active' AND sign_count = ?5",
        params![
            i64::from(observed_sign_count),
            observed_backup_eligible,
            observed_backup_state,
            credential_id,
            i64::from(device.enrollment().sign_count()),
        ],
    )?;
    (changed == 1)
        .then_some(())
        .ok_or(StoreError::BrowserDeviceCounterConflict)?;
    record.sign_count = i64::from(observed_sign_count);
    record.backup_eligible = observed_backup_eligible;
    record.backup_state = observed_backup_state;
    transaction.commit()?;
    record.decode()
}

fn ensure_admission_owner(
    transaction: &Transaction<'_>,
    enrollment: &BrowserDeviceEnrollmentV1,
) -> Result<(), StoreError> {
    let owner_id = transaction
        .query_row(
            "SELECT owner_id FROM makosh_kernel_initial_owner_identity WHERE singleton = 1",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .ok_or(StoreError::BrowserDeviceOwnerMissing)?;
    (owner_id == enrollment.owner_id())
        .then_some(())
        .ok_or(StoreError::BrowserDeviceOwnerMismatch)
}

fn ensure_expected_epoch(current: u64, expected: u64) -> Result<(), StoreError> {
    (current == expected && expected != 0)
        .then_some(())
        .ok_or(StoreError::BrowserDeviceIdentityEpochConflict)
}

fn ensure_browser_device_absent(
    transaction: &Transaction<'_>,
    enrollment: &BrowserDeviceEnrollmentV1,
) -> Result<(), StoreError> {
    let exists = transaction.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM makosh_kernel_browser_device_identity
            WHERE device_id = ?1 OR credential_id = ?2
        )",
        params![enrollment.device_id(), enrollment.credential_id()],
        |row| row.get::<_, bool>(0),
    )?;
    (!exists)
        .then_some(())
        .ok_or(StoreError::BrowserDeviceAlreadyExists)
}

fn insert_browser_device(
    transaction: &Transaction<'_>,
    enrollment: &BrowserDeviceEnrollmentV1,
    identity_epoch: u64,
) -> Result<(), StoreError> {
    transaction.execute(
        "INSERT INTO makosh_kernel_browser_device_identity (
            device_id, owner_id, credential_id, cose_public_key, browser_key_public_key, rp_id, sign_count, backup_eligible, backup_state, state, identity_epoch
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'active', ?10)",
        params![
            enrollment.device_id(),
            enrollment.owner_id(),
            enrollment.credential_id(),
            enrollment.cose_public_key(),
            enrollment.browser_key_public_key(),
            enrollment.rp_id(),
            i64::from(enrollment.sign_count()),
            enrollment.backup_eligible(),
            enrollment.backup_state(),
            as_sqlite_integer(identity_epoch)?,
        ],
    )?;
    Ok(())
}

fn mark_browser_device_revoked(
    transaction: &Transaction<'_>,
    device_id: &str,
    identity_epoch: u64,
) -> Result<(), StoreError> {
    let changed = transaction.execute(
        "UPDATE makosh_kernel_browser_device_identity
         SET state = 'revoked', identity_epoch = ?1
         WHERE device_id = ?2 AND state = 'active'",
        params![as_sqlite_integer(identity_epoch)?, device_id],
    )?;
    if changed == 1 {
        Ok(())
    } else if browser_device_exists(transaction, device_id)? {
        Err(StoreError::BrowserDeviceStateConflict)
    } else {
        Err(StoreError::BrowserDeviceMissing)
    }
}

fn browser_device_exists(
    transaction: &Transaction<'_>,
    device_id: &str,
) -> Result<bool, StoreError> {
    transaction
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM makosh_kernel_browser_device_identity WHERE device_id = ?1)",
            [device_id],
            |row| row.get(0),
        )
        .map_err(StoreError::from)
}

fn counter_progresses(current: u32, observed: u32) -> bool {
    current == 0 || observed > current
}

fn load_browser_device(
    connection: &mut Connection,
    device_id: &str,
) -> Result<Option<BrowserDeviceIdentityV1>, StoreError> {
    let record = connection
        .query_row(
            "SELECT owner_id, device_id, credential_id, cose_public_key, browser_key_public_key, rp_id, sign_count, backup_eligible, backup_state, state, identity_epoch
             FROM makosh_kernel_browser_device_identity WHERE device_id = ?1",
            [device_id],
            |row| {
                Ok(BrowserDeviceRecord {
                    owner_id: row.get(0)?,
                    device_id: row.get(1)?,
                    credential_id: row.get(2)?,
                    cose_public_key: row.get(3)?,
                    browser_key_public_key: row.get(4)?,
                    rp_id: row.get(5)?,
                    sign_count: row.get(6)?,
                    backup_eligible: row.get(7)?,
                    backup_state: row.get(8)?,
                    state: row.get(9)?,
                    identity_epoch: row.get(10)?,
                })
            },
        )
        .optional()?;
    record.map(BrowserDeviceRecord::decode).transpose()
}

fn load_browser_device_by_credential_id(
    connection: &mut Connection,
    credential_id: &[u8],
) -> Result<Option<BrowserDeviceIdentityV1>, StoreError> {
    let record = connection
        .query_row(
            "SELECT owner_id, device_id, credential_id, cose_public_key, browser_key_public_key, rp_id, sign_count, backup_eligible, backup_state, state, identity_epoch
             FROM makosh_kernel_browser_device_identity WHERE credential_id = ?1",
            [credential_id],
            |row| {
                Ok(BrowserDeviceRecord {
                    owner_id: row.get(0)?,
                    device_id: row.get(1)?,
                    credential_id: row.get(2)?,
                    cose_public_key: row.get(3)?,
                    browser_key_public_key: row.get(4)?,
                    rp_id: row.get(5)?,
                    sign_count: row.get(6)?,
                    backup_eligible: row.get(7)?,
                    backup_state: row.get(8)?,
                    state: row.get(9)?,
                    identity_epoch: row.get(10)?,
                })
            },
        )
        .optional()?;
    record.map(BrowserDeviceRecord::decode).transpose()
}

fn load_browser_device_by_credential_id_in_transaction(
    transaction: &Transaction<'_>,
    credential_id: &[u8],
) -> Result<Option<BrowserDeviceRecord>, StoreError> {
    transaction
        .query_row(
            "SELECT owner_id, device_id, credential_id, cose_public_key, browser_key_public_key, rp_id, sign_count, backup_eligible, backup_state, state, identity_epoch
             FROM makosh_kernel_browser_device_identity WHERE credential_id = ?1",
            [credential_id],
            browser_device_record_from_row,
        )
        .optional()
        .map_err(StoreError::from)
}

#[derive(Clone)]
struct BrowserDeviceRecord {
    owner_id: String,
    device_id: String,
    credential_id: Vec<u8>,
    cose_public_key: Vec<u8>,
    browser_key_public_key: Vec<u8>,
    rp_id: String,
    sign_count: i64,
    backup_eligible: bool,
    backup_state: bool,
    state: String,
    identity_epoch: i64,
}

fn browser_device_record_from_row(
    row: &rusqlite::Row<'_>,
) -> Result<BrowserDeviceRecord, rusqlite::Error> {
    Ok(BrowserDeviceRecord {
        owner_id: row.get(0)?,
        device_id: row.get(1)?,
        credential_id: row.get(2)?,
        cose_public_key: row.get(3)?,
        browser_key_public_key: row.get(4)?,
        rp_id: row.get(5)?,
        sign_count: row.get(6)?,
        backup_eligible: row.get(7)?,
        backup_state: row.get(8)?,
        state: row.get(9)?,
        identity_epoch: row.get(10)?,
    })
}

impl BrowserDeviceRecord {
    fn decode(self) -> Result<BrowserDeviceIdentityV1, StoreError> {
        let enrollment = BrowserDeviceEnrollmentV1::new(BrowserDeviceEnrollmentInputV1 {
            owner_id: self.owner_id,
            device_id: self.device_id,
            credential_id: self.credential_id,
            cose_public_key: self.cose_public_key,
            browser_key_public_key: self.browser_key_public_key,
            rp_id: self.rp_id,
            sign_count: u32::try_from(self.sign_count)
                .map_err(|_| StoreError::InvalidBrowserDeviceIdentity)?,
            backup_eligible: self.backup_eligible,
            backup_state: self.backup_state,
        })
        .map_err(|_| StoreError::InvalidBrowserDeviceIdentity)?;
        let state = BrowserDeviceStateV1::parse(&self.state)
            .ok_or(StoreError::InvalidBrowserDeviceIdentity)?;
        BrowserDeviceIdentityV1::new(enrollment, state, as_u64(self.identity_epoch)?)
            .map_err(|_| StoreError::InvalidBrowserDeviceIdentity)
    }
}

struct Metadata {
    instance_id: String,
    generation: u64,
    identity_epoch: u64,
    grant_epoch: u64,
}

fn read_metadata(transaction: &Transaction<'_>) -> Result<Metadata, StoreError> {
    transaction
        .query_row(
            "SELECT instance_id, generation, identity_epoch, grant_epoch
         FROM makosh_kernel_control_store_metadata WHERE singleton = 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )
        .map_err(StoreError::from)
        .and_then(|(instance_id, generation, identity_epoch, grant_epoch)| {
            Ok(Metadata {
                instance_id,
                generation: as_u64(generation)?,
                identity_epoch: as_u64(identity_epoch)?,
                grant_epoch: as_u64(grant_epoch)?,
            })
        })
}

fn current_identity_epoch(transaction: &Transaction<'_>) -> Result<u64, StoreError> {
    Ok(read_metadata(transaction)?.identity_epoch)
}

fn as_u64(value: i64) -> Result<u64, StoreError> {
    u64::try_from(value).map_err(|_| StoreError::InvalidBrowserDeviceIdentity)
}

fn as_sqlite_integer(value: u64) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::RecoveryFenceOverflow)
}
