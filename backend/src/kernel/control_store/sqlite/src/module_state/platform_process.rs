//! SQLite persistence for Kernel-owned platform process trust and fencing.

use makosh_kernel_control_store::{PlatformManagedProcessBinding, PlatformManagedProcessLaunch};
use rusqlite::{OptionalExtension, params};

use crate::{SqliteControlStore, StoreError, valid_identity_token};

impl SqliteControlStore {
    pub fn record_platform_managed_process_binding(
        &self,
        binding: &PlatformManagedProcessBinding,
    ) -> Result<(), StoreError> {
        if !valid_binding(binding) {
            return Err(StoreError::InvalidPlatformManagedProcessBinding);
        }
        let binding = binding.clone();
        self.with_connection(move |connection| {
            let changed = connection.execute(
                "INSERT INTO makosh_kernel_platform_managed_process_binding (process_id, binding_revision, distribution_id, artifact_id, executable_sha256, descriptor_sha256, settings_schema_sha256) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) ON CONFLICT(process_id) DO UPDATE SET binding_revision=excluded.binding_revision, distribution_id=excluded.distribution_id, artifact_id=excluded.artifact_id, executable_sha256=excluded.executable_sha256, descriptor_sha256=excluded.descriptor_sha256, settings_schema_sha256=excluded.settings_schema_sha256 WHERE excluded.binding_revision = makosh_kernel_platform_managed_process_binding.binding_revision + 1",
                params![binding.process_id(), as_sql(binding.binding_revision())?, binding.distribution_id(), binding.artifact_id(), binding.executable_sha256().as_slice(), binding.descriptor_sha256().as_slice(), binding.settings_schema_sha256().map(|digest| digest.as_slice())],
            )?;
            if changed == 1 { Ok(()) } else { Err(StoreError::PlatformManagedProcessBindingRevisionConflict) }
        })
    }

    pub fn platform_managed_process_binding(
        &self,
        process_id: &str,
    ) -> Result<Option<PlatformManagedProcessBinding>, StoreError> {
        let process_id = process_id.to_owned();
        self.with_connection(move |connection| connection.query_row(
            "SELECT binding_revision, distribution_id, artifact_id, executable_sha256, descriptor_sha256, settings_schema_sha256 FROM makosh_kernel_platform_managed_process_binding WHERE process_id = ?1", [&process_id],
            |row| decode_binding(row, &process_id),
        ).optional().map_err(StoreError::from))
    }

    pub fn record_platform_managed_process_launch(
        &self,
        launch: &PlatformManagedProcessLaunch,
    ) -> Result<(), StoreError> {
        if !valid_launch(launch) || launch.kernel_generation() != self.snapshot().generation() {
            return Err(StoreError::InvalidPlatformManagedProcessLaunch);
        }
        let launch = launch.clone();
        self.with_connection(move |connection| {
            let changed = connection.execute(
                "INSERT INTO makosh_kernel_platform_managed_process_launch (process_id, binding_revision, kernel_generation, runtime_generation, grant_epoch) SELECT ?1, ?2, ?3, ?4, ?5 WHERE EXISTS (SELECT 1 FROM makosh_kernel_platform_managed_process_binding WHERE process_id = ?1 AND binding_revision = ?2) ON CONFLICT(process_id) DO UPDATE SET binding_revision=excluded.binding_revision, kernel_generation=excluded.kernel_generation, runtime_generation=excluded.runtime_generation, grant_epoch=excluded.grant_epoch WHERE excluded.runtime_generation > makosh_kernel_platform_managed_process_launch.runtime_generation",
                params![launch.process_id(), as_sql(launch.binding_revision())?, as_sql(launch.kernel_generation())?, as_sql(launch.runtime_generation())?, as_sql(launch.grant_epoch())?],
            )?;
            if changed == 1 { Ok(()) } else { Err(StoreError::StalePlatformManagedProcessLaunch) }
        })
    }

    pub fn platform_managed_process_launch(
        &self,
        process_id: &str,
    ) -> Result<Option<PlatformManagedProcessLaunch>, StoreError> {
        let process_id = process_id.to_owned();
        self.with_connection(move |connection| connection.query_row(
            "SELECT binding_revision, kernel_generation, runtime_generation, grant_epoch FROM makosh_kernel_platform_managed_process_launch WHERE process_id = ?1", [&process_id],
            |row| Ok(PlatformManagedProcessLaunch::new(&process_id, as_u64(row.get(0)?, 0)?, as_u64(row.get(1)?, 1)?, as_u64(row.get(2)?, 2)?, as_u64(row.get(3)?, 3)?)),
        ).optional().map_err(StoreError::from))
    }
}

fn decode_binding(
    row: &rusqlite::Row<'_>,
    process_id: &str,
) -> Result<PlatformManagedProcessBinding, rusqlite::Error> {
    let executable: Vec<u8> = row.get(3)?;
    let descriptor: Vec<u8> = row.get(4)?;
    let settings: Option<Vec<u8>> = row.get(5)?;
    Ok(PlatformManagedProcessBinding::new(
        process_id,
        as_u64(row.get(0)?, 0)?,
        row.get::<_, String>(1)?,
        row.get::<_, String>(2)?,
        executable
            .try_into()
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(3, 32))?,
        descriptor
            .try_into()
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(4, 32))?,
        settings
            .map(|value| {
                value
                    .try_into()
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(5, 32))
            })
            .transpose()?,
    ))
}

fn valid_binding(value: &PlatformManagedProcessBinding) -> bool {
    valid_identity_token(value.process_id())
        && value.binding_revision() > 0
        && valid_identity_token(value.distribution_id())
        && valid_identity_token(value.artifact_id())
}
fn valid_launch(value: &PlatformManagedProcessLaunch) -> bool {
    valid_identity_token(value.process_id())
        && value.binding_revision() > 0
        && value.kernel_generation() > 0
        && value.runtime_generation() > 0
        && value.grant_epoch() > 0
}
fn as_sql(value: u64) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::InvalidPlatformManagedProcessLaunch)
}
fn as_u64(value: i64, index: usize) -> Result<u64, rusqlite::Error> {
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, 0))
}
