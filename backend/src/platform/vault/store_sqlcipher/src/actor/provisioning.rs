//! Durable idempotent write-only provisioning transaction.

use makosh_vault_protocol::{
    VaultActionV1, VaultProvisioningReceiptV1, VaultProvisioningStateV1, state_for_action,
};
use rusqlite::{Connection, OptionalExtension, Transaction};
use sha2::{Digest, Sha256};

use crate::database::store::{VaultStoreError, VaultStoreResult};
use crate::provisioning::{VaultProvisioningMutationReceiptV1, VaultProvisioningMutationV1};
use crate::records::secret::{self as secret_record, SecretRecordScope};

use super::handle::{decrypt_record, read_record};
use super::lifecycle::{self, SecretLifecycleMutation};

pub(super) fn provision(
    connection: &mut Connection,
    record_key: &[u8; 32],
    mutation: &VaultProvisioningMutationV1,
) -> VaultStoreResult<VaultProvisioningMutationReceiptV1> {
    let intent_digest = intent_digest(mutation);
    let transaction = connection
        .unchecked_transaction()
        .map_err(VaultStoreError::Sqlite)?;
    if let Some(receipt) = prior_receipt(&transaction, mutation.operation_id(), &intent_digest)? {
        return Ok(receipt);
    }

    match mutation.action() {
        VaultActionV1::Create => insert_secret(
            &transaction,
            record_key,
            mutation.scope(),
            mutation.payload(),
        )?,
        VaultActionV1::ReplaceCas => replace_secret(
            &transaction,
            record_key,
            mutation.scope(),
            mutation.payload(),
        )?,
        VaultActionV1::Retire => lifecycle::apply(
            &transaction,
            mutation.scope(),
            mutation.changed_at_unix_seconds(),
            SecretLifecycleMutation::Retire,
        )?,
        VaultActionV1::Delete => lifecycle::apply(
            &transaction,
            mutation.scope(),
            mutation.changed_at_unix_seconds(),
            SecretLifecycleMutation::Delete,
        )?,
        _ => return Err(VaultStoreError::ProvisioningConflict),
    }

    let (_, _, _, _, revision) = mutation.scope().metadata();
    let revision = u64::try_from(revision).map_err(|_| VaultStoreError::ProvisioningConflict)?;
    let receipt = VaultProvisioningReceiptV1::new(
        *mutation.operation_id(),
        mutation.action(),
        revision,
        state_for_action(mutation.action()),
    )
    .map_err(|_| VaultStoreError::ProvisioningConflict)?;
    transaction
        .execute(
            "INSERT INTO vault_owner_provisioning_receipts (
                operation_id, intent_digest, action, secret_revision, state,
                changed_at_unix_seconds
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                mutation.operation_id().as_slice(),
                intent_digest.as_slice(),
                mutation.action().code(),
                revision,
                receipt.state().code(),
                mutation.changed_at_unix_seconds(),
            ],
        )
        .map_err(|_| VaultStoreError::ProvisioningConflict)?;
    transaction.commit().map_err(VaultStoreError::Sqlite)?;
    Ok(receipt)
}

fn prior_receipt(
    transaction: &Transaction<'_>,
    operation_id: &[u8; 16],
    expected_intent_digest: &[u8; 32],
) -> VaultStoreResult<Option<VaultProvisioningReceiptV1>> {
    let stored = transaction
        .query_row(
            "SELECT intent_digest, action, secret_revision, state
             FROM vault_owner_provisioning_receipts
             WHERE operation_id = ?1",
            rusqlite::params![operation_id.as_slice()],
            |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )
        .optional()
        .map_err(VaultStoreError::Sqlite)?;
    let Some((intent_digest, action, revision, state)) = stored else {
        return Ok(None);
    };
    if intent_digest.as_slice() != expected_intent_digest {
        return Err(VaultStoreError::ProvisioningConflict);
    }
    let action = VaultActionV1::from_code(action).ok_or(VaultStoreError::ProvisioningConflict)?;
    let revision = u64::try_from(revision)
        .ok()
        .filter(|revision| *revision > 0)
        .ok_or(VaultStoreError::ProvisioningConflict)?;
    let state = u8::try_from(state)
        .ok()
        .and_then(VaultProvisioningStateV1::from_code)
        .ok_or(VaultStoreError::ProvisioningConflict)?;
    VaultProvisioningReceiptV1::new(*operation_id, action, revision, state)
        .map(Some)
        .map_err(|_| VaultStoreError::ProvisioningConflict)
}

fn insert_secret(
    transaction: &Transaction<'_>,
    record_key: &[u8; 32],
    scope: &SecretRecordScope,
    payload: &[u8],
) -> VaultStoreResult<()> {
    let encrypted =
        secret_record::encrypt(scope, payload, record_key).map_err(VaultStoreError::Record)?;
    insert_encrypted(transaction, scope, &encrypted)
}

fn replace_secret(
    transaction: &Transaction<'_>,
    record_key: &[u8; 32],
    next_scope: &SecretRecordScope,
    payload: &[u8],
) -> VaultStoreResult<()> {
    let (_, _, _, _, next_revision) = next_scope.metadata();
    let prior_revision = u64::try_from(next_revision)
        .ok()
        .and_then(|revision| revision.checked_sub(1))
        .filter(|revision| *revision > 0)
        .ok_or(VaultStoreError::ProvisioningConflict)?;
    let prior_scope = next_scope
        .with_revision(prior_revision)
        .map_err(VaultStoreError::Record)?;
    let (owner, configuration, purpose, class, revision) = prior_scope.metadata();
    let mut statement = transaction
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
        .ok_or(VaultStoreError::ProvisioningConflict)?;
    let stored = read_record(row).map_err(VaultStoreError::Sqlite)?;
    if rows.next().map_err(VaultStoreError::Sqlite)?.is_some() {
        return Err(VaultStoreError::AmbiguousScope);
    }
    let prior_record_id = stored.0.clone();
    let _ = decrypt_record(&prior_scope, record_key, stored)?;
    drop(rows);
    drop(statement);

    let encrypted =
        secret_record::encrypt(next_scope, payload, record_key).map_err(VaultStoreError::Record)?;
    transaction
        .execute(
            "DELETE FROM vault_secret_records WHERE record_id = ?1",
            rusqlite::params![prior_record_id],
        )
        .map_err(VaultStoreError::Sqlite)?;
    insert_encrypted(transaction, next_scope, &encrypted)
}

fn insert_encrypted(
    transaction: &Transaction<'_>,
    scope: &SecretRecordScope,
    encrypted: &secret_record::EncryptedSecretRecord,
) -> VaultStoreResult<()> {
    let (owner, configuration, purpose, class, revision) = scope.metadata();
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
                &encrypted.ciphertext,
            ],
        )
        .map_err(|_| VaultStoreError::ProvisioningConflict)?;
    Ok(())
}

fn intent_digest(mutation: &VaultProvisioningMutationV1) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.owner-vault-provisioning.intent.v1\0");
    digest.update([mutation.action().code() as u8]);
    let (owner, configuration, purpose, class, revision) = mutation.scope().metadata();
    append(&mut digest, owner.as_bytes());
    append(&mut digest, configuration.as_bytes());
    append(&mut digest, purpose.as_bytes());
    digest.update(class.to_be_bytes());
    digest.update(revision.to_be_bytes());
    digest.update(Sha256::digest(mutation.payload()));
    digest.finalize().into()
}

fn append(digest: &mut Sha256, value: &[u8]) {
    digest.update(u32::try_from(value.len()).unwrap_or(u32::MAX).to_be_bytes());
    digest.update(value);
}
