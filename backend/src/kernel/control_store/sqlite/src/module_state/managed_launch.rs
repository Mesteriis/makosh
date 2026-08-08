//! Signed-release managed launch authority and runtime fencing.

use makosh_kernel_control_store::{
    BundledManagedLaunchBinding, ManagedLaunchRecord, ModuleRegistrationState,
};
use rusqlite::{OptionalExtension, params};

use crate::module_state::registry::read_required_registration;
use crate::{SqliteControlStore, StoreError, valid_identity_token};

impl SqliteControlStore {
    pub fn record_bundled_managed_launch_binding(
        &self,
        binding: &BundledManagedLaunchBinding,
    ) -> Result<(), StoreError> {
        if !valid_binding(binding) {
            return Err(StoreError::InvalidBundledManagedLaunchBinding);
        }
        let binding = binding.clone();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            let registration = read_required_registration(&transaction, binding.registration_id())?;
            if registration.state() != ModuleRegistrationState::Approved
                || registration.descriptor_sha256() != binding.descriptor_sha256()
            {
                return Err(StoreError::InvalidBundledManagedLaunchBinding);
            }
            let changed = transaction.execute(
                "INSERT INTO makosh_kernel_bundled_managed_launch_binding
                 (registration_id, binding_revision, distribution_id, artifact_id,
                  executable_sha256, descriptor_sha256, settings_schema_sha256)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(registration_id) DO UPDATE SET
                 binding_revision=excluded.binding_revision, distribution_id=excluded.distribution_id,
                 artifact_id=excluded.artifact_id, executable_sha256=excluded.executable_sha256,
                 descriptor_sha256=excluded.descriptor_sha256,
                 settings_schema_sha256=excluded.settings_schema_sha256
                 WHERE excluded.binding_revision = makosh_kernel_bundled_managed_launch_binding.binding_revision + 1",
                params![binding.registration_id(), as_sql(binding.binding_revision())?, binding.distribution_id(), binding.artifact_id(), binding.executable_sha256().as_slice(), binding.descriptor_sha256().as_slice(), binding.settings_schema_sha256().map(|digest| digest.as_slice())],
            )?;
            if changed != 1 {
                return Err(StoreError::BundledManagedLaunchBindingRevisionConflict);
            }
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn effective_bundled_managed_launch_binding(
        &self,
        registration_id: &str,
    ) -> Result<Option<BundledManagedLaunchBinding>, StoreError> {
        let registration_id = registration_id.to_owned();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            let registration = read_required_registration(&transaction, &registration_id)?;
            if registration.state() != ModuleRegistrationState::Approved {
                return Ok(None);
            }
            let binding = transaction
                .query_row(
                    "SELECT binding_revision, distribution_id, artifact_id, executable_sha256,
                 descriptor_sha256, settings_schema_sha256
                 FROM makosh_kernel_bundled_managed_launch_binding WHERE registration_id = ?1",
                    [&registration_id],
                    |row| decode_binding(row, &registration_id),
                )
                .optional()?;
            transaction.commit()?;
            Ok(binding)
        })
    }

    pub fn record_managed_launch(&self, record: &ManagedLaunchRecord) -> Result<(), StoreError> {
        if !valid_record(record) || record.kernel_generation() != self.snapshot().generation() {
            return Err(StoreError::InvalidManagedLaunchRecord);
        }
        let record = record.clone();
        self.with_connection(move |connection| {
            let changed = connection.execute(
                "INSERT INTO makosh_kernel_managed_launch_record
                 (registration_id, runtime_instance_id, binding_revision, kernel_generation, runtime_generation, grant_epoch)
                 SELECT ?1, ?2, ?3, ?4, ?5, ?6 WHERE EXISTS (
                   SELECT 1 FROM makosh_kernel_module_registration AS registration
                   JOIN makosh_kernel_bundled_managed_launch_binding AS binding
                     ON binding.registration_id = registration.registration_id
                   WHERE registration.registration_id = ?1 AND registration.state = 'approved'
                     AND registration.grant_epoch = ?6 AND binding.binding_revision = ?3)
                 ON CONFLICT(registration_id) DO UPDATE SET runtime_instance_id=excluded.runtime_instance_id,
                 binding_revision=excluded.binding_revision,
                 kernel_generation=excluded.kernel_generation,
                 runtime_generation=excluded.runtime_generation, grant_epoch=excluded.grant_epoch
                 WHERE excluded.runtime_generation > makosh_kernel_managed_launch_record.runtime_generation",
                params![record.registration_id(), record.runtime_instance_id(), as_sql(record.binding_revision())?, as_sql(record.kernel_generation())?, as_sql(record.runtime_generation())?, as_sql(record.grant_epoch())?],
            )?;
            if changed == 1 { Ok(()) } else { Err(StoreError::StaleManagedLaunchRecord) }
        })
    }

    pub fn effective_managed_launch_record(
        &self,
        registration_id: &str,
    ) -> Result<Option<ManagedLaunchRecord>, StoreError> {
        let registration_id = registration_id.to_owned();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            let registration = read_required_registration(&transaction, &registration_id)?;
            if registration.state() != ModuleRegistrationState::Approved {
                return Ok(None);
            }
            let record = transaction
                .query_row(
                    "SELECT runtime_instance_id, binding_revision, kernel_generation, runtime_generation, grant_epoch
                 FROM makosh_kernel_managed_launch_record
                 WHERE registration_id = ?1 AND grant_epoch = ?2",
                    params![&registration_id, as_sql(registration.grant_epoch())?],
                    |row| {
                        Ok(ManagedLaunchRecord::new(
                            &registration_id,
                            row.get::<_, String>(0)?,
                            as_u64(row.get(1)?, 1)?,
                            as_u64(row.get(2)?, 2)?,
                            as_u64(row.get(3)?, 3)?,
                            as_u64(row.get(4)?, 4)?,
                        ))
                    },
                )
                .optional()?;
            transaction.commit()?;
            Ok(record)
        })
    }

    pub fn managed_launch_generation_high_watermark(
        &self,
        registration_id: &str,
    ) -> Result<Option<u64>, StoreError> {
        let registration_id = registration_id.to_owned();
        self.with_connection(move |connection| {
            connection
                .query_row(
                    "SELECT runtime_generation FROM makosh_kernel_managed_launch_record
                     WHERE registration_id = ?1",
                    [&registration_id],
                    |row| as_u64(row.get(0)?, 0),
                )
                .optional()
                .map_err(StoreError::from)
        })
    }
}

fn decode_binding(
    row: &rusqlite::Row<'_>,
    registration_id: &str,
) -> Result<BundledManagedLaunchBinding, rusqlite::Error> {
    let executable: Vec<u8> = row.get(3)?;
    let descriptor: Vec<u8> = row.get(4)?;
    let settings: Option<Vec<u8>> = row.get(5)?;
    Ok(BundledManagedLaunchBinding::new(
        registration_id,
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
            .map(|digest| {
                digest
                    .try_into()
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(5, 32))
            })
            .transpose()?,
    ))
}

fn valid_binding(binding: &BundledManagedLaunchBinding) -> bool {
    valid_identity_token(binding.registration_id())
        && binding.binding_revision() > 0
        && valid_identity_token(binding.distribution_id())
        && valid_identity_token(binding.artifact_id())
}

fn valid_record(record: &ManagedLaunchRecord) -> bool {
    valid_identity_token(record.registration_id())
        && valid_identity_token(record.runtime_instance_id())
        && record.binding_revision() > 0
        && record.kernel_generation() > 0
        && record.runtime_generation() > 0
        && record.grant_epoch() > 0
}

fn as_sql(value: u64) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::InvalidManagedLaunchRecord)
}

fn as_u64(value: i64, index: usize) -> Result<u64, rusqlite::Error> {
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, 0))
}
