//! Public Vault store facade and encrypted-database bootstrap validation.

use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use makosh_vault_key_provider::WrappingKey;
use rusqlite::Connection;
use zeroize::Zeroizing;

use crate::VaultRecoveryKeyV1;
use crate::actor::VaultStoreHandle;
use crate::database::connection::{configure, create_keyed_connection, open_keyed_connection};
use crate::identity::anchor as vault_anchor;
use crate::records::secret::{self as secret_record, SecretRecordId, SecretRecordScope};
use crate::recovery::root_rotation;

const SCHEMA_VERSION: i64 = 4;

pub(crate) type VaultStoreResult<T> = Result<T, VaultStoreError>;

pub struct VaultStore {
    path: PathBuf,
    instance_id: String,
    handle: VaultStoreHandle,
}

impl VaultStore {
    pub fn initialize(
        path: &Path,
        anchor_path: &Path,
        instance_id: &str,
        wrapping_key: &WrappingKey,
    ) -> Result<Self, VaultStoreError> {
        let path = canonical_store_path(path)?;
        validate_store_parent(&path)?;
        validate_store_parent(anchor_path)?;
        if path.exists() {
            return Err(VaultStoreError::AlreadyInitialized);
        }
        let root = vault_anchor::create_anchor(anchor_path, instance_id, wrapping_key)
            .map_err(|_| VaultStoreError::Anchor)?;
        let result = initialize_database(&path, instance_id, &root);
        if result.is_err() {
            let _ = fs::remove_file(&path);
            let _ = fs::remove_file(anchor_path);
        }
        result
    }

    pub fn open(
        path: &Path,
        anchor_path: &Path,
        wrapping_key: &WrappingKey,
    ) -> Result<Self, VaultStoreError> {
        let path = canonical_store_path(path)?;
        validate_store_parent(&path)?;
        if root_rotation::rotation_pending(&path)? {
            return Err(VaultStoreError::RootRotationPending);
        }
        let (instance_id, root) = vault_anchor::open_anchor(anchor_path, wrapping_key)
            .map_err(|_| VaultStoreError::Anchor)?;
        open_existing_store(path, instance_id, &root)
    }

    pub fn open_with_recovery(
        path: &Path,
        anchor_path: &Path,
        recovery_key: &VaultRecoveryKeyV1,
    ) -> Result<Self, VaultStoreError> {
        let path = canonical_store_path(path)?;
        validate_store_parent(&path)?;
        if root_rotation::rotation_pending(&path)? {
            return Err(VaultStoreError::RootRotationPending);
        }
        let (instance_id, root) =
            vault_anchor::open_anchor_with_recovery(anchor_path, recovery_key)
                .map_err(|_| VaultStoreError::Anchor)?;
        open_existing_store(path, instance_id, &root)
    }

    pub fn add_recovery_slot(
        anchor_path: &Path,
        wrapping_key: &WrappingKey,
        recovery_key: &VaultRecoveryKeyV1,
    ) -> Result<(), VaultStoreError> {
        vault_anchor::add_recovery_slot(anchor_path, wrapping_key, recovery_key)
            .map_err(|_| VaultStoreError::Anchor)
    }

    pub fn rotate_recovery_slot(
        anchor_path: &Path,
        wrapping_key: &WrappingKey,
        current_recovery_key: &VaultRecoveryKeyV1,
        next_recovery_key: &VaultRecoveryKeyV1,
    ) -> Result<(), VaultStoreError> {
        vault_anchor::rotate_recovery_slot(
            anchor_path,
            wrapping_key,
            current_recovery_key,
            next_recovery_key,
        )
        .map_err(|_| VaultStoreError::Anchor)
    }

    pub fn rotate_root_offline(
        path: &Path,
        anchor_path: &Path,
        wrapping_key: &WrappingKey,
        recovery_key: Option<&VaultRecoveryKeyV1>,
    ) -> Result<(), VaultStoreError> {
        let path = canonical_store_path(path)?;
        validate_store_parent(&path)?;
        validate_store_parent(anchor_path)?;
        let (instance_id, root) = vault_anchor::open_anchor(anchor_path, wrapping_key)
            .map_err(|_| VaultStoreError::Anchor)?;
        root_rotation::rotate(
            &path,
            anchor_path,
            &instance_id,
            &root,
            wrapping_key,
            recovery_key,
        )
    }

    #[must_use]
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn store_secret(
        &self,
        scope: &SecretRecordScope,
        payload: &[u8],
    ) -> Result<SecretRecordId, VaultStoreError> {
        self.handle.store_secret(scope, payload)
    }

    pub fn store_secrets_atomically(
        &self,
        secrets: Vec<(SecretRecordScope, Zeroizing<Vec<u8>>)>,
    ) -> Result<Vec<SecretRecordId>, VaultStoreError> {
        self.handle.store_secrets_atomically(secrets)
    }

    pub fn reconcile_bootstrap_secrets_atomically(
        &self,
        secrets: Vec<(SecretRecordScope, Zeroizing<Vec<u8>>)>,
    ) -> Result<Vec<SecretRecordId>, VaultStoreError> {
        self.handle.reconcile_bootstrap_secrets_atomically(secrets)
    }

    pub fn resolve_scoped_secret(
        &self,
        record_id: &SecretRecordId,
        scope: &SecretRecordScope,
    ) -> Result<Zeroizing<Vec<u8>>, VaultStoreError> {
        self.handle.resolve_scoped_secret(record_id, scope)
    }

    pub fn resolve_current_secret(
        &self,
        scope: &SecretRecordScope,
    ) -> Result<Zeroizing<Vec<u8>>, VaultStoreError> {
        self.handle.resolve_current_secret(scope)
    }

    pub fn replace_secret(
        &self,
        prior_record_id: &SecretRecordId,
        prior_scope: &SecretRecordScope,
        next_scope: &SecretRecordScope,
        payload: &[u8],
    ) -> Result<SecretRecordId, VaultStoreError> {
        self.handle
            .replace_secret(prior_record_id, prior_scope, next_scope, payload)
    }

    pub fn retire_secret(
        &self,
        scope: &SecretRecordScope,
        changed_at_unix_seconds: u64,
    ) -> Result<(), VaultStoreError> {
        self.handle.retire_secret(scope, changed_at_unix_seconds)
    }

    pub fn delete_secret(
        &self,
        scope: &SecretRecordScope,
        changed_at_unix_seconds: u64,
    ) -> Result<(), VaultStoreError> {
        self.handle.delete_secret(scope, changed_at_unix_seconds)
    }

    pub fn provision(
        &self,
        mutation: crate::VaultProvisioningMutationV1,
    ) -> Result<crate::VaultProvisioningMutationReceiptV1, VaultStoreError> {
        self.handle.provision(mutation)
    }

    fn from_connection(
        path: PathBuf,
        instance_id: String,
        connection: Connection,
        record_key: Zeroizing<[u8; 32]>,
    ) -> Result<Self, VaultStoreError> {
        let handle = VaultStoreHandle::start(connection, record_key)?;
        Ok(Self {
            path,
            instance_id,
            handle,
        })
    }
}

fn open_existing_store(
    path: PathBuf,
    instance_id: String,
    root: &vault_anchor::VaultRootKey,
) -> Result<VaultStore, VaultStoreError> {
    let current_sqlcipher_key = root
        .derive_sqlcipher_key(&instance_id)
        .map_err(|_| VaultStoreError::Anchor)?;
    let current_connection = open_keyed_connection(&path, &current_sqlcipher_key)?;
    let (mut connection, record_key) = match validate_encrypted_connection(&current_connection) {
        Ok(()) => {
            let record_key = root
                .derive_record_key(&instance_id, secret_record::CURRENT_KEY_EPOCH)
                .map_err(|_| VaultStoreError::Anchor)?;
            (current_connection, record_key)
        }
        Err(error) if is_not_a_database(&error) => {
            drop(current_connection);
            let legacy_sqlcipher_key = root
                .derive_legacy_sqlcipher_key(&instance_id)
                .map_err(|_| VaultStoreError::Anchor)?;
            let legacy_connection = open_keyed_connection(&path, &legacy_sqlcipher_key)?;
            validate_encrypted_connection(&legacy_connection)?;
            let record_key = root
                .derive_legacy_record_key(&instance_id, secret_record::CURRENT_KEY_EPOCH)
                .map_err(|_| VaultStoreError::Anchor)?;
            (legacy_connection, record_key)
        }
        Err(error) => return Err(error),
    };
    configure(&connection)?;
    migrate_schema(&mut connection)?;
    validate_metadata(&connection, &instance_id)?;
    VaultStore::from_connection(path, instance_id, connection, record_key)
}

fn validate_encrypted_connection(connection: &Connection) -> Result<(), VaultStoreError> {
    connection
        .query_row("SELECT COUNT(*) FROM sqlite_master", [], |_| Ok(()))
        .map_err(VaultStoreError::Sqlite)
}

fn is_not_a_database(error: &VaultStoreError) -> bool {
    matches!(
        error,
        VaultStoreError::Sqlite(rusqlite::Error::SqliteFailure(sqlite, _))
            if sqlite.code == rusqlite::ErrorCode::NotADatabase
    )
}

fn initialize_database(
    path: &Path,
    instance_id: &str,
    root: &vault_anchor::VaultRootKey,
) -> Result<VaultStore, VaultStoreError> {
    let sqlcipher_key = root
        .derive_sqlcipher_key(instance_id)
        .map_err(|_| VaultStoreError::Anchor)?;
    let record_key = root
        .derive_record_key(instance_id, secret_record::CURRENT_KEY_EPOCH)
        .map_err(|_| VaultStoreError::Anchor)?;
    initialize_database_with_keys(path, instance_id, &sqlcipher_key, record_key)
}

fn initialize_database_with_keys(
    path: &Path,
    instance_id: &str,
    sqlcipher_key: &[u8; 32],
    record_key: Zeroizing<[u8; 32]>,
) -> Result<VaultStore, VaultStoreError> {
    let connection = create_keyed_connection(path, sqlcipher_key)?;
    configure(&connection)?;
    connection
        .execute_batch(
            "CREATE TABLE vault_metadata (
                singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                schema_version INTEGER NOT NULL CHECK (schema_version = 4),
                instance_id TEXT NOT NULL
            ) STRICT;
            CREATE TABLE vault_secret_records (
                record_id BLOB PRIMARY KEY CHECK (length(record_id) = 16),
                logical_owner_id TEXT NOT NULL,
                configuration_instance_id TEXT NOT NULL,
                purpose_id TEXT NOT NULL,
                secret_class INTEGER NOT NULL,
                secret_revision INTEGER NOT NULL CHECK (secret_revision > 0),
                key_epoch INTEGER NOT NULL CHECK (key_epoch > 0),
                nonce BLOB NOT NULL CHECK (length(nonce) = 24),
                ciphertext BLOB NOT NULL CHECK (length(ciphertext) > 16)
            ) STRICT;
            CREATE UNIQUE INDEX vault_secret_records_scope_revision
                ON vault_secret_records (
                    logical_owner_id, configuration_instance_id, purpose_id,
                    secret_class, secret_revision, key_epoch
                );
            CREATE TABLE vault_secret_tombstones (
                logical_owner_id TEXT NOT NULL,
                configuration_instance_id TEXT NOT NULL,
                purpose_id TEXT NOT NULL,
                secret_class INTEGER NOT NULL,
                secret_revision INTEGER NOT NULL CHECK (secret_revision > 0),
                state INTEGER NOT NULL CHECK (state IN (1, 2)),
                changed_at_unix_seconds INTEGER NOT NULL CHECK (changed_at_unix_seconds > 0),
                PRIMARY KEY (
                    logical_owner_id, configuration_instance_id, purpose_id,
                    secret_class, secret_revision
                )
            ) STRICT;
            CREATE TABLE vault_owner_provisioning_receipts (
                operation_id BLOB PRIMARY KEY CHECK (length(operation_id) = 16),
                intent_digest BLOB NOT NULL CHECK (length(intent_digest) = 32),
                action INTEGER NOT NULL CHECK (action IN (2, 3, 4, 5)),
                secret_revision INTEGER NOT NULL CHECK (secret_revision > 0),
                state INTEGER NOT NULL CHECK (state IN (1, 2, 3)),
                changed_at_unix_seconds INTEGER NOT NULL
                    CHECK (changed_at_unix_seconds > 0)
            ) STRICT;
            CREATE TRIGGER vault_secret_records_reject_tombstone
                BEFORE INSERT ON vault_secret_records
                WHEN EXISTS (
                    SELECT 1 FROM vault_secret_tombstones
                    WHERE logical_owner_id = NEW.logical_owner_id
                      AND configuration_instance_id = NEW.configuration_instance_id
                      AND purpose_id = NEW.purpose_id
                      AND secret_class = NEW.secret_class
                      AND secret_revision = NEW.secret_revision
                )
                BEGIN
                    SELECT RAISE(ABORT, 'retired Vault secret revision');
                END;",
        )
        .map_err(VaultStoreError::Sqlite)?;
    connection
        .execute(
            "INSERT INTO vault_metadata (singleton, schema_version, instance_id) VALUES (1, ?1, ?2)",
            rusqlite::params![SCHEMA_VERSION, instance_id],
        )
        .map_err(VaultStoreError::Sqlite)?;
    connection
        .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(VaultStoreError::Sqlite)?;
    VaultStore::from_connection(
        path.to_owned(),
        instance_id.to_owned(),
        connection,
        record_key,
    )
}

fn validate_metadata(
    connection: &Connection,
    expected_instance_id: &str,
) -> Result<(), VaultStoreError> {
    let (version, actual_instance_id): (i64, String) = connection
        .query_row(
            "SELECT schema_version, instance_id FROM vault_metadata WHERE singleton = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(VaultStoreError::Sqlite)?;
    if version != SCHEMA_VERSION {
        return Err(VaultStoreError::UnsupportedSchema);
    }
    if actual_instance_id == expected_instance_id {
        Ok(())
    } else {
        Err(VaultStoreError::Anchor)
    }
}

fn migrate_schema(connection: &mut Connection) -> Result<(), VaultStoreError> {
    let version: i64 = connection
        .query_row(
            "SELECT schema_version FROM vault_metadata WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .map_err(VaultStoreError::Sqlite)?;
    match version {
        SCHEMA_VERSION => Ok(()),
        1 => {
            migrate_v1_to_v2(connection)?;
            migrate_v2_to_v3(connection)?;
            migrate_v3_to_v4(connection)
        }
        2 => {
            migrate_v2_to_v3(connection)?;
            migrate_v3_to_v4(connection)
        }
        3 => migrate_v3_to_v4(connection),
        _ => Err(VaultStoreError::UnsupportedSchema),
    }
}

fn migrate_v1_to_v2(connection: &mut Connection) -> Result<(), VaultStoreError> {
    let transaction = connection
        .unchecked_transaction()
        .map_err(VaultStoreError::Sqlite)?;
    transaction
        .execute_batch(
            "CREATE TABLE vault_metadata_v2 (
                singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                schema_version INTEGER NOT NULL CHECK (schema_version = 2),
                instance_id TEXT NOT NULL
             ) STRICT;
             INSERT INTO vault_metadata_v2 (singleton, schema_version, instance_id)
                SELECT singleton, 2, instance_id FROM vault_metadata;
             DROP TABLE vault_metadata;
             ALTER TABLE vault_metadata_v2 RENAME TO vault_metadata;
             CREATE UNIQUE INDEX vault_secret_records_scope_revision
                ON vault_secret_records (
                    logical_owner_id, configuration_instance_id, purpose_id,
                    secret_class, secret_revision, key_epoch
                );",
        )
        .map_err(VaultStoreError::Sqlite)?;
    transaction.commit().map_err(VaultStoreError::Sqlite)
}

fn migrate_v2_to_v3(connection: &mut Connection) -> Result<(), VaultStoreError> {
    let transaction = connection
        .unchecked_transaction()
        .map_err(VaultStoreError::Sqlite)?;
    transaction
        .execute_batch(
            "CREATE TABLE vault_metadata_v3 (
                singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                schema_version INTEGER NOT NULL CHECK (schema_version = 3),
                instance_id TEXT NOT NULL
             ) STRICT;
             INSERT INTO vault_metadata_v3 (singleton, schema_version, instance_id)
                SELECT singleton, 3, instance_id FROM vault_metadata;
             DROP TABLE vault_metadata;
             ALTER TABLE vault_metadata_v3 RENAME TO vault_metadata;
             CREATE TABLE vault_secret_tombstones (
                logical_owner_id TEXT NOT NULL,
                configuration_instance_id TEXT NOT NULL,
                purpose_id TEXT NOT NULL,
                secret_class INTEGER NOT NULL,
                secret_revision INTEGER NOT NULL CHECK (secret_revision > 0),
                state INTEGER NOT NULL CHECK (state IN (1, 2)),
                changed_at_unix_seconds INTEGER NOT NULL CHECK (changed_at_unix_seconds > 0),
                PRIMARY KEY (
                    logical_owner_id, configuration_instance_id, purpose_id,
                    secret_class, secret_revision
                )
             ) STRICT;
             CREATE TRIGGER vault_secret_records_reject_tombstone
                BEFORE INSERT ON vault_secret_records
                WHEN EXISTS (
                    SELECT 1 FROM vault_secret_tombstones
                    WHERE logical_owner_id = NEW.logical_owner_id
                      AND configuration_instance_id = NEW.configuration_instance_id
                      AND purpose_id = NEW.purpose_id
                      AND secret_class = NEW.secret_class
                      AND secret_revision = NEW.secret_revision
                )
                BEGIN
                    SELECT RAISE(ABORT, 'retired Vault secret revision');
                END;",
        )
        .map_err(VaultStoreError::Sqlite)?;
    transaction.commit().map_err(VaultStoreError::Sqlite)
}

fn migrate_v3_to_v4(connection: &mut Connection) -> Result<(), VaultStoreError> {
    let transaction = connection
        .unchecked_transaction()
        .map_err(VaultStoreError::Sqlite)?;
    transaction
        .execute_batch(
            "CREATE TABLE vault_metadata_v4 (
                singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                schema_version INTEGER NOT NULL CHECK (schema_version = 4),
                instance_id TEXT NOT NULL
             ) STRICT;
             INSERT INTO vault_metadata_v4 (singleton, schema_version, instance_id)
                SELECT singleton, 4, instance_id FROM vault_metadata;
             DROP TABLE vault_metadata;
             ALTER TABLE vault_metadata_v4 RENAME TO vault_metadata;
             CREATE TABLE vault_owner_provisioning_receipts (
                operation_id BLOB PRIMARY KEY CHECK (length(operation_id) = 16),
                intent_digest BLOB NOT NULL CHECK (length(intent_digest) = 32),
                action INTEGER NOT NULL CHECK (action IN (2, 3, 4, 5)),
                secret_revision INTEGER NOT NULL CHECK (secret_revision > 0),
                state INTEGER NOT NULL CHECK (state IN (1, 2, 3)),
                changed_at_unix_seconds INTEGER NOT NULL
                    CHECK (changed_at_unix_seconds > 0)
             ) STRICT;",
        )
        .map_err(VaultStoreError::Sqlite)?;
    transaction.commit().map_err(VaultStoreError::Sqlite)
}

pub(crate) fn validate_store_parent(path: &Path) -> Result<(), VaultStoreError> {
    let parent = path.parent().ok_or(VaultStoreError::InsecurePath)?;
    let metadata = fs::symlink_metadata(parent).map_err(|_| VaultStoreError::InsecurePath)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() || metadata.mode() & 0o077 != 0 {
        return Err(VaultStoreError::InsecurePath);
    }
    if let Ok(metadata) = fs::symlink_metadata(path)
        && (metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.mode() & 0o077 != 0)
    {
        return Err(VaultStoreError::InsecurePath);
    }
    Ok(())
}

pub(crate) fn canonical_store_path(path: &Path) -> Result<PathBuf, VaultStoreError> {
    let parent = path.parent().ok_or(VaultStoreError::InsecurePath)?;
    let name = path.file_name().ok_or(VaultStoreError::InsecurePath)?;
    let parent = fs::canonicalize(parent).map_err(|_| VaultStoreError::InsecurePath)?;
    Ok(parent.join(name))
}

#[derive(Debug)]
pub enum VaultStoreError {
    AlreadyInitialized,
    InsecurePath,
    Anchor,
    Record(secret_record::SecretRecordError),
    RecordNotFound,
    UnsupportedSchema,
    RootRotationPending,
    Backup,
    QueueFull,
    DeadlineExceeded,
    ActorStopped,
    AmbiguousScope,
    LifecycleConflict,
    ProvisioningConflict,
    Sqlite(rusqlite::Error),
}

impl std::fmt::Display for VaultStoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::AlreadyInitialized => "Vault store is already initialized",
            Self::InsecurePath => "Vault store path is not private and regular",
            Self::Anchor => "Vault root-key slot is unavailable or invalid",
            Self::Record(_) => "Vault credential record is unavailable or invalid",
            Self::RecordNotFound => "Vault credential scope has no current record",
            Self::UnsupportedSchema => "Vault store schema is unsupported",
            Self::RootRotationPending => {
                "Vault root-key rotation requires explicit offline recovery"
            }
            Self::Backup => "Vault backup is unavailable or invalid",
            Self::QueueFull => "Vault store request queue is full",
            Self::DeadlineExceeded => "Vault store operation deadline exceeded",
            Self::ActorStopped => "Vault store actor is stopped",
            Self::AmbiguousScope => "Vault credential scope is ambiguous",
            Self::LifecycleConflict => "Vault credential lifecycle state conflicts",
            Self::ProvisioningConflict => "Vault owner provisioning intent conflicts",
            Self::Sqlite(_) => "Vault encrypted store operation failed",
        })
    }
}

impl std::error::Error for VaultStoreError {}

impl VaultStoreError {
    #[must_use]
    pub const fn is_malformed_record(&self) -> bool {
        matches!(
            self,
            Self::Record(secret_record::SecretRecordError::MalformedRecord)
        )
    }

    #[must_use]
    pub const fn is_record_not_found(&self) -> bool {
        matches!(self, Self::RecordNotFound)
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn create() -> Self {
            for attempt in 0..16 {
                let path = std::env::temp_dir().join(format!(
                    "makosh-vault-legacy-derivation-{}-{attempt}",
                    std::process::id()
                ));
                match std::fs::create_dir(&path) {
                    Ok(()) => {
                        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
                            .expect("make test directory private");
                        return Self(path);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("create test directory: {error}"),
                }
            }
            panic!("create unique test directory")
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn opens_store_created_with_legacy_key_derivation() {
        let directory = TestDirectory::create();
        let database = canonical_store_path(&directory.0.join("vault.db"))
            .expect("canonical test database path");
        let anchor = directory.0.join("vault.anchor");
        let instance_id = "legacy-vault-instance";
        let wrapping_key = WrappingKey::from_bytes([7; 32]);
        let root = vault_anchor::create_anchor(&anchor, instance_id, &wrapping_key)
            .expect("create anchor");
        let sqlcipher_key = root
            .derive_legacy_sqlcipher_key(instance_id)
            .expect("legacy sqlcipher key");
        let record_key = root
            .derive_legacy_record_key(instance_id, secret_record::CURRENT_KEY_EPOCH)
            .expect("legacy record key");
        let legacy_store =
            initialize_database_with_keys(&database, instance_id, &sqlcipher_key, record_key)
                .expect("create legacy-derived store");
        drop(legacy_store);

        let reopened =
            VaultStore::open(&database, &anchor, &wrapping_key).expect("open legacy-derived store");

        assert_eq!(reopened.instance_id, instance_id);
    }
}
