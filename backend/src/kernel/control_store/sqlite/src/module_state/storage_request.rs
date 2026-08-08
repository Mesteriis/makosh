//! SQLite persistence for descriptor-declared Storage namespace requests.

use std::collections::BTreeSet;

use makosh_kernel_control_store::{ModuleRegistration, ModuleStorageRequestV1};
use rusqlite::{Connection, OptionalExtension, params};

use crate::{SqliteControlStore, StoreError, valid_capability_ids};

impl SqliteControlStore {
    pub fn module_storage_request(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Option<ModuleStorageRequestV1>, StoreError> {
        let registration_id = registration_id.to_owned();
        let capability_id = capability_id.to_owned();
        self.with_connection(move |connection| {
            read_storage_request(connection, &registration_id, &capability_id)
        })
    }
}

pub(crate) fn validate_storage_requests(
    registration: &ModuleRegistration,
    capabilities: &[String],
    requests: &[ModuleStorageRequestV1],
) -> Result<(), StoreError> {
    let requested = capabilities
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let valid = requests.iter().all(|request| {
        request.registration_id() == registration.registration_id()
            && request.owner_id() == registration.owner_id()
            && requested.contains(request.capability_id())
            && valid_capability_ids(&[request.capability_id().to_owned()])
            && request.connection_budget() > 0
            && request.statement_timeout_millis() > 0
            && seen.insert(request.capability_id())
    });
    valid
        .then_some(())
        .ok_or(StoreError::InvalidModuleStorageRequest)
}

pub(crate) fn insert_storage_requests(
    connection: &Connection,
    requests: &[ModuleStorageRequestV1],
) -> Result<(), StoreError> {
    for request in requests {
        connection.execute(
            "INSERT INTO makosh_kernel_module_storage_request
             (registration_id, capability_id, owner_id, connection_budget, statement_timeout_millis)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                request.registration_id(),
                request.capability_id(),
                request.owner_id(),
                i64::from(request.connection_budget()),
                i64::from(request.statement_timeout_millis()),
            ],
        )?;
    }
    Ok(())
}

fn read_storage_request(
    connection: &Connection,
    registration_id: &str,
    capability_id: &str,
) -> Result<Option<ModuleStorageRequestV1>, StoreError> {
    connection
        .query_row(
            "SELECT owner_id, connection_budget, statement_timeout_millis
             FROM makosh_kernel_module_storage_request
             WHERE registration_id = ?1 AND capability_id = ?2",
            params![registration_id, capability_id],
            |row| {
                let budget = u16::try_from(row.get::<_, i64>(1)?)
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(1, 0))?;
                let timeout = u32::try_from(row.get::<_, i64>(2)?)
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(2, 0))?;
                Ok(ModuleStorageRequestV1::new(
                    registration_id,
                    capability_id,
                    row.get::<_, String>(0)?,
                    budget,
                    timeout,
                ))
            },
        )
        .optional()
        .map_err(StoreError::from)
}
