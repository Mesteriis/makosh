//! SQLite persistence for descriptor-declared Scheduler JobKind requests.

use std::collections::BTreeSet;

use makosh_kernel_control_store::{ModuleRegistration, ModuleSchedulerJobRequestV1};
use rusqlite::{Connection, params};

use crate::{SqliteControlStore, StoreError, valid_capability_ids, valid_identity_token};

impl SqliteControlStore {
    pub fn module_scheduler_job_requests(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Vec<ModuleSchedulerJobRequestV1>, StoreError> {
        let registration_id = registration_id.to_owned();
        let capability_id = capability_id.to_owned();
        self.with_connection(move |connection| {
            read_scheduler_job_requests(connection, &registration_id, &capability_id)
        })
    }
}

pub(crate) fn validate_scheduler_job_requests(
    registration: &ModuleRegistration,
    capabilities: &[String],
    requests: &[ModuleSchedulerJobRequestV1],
) -> Result<(), StoreError> {
    let requested = capabilities
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let valid = requests.iter().all(|request| {
        request.registration_id() == registration.registration_id()
            && request.owner() == registration.owner_id()
            && requested.contains(request.capability_id())
            && valid_capability_ids(&[request.capability_id().to_owned()])
            && valid_identity_token(request.name())
            && u16::try_from(request.major()).is_ok_and(|major| major > 0)
            && request.revision() > 0
            && request.schema_sha256().iter().any(|byte| *byte != 0)
            && seen.insert((
                request.capability_id(),
                request.owner(),
                request.name(),
                request.major(),
            ))
    });
    valid
        .then_some(())
        .ok_or(StoreError::InvalidModuleSchedulerJobRequest)
}

pub(crate) fn insert_scheduler_job_requests(
    connection: &Connection,
    requests: &[ModuleSchedulerJobRequestV1],
) -> Result<(), StoreError> {
    for request in requests {
        connection.execute(
            "INSERT INTO makosh_kernel_module_scheduler_job_request
             (registration_id, capability_id, job_owner, job_name, job_major, job_revision,
              contract_schema_sha256)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                request.registration_id(),
                request.capability_id(),
                request.owner(),
                request.name(),
                i64::from(request.major()),
                i64::from(request.revision()),
                request.schema_sha256().as_slice(),
            ],
        )?;
    }
    Ok(())
}

fn read_scheduler_job_requests(
    connection: &Connection,
    registration_id: &str,
    capability_id: &str,
) -> Result<Vec<ModuleSchedulerJobRequestV1>, StoreError> {
    let mut statement = connection.prepare_cached(
        "SELECT job_owner, job_name, job_major, job_revision, contract_schema_sha256
         FROM makosh_kernel_module_scheduler_job_request
         WHERE registration_id = ?1 AND capability_id = ?2
         ORDER BY job_owner, job_name, job_major",
    )?;
    let rows = statement.query_map(params![registration_id, capability_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, Vec<u8>>(4)?,
        ))
    })?;
    rows.map(|row| {
        let (owner, name, major, revision, digest) = row?;
        let major =
            u32::try_from(major).map_err(|_| StoreError::InvalidModuleSchedulerJobRequest)?;
        let revision =
            u32::try_from(revision).map_err(|_| StoreError::InvalidModuleSchedulerJobRequest)?;
        let digest: [u8; 32] = digest
            .try_into()
            .map_err(|_| StoreError::InvalidModuleSchedulerJobRequest)?;
        Ok(ModuleSchedulerJobRequestV1::new(
            registration_id,
            capability_id,
            owner,
            name,
            major,
            revision,
            digest,
        ))
    })
    .collect()
}
