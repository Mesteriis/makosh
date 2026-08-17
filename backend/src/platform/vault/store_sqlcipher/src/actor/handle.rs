//! Bounded single-writer actor for all unlocked Vault SQLite operations.

use std::collections::BTreeSet;
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, TrySendError, sync_channel};
use std::time::Duration;

use rusqlite::Connection;
use zeroize::Zeroizing;

use crate::database::store::VaultStoreError;
use crate::provisioning::{VaultProvisioningMutationReceiptV1, VaultProvisioningMutationV1};
use crate::records::secret::{self as secret_record, SecretRecordId, SecretRecordScope};

use super::lifecycle::{self, SecretLifecycleMutation};
use super::provisioning::provision;

const ACTOR_QUEUE_CAPACITY: usize = 64;
const OPERATION_DEADLINE: Duration = Duration::from_secs(2);

pub(crate) struct VaultStoreHandle {
    sender: Option<SyncSender<VaultStoreRequest>>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl VaultStoreHandle {
    pub(crate) fn start(
        connection: Connection,
        record_key: Zeroizing<[u8; 32]>,
    ) -> Result<Self, VaultStoreError> {
        let (sender, receiver) = sync_channel(ACTOR_QUEUE_CAPACITY);
        let worker = std::thread::Builder::new()
            .name("makosh-vault-sqlite".to_owned())
            .spawn(move || actor_loop(connection, record_key, receiver))
            .map_err(|_| VaultStoreError::ActorStopped)?;
        Ok(Self {
            sender: Some(sender),
            worker: Some(worker),
        })
    }

    pub(crate) fn store_secret(
        &self,
        scope: &SecretRecordScope,
        payload: &[u8],
    ) -> Result<SecretRecordId, VaultStoreError> {
        let (response, receiver) = sync_channel(1);
        self.submit(VaultStoreRequest::Store {
            scope: scope.clone(),
            payload: Zeroizing::new(payload.to_vec()),
            response,
        })?;
        receive(receiver)
    }

    pub(crate) fn resolve_scoped_secret(
        &self,
        record_id: &SecretRecordId,
        scope: &SecretRecordScope,
    ) -> Result<Zeroizing<Vec<u8>>, VaultStoreError> {
        let (response, receiver) = sync_channel(1);
        self.submit(VaultStoreRequest::Resolve {
            record_id: record_id.clone(),
            scope: scope.clone(),
            response,
        })?;
        receive(receiver)
    }

    pub(crate) fn store_secrets_atomically(
        &self,
        secrets: Vec<(SecretRecordScope, Zeroizing<Vec<u8>>)>,
    ) -> Result<Vec<SecretRecordId>, VaultStoreError> {
        if secrets.is_empty() {
            return Err(VaultStoreError::Record(
                secret_record::SecretRecordError::InvalidPayload,
            ));
        }
        let (response, receiver) = sync_channel(1);
        self.submit(VaultStoreRequest::StoreMany { secrets, response })?;
        receive(receiver)
    }

    pub(crate) fn reconcile_bootstrap_secrets_atomically(
        &self,
        secrets: Vec<(SecretRecordScope, Zeroizing<Vec<u8>>)>,
    ) -> Result<Vec<SecretRecordId>, VaultStoreError> {
        if secrets.is_empty() {
            return Err(VaultStoreError::Record(
                secret_record::SecretRecordError::InvalidPayload,
            ));
        }
        let (response, receiver) = sync_channel(1);
        self.submit(VaultStoreRequest::ReconcileBootstrapMany { secrets, response })?;
        receive(receiver)
    }

    pub(crate) fn resolve_current_secret(
        &self,
        scope: &SecretRecordScope,
    ) -> Result<Zeroizing<Vec<u8>>, VaultStoreError> {
        let (response, receiver) = sync_channel(1);
        self.submit(VaultStoreRequest::ResolveCurrent {
            scope: scope.clone(),
            response,
        })?;
        receive(receiver)
    }

    pub(crate) fn replace_secret(
        &self,
        prior_record_id: &SecretRecordId,
        prior_scope: &SecretRecordScope,
        next_scope: &SecretRecordScope,
        payload: &[u8],
    ) -> Result<SecretRecordId, VaultStoreError> {
        let (response, receiver) = sync_channel(1);
        self.submit(VaultStoreRequest::Replace {
            prior_record_id: prior_record_id.clone(),
            prior_scope: prior_scope.clone(),
            next_scope: next_scope.clone(),
            payload: Zeroizing::new(payload.to_vec()),
            response,
        })?;
        receive(receiver)
    }

    pub(crate) fn retire_secret(
        &self,
        scope: &SecretRecordScope,
        changed_at_unix_seconds: u64,
    ) -> Result<(), VaultStoreError> {
        let (response, receiver) = sync_channel(1);
        self.submit(VaultStoreRequest::Retire {
            scope: scope.clone(),
            changed_at_unix_seconds,
            response,
        })?;
        receive(receiver)
    }

    pub(crate) fn delete_secret(
        &self,
        scope: &SecretRecordScope,
        changed_at_unix_seconds: u64,
    ) -> Result<(), VaultStoreError> {
        let (response, receiver) = sync_channel(1);
        self.submit(VaultStoreRequest::Delete {
            scope: scope.clone(),
            changed_at_unix_seconds,
            response,
        })?;
        receive(receiver)
    }

    pub(crate) fn provision(
        &self,
        mutation: VaultProvisioningMutationV1,
    ) -> Result<VaultProvisioningMutationReceiptV1, VaultStoreError> {
        let (response, receiver) = sync_channel(1);
        self.submit(VaultStoreRequest::Provision { mutation, response })?;
        receive(receiver)
    }

    fn submit(&self, request: VaultStoreRequest) -> Result<(), VaultStoreError> {
        let sender = self.sender.as_ref().ok_or(VaultStoreError::ActorStopped)?;
        sender.try_send(request).map_err(|error| match error {
            TrySendError::Full(_) => VaultStoreError::QueueFull,
            TrySendError::Disconnected(_) => VaultStoreError::ActorStopped,
        })
    }
}

impl Drop for VaultStoreHandle {
    fn drop(&mut self) {
        self.sender.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

enum VaultStoreRequest {
    Store {
        scope: SecretRecordScope,
        payload: Zeroizing<Vec<u8>>,
        response: SyncSender<Result<SecretRecordId, VaultStoreError>>,
    },
    StoreMany {
        secrets: Vec<(SecretRecordScope, Zeroizing<Vec<u8>>)>,
        response: SyncSender<Result<Vec<SecretRecordId>, VaultStoreError>>,
    },
    ReconcileBootstrapMany {
        secrets: Vec<(SecretRecordScope, Zeroizing<Vec<u8>>)>,
        response: SyncSender<Result<Vec<SecretRecordId>, VaultStoreError>>,
    },
    Resolve {
        record_id: SecretRecordId,
        scope: SecretRecordScope,
        response: SyncSender<Result<Zeroizing<Vec<u8>>, VaultStoreError>>,
    },
    ResolveCurrent {
        scope: SecretRecordScope,
        response: SyncSender<Result<Zeroizing<Vec<u8>>, VaultStoreError>>,
    },
    Replace {
        prior_record_id: SecretRecordId,
        prior_scope: SecretRecordScope,
        next_scope: SecretRecordScope,
        payload: Zeroizing<Vec<u8>>,
        response: SyncSender<Result<SecretRecordId, VaultStoreError>>,
    },
    Retire {
        scope: SecretRecordScope,
        changed_at_unix_seconds: u64,
        response: SyncSender<Result<(), VaultStoreError>>,
    },
    Delete {
        scope: SecretRecordScope,
        changed_at_unix_seconds: u64,
        response: SyncSender<Result<(), VaultStoreError>>,
    },
    Provision {
        mutation: VaultProvisioningMutationV1,
        response: SyncSender<Result<VaultProvisioningMutationReceiptV1, VaultStoreError>>,
    },
}

fn receive<T>(receiver: Receiver<Result<T, VaultStoreError>>) -> Result<T, VaultStoreError> {
    match receiver.recv_timeout(OPERATION_DEADLINE) {
        Ok(result) => result,
        Err(RecvTimeoutError::Timeout) => Err(VaultStoreError::DeadlineExceeded),
        Err(RecvTimeoutError::Disconnected) => Err(VaultStoreError::ActorStopped),
    }
}

fn actor_loop(
    mut connection: Connection,
    record_key: Zeroizing<[u8; 32]>,
    receiver: Receiver<VaultStoreRequest>,
) {
    while let Ok(request) = receiver.recv() {
        match request {
            VaultStoreRequest::Store {
                scope,
                payload,
                response,
            } => {
                let _ = response.send(store_secret(&mut connection, &record_key, &scope, &payload));
            }
            VaultStoreRequest::StoreMany { secrets, response } => {
                let _ = response.send(store_secrets(&mut connection, &record_key, secrets));
            }
            VaultStoreRequest::ReconcileBootstrapMany { secrets, response } => {
                let _ = response.send(reconcile_bootstrap_secrets(
                    &mut connection,
                    &record_key,
                    secrets,
                ));
            }
            VaultStoreRequest::Resolve {
                record_id,
                scope,
                response,
            } => {
                let _ = response.send(resolve_secret(&connection, &record_key, &record_id, &scope));
            }
            VaultStoreRequest::ResolveCurrent { scope, response } => {
                let _ = response.send(resolve_current_secret(&connection, &record_key, &scope));
            }
            VaultStoreRequest::Replace {
                prior_record_id,
                prior_scope,
                next_scope,
                payload,
                response,
            } => {
                let _ = response.send(replace_secret(
                    &mut connection,
                    &record_key,
                    &prior_record_id,
                    &prior_scope,
                    &next_scope,
                    &payload,
                ));
            }
            VaultStoreRequest::Retire {
                scope,
                changed_at_unix_seconds,
                response,
            } => {
                let _ = response.send(retire_secret(
                    &mut connection,
                    &scope,
                    changed_at_unix_seconds,
                ));
            }
            VaultStoreRequest::Delete {
                scope,
                changed_at_unix_seconds,
                response,
            } => {
                let _ = response.send(delete_secret(
                    &mut connection,
                    &scope,
                    changed_at_unix_seconds,
                ));
            }
            VaultStoreRequest::Provision { mutation, response } => {
                let _ = response.send(provision(&mut connection, &record_key, &mutation));
            }
        }
    }
}

fn store_secret(
    connection: &mut Connection,
    record_key: &[u8; 32],
    scope: &SecretRecordScope,
    payload: &[u8],
) -> Result<SecretRecordId, VaultStoreError> {
    store_secrets(
        connection,
        record_key,
        vec![(scope.clone(), Zeroizing::new(payload.to_vec()))],
    )
    .map(|mut records| records.remove(0))
}

fn store_secrets(
    connection: &mut Connection,
    record_key: &[u8; 32],
    secrets: Vec<(SecretRecordScope, Zeroizing<Vec<u8>>)>,
) -> Result<Vec<SecretRecordId>, VaultStoreError> {
    let encrypted = secrets
        .iter()
        .map(|(scope, payload)| {
            secret_record::encrypt(scope, payload, record_key).map_err(VaultStoreError::Record)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let transaction = connection
        .unchecked_transaction()
        .map_err(VaultStoreError::Sqlite)?;
    for ((scope, _), record) in secrets.into_iter().zip(encrypted.iter()) {
        let (owner, configuration, purpose, class, revision) = scope.metadata();
        transaction
            .execute(
                "INSERT INTO vault_secret_records (
                record_id, logical_owner_id, configuration_instance_id, purpose_id,
                secret_class, secret_revision, key_epoch, nonce, ciphertext
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    record.record_id.as_bytes().as_slice(),
                    owner,
                    configuration,
                    purpose,
                    class,
                    revision,
                    i64::from(secret_record::CURRENT_KEY_EPOCH),
                    record.nonce.as_slice(),
                    &record.ciphertext,
                ],
            )
            .map_err(VaultStoreError::Sqlite)?;
    }
    transaction.commit().map_err(VaultStoreError::Sqlite)?;
    Ok(encrypted
        .into_iter()
        .map(|record| record.record_id)
        .collect())
}

fn reconcile_bootstrap_secrets(
    connection: &mut Connection,
    record_key: &[u8; 32],
    secrets: Vec<(SecretRecordScope, Zeroizing<Vec<u8>>)>,
) -> Result<Vec<SecretRecordId>, VaultStoreError> {
    let mut scopes = BTreeSet::new();
    for (scope, _) in &secrets {
        if !scopes.insert(scope.metadata()) {
            return Err(VaultStoreError::ProvisioningConflict);
        }
    }
    let encrypted = secrets
        .iter()
        .map(|(scope, payload)| {
            secret_record::encrypt(scope, payload, record_key).map_err(VaultStoreError::Record)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let transaction = connection
        .unchecked_transaction()
        .map_err(VaultStoreError::Sqlite)?;
    for ((scope, _), record) in secrets.into_iter().zip(encrypted.iter()) {
        let (owner, configuration, purpose, class, revision) = scope.metadata();
        transaction
            .execute(
                "DELETE FROM vault_secret_records
                 WHERE logical_owner_id = ?1 AND configuration_instance_id = ?2
                   AND purpose_id = ?3 AND secret_class = ?4 AND secret_revision = ?5",
                rusqlite::params![owner, configuration, purpose, class, revision],
            )
            .map_err(VaultStoreError::Sqlite)?;
        transaction
            .execute(
                "INSERT INTO vault_secret_records (
                    record_id, logical_owner_id, configuration_instance_id, purpose_id,
                    secret_class, secret_revision, key_epoch, nonce, ciphertext
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    record.record_id.as_bytes().as_slice(),
                    owner,
                    configuration,
                    purpose,
                    class,
                    revision,
                    i64::from(secret_record::CURRENT_KEY_EPOCH),
                    record.nonce.as_slice(),
                    &record.ciphertext,
                ],
            )
            .map_err(VaultStoreError::Sqlite)?;
    }
    transaction.commit().map_err(VaultStoreError::Sqlite)?;
    Ok(encrypted
        .into_iter()
        .map(|record| record.record_id)
        .collect())
}

fn resolve_secret(
    connection: &Connection,
    record_key: &[u8; 32],
    record_id: &SecretRecordId,
    scope: &SecretRecordScope,
) -> Result<Zeroizing<Vec<u8>>, VaultStoreError> {
    let record = connection
        .query_row(
            "SELECT record_id, logical_owner_id, configuration_instance_id, purpose_id,
                    secret_class, secret_revision, key_epoch, nonce, ciphertext
             FROM vault_secret_records WHERE record_id = ?1",
            rusqlite::params![record_id.as_bytes().as_slice()],
            read_record,
        )
        .map_err(VaultStoreError::Sqlite)?;
    decrypt_record(scope, record_key, record)
}

fn resolve_current_secret(
    connection: &Connection,
    record_key: &[u8; 32],
    scope: &SecretRecordScope,
) -> Result<Zeroizing<Vec<u8>>, VaultStoreError> {
    let (owner, configuration, purpose, class, revision) = scope.metadata();
    let mut statement = connection
        .prepare(
            "SELECT record_id, logical_owner_id, configuration_instance_id, purpose_id,
                    secret_class, secret_revision, key_epoch, nonce, ciphertext
             FROM vault_secret_records
             WHERE logical_owner_id = ?1 AND configuration_instance_id = ?2 AND purpose_id = ?3
               AND secret_class = ?4 AND secret_revision = ?5 AND key_epoch = ?6",
        )
        .map_err(VaultStoreError::Sqlite)?;
    let mut rows = statement
        .query(rusqlite::params![
            owner,
            configuration,
            purpose,
            class,
            revision,
            i64::from(secret_record::CURRENT_KEY_EPOCH),
        ])
        .map_err(VaultStoreError::Sqlite)?;
    let row = rows
        .next()
        .map_err(VaultStoreError::Sqlite)?
        .ok_or(VaultStoreError::RecordNotFound)?;
    let record = read_record(row).map_err(VaultStoreError::Sqlite)?;
    if rows.next().map_err(VaultStoreError::Sqlite)?.is_some() {
        return Err(VaultStoreError::AmbiguousScope);
    }
    decrypt_record(scope, record_key, record)
}

fn replace_secret(
    connection: &mut Connection,
    record_key: &[u8; 32],
    prior_record_id: &SecretRecordId,
    prior_scope: &SecretRecordScope,
    next_scope: &SecretRecordScope,
    payload: &[u8],
) -> Result<SecretRecordId, VaultStoreError> {
    if !next_scope.replaces(prior_scope) {
        return Err(VaultStoreError::Record(
            secret_record::SecretRecordError::InvalidReplacement,
        ));
    }
    let _ = resolve_secret(connection, record_key, prior_record_id, prior_scope)?;
    let encrypted =
        secret_record::encrypt(next_scope, payload, record_key).map_err(VaultStoreError::Record)?;
    let transaction = connection
        .unchecked_transaction()
        .map_err(VaultStoreError::Sqlite)?;
    transaction
        .execute(
            "DELETE FROM vault_secret_records WHERE record_id = ?1",
            rusqlite::params![prior_record_id.as_bytes().as_slice()],
        )
        .map_err(VaultStoreError::Sqlite)?;
    let (owner, configuration, purpose, class, revision) = next_scope.metadata();
    transaction
        .execute(
            "INSERT INTO vault_secret_records (
                record_id, logical_owner_id, configuration_instance_id, purpose_id,
                secret_class, secret_revision, key_epoch, nonce, ciphertext
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                encrypted.record_id.as_bytes().as_slice(),
                owner,
                configuration,
                purpose,
                class,
                revision,
                i64::from(secret_record::CURRENT_KEY_EPOCH),
                encrypted.nonce.as_slice(),
                encrypted.ciphertext,
            ],
        )
        .map_err(VaultStoreError::Sqlite)?;
    transaction.commit().map_err(VaultStoreError::Sqlite)?;
    Ok(encrypted.record_id)
}

fn retire_secret(
    connection: &mut Connection,
    scope: &SecretRecordScope,
    changed_at_unix_seconds: u64,
) -> Result<(), VaultStoreError> {
    lifecycle::mutate(
        connection,
        scope,
        changed_at_unix_seconds,
        SecretLifecycleMutation::Retire,
    )
}

fn delete_secret(
    connection: &mut Connection,
    scope: &SecretRecordScope,
    changed_at_unix_seconds: u64,
) -> Result<(), VaultStoreError> {
    lifecycle::mutate(
        connection,
        scope,
        changed_at_unix_seconds,
        SecretLifecycleMutation::Delete,
    )
}

pub(super) type StoredRecord = (
    Vec<u8>,
    String,
    String,
    String,
    i64,
    i64,
    i64,
    Vec<u8>,
    Vec<u8>,
);

pub(super) fn read_record(row: &rusqlite::Row<'_>) -> Result<StoredRecord, rusqlite::Error> {
    Ok((
        row.get::<_, Vec<u8>>(0)?,
        row.get::<_, String>(1)?,
        row.get::<_, String>(2)?,
        row.get::<_, String>(3)?,
        row.get::<_, i64>(4)?,
        row.get::<_, i64>(5)?,
        row.get::<_, i64>(6)?,
        row.get::<_, Vec<u8>>(7)?,
        row.get::<_, Vec<u8>>(8)?,
    ))
}

pub(super) fn decrypt_record(
    scope: &SecretRecordScope,
    record_key: &[u8; 32],
    record: StoredRecord,
) -> Result<Zeroizing<Vec<u8>>, VaultStoreError> {
    let (record_id, owner, configuration, purpose, class, revision, key_epoch, nonce, ciphertext) =
        record;
    if !scope.matches_metadata(&owner, &configuration, &purpose, class, revision, key_epoch) {
        return Err(VaultStoreError::Record(
            secret_record::SecretRecordError::MalformedRecord,
        ));
    }
    let record_id = SecretRecordId::from_slice(&record_id).map_err(VaultStoreError::Record)?;
    secret_record::decrypt(scope, record_id, &nonce, &ciphertext, record_key)
        .map_err(VaultStoreError::Record)
}
