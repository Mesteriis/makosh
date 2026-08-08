use std::fs::File;
use std::path::{Path, PathBuf};

use makosh_kernel_control_store::{ControlStore, RecoveryFences};
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};

use crate::database::connection::{configure_writable, validate_quick_check};
use crate::{SqliteControlStore, StoreError};

pub struct StagedControlStoreRestore {
    path: PathBuf,
    snapshot: ControlStore,
    sha256: [u8; 32],
}

impl StagedControlStoreRestore {
    pub fn prepare(
        source: &Path,
        staged_path: &Path,
        expected_instance_id: &str,
        fences: RecoveryFences,
    ) -> Result<Self, StoreError> {
        let source_store = SqliteControlStore::open(source)?;
        if source_store.snapshot().instance_id() != expected_instance_id {
            return Err(StoreError::InstallationIdentityMismatch);
        }
        if staged_path.exists() {
            return Err(StoreError::InvalidExportDestination);
        }
        source_store.export_to(staged_path)?;
        let snapshot = fence_staged_store(staged_path, expected_instance_id, fences)?;
        File::open(staged_path)?.sync_all()?;
        let sha256 = Sha256::digest(std::fs::read(staged_path)?).into();
        Ok(Self {
            path: staged_path.to_owned(),
            snapshot,
            sha256,
        })
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn snapshot(&self) -> &ControlStore {
        &self.snapshot
    }

    #[must_use]
    pub fn sha256(&self) -> &[u8; 32] {
        &self.sha256
    }
}

fn fence_staged_store(
    path: &Path,
    instance_id: &str,
    fences: RecoveryFences,
) -> Result<ControlStore, StoreError> {
    let connection = Connection::open(path)?;
    configure_writable(&connection)?;
    let transaction = connection.unchecked_transaction()?;
    transaction.execute(
        "UPDATE makosh_kernel_control_store_metadata
         SET generation = ?1, identity_epoch = ?2, grant_epoch = ?3
         WHERE singleton = 1 AND instance_id = ?4",
        params![
            as_sqlite_integer(fences.generation())?,
            as_sqlite_integer(fences.identity_epoch())?,
            as_sqlite_integer(fences.grant_epoch())?,
            instance_id,
        ],
    )?;
    transaction.execute(
        "UPDATE makosh_kernel_module_registration
         SET state = CASE WHEN state = 'revoked' THEN 'revoked' ELSE 'suspended' END,
             grant_epoch = ?1",
        [as_sqlite_integer(fences.grant_epoch())?],
    )?;
    transaction.execute(
        "UPDATE makosh_kernel_module_registration_capability SET approved = 0",
        [],
    )?;
    transaction.execute("DELETE FROM makosh_kernel_external_runtime_attestation", [])?;
    transaction.execute("DELETE FROM makosh_kernel_managed_launch_record", [])?;
    transaction.execute("DELETE FROM makosh_kernel_server_bootstrap_pairing", [])?;
    transaction.commit()?;
    validate_quick_check(&connection)?;
    Ok(ControlStore::with_recovery_fences(
        instance_id,
        fences.generation(),
        fences.identity_epoch(),
        fences.grant_epoch(),
    ))
}

fn as_sqlite_integer(value: u64) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::RecoveryFenceOverflow)
}
