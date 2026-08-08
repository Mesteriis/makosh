//! Owner-pinned external artifact bindings.

use makosh_kernel_control_store::{
    ModuleRegistrationState, OwnerPinnedArtifactBinding, OwnerPinnedArtifactBindingInputV1,
};
use rusqlite::{OptionalExtension, params};

use crate::module_state::registry::read_required_registration;
use crate::{SqliteControlStore, StoreError, valid_owner_pinned_artifact_binding};

impl SqliteControlStore {
    pub fn record_owner_pinned_artifact_binding(
        &self,
        binding: &OwnerPinnedArtifactBinding,
    ) -> Result<(), StoreError> {
        if !valid_owner_pinned_artifact_binding(binding) {
            return Err(StoreError::InvalidOwnerPinnedArtifactBinding);
        }
        let binding = binding.clone();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            let registration = read_required_registration(&transaction, binding.registration_id())?;
            if registration.state() != ModuleRegistrationState::Approved {
                return Err(StoreError::InvalidOwnerPinnedArtifactBinding);
            }
            let changed = transaction.execute(
                "INSERT INTO makosh_kernel_owner_pinned_artifact_binding
                 (registration_id, binding_revision, canonical_artifact_path, artifact_sha256,
                  artifact_size, artifact_device, artifact_inode, owner_signature_raw)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(registration_id) DO UPDATE SET binding_revision=excluded.binding_revision,
                 canonical_artifact_path=excluded.canonical_artifact_path,
                 artifact_sha256=excluded.artifact_sha256, artifact_size=excluded.artifact_size,
                 artifact_device=excluded.artifact_device, artifact_inode=excluded.artifact_inode,
                 owner_signature_raw=excluded.owner_signature_raw
                 WHERE excluded.binding_revision = makosh_kernel_owner_pinned_artifact_binding.binding_revision + 1",
                params![binding.registration_id(), as_sql(binding.binding_revision())?, binding.canonical_artifact_path(), binding.artifact_sha256().as_slice(), as_sql(binding.artifact_size())?, as_sql(binding.artifact_device())?, as_sql(binding.artifact_inode())?, binding.owner_signature_raw().as_slice()],
            )?;
            if changed != 1 {
                return Err(StoreError::OwnerPinnedArtifactBindingRevisionConflict);
            }
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn effective_owner_pinned_artifact_binding(
        &self,
        registration_id: &str,
    ) -> Result<Option<OwnerPinnedArtifactBinding>, StoreError> {
        let registration_id = registration_id.to_owned();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            let registration = read_required_registration(&transaction, &registration_id)?;
            if registration.state() != ModuleRegistrationState::Approved {
                return Ok(None);
            }
            let binding = transaction.query_row(
                "SELECT binding_revision, canonical_artifact_path, artifact_sha256, artifact_size,
                 artifact_device, artifact_inode, owner_signature_raw
                 FROM makosh_kernel_owner_pinned_artifact_binding WHERE registration_id = ?1",
                [&registration_id],
                |row| decode_binding(row, &registration_id),
            ).optional()?;
            transaction.commit()?;
            Ok(binding)
        })
    }
}

fn decode_binding(
    row: &rusqlite::Row<'_>,
    registration_id: &str,
) -> Result<OwnerPinnedArtifactBinding, rusqlite::Error> {
    let digest: Vec<u8> = row.get(2)?;
    let signature: Vec<u8> = row.get(6)?;
    Ok(OwnerPinnedArtifactBinding::new(
        OwnerPinnedArtifactBindingInputV1 {
            registration_id: registration_id.to_owned(),
            binding_revision: as_u64(row.get(0)?, 0)?,
            canonical_artifact_path: row.get(1)?,
            artifact_sha256: digest
                .try_into()
                .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(2, 32))?,
            artifact_size: as_u64(row.get(3)?, 3)?,
            artifact_device: as_u64(row.get(4)?, 4)?,
            artifact_inode: as_u64(row.get(5)?, 5)?,
            owner_signature_raw: signature
                .try_into()
                .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(6, 64))?,
        },
    ))
}

fn as_sql(value: u64) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::InvalidOwnerPinnedArtifactBinding)
}

fn as_u64(value: i64, index: usize) -> Result<u64, rusqlite::Error> {
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, 0))
}
