//! SQLite persistence for durable non-secret Storage binding fences.

use makosh_kernel_control_store::{
    PlatformStorageBindingInputV1, PlatformStorageBindingStateV1, PlatformStorageBindingV1,
};
use rusqlite::{OptionalExtension, params};

use crate::{SqliteControlStore, StoreError};

impl SqliteControlStore {
    pub fn record_platform_storage_binding(
        &self,
        binding: &PlatformStorageBindingV1,
    ) -> Result<(), StoreError> {
        let binding = binding.clone();
        self.with_connection(move |connection| {
            verify_initial_revision(connection, &binding)?;
            let changed = connection.execute(
                "INSERT INTO makosh_kernel_platform_storage_binding
                 (registration_id, capability_id, owner_id, binding_revision, topology_revision,
                  storage_generation, runtime_instance_id, runtime_generation, grant_epoch,
                  role_epoch, runtime_principal, connection_budget, statement_timeout_millis,
                  credential_lease_revision, storage_bundle_revision, storage_bundle_digest, state)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
                 ON CONFLICT(registration_id, capability_id) DO UPDATE SET
                   owner_id=excluded.owner_id,
                   binding_revision=excluded.binding_revision,
                   topology_revision=excluded.topology_revision,
                   storage_generation=excluded.storage_generation,
                   runtime_instance_id=excluded.runtime_instance_id,
                   runtime_generation=excluded.runtime_generation,
                   grant_epoch=excluded.grant_epoch,
                   role_epoch=excluded.role_epoch,
                   runtime_principal=excluded.runtime_principal,
                   connection_budget=excluded.connection_budget,
                   statement_timeout_millis=excluded.statement_timeout_millis,
                   credential_lease_revision=excluded.credential_lease_revision,
                   storage_bundle_revision=excluded.storage_bundle_revision,
                   storage_bundle_digest=excluded.storage_bundle_digest,
                   state=excluded.state
                 WHERE excluded.binding_revision = makosh_kernel_platform_storage_binding.binding_revision + 1",
                params![
                    binding.registration_id(), binding.capability_id(), binding.owner_id(),
                    as_sql(binding.binding_revision())?, as_sql(binding.topology_revision())?,
                    as_sql(binding.storage_generation())?, binding.runtime_instance_id(),
                    as_sql(binding.runtime_generation())?, as_sql(binding.grant_epoch())?,
                    as_sql(binding.role_epoch())?, binding.runtime_principal(),
                    i64::from(binding.connection_budget()),
                    i64::from(binding.statement_timeout_millis()),
                    as_sql(binding.credential_lease_revision())?,
                    as_sql(binding.storage_bundle_revision())?,
                    binding.storage_bundle_digest().as_slice(),
                    binding.state().as_sql(),
                ],
            )?;
            (changed == 1).then_some(()).ok_or(StoreError::PlatformStorageBindingRevisionConflict)
        })
    }

    pub fn platform_storage_binding(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Option<PlatformStorageBindingV1>, StoreError> {
        let registration_id = registration_id.to_owned();
        let capability_id = capability_id.to_owned();
        self.with_connection(move |connection| {
            connection
                .query_row(
                    "SELECT owner_id, binding_revision, topology_revision, storage_generation,
                    runtime_instance_id, runtime_generation, grant_epoch, role_epoch,
                    runtime_principal, connection_budget, statement_timeout_millis,
                    credential_lease_revision, storage_bundle_revision, storage_bundle_digest, state
             FROM makosh_kernel_platform_storage_binding
             WHERE registration_id = ?1 AND capability_id = ?2",
                    params![registration_id, capability_id],
                    |row| decode_binding(row, &registration_id, &capability_id),
                )
                .optional()
                .map_err(StoreError::from)
        })
    }

    pub fn begin_platform_storage_binding_revocation(
        &self,
        registration_id: &str,
        capability_id: &str,
        binding_revision: u64,
    ) -> Result<PlatformStorageBindingV1, StoreError> {
        let registration_id = registration_id.to_owned();
        let capability_id = capability_id.to_owned();
        self.with_connection(move |connection| {
            let changed = connection.execute(
                "UPDATE makosh_kernel_platform_storage_binding
                 SET state = 2
                 WHERE registration_id = ?1 AND capability_id = ?2 AND binding_revision = ?3 AND state = 1",
                params![registration_id, capability_id, as_sql(binding_revision)?],
            )?;
            let binding = connection
                .query_row(
                    "SELECT owner_id, binding_revision, topology_revision, storage_generation,
                        runtime_instance_id, runtime_generation, grant_epoch, role_epoch,
                        runtime_principal, connection_budget, statement_timeout_millis,
                        credential_lease_revision, storage_bundle_revision, storage_bundle_digest, state
                 FROM makosh_kernel_platform_storage_binding
                 WHERE registration_id = ?1 AND capability_id = ?2",
                    params![registration_id, capability_id],
                    |row| decode_binding(row, &registration_id, &capability_id),
                )
                .optional()?
                .ok_or(StoreError::PlatformStorageBindingStateConflict)?;
            if changed == 1
                || (binding.binding_revision() == binding_revision
                    && binding.state() == PlatformStorageBindingStateV1::Revoking)
            {
                Ok(binding)
            } else {
                Err(StoreError::PlatformStorageBindingStateConflict)
            }
        })
    }

    pub fn begin_registration_storage_bindings_revocation(
        &self,
        registration_id: &str,
    ) -> Result<Vec<PlatformStorageBindingV1>, StoreError> {
        let registration_id = registration_id.to_owned();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            let active_capability_ids = {
                let mut statement = transaction.prepare(
                    "SELECT capability_id
                     FROM makosh_kernel_platform_storage_binding
                     WHERE registration_id = ?1 AND state = 1
                     ORDER BY capability_id",
                )?;
                statement
                    .query_map(params![registration_id], |row| row.get::<_, String>(0))?
                    .collect::<Result<Vec<_>, _>>()?
            };
            let changed = transaction.execute(
                "UPDATE makosh_kernel_platform_storage_binding
                 SET state = 2
                 WHERE registration_id = ?1 AND state = 1",
                params![registration_id],
            )?;
            if changed != active_capability_ids.len() {
                return Err(StoreError::PlatformStorageBindingStateConflict);
            }
            let reserved = {
                let mut statement = transaction.prepare(
                    "SELECT registration_id, capability_id, owner_id, binding_revision,
                            topology_revision, storage_generation, runtime_instance_id,
                            runtime_generation, grant_epoch, role_epoch, runtime_principal,
                            connection_budget, statement_timeout_millis,
                            credential_lease_revision, storage_bundle_revision,
                            storage_bundle_digest, state
                     FROM makosh_kernel_platform_storage_binding
                     WHERE registration_id = ?1 AND state = 2
                     ORDER BY capability_id",
                )?;
                statement
                    .query_map(params![registration_id], decode_listed_binding)?
                    .collect::<Result<Vec<_>, _>>()?
            };
            if reserved.len() < active_capability_ids.len() {
                return Err(StoreError::PlatformStorageBindingStateConflict);
            }
            transaction.commit()?;
            Ok(reserved)
        })
    }

    pub fn platform_storage_bindings(&self) -> Result<Vec<PlatformStorageBindingV1>, StoreError> {
        self.with_connection(move |connection| {
            let mut statement = connection.prepare(
                "SELECT registration_id, capability_id, owner_id, binding_revision,
                        topology_revision, storage_generation, runtime_instance_id,
                        runtime_generation, grant_epoch, role_epoch, runtime_principal,
                        connection_budget, statement_timeout_millis, credential_lease_revision,
                        storage_bundle_revision, storage_bundle_digest, state
                 FROM makosh_kernel_platform_storage_binding
                 ORDER BY registration_id, capability_id",
            )?;
            statement
                .query_map([], decode_listed_binding)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(StoreError::from)
        })
    }
}

fn verify_initial_revision(
    connection: &rusqlite::Connection,
    binding: &PlatformStorageBindingV1,
) -> Result<(), StoreError> {
    let exists = connection.query_row(
        "SELECT EXISTS(
             SELECT 1 FROM makosh_kernel_platform_storage_binding
             WHERE registration_id = ?1 AND capability_id = ?2
         )",
        params![binding.registration_id(), binding.capability_id()],
        |row| row.get::<_, bool>(0),
    )?;
    if !exists && binding.binding_revision() != 1 {
        return Err(StoreError::PlatformStorageBindingRevisionConflict);
    }
    Ok(())
}

fn decode_binding(
    row: &rusqlite::Row<'_>,
    registration_id: &str,
    capability_id: &str,
) -> Result<PlatformStorageBindingV1, rusqlite::Error> {
    let digest: Vec<u8> = row.get(13)?;
    let state = state(row.get(14)?)?;
    PlatformStorageBindingV1::new(PlatformStorageBindingInputV1 {
        registration_id: registration_id.to_owned(),
        capability_id: capability_id.to_owned(),
        owner_id: row.get(0)?,
        binding_revision: as_u64(row.get(1)?, 1)?,
        topology_revision: as_u64(row.get(2)?, 2)?,
        storage_generation: as_u64(row.get(3)?, 3)?,
        runtime_instance_id: row.get(4)?,
        runtime_generation: as_u64(row.get(5)?, 5)?,
        grant_epoch: as_u64(row.get(6)?, 6)?,
        role_epoch: as_u64(row.get(7)?, 7)?,
        runtime_principal: row.get(8)?,
        connection_budget: as_u16(row.get(9)?, 9)?,
        statement_timeout_millis: as_u32(row.get(10)?, 10)?,
        credential_lease_revision: as_u64(row.get(11)?, 11)?,
        storage_bundle_revision: as_u64(row.get(12)?, 12)?,
        storage_bundle_digest: digest
            .try_into()
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(13, 32))?,
    })
    .map_err(|_| rusqlite::Error::InvalidQuery)
    .map(|binding| binding.restore_state(state))
}

fn decode_listed_binding(
    row: &rusqlite::Row<'_>,
) -> Result<PlatformStorageBindingV1, rusqlite::Error> {
    let registration_id: String = row.get(0)?;
    let capability_id: String = row.get(1)?;
    decode_binding_from_list(row, &registration_id, &capability_id)
}

fn decode_binding_from_list(
    row: &rusqlite::Row<'_>,
    registration_id: &str,
    capability_id: &str,
) -> Result<PlatformStorageBindingV1, rusqlite::Error> {
    let digest: Vec<u8> = row.get(15)?;
    let state = state(row.get(16)?)?;
    PlatformStorageBindingV1::new(PlatformStorageBindingInputV1 {
        registration_id: registration_id.to_owned(),
        capability_id: capability_id.to_owned(),
        owner_id: row.get(2)?,
        binding_revision: as_u64(row.get(3)?, 3)?,
        topology_revision: as_u64(row.get(4)?, 4)?,
        storage_generation: as_u64(row.get(5)?, 5)?,
        runtime_instance_id: row.get(6)?,
        runtime_generation: as_u64(row.get(7)?, 7)?,
        grant_epoch: as_u64(row.get(8)?, 8)?,
        role_epoch: as_u64(row.get(9)?, 9)?,
        runtime_principal: row.get(10)?,
        connection_budget: as_u16(row.get(11)?, 11)?,
        statement_timeout_millis: as_u32(row.get(12)?, 12)?,
        credential_lease_revision: as_u64(row.get(13)?, 13)?,
        storage_bundle_revision: as_u64(row.get(14)?, 14)?,
        storage_bundle_digest: digest
            .try_into()
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(15, 32))?,
    })
    .map_err(|_| rusqlite::Error::InvalidQuery)
    .map(|binding| binding.restore_state(state))
}

fn state(value: i64) -> Result<PlatformStorageBindingStateV1, rusqlite::Error> {
    PlatformStorageBindingStateV1::from_sql(value).ok_or(rusqlite::Error::InvalidQuery)
}

fn as_sql(value: u64) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::InvalidPlatformStorageBinding)
}

fn as_u64(value: i64, index: usize) -> Result<u64, rusqlite::Error> {
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, 0))
}

fn as_u16(value: i64, index: usize) -> Result<u16, rusqlite::Error> {
    u16::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, 0))
}

fn as_u32(value: i64, index: usize) -> Result<u32, rusqlite::Error> {
    u32::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, 0))
}
