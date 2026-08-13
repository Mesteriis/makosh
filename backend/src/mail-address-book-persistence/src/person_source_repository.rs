use std::collections::BTreeSet;

use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{
        ActorKindV1, DurableEnvelopeV1, FenceKindV1, ResultOutcomeV1,
        durable_envelope_v1::Semantics,
    },
    validation::envelope::validate_envelope_v1,
};
use makosh_mail_address_book_contract::{
    MAIL_PERSON_SOURCE_CAPABILITY_ID_V1, MAIL_RUNTIME_MODULE_ID_V1, MailPersonSourceContractV1,
    mail_person_source_tombstone_digest_v1, validate_fetch_mail_person_source_page_v1,
    validate_mail_person_source_observed_v1, validate_mail_person_source_page_completed_v1,
    validate_mail_person_source_removed_v1, validate_mail_person_source_updated_v1,
    wire_person_source::{
        FetchMailPersonSourcePageCommandV1, MailPersonSourceObservedV1,
        MailPersonSourcePageCompletedV1, MailPersonSourceRemovedV1, MailPersonSourceUpdatedV1,
    },
};
use prost::Message;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};

use crate::{MailAddressBookPersistenceErrorV1, MailAddressBookPersistenceV1};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonSourceAccountMappingV1 {
    pub integration_public_id: [u8; 16],
    pub account_public_id: [u8; 16],
    pub mapping_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonSourceAccountBindingV1 {
    pub private_account_key: String,
    pub mapping: MailPersonSourceAccountMappingV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonSourceFetchStateV1 {
    pub provider_cursor: Option<Vec<u8>>,
    pub page_sequence: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonSourceChangeKindV1 {
    Observed,
    Unchanged,
    Updated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonSourceObservationV1 {
    pub logical_owner_id: String,
    pub account_public_id: [u8; 16],
    pub provider_record_key: Vec<u8>,
    pub provider_record_etag: Option<Vec<u8>>,
    pub proposed_source_public_id: [u8; 16],
    pub claims_digest: [u8; 32],
    pub observed_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MailPersonSourceObservationOutcomeV1 {
    pub provider_source_contact_public_id: [u8; 16],
    pub source_revision: u64,
    pub change_kind: MailPersonSourceChangeKindV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonSourceEnvelopeRecordV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

impl MailPersonSourceEnvelopeRecordV1 {
    #[must_use]
    pub fn from_outbox(record: &OutboxRecordV1) -> Self {
        Self {
            message_id: *record.message_id(),
            envelope_sha256: *record.envelope_sha256(),
            envelope_bytes: record.exact_bytes().to_vec(),
        }
    }

    fn validate(&self) -> Result<(), MailAddressBookPersistenceErrorV1> {
        self.decode().map(|_| ())
    }

    fn decode(&self) -> Result<DurableEnvelopeV1, MailAddressBookPersistenceErrorV1> {
        let accepted = OutboxRecordV1::accept(self.envelope_bytes.clone())
            .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
        let actual_sha256: [u8; 32] = Sha256::digest(&self.envelope_bytes).into();
        if accepted.message_id() != &self.message_id
            || accepted.envelope_sha256() != &self.envelope_sha256
            || actual_sha256 != self.envelope_sha256
        {
            return Err(MailAddressBookPersistenceErrorV1::HashMismatch);
        }
        let envelope = DurableEnvelopeV1::decode(self.envelope_bytes.as_slice())
            .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
        validate_envelope_v1(&envelope)
            .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
        if envelope.encode_to_vec() != self.envelope_bytes {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        Ok(envelope)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonSourceFetchOutputV1 {
    pub semantic_order_key: Vec<u8>,
    pub record: MailPersonSourceEnvelopeRecordV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonSourceAtomicFetchCommitV1 {
    pub logical_owner_id: String,
    pub account_public_id: [u8; 16],
    pub run_id: [u8; 16],
    pub page_sequence: u64,
    pub expected_provider_cursor: Option<Vec<u8>>,
    pub next_provider_cursor: Option<Vec<u8>>,
    /// Public workflow continuation may remain true after the provider cursor
    /// is exhausted while deterministic synthetic-removal pages are pending.
    pub public_has_more: bool,
    pub has_more: bool,
    pub command: MailPersonSourceEnvelopeRecordV1,
    pub processed_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonSourceAtomicFetchOutcomeV1 {
    pub replayed: bool,
    /// True only when an exact durable fetch replay belongs to a run whose
    /// terminal full-snapshot transaction has already committed. Consumers may
    /// acknowledge that delivery without attempting a second terminal plan.
    pub terminal_snapshot_succeeded: bool,
    pub processed_at_unix_millis: i64,
    pub changes: Vec<MailPersonSourceObservationOutcomeV1>,
    pub outputs: Vec<MailPersonSourceFetchOutputV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonSourcePendingOutboxV1 {
    pub record: MailPersonSourceEnvelopeRecordV1,
    pub semantic_order_key: Vec<u8>,
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonSourceLifecycleOutboxV1 {
    pub record: MailPersonSourceEnvelopeRecordV1,
    pub account_public_id: [u8; 16],
    pub mapping_revision: u64,
    pub retired: bool,
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MailPersonSourceRemovalStateV1 {
    pub integration_public_id: [u8; 16],
    pub account_public_id: [u8; 16],
    pub provider_source_contact_public_id: [u8; 16],
    pub source_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonSourceRemovalPageCommitV1 {
    pub page_sequence: u64,
    pub source_ids: Vec<[u8; 16]>,
    pub outputs: Vec<MailPersonSourceFetchOutputV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonSourceSnapshotCommitV1 {
    pub logical_owner_id: String,
    pub account_public_id: [u8; 16],
    pub run_id: [u8; 16],
    pub seen_public_source_ids: Vec<[u8; 16]>,
    pub expected_removals: Vec<MailPersonSourceRemovalStateV1>,
    pub removal_pages: Vec<MailPersonSourceRemovalPageCommitV1>,
    pub terminal_command: MailPersonSourceEnvelopeRecordV1,
    pub completed_at_unix_millis: i64,
}

/// Returns the canonical run-global ordering key for one public source output.
///
/// Page sequence is encoded before the one-based ordinal, so independently
/// committed pages cannot collide and pending delivery remains page ordered.
pub fn mail_person_source_semantic_order_key_v1(
    page_sequence: u64,
    ordinal: u16,
) -> Result<Vec<u8>, MailAddressBookPersistenceErrorV1> {
    if !(1..=4_096).contains(&page_sequence) || !(1..=501).contains(&ordinal) {
        return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
    }
    let mut key = Vec::with_capacity(10);
    key.extend_from_slice(&page_sequence.to_be_bytes());
    key.extend_from_slice(&ordinal.to_be_bytes());
    Ok(key)
}

impl MailAddressBookPersistenceV1 {
    pub async fn load_person_source_account_lifecycle_record(
        &self,
        logical_owner_id: &str,
        mapping: &MailPersonSourceAccountMappingV1,
        retired: bool,
    ) -> Result<Option<MailPersonSourceEnvelopeRecordV1>, MailAddressBookPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        if !nonzero(&mapping.account_public_id)
            || !nonzero(&mapping.integration_public_id)
            || mapping.mapping_revision == 0
        {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT message_id,envelope_sha256,envelope_bytes FROM \
             makosh_data.mail_address_book_person_source_lifecycle_outbox \
             WHERE logical_owner_id=$1 AND account_public_id=$2 AND mapping_revision=$3 AND semantic_kind=$4",
        )
        .bind(logical_owner_id)
        .bind(mapping.account_public_id.as_slice())
        .bind(
            i64::try_from(mapping.mapping_revision)
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?,
        )
        .bind(if retired { 2_i16 } else { 1_i16 })
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?;
        let record = row
            .map(|row| {
                Ok(MailPersonSourceEnvelopeRecordV1 {
                    message_id: bytes::<16>(&row, "message_id")?,
                    envelope_sha256: bytes::<32>(&row, "envelope_sha256")?,
                    envelope_bytes: row.try_get("envelope_bytes").map_err(storage)?,
                })
            })
            .transpose()?;
        if let Some(record) = &record {
            record.validate()?;
        }
        transaction.rollback().await.map_err(storage)?;
        Ok(record)
    }

    pub async fn record_person_source_account_lifecycle_once(
        &self,
        logical_owner_id: &str,
        mapping: MailPersonSourceAccountMappingV1,
        retired: bool,
        record: MailPersonSourceEnvelopeRecordV1,
        created_at_unix_millis: i64,
    ) -> Result<(), MailAddressBookPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        record.validate()?;
        if !nonzero(&mapping.account_public_id)
            || !nonzero(&mapping.integration_public_id)
            || mapping.mapping_revision == 0
            || created_at_unix_millis <= 0
        {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let exists = sqlx::query(
            "SELECT integration_public_id,mapping_revision FROM makosh_data.mail_address_book_person_source_accounts \
             WHERE logical_owner_id=$1 AND account_public_id=$2 FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(mapping.account_public_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?
        .ok_or(MailAddressBookPersistenceErrorV1::NotFound)?;
        if bytes::<16>(&exists, "integration_public_id")? != mapping.integration_public_id
            || exists
                .try_get::<i64, _>("mapping_revision")
                .map_err(storage)?
                != i64::try_from(mapping.mapping_revision)
                    .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?
        {
            return Err(MailAddressBookPersistenceErrorV1::Conflict);
        }
        sqlx::query(
            "INSERT INTO makosh_data.mail_address_book_person_source_lifecycle_outbox \
             (logical_owner_id,message_id,envelope_sha256,envelope_bytes,account_public_id,mapping_revision,semantic_kind,created_at_unix_millis) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT (logical_owner_id,account_public_id,mapping_revision,semantic_kind) DO NOTHING",
        )
        .bind(logical_owner_id)
        .bind(record.message_id.as_slice())
        .bind(record.envelope_sha256.as_slice())
        .bind(&record.envelope_bytes)
        .bind(mapping.account_public_id.as_slice())
        .bind(i64::try_from(mapping.mapping_revision).map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?)
        .bind(if retired { 2_i16 } else { 1_i16 })
        .bind(created_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        let stored = sqlx::query(
            "SELECT message_id,envelope_sha256,envelope_bytes FROM makosh_data.mail_address_book_person_source_lifecycle_outbox \
             WHERE logical_owner_id=$1 AND account_public_id=$2 AND mapping_revision=$3 AND semantic_kind=$4",
        )
        .bind(logical_owner_id)
        .bind(mapping.account_public_id.as_slice())
        .bind(i64::try_from(mapping.mapping_revision).map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?)
        .bind(if retired { 2_i16 } else { 1_i16 })
        .fetch_one(&mut *transaction)
        .await
        .map_err(storage)?;
        let replay = MailPersonSourceEnvelopeRecordV1 {
            message_id: bytes::<16>(&stored, "message_id")?,
            envelope_sha256: bytes::<32>(&stored, "envelope_sha256")?,
            envelope_bytes: stored.try_get("envelope_bytes").map_err(storage)?,
        };
        replay.validate()?;
        if replay != record {
            return Err(MailAddressBookPersistenceErrorV1::Conflict);
        }
        transaction.commit().await.map_err(storage)
    }

    pub async fn load_pending_person_source_lifecycle_outbox(
        &self,
        logical_owner_id: &str,
    ) -> Result<Option<MailPersonSourceLifecycleOutboxV1>, MailAddressBookPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT message_id,envelope_sha256,envelope_bytes,account_public_id,mapping_revision,semantic_kind,created_at_unix_millis \
             FROM makosh_data.mail_address_book_person_source_lifecycle_outbox \
             WHERE logical_owner_id=$1 AND published_at_unix_millis IS NULL ORDER BY outbox_sequence LIMIT 1",
        )
        .bind(logical_owner_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?;
        let result = row
            .map(|row| {
                let record = MailPersonSourceEnvelopeRecordV1 {
                    message_id: bytes::<16>(&row, "message_id")?,
                    envelope_sha256: bytes::<32>(&row, "envelope_sha256")?,
                    envelope_bytes: row.try_get("envelope_bytes").map_err(storage)?,
                };
                record.validate()?;
                let semantic_kind = row.try_get::<i16, _>("semantic_kind").map_err(storage)?;
                if !matches!(semantic_kind, 1 | 2) {
                    return Err(MailAddressBookPersistenceErrorV1::InvalidRow);
                }
                Ok(MailPersonSourceLifecycleOutboxV1 {
                    record,
                    account_public_id: bytes::<16>(&row, "account_public_id")?,
                    mapping_revision: row
                        .try_get::<i64, _>("mapping_revision")
                        .map_err(storage)?
                        .try_into()
                        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?,
                    retired: semantic_kind == 2,
                    created_at_unix_millis: row
                        .try_get("created_at_unix_millis")
                        .map_err(storage)?,
                })
            })
            .transpose()?;
        transaction.rollback().await.map_err(storage)?;
        Ok(result)
    }

    pub async fn mark_person_source_lifecycle_outbox_published(
        &self,
        logical_owner_id: &str,
        message_id: [u8; 16],
        expected_envelope_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), MailAddressBookPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT envelope_sha256,envelope_bytes,created_at_unix_millis,published_at_unix_millis \
             FROM makosh_data.mail_address_book_person_source_lifecycle_outbox \
             WHERE logical_owner_id=$1 AND message_id=$2 FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(message_id.as_slice())
        .fetch_one(&mut *transaction)
        .await
        .map_err(storage)?;
        let stored = MailPersonSourceEnvelopeRecordV1 {
            message_id,
            envelope_sha256: bytes::<32>(&row, "envelope_sha256")?,
            envelope_bytes: row.try_get("envelope_bytes").map_err(storage)?,
        };
        stored.validate()?;
        if stored.envelope_sha256 != expected_envelope_sha256
            || published_at_unix_millis
                < row
                    .try_get::<i64, _>("created_at_unix_millis")
                    .map_err(storage)?
        {
            return Err(MailAddressBookPersistenceErrorV1::HashMismatch);
        }
        if row
            .try_get::<Option<i64>, _>("published_at_unix_millis")
            .map_err(storage)?
            .is_none()
        {
            sqlx::query(
                "UPDATE makosh_data.mail_address_book_person_source_lifecycle_outbox SET published_at_unix_millis=$3 \
                 WHERE logical_owner_id=$1 AND message_id=$2 AND envelope_sha256=$4 AND published_at_unix_millis IS NULL",
            )
            .bind(logical_owner_id)
            .bind(message_id.as_slice())
            .bind(published_at_unix_millis)
            .bind(expected_envelope_sha256.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        }
        transaction.commit().await.map_err(storage)
    }

    pub async fn ensure_person_source_account_mapping(
        &self,
        logical_owner_id: &str,
        private_account_key: &str,
        proposed: MailPersonSourceAccountMappingV1,
        created_at_unix_millis: i64,
    ) -> Result<MailPersonSourceAccountMappingV1, MailAddressBookPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        if private_account_key.is_empty()
            || private_account_key.len() > 256
            || !nonzero(&proposed.integration_public_id)
            || !nonzero(&proposed.account_public_id)
            || proposed.integration_public_id == proposed.account_public_id
            || proposed.mapping_revision == 0
            || created_at_unix_millis <= 0
        {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, logical_owner_id).await?;
        sqlx::query(
            "INSERT INTO makosh_data.mail_address_book_person_source_accounts \
             (logical_owner_id,private_account_key,integration_public_id,account_public_id,mapping_revision,created_at_unix_millis) \
             VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT (logical_owner_id,private_account_key) DO NOTHING",
        )
        .bind(logical_owner_id)
        .bind(private_account_key)
        .bind(proposed.integration_public_id.as_slice())
        .bind(proposed.account_public_id.as_slice())
        .bind(i64::try_from(proposed.mapping_revision).map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?)
        .bind(created_at_unix_millis)
        .execute(&mut *transaction).await.map_err(storage)?;
        let row = sqlx::query(
            "SELECT integration_public_id,account_public_id,mapping_revision FROM \
             makosh_data.mail_address_book_person_source_accounts \
             WHERE logical_owner_id=$1 AND private_account_key=$2 FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(private_account_key)
        .fetch_one(&mut *transaction)
        .await
        .map_err(storage)?;
        let mapping = MailPersonSourceAccountMappingV1 {
            integration_public_id: bytes::<16>(&row, "integration_public_id")?,
            account_public_id: bytes::<16>(&row, "account_public_id")?,
            mapping_revision: row
                .try_get::<i64, _>("mapping_revision")
                .map_err(storage)?
                .try_into()
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?,
        };
        transaction.commit().await.map_err(storage)?;
        Ok(mapping)
    }

    pub async fn load_person_source_account_mapping(
        &self,
        logical_owner_id: &str,
        private_account_key: &str,
    ) -> Result<MailPersonSourceAccountMappingV1, MailAddressBookPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        if private_account_key.is_empty() || private_account_key.len() > 256 {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT integration_public_id,account_public_id,mapping_revision FROM \
             makosh_data.mail_address_book_person_source_accounts \
             WHERE logical_owner_id=$1 AND private_account_key=$2",
        )
        .bind(logical_owner_id)
        .bind(private_account_key)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?
        .ok_or(MailAddressBookPersistenceErrorV1::NotFound)?;
        let mapping = MailPersonSourceAccountMappingV1 {
            integration_public_id: bytes::<16>(&row, "integration_public_id")?,
            account_public_id: bytes::<16>(&row, "account_public_id")?,
            mapping_revision: row
                .try_get::<i64, _>("mapping_revision")
                .map_err(storage)?
                .try_into()
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?,
        };
        transaction.rollback().await.map_err(storage)?;
        Ok(mapping)
    }

    pub async fn load_person_source_account_binding_by_public_id(
        &self,
        logical_owner_id: &str,
        account_public_id: [u8; 16],
    ) -> Result<MailPersonSourceAccountBindingV1, MailAddressBookPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        if !nonzero(&account_public_id) {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT private_account_key,integration_public_id,mapping_revision FROM \
             makosh_data.mail_address_book_person_source_accounts \
             WHERE logical_owner_id=$1 AND account_public_id=$2",
        )
        .bind(logical_owner_id)
        .bind(account_public_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?
        .ok_or(MailAddressBookPersistenceErrorV1::NotFound)?;
        let result = MailPersonSourceAccountBindingV1 {
            private_account_key: row.try_get("private_account_key").map_err(storage)?,
            mapping: MailPersonSourceAccountMappingV1 {
                integration_public_id: bytes::<16>(&row, "integration_public_id")?,
                account_public_id,
                mapping_revision: row
                    .try_get::<i64, _>("mapping_revision")
                    .map_err(storage)?
                    .try_into()
                    .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?,
            },
        };
        transaction.rollback().await.map_err(storage)?;
        Ok(result)
    }

    pub async fn load_person_source_fetch_replay(
        &self,
        logical_owner_id: &str,
        account_public_id: [u8; 16],
        run_id: [u8; 16],
        page_sequence: u64,
        command: &MailPersonSourceEnvelopeRecordV1,
    ) -> Result<Option<MailPersonSourceAtomicFetchOutcomeV1>, MailAddressBookPersistenceErrorV1>
    {
        validate_owner(logical_owner_id)?;
        command.validate()?;
        if !nonzero(&account_public_id)
            || !nonzero(&run_id)
            || !(1..=4_096).contains(&page_sequence)
        {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT envelope_sha256,envelope_bytes,account_public_id,run_id,page_sequence,processed_at_unix_millis \
             FROM makosh_data.mail_address_book_person_source_fetch_inbox \
             WHERE logical_owner_id=$1 AND command_id=$2 FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(command.message_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?;
        let Some(row) = row else {
            transaction.rollback().await.map_err(storage)?;
            return Ok(None);
        };
        if bytes::<32>(&row, "envelope_sha256")? != command.envelope_sha256
            || row
                .try_get::<Vec<u8>, _>("envelope_bytes")
                .map_err(storage)?
                != command.envelope_bytes
            || bytes::<16>(&row, "account_public_id")? != account_public_id
            || bytes::<16>(&row, "run_id")? != run_id
            || row.try_get::<i64, _>("page_sequence").map_err(storage)?
                != i64::try_from(page_sequence)
                    .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?
        {
            return Err(MailAddressBookPersistenceErrorV1::Conflict);
        }
        let processed_at_unix_millis = row.try_get("processed_at_unix_millis").map_err(storage)?;
        let rows = sqlx::query(
            "SELECT message_id,envelope_sha256,envelope_bytes,semantic_order_key \
             FROM makosh_data.mail_address_book_person_source_fetch_outbox \
             WHERE logical_owner_id=$1 AND account_public_id=$2 AND run_id=$3 AND page_sequence=$4 \
             ORDER BY semantic_order_key,message_id",
        )
        .bind(logical_owner_id)
        .bind(account_public_id.as_slice())
        .bind(run_id.as_slice())
        .bind(
            i64::try_from(page_sequence)
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?,
        )
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage)?;
        let mut outputs = Vec::with_capacity(rows.len());
        for row in rows {
            let record = MailPersonSourceEnvelopeRecordV1 {
                message_id: bytes::<16>(&row, "message_id")?,
                envelope_sha256: bytes::<32>(&row, "envelope_sha256")?,
                envelope_bytes: row.try_get("envelope_bytes").map_err(storage)?,
            };
            record.validate()?;
            outputs.push(MailPersonSourceFetchOutputV1 {
                semantic_order_key: row.try_get("semantic_order_key").map_err(storage)?,
                record,
            });
        }
        let terminal = outputs
            .last()
            .ok_or(MailAddressBookPersistenceErrorV1::InvalidRow)?;
        let terminal_envelope = terminal.record.decode()?;
        let terminal_payload =
            MailPersonSourcePageCompletedV1::decode(terminal_envelope.payload.as_slice())
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?;
        let replay_input = MailPersonSourceAtomicFetchCommitV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            account_public_id,
            run_id,
            page_sequence,
            expected_provider_cursor: None,
            next_provider_cursor: None,
            public_has_more: terminal_payload.has_more,
            has_more: false,
            command: command.clone(),
            processed_at_unix_millis,
        };
        validate_replayed_fetch_outputs_v1(&replay_input, &outputs)?;
        let run = sqlx::query(
            "SELECT state,terminal_snapshot_succeeded FROM makosh_data.mail_address_book_person_source_runs \
             WHERE logical_owner_id=$1 AND account_public_id=$2 AND run_id=$3 FOR UPDATE",
        )
        .bind(logical_owner_id).bind(account_public_id.as_slice()).bind(run_id.as_slice())
        .fetch_one(&mut *transaction).await.map_err(storage)?;
        let state = run.try_get::<i16, _>("state").map_err(storage)?;
        let terminal_snapshot_succeeded = run
            .try_get::<bool, _>("terminal_snapshot_succeeded")
            .map_err(storage)?;
        if !(1..=3).contains(&state) || terminal_snapshot_succeeded != (state == 3) {
            return Err(MailAddressBookPersistenceErrorV1::InvalidRow);
        }
        transaction.rollback().await.map_err(storage)?;
        Ok(Some(MailPersonSourceAtomicFetchOutcomeV1 {
            replayed: true,
            terminal_snapshot_succeeded,
            processed_at_unix_millis,
            changes: Vec::new(),
            outputs,
        }))
    }

    /// Durably accepts a workflow FetchPage that names an already-materialized
    /// synthetic removal page. The terminal Mail run never re-enters provider
    /// I/O; exact redelivery is ACK-safe and altered bytes fail closed.
    pub async fn accept_person_source_synthetic_fetch_continuation_once(
        &self,
        logical_owner_id: &str,
        account_public_id: [u8; 16],
        run_id: [u8; 16],
        page_sequence: u64,
        command: &MailPersonSourceEnvelopeRecordV1,
        received_at_unix_millis: i64,
    ) -> Result<bool, MailAddressBookPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        command.validate()?;
        if !nonzero(&account_public_id)
            || !nonzero(&run_id)
            || !(1..=4_096).contains(&page_sequence)
            || received_at_unix_millis <= 0
        {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let command_input = MailPersonSourceAtomicFetchCommitV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            account_public_id,
            run_id,
            page_sequence,
            expected_provider_cursor: None,
            next_provider_cursor: None,
            public_has_more: false,
            has_more: false,
            command: command.clone(),
            processed_at_unix_millis: received_at_unix_millis,
        };
        validate_fetch_command_envelope_v1(&command_input)?;

        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let run = sqlx::query(
            "SELECT page_sequence,state,terminal_snapshot_succeeded FROM \
             makosh_data.mail_address_book_person_source_runs WHERE logical_owner_id=$1 \
             AND account_public_id=$2 AND run_id=$3 FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(account_public_id.as_slice())
        .bind(run_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?;
        let Some(run) = run else {
            transaction.rollback().await.map_err(storage)?;
            return Ok(false);
        };
        let provider_terminal_page = run
            .try_get::<i64, _>("page_sequence")
            .map_err(storage)?
            .try_into()
            .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?;
        let state = run.try_get::<i16, _>("state").map_err(storage)?;
        let terminal_snapshot_succeeded = run
            .try_get::<bool, _>("terminal_snapshot_succeeded")
            .map_err(storage)?;
        if state != 3 || !terminal_snapshot_succeeded || page_sequence <= provider_terminal_page {
            transaction.rollback().await.map_err(storage)?;
            return Ok(false);
        }

        let synthetic = sqlx::query(
            "SELECT message_id,envelope_sha256,envelope_bytes FROM \
             makosh_data.mail_address_book_person_source_fetch_outbox WHERE logical_owner_id=$1 \
             AND account_public_id=$2 AND run_id=$3 AND page_sequence=$4 \
             ORDER BY semantic_order_key DESC,message_id DESC LIMIT 1 FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(account_public_id.as_slice())
        .bind(run_id.as_slice())
        .bind(
            i64::try_from(page_sequence)
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?,
        )
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?;
        let Some(synthetic) = synthetic else {
            transaction.rollback().await.map_err(storage)?;
            return Ok(false);
        };
        let synthetic = MailPersonSourceEnvelopeRecordV1 {
            message_id: bytes::<16>(&synthetic, "message_id")?,
            envelope_sha256: bytes::<32>(&synthetic, "envelope_sha256")?,
            envelope_bytes: synthetic.try_get("envelope_bytes").map_err(storage)?,
        };
        validate_synthetic_fetch_continuation_page_v1(
            logical_owner_id,
            account_public_id,
            run_id,
            page_sequence,
            command.message_id,
            &synthetic,
        )?;
        let synthetic_request_sha256: [u8; 32] = Sha256::new()
            .chain_update(b"makosh.mail.person-source.synthetic-fetch-request.v1")
            .chain_update((command.envelope_bytes.len() as u64).to_be_bytes())
            .chain_update(&command.envelope_bytes)
            .finalize()
            .into();
        let synthetic_plan_sha256: [u8; 32] = Sha256::new()
            .chain_update(b"makosh.mail.person-source.synthetic-fetch-plan.v1")
            .chain_update((synthetic.envelope_bytes.len() as u64).to_be_bytes())
            .chain_update(&synthetic.envelope_bytes)
            .finalize()
            .into();

        if let Some(stored) = sqlx::query(
            "SELECT envelope_sha256,envelope_bytes,request_sha256,plan_sha256,account_public_id,run_id,page_sequence FROM \
             makosh_data.mail_address_book_person_source_fetch_inbox WHERE logical_owner_id=$1 \
             AND command_id=$2 FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(command.message_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?
        {
            let exact = bytes::<32>(&stored, "envelope_sha256")? == command.envelope_sha256
                && stored
                    .try_get::<Vec<u8>, _>("envelope_bytes")
                    .map_err(storage)?
                    == command.envelope_bytes
                && bytes::<32>(&stored, "request_sha256")? == synthetic_request_sha256
                && bytes::<32>(&stored, "plan_sha256")? == synthetic_plan_sha256
                && bytes::<16>(&stored, "account_public_id")? == account_public_id
                && bytes::<16>(&stored, "run_id")? == run_id
                && stored.try_get::<i64, _>("page_sequence").map_err(storage)?
                    == i64::try_from(page_sequence)
                        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
            transaction.commit().await.map_err(storage)?;
            return if exact {
                Ok(true)
            } else {
                Err(MailAddressBookPersistenceErrorV1::Conflict)
            };
        }

        sqlx::query(
            "INSERT INTO makosh_data.mail_address_book_person_source_fetch_inbox \
             (logical_owner_id,command_id,envelope_sha256,envelope_bytes,request_sha256,plan_sha256, \
              account_public_id,run_id,page_sequence,processed_at_unix_millis) \
              VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) \
              ON CONFLICT DO NOTHING",
        )
        .bind(logical_owner_id)
        .bind(command.message_id.as_slice())
        .bind(command.envelope_sha256.as_slice())
        .bind(&command.envelope_bytes)
        .bind(synthetic_request_sha256.as_slice())
        .bind(synthetic_plan_sha256.as_slice())
        .bind(account_public_id.as_slice())
        .bind(run_id.as_slice())
        .bind(
            i64::try_from(page_sequence)
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?,
        )
        .bind(received_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        let stored = sqlx::query(
            "SELECT command_id,envelope_sha256,envelope_bytes,request_sha256,plan_sha256,account_public_id,run_id,page_sequence \
             FROM makosh_data.mail_address_book_person_source_fetch_inbox WHERE logical_owner_id=$1 \
             AND account_public_id=$2 AND run_id=$3 AND page_sequence=$4 FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(account_public_id.as_slice())
        .bind(run_id.as_slice())
        .bind(
            i64::try_from(page_sequence)
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?,
        )
        .fetch_one(&mut *transaction)
        .await
        .map_err(storage)?;
        let exact = bytes::<16>(&stored, "command_id")? == command.message_id
            && bytes::<32>(&stored, "envelope_sha256")? == command.envelope_sha256
            && stored
                .try_get::<Vec<u8>, _>("envelope_bytes")
                .map_err(storage)?
                == command.envelope_bytes
            && bytes::<32>(&stored, "request_sha256")? == synthetic_request_sha256
            && bytes::<32>(&stored, "plan_sha256")? == synthetic_plan_sha256
            && bytes::<16>(&stored, "account_public_id")? == account_public_id
            && bytes::<16>(&stored, "run_id")? == run_id
            && stored.try_get::<i64, _>("page_sequence").map_err(storage)?
                == i64::try_from(page_sequence)
                    .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
        transaction.commit().await.map_err(storage)?;
        if exact {
            Ok(true)
        } else {
            Err(MailAddressBookPersistenceErrorV1::Conflict)
        }
    }

    pub async fn load_person_source_fetch_state(
        &self,
        logical_owner_id: &str,
        account_public_id: [u8; 16],
        run_id: [u8; 16],
        page_sequence: u64,
    ) -> Result<Option<MailPersonSourceFetchStateV1>, MailAddressBookPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        if !nonzero(&account_public_id)
            || !nonzero(&run_id)
            || !(1..=4_096).contains(&page_sequence)
        {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        if page_sequence == 1 {
            return Ok(None);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT provider_cursor,page_sequence,state FROM \
             makosh_data.mail_address_book_person_source_runs \
             WHERE logical_owner_id=$1 AND account_public_id=$2 AND run_id=$3",
        )
        .bind(logical_owner_id)
        .bind(account_public_id.as_slice())
        .bind(run_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?
        .ok_or(MailAddressBookPersistenceErrorV1::NotFound)?;
        if row.try_get::<i16, _>("state").map_err(storage)? != 1 {
            return Err(MailAddressBookPersistenceErrorV1::Conflict);
        }
        let stored_page: u64 = row
            .try_get::<i64, _>("page_sequence")
            .map_err(storage)?
            .try_into()
            .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?;
        if stored_page != page_sequence {
            return Err(MailAddressBookPersistenceErrorV1::Conflict);
        }
        let result = MailPersonSourceFetchStateV1 {
            provider_cursor: row.try_get("provider_cursor").map_err(storage)?,
            page_sequence: stored_page,
        };
        transaction.rollback().await.map_err(storage)?;
        Ok(Some(result))
    }

    pub async fn load_person_source_contact_public_id(
        &self,
        logical_owner_id: &str,
        account_public_id: [u8; 16],
        provider_record_key: &[u8],
    ) -> Result<Option<[u8; 16]>, MailAddressBookPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        if !nonzero(&account_public_id)
            || provider_record_key.is_empty()
            || provider_record_key.len() > 512
        {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let row = sqlx::query_scalar::<_, Vec<u8>>(
            "SELECT provider_source_contact_public_id FROM \
             makosh_data.mail_address_book_person_sources \
             WHERE logical_owner_id=$1 AND account_public_id=$2 AND provider_record_key=$3",
        )
        .bind(logical_owner_id)
        .bind(account_public_id.as_slice())
        .bind(provider_record_key)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?;
        let result = row
            .map(|value| {
                value
                    .try_into()
                    .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)
            })
            .transpose()?;
        transaction.rollback().await.map_err(storage)?;
        Ok(result)
    }

    #[allow(clippy::too_many_arguments)]
    #[cfg(feature = "conformance-test-support")]
    pub async fn ensure_person_source_contact_mapping(
        &self,
        logical_owner_id: &str,
        account_public_id: [u8; 16],
        provider_record_key: &[u8],
        proposed_source_public_id: [u8; 16],
        claims_digest: [u8; 32],
        source_revision: u64,
        updated_at_unix_millis: i64,
    ) -> Result<[u8; 16], MailAddressBookPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        if !nonzero(&account_public_id)
            || provider_record_key.is_empty()
            || provider_record_key.len() > 512
            || !nonzero(&proposed_source_public_id)
            || !nonzero(&claims_digest)
            || source_revision == 0
            || updated_at_unix_millis <= 0
        {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, logical_owner_id).await?;
        sqlx::query(
            "INSERT INTO makosh_data.mail_address_book_person_sources \
             (logical_owner_id,account_public_id,provider_record_key,provider_record_etag, \
              provider_source_contact_public_id,claims_digest,source_revision,active,last_terminal_run_id,updated_at_unix_millis) \
             VALUES ($1,$2,$3,NULL,$4,$5,$6,TRUE,NULL,$7) \
             ON CONFLICT (logical_owner_id,account_public_id,provider_record_key) DO NOTHING",
        ).bind(logical_owner_id).bind(account_public_id.as_slice()).bind(provider_record_key)
            .bind(proposed_source_public_id.as_slice()).bind(claims_digest.as_slice())
            .bind(i64::try_from(source_revision).map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?)
            .bind(updated_at_unix_millis).execute(&mut *transaction).await.map_err(storage)?;
        let source_id = sqlx::query_scalar::<_,Vec<u8>>(
            "SELECT provider_source_contact_public_id FROM makosh_data.mail_address_book_person_sources \
             WHERE logical_owner_id=$1 AND account_public_id=$2 AND provider_record_key=$3 FOR UPDATE",
        ).bind(logical_owner_id).bind(account_public_id.as_slice()).bind(provider_record_key)
            .fetch_one(&mut *transaction).await.map_err(storage)?
            .try_into().map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?;
        transaction.commit().await.map_err(storage)?;
        Ok(source_id)
    }

    #[cfg(feature = "conformance-test-support")]
    pub async fn observe_person_source_contact(
        &self,
        input: &MailPersonSourceObservationV1,
    ) -> Result<MailPersonSourceObservationOutcomeV1, MailAddressBookPersistenceErrorV1> {
        validate_owner(&input.logical_owner_id)?;
        if !nonzero(&input.account_public_id)
            || input.provider_record_key.is_empty()
            || input.provider_record_key.len() > 512
            || input
                .provider_record_etag
                .as_ref()
                .is_some_and(|value| value.is_empty() || value.len() > 512)
            || !nonzero(&input.proposed_source_public_id)
            || !nonzero(&input.claims_digest)
            || input.observed_at_unix_millis <= 0
        {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, &input.logical_owner_id).await?;
        let existing = sqlx::query(
            "SELECT provider_source_contact_public_id,claims_digest,source_revision,active,updated_at_unix_millis \
             FROM makosh_data.mail_address_book_person_sources \
             WHERE logical_owner_id=$1 AND account_public_id=$2 AND provider_record_key=$3 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.account_public_id.as_slice())
        .bind(&input.provider_record_key)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?;
        let outcome = if let Some(row) = existing {
            let public_id = bytes::<16>(&row, "provider_source_contact_public_id")?;
            let stored_digest = bytes::<32>(&row, "claims_digest")?;
            let revision = row
                .try_get::<i64, _>("source_revision")
                .map_err(storage)?
                .try_into()
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?;
            let active = row.try_get::<bool, _>("active").map_err(storage)?;
            let updated_at = row
                .try_get::<i64, _>("updated_at_unix_millis")
                .map_err(storage)?;
            if input.observed_at_unix_millis < updated_at {
                return Err(MailAddressBookPersistenceErrorV1::Conflict);
            }
            if active && stored_digest == input.claims_digest {
                sqlx::query(
                    "UPDATE makosh_data.mail_address_book_person_sources SET provider_record_etag=$4,updated_at_unix_millis=$5 \
                     WHERE logical_owner_id=$1 AND account_public_id=$2 AND provider_record_key=$3",
                )
                .bind(&input.logical_owner_id)
                .bind(input.account_public_id.as_slice())
                .bind(&input.provider_record_key)
                .bind(input.provider_record_etag.as_deref())
                .bind(input.observed_at_unix_millis)
                .execute(&mut *transaction)
                .await
                .map_err(storage)?;
                MailPersonSourceObservationOutcomeV1 {
                    provider_source_contact_public_id: public_id,
                    source_revision: revision,
                    change_kind: MailPersonSourceChangeKindV1::Unchanged,
                }
            } else {
                let next_revision = revision
                    .checked_add(1)
                    .ok_or(MailAddressBookPersistenceErrorV1::Conflict)?;
                sqlx::query(
                    "UPDATE makosh_data.mail_address_book_person_sources SET provider_record_etag=$4,claims_digest=$5,source_revision=$6,active=TRUE,last_terminal_run_id=NULL,updated_at_unix_millis=$7 \
                     WHERE logical_owner_id=$1 AND account_public_id=$2 AND provider_record_key=$3",
                )
                .bind(&input.logical_owner_id)
                .bind(input.account_public_id.as_slice())
                .bind(&input.provider_record_key)
                .bind(input.provider_record_etag.as_deref())
                .bind(input.claims_digest.as_slice())
                .bind(i64::try_from(next_revision).map_err(|_| MailAddressBookPersistenceErrorV1::Conflict)?)
                .bind(input.observed_at_unix_millis)
                .execute(&mut *transaction)
                .await
                .map_err(storage)?;
                MailPersonSourceObservationOutcomeV1 {
                    provider_source_contact_public_id: public_id,
                    source_revision: next_revision,
                    change_kind: if active {
                        MailPersonSourceChangeKindV1::Updated
                    } else {
                        MailPersonSourceChangeKindV1::Observed
                    },
                }
            }
        } else {
            sqlx::query(
                "INSERT INTO makosh_data.mail_address_book_person_sources \
                 (logical_owner_id,account_public_id,provider_record_key,provider_record_etag,provider_source_contact_public_id,claims_digest,source_revision,active,last_terminal_run_id,updated_at_unix_millis) \
                 VALUES ($1,$2,$3,$4,$5,$6,1,TRUE,NULL,$7)",
            )
            .bind(&input.logical_owner_id)
            .bind(input.account_public_id.as_slice())
            .bind(&input.provider_record_key)
            .bind(input.provider_record_etag.as_deref())
            .bind(input.proposed_source_public_id.as_slice())
            .bind(input.claims_digest.as_slice())
            .bind(input.observed_at_unix_millis)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
            MailPersonSourceObservationOutcomeV1 {
                provider_source_contact_public_id: input.proposed_source_public_id,
                source_revision: 1,
                change_kind: MailPersonSourceChangeKindV1::Observed,
            }
        };
        transaction.commit().await.map_err(storage)?;
        Ok(outcome)
    }

    pub async fn commit_person_source_fetch_atomically_once<P, F>(
        &self,
        input: &MailPersonSourceAtomicFetchCommitV1,
        prepare_observations: P,
        build_outputs: F,
    ) -> Result<MailPersonSourceAtomicFetchOutcomeV1, MailAddressBookPersistenceErrorV1>
    where
        P: FnOnce()
            -> Result<Vec<MailPersonSourceObservationV1>, MailAddressBookPersistenceErrorV1>,
        F: FnOnce(
            &[MailPersonSourceObservationOutcomeV1],
        )
            -> Result<Vec<MailPersonSourceFetchOutputV1>, MailAddressBookPersistenceErrorV1>,
    {
        let request_sha256 = validate_atomic_fetch_request(input)?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, &input.logical_owner_id).await?;

        let existing = sqlx::query(
            "SELECT envelope_sha256,envelope_bytes,request_sha256,account_public_id,run_id,page_sequence,processed_at_unix_millis \
             FROM makosh_data.mail_address_book_person_source_fetch_inbox \
             WHERE logical_owner_id=$1 AND command_id=$2 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.command.message_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?;
        if let Some(row) = existing {
            let exact = bytes::<32>(&row, "envelope_sha256")? == input.command.envelope_sha256
                && row
                    .try_get::<Vec<u8>, _>("envelope_bytes")
                    .map_err(storage)?
                    == input.command.envelope_bytes
                && bytes::<32>(&row, "request_sha256")? == request_sha256
                && bytes::<16>(&row, "account_public_id")? == input.account_public_id
                && bytes::<16>(&row, "run_id")? == input.run_id
                && row.try_get::<i64, _>("page_sequence").map_err(storage)?
                    == i64::try_from(input.page_sequence)
                        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
            if !exact {
                return Err(MailAddressBookPersistenceErrorV1::Conflict);
            }
            let mut replay_input = input.clone();
            replay_input.processed_at_unix_millis =
                row.try_get("processed_at_unix_millis").map_err(storage)?;
            let outputs = load_atomic_fetch_outputs(&mut transaction, &replay_input).await?;
            let terminal_snapshot_succeeded = sqlx::query(
                "SELECT state,terminal_snapshot_succeeded FROM makosh_data.mail_address_book_person_source_runs \
                 WHERE logical_owner_id=$1 AND account_public_id=$2 AND run_id=$3 FOR UPDATE",
            )
            .bind(&input.logical_owner_id)
            .bind(input.account_public_id.as_slice())
            .bind(input.run_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(storage)?
            .ok_or(MailAddressBookPersistenceErrorV1::InvalidRow)?;
            let run_state = terminal_snapshot_succeeded
                .try_get::<i16, _>("state")
                .map_err(storage)?;
            let snapshot_succeeded = terminal_snapshot_succeeded
                .try_get::<bool, _>("terminal_snapshot_succeeded")
                .map_err(storage)?;
            if !(1..=3).contains(&run_state) || snapshot_succeeded != (run_state == 3) {
                return Err(MailAddressBookPersistenceErrorV1::InvalidRow);
            }
            transaction.rollback().await.map_err(storage)?;
            return Ok(MailPersonSourceAtomicFetchOutcomeV1 {
                replayed: true,
                terminal_snapshot_succeeded: snapshot_succeeded,
                processed_at_unix_millis: replay_input.processed_at_unix_millis,
                changes: Vec::new(),
                outputs,
            });
        }

        validate_fetch_command_freshness_v1(input)?;

        if input.page_sequence == 1 && input.expected_provider_cursor.is_none() {
            sqlx::query(
                "INSERT INTO makosh_data.mail_address_book_person_source_runs \
                 (logical_owner_id,account_public_id,run_id,provider_snapshot_generation,provider_cursor,page_sequence,state,terminal_snapshot_succeeded,created_at_unix_millis,updated_at_unix_millis) \
                 VALUES ($1,$2,$3,$3,NULL,1,1,FALSE,$4,$4) \
                 ON CONFLICT (logical_owner_id,account_public_id,run_id) DO NOTHING",
            )
            .bind(&input.logical_owner_id)
            .bind(input.account_public_id.as_slice())
            .bind(input.run_id.as_slice())
            .bind(input.processed_at_unix_millis)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        }
        let run = sqlx::query(
            "SELECT provider_cursor,page_sequence,state,updated_at_unix_millis \
             FROM makosh_data.mail_address_book_person_source_runs \
             WHERE logical_owner_id=$1 AND account_public_id=$2 AND run_id=$3 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.account_public_id.as_slice())
        .bind(input.run_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?
        .ok_or(MailAddressBookPersistenceErrorV1::Conflict)?;
        let stored_cursor = run
            .try_get::<Option<Vec<u8>>, _>("provider_cursor")
            .map_err(storage)?;
        if run.try_get::<i16, _>("state").map_err(storage)? != 1
            || run.try_get::<i64, _>("page_sequence").map_err(storage)?
                != i64::try_from(input.page_sequence)
                    .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?
            || stored_cursor != input.expected_provider_cursor
            || input.processed_at_unix_millis
                < run
                    .try_get::<i64, _>("updated_at_unix_millis")
                    .map_err(storage)?
        {
            return Err(MailAddressBookPersistenceErrorV1::Conflict);
        }

        let observations = prepare_observations()?;
        let plan_sha256 = validate_atomic_fetch_plan(input, &observations)?;

        let mut changes = Vec::with_capacity(observations.len());
        for observation in &observations {
            let outcome =
                observe_person_source_contact_in_transaction(&mut transaction, observation).await?;
            sqlx::query(
                "INSERT INTO makosh_data.mail_address_book_person_source_seen \
                 (logical_owner_id,account_public_id,run_id,provider_source_contact_public_id) \
                 VALUES ($1,$2,$3,$4) ON CONFLICT DO NOTHING",
            )
            .bind(&input.logical_owner_id)
            .bind(input.account_public_id.as_slice())
            .bind(input.run_id.as_slice())
            .bind(outcome.provider_source_contact_public_id.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
            changes.push(outcome);
        }
        let outputs = build_outputs(&changes)?;
        let integration_public_id = sqlx::query_scalar::<_, Vec<u8>>(
            "SELECT integration_public_id FROM makosh_data.mail_address_book_person_source_accounts \
             WHERE logical_owner_id=$1 AND account_public_id=$2",
        )
        .bind(&input.logical_owner_id)
        .bind(input.account_public_id.as_slice())
        .fetch_one(&mut *transaction)
        .await
        .map_err(storage)?
        .try_into()
        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?;
        validate_fetch_output_envelopes_v1(
            input,
            integration_public_id,
            &observations,
            &changes,
            &outputs,
        )?;

        sqlx::query(
            "INSERT INTO makosh_data.mail_address_book_person_source_fetch_inbox \
             (logical_owner_id,command_id,envelope_sha256,envelope_bytes,request_sha256,plan_sha256,account_public_id,run_id,page_sequence,processed_at_unix_millis) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.command.message_id.as_slice())
        .bind(input.command.envelope_sha256.as_slice())
        .bind(&input.command.envelope_bytes)
        .bind(request_sha256.as_slice())
        .bind(plan_sha256.as_slice())
        .bind(input.account_public_id.as_slice())
        .bind(input.run_id.as_slice())
        .bind(
            i64::try_from(input.page_sequence)
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?,
        )
        .bind(input.processed_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage)?;
        for output in &outputs {
            sqlx::query(
                "INSERT INTO makosh_data.mail_address_book_person_source_fetch_outbox \
                 (logical_owner_id,message_id,envelope_sha256,envelope_bytes,account_public_id,run_id,page_sequence,semantic_order_key,created_at_unix_millis,published_at_unix_millis) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,NULL)",
            )
            .bind(&input.logical_owner_id)
            .bind(output.record.message_id.as_slice())
            .bind(output.record.envelope_sha256.as_slice())
            .bind(&output.record.envelope_bytes)
            .bind(input.account_public_id.as_slice())
            .bind(input.run_id.as_slice())
            .bind(
                i64::try_from(input.page_sequence)
                    .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?,
            )
            .bind(&output.semantic_order_key)
            .bind(input.processed_at_unix_millis)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        }
        let next_page_sequence = if input.has_more {
            input
                .page_sequence
                .checked_add(1)
                .ok_or(MailAddressBookPersistenceErrorV1::Conflict)?
        } else {
            input.page_sequence
        };
        let next_state = if input.has_more { 1_i16 } else { 2_i16 };
        let advanced = sqlx::query(
            "UPDATE makosh_data.mail_address_book_person_source_runs \
             SET provider_cursor=$4,page_sequence=$5,state=$6,updated_at_unix_millis=$7 \
             WHERE logical_owner_id=$1 AND account_public_id=$2 AND run_id=$3 \
             AND state=1 AND page_sequence=$8 AND provider_cursor IS NOT DISTINCT FROM $9",
        )
        .bind(&input.logical_owner_id)
        .bind(input.account_public_id.as_slice())
        .bind(input.run_id.as_slice())
        .bind(input.next_provider_cursor.as_deref())
        .bind(
            i64::try_from(next_page_sequence)
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?,
        )
        .bind(next_state)
        .bind(input.processed_at_unix_millis)
        .bind(
            i64::try_from(input.page_sequence)
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?,
        )
        .bind(input.expected_provider_cursor.as_deref())
        .execute(&mut *transaction)
        .await
        .map_err(storage)?
        .rows_affected();
        if advanced != 1 {
            return Err(MailAddressBookPersistenceErrorV1::Conflict);
        }
        transaction.commit().await.map_err(storage)?;
        Ok(MailPersonSourceAtomicFetchOutcomeV1 {
            replayed: false,
            terminal_snapshot_succeeded: false,
            processed_at_unix_millis: input.processed_at_unix_millis,
            changes,
            outputs,
        })
    }

    pub async fn load_pending_person_source_fetch_outbox(
        &self,
        logical_owner_id: &str,
    ) -> Result<Vec<MailPersonSourcePendingOutboxV1>, MailAddressBookPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let rows = sqlx::query(
            "SELECT message_id,envelope_sha256,envelope_bytes,semantic_order_key,created_at_unix_millis \
             FROM makosh_data.mail_address_book_person_source_fetch_outbox \
             WHERE logical_owner_id=$1 AND published_at_unix_millis IS NULL \
             ORDER BY account_public_id,run_id,page_sequence,semantic_order_key,message_id LIMIT 512",
        )
        .bind(logical_owner_id)
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage)?;
        let mut pending = Vec::with_capacity(rows.len());
        for row in rows {
            let record = MailPersonSourceEnvelopeRecordV1 {
                message_id: bytes::<16>(&row, "message_id")?,
                envelope_sha256: bytes::<32>(&row, "envelope_sha256")?,
                envelope_bytes: row.try_get("envelope_bytes").map_err(storage)?,
            };
            record
                .validate()
                .map_err(|_| MailAddressBookPersistenceErrorV1::HashMismatch)?;
            pending.push(MailPersonSourcePendingOutboxV1 {
                record,
                semantic_order_key: row.try_get("semantic_order_key").map_err(storage)?,
                created_at_unix_millis: row.try_get("created_at_unix_millis").map_err(storage)?,
            });
        }
        transaction.rollback().await.map_err(storage)?;
        Ok(pending)
    }

    pub async fn mark_person_source_fetch_outbox_published(
        &self,
        logical_owner_id: &str,
        message_id: [u8; 16],
        expected_envelope_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), MailAddressBookPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        if !nonzero(&message_id)
            || !nonzero(&expected_envelope_sha256)
            || published_at_unix_millis <= 0
        {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT envelope_sha256,envelope_bytes,created_at_unix_millis,published_at_unix_millis \
             FROM makosh_data.mail_address_book_person_source_fetch_outbox \
             WHERE logical_owner_id=$1 AND message_id=$2 FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(message_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?
        .ok_or(MailAddressBookPersistenceErrorV1::NotFound)?;
        let stored_sha = bytes::<32>(&row, "envelope_sha256")?;
        let stored_bytes: Vec<u8> = row.try_get("envelope_bytes").map_err(storage)?;
        let actual_sha256: [u8; 32] = Sha256::digest(&stored_bytes).into();
        if stored_sha != expected_envelope_sha256
            || actual_sha256 != stored_sha
            || published_at_unix_millis
                < row
                    .try_get::<i64, _>("created_at_unix_millis")
                    .map_err(storage)?
        {
            return Err(MailAddressBookPersistenceErrorV1::HashMismatch);
        }
        if row
            .try_get::<Option<i64>, _>("published_at_unix_millis")
            .map_err(storage)?
            .is_none()
        {
            sqlx::query(
                "UPDATE makosh_data.mail_address_book_person_source_fetch_outbox SET published_at_unix_millis=$3 \
                 WHERE logical_owner_id=$1 AND message_id=$2 AND published_at_unix_millis IS NULL",
            )
            .bind(logical_owner_id)
            .bind(message_id.as_slice())
            .bind(published_at_unix_millis)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?;
        }
        transaction.commit().await.map_err(storage)
    }

    pub async fn preview_person_source_removals(
        &self,
        logical_owner_id: &str,
        account_public_id: [u8; 16],
        seen_public_source_ids: &[[u8; 16]],
    ) -> Result<Vec<MailPersonSourceRemovalStateV1>, MailAddressBookPersistenceErrorV1> {
        validate_snapshot_identity(logical_owner_id, account_public_id, seen_public_source_ids)?;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let removals = select_person_source_removals(
            &mut transaction,
            logical_owner_id,
            account_public_id,
            seen_public_source_ids,
        )
        .await?;
        transaction.rollback().await.map_err(storage)?;
        Ok(removals)
    }

    pub async fn load_person_source_run_seen_ids(
        &self,
        logical_owner_id: &str,
        account_public_id: [u8; 16],
        run_id: [u8; 16],
    ) -> Result<Vec<[u8; 16]>, MailAddressBookPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        if !nonzero(&account_public_id) || !nonzero(&run_id) {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let rows = sqlx::query_scalar::<_, Vec<u8>>(
            "SELECT provider_source_contact_public_id FROM \
             makosh_data.mail_address_book_person_source_seen \
             WHERE logical_owner_id=$1 AND account_public_id=$2 AND run_id=$3 \
             ORDER BY provider_source_contact_public_id",
        )
        .bind(logical_owner_id)
        .bind(account_public_id.as_slice())
        .bind(run_id.as_slice())
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage)?;
        let ids = rows
            .into_iter()
            .map(|value| {
                value
                    .try_into()
                    .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)
            })
            .collect::<Result<Vec<[u8; 16]>, _>>()?;
        transaction.rollback().await.map_err(storage)?;
        Ok(ids)
    }

    pub async fn commit_person_source_snapshot_once(
        &self,
        input: &MailPersonSourceSnapshotCommitV1,
    ) -> Result<Vec<MailPersonSourceRemovalStateV1>, MailAddressBookPersistenceErrorV1> {
        let validation = validate_snapshot_commit(input)?;
        let plan_sha256 = validation.plan_sha256;
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, &input.logical_owner_id).await?;
        let run = sqlx::query(
            "SELECT state,page_sequence,updated_at_unix_millis,terminal_command_id,terminal_envelope_sha256, \
             terminal_envelope_bytes,terminal_fingerprint,terminal_plan_sha256 \
             FROM makosh_data.mail_address_book_person_source_runs \
             WHERE logical_owner_id=$1 AND account_public_id=$2 AND run_id=$3 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.account_public_id.as_slice())
        .bind(input.run_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?
        .ok_or(MailAddressBookPersistenceErrorV1::NotFound)?;
        let durable_seen = sqlx::query_scalar::<_, Vec<u8>>(
            "SELECT provider_source_contact_public_id FROM \
             makosh_data.mail_address_book_person_source_seen \
             WHERE logical_owner_id=$1 AND account_public_id=$2 AND run_id=$3 \
             ORDER BY provider_source_contact_public_id FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.account_public_id.as_slice())
        .bind(input.run_id.as_slice())
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage)?
        .into_iter()
        .map(|value| {
            value
                .try_into()
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)
        })
        .collect::<Result<Vec<[u8; 16]>, _>>()?;
        if durable_seen != input.seen_public_source_ids {
            return Err(MailAddressBookPersistenceErrorV1::Conflict);
        }
        let stored_terminal_row = sqlx::query(
            "SELECT message_id,envelope_sha256,envelope_bytes FROM \
             makosh_data.mail_address_book_person_source_fetch_outbox \
             WHERE logical_owner_id=$1 AND account_public_id=$2 AND run_id=$3 AND page_sequence=$4 \
             ORDER BY semantic_order_key DESC,message_id DESC LIMIT 1 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.account_public_id.as_slice())
        .bind(input.run_id.as_slice())
        .bind(
            i64::try_from(validation.terminal_page_sequence)
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?,
        )
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?
        .ok_or(MailAddressBookPersistenceErrorV1::Conflict)?;
        let stored_final_page_terminal = MailPersonSourceEnvelopeRecordV1 {
            message_id: bytes::<16>(&stored_terminal_row, "message_id")?,
            envelope_sha256: bytes::<32>(&stored_terminal_row, "envelope_sha256")?,
            envelope_bytes: stored_terminal_row
                .try_get("envelope_bytes")
                .map_err(storage)?,
        };
        stored_final_page_terminal.validate()?;
        if stored_final_page_terminal != input.terminal_command {
            return Err(MailAddressBookPersistenceErrorV1::Conflict);
        }
        let state = run.try_get::<i16, _>("state").map_err(storage)?;
        if state == 3 {
            let stored_terminal = MailPersonSourceEnvelopeRecordV1 {
                message_id: bytes::<16>(&run, "terminal_command_id")?,
                envelope_sha256: bytes::<32>(&run, "terminal_envelope_sha256")?,
                envelope_bytes: run.try_get("terminal_envelope_bytes").map_err(storage)?,
            };
            stored_terminal.validate()?;
            let exact = stored_terminal == input.terminal_command
                && bytes::<32>(&run, "terminal_fingerprint")? == validation.terminal_fingerprint
                && bytes::<32>(&run, "terminal_plan_sha256")? == plan_sha256
                && run.try_get::<i64, _>("page_sequence").map_err(storage)?
                    == i64::try_from(validation.terminal_page_sequence)
                        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
            if !exact {
                return Err(MailAddressBookPersistenceErrorV1::Conflict);
            }
            for page in &input.removal_pages {
                let rows = sqlx::query(
                    "SELECT message_id,envelope_sha256,envelope_bytes,semantic_order_key \
                     FROM makosh_data.mail_address_book_person_source_fetch_outbox \
                     WHERE logical_owner_id=$1 AND account_public_id=$2 AND run_id=$3 AND page_sequence=$4 \
                     ORDER BY semantic_order_key,message_id",
                )
                .bind(&input.logical_owner_id)
                .bind(input.account_public_id.as_slice())
                .bind(input.run_id.as_slice())
                .bind(
                    i64::try_from(page.page_sequence)
                        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?,
                )
                .fetch_all(&mut *transaction)
                .await
                .map_err(storage)?;
                if rows.len() != page.outputs.len() {
                    return Err(MailAddressBookPersistenceErrorV1::Conflict);
                }
                for (row, expected) in rows.iter().zip(&page.outputs) {
                    let stored = MailPersonSourceEnvelopeRecordV1 {
                        message_id: bytes::<16>(row, "message_id")?,
                        envelope_sha256: bytes::<32>(row, "envelope_sha256")?,
                        envelope_bytes: row.try_get("envelope_bytes").map_err(storage)?,
                    };
                    stored.validate()?;
                    if stored != expected.record
                        || row
                            .try_get::<Vec<u8>, _>("semantic_order_key")
                            .map_err(storage)?
                            != expected.semantic_order_key
                    {
                        return Err(MailAddressBookPersistenceErrorV1::Conflict);
                    }
                }
            }
            transaction.rollback().await.map_err(storage)?;
            return Ok(input.expected_removals.clone());
        }
        if state != 2
            || run.try_get::<i64, _>("page_sequence").map_err(storage)?
                != i64::try_from(validation.terminal_page_sequence)
                    .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?
            || input.completed_at_unix_millis
                < run
                    .try_get::<i64, _>("updated_at_unix_millis")
                    .map_err(storage)?
        {
            return Err(MailAddressBookPersistenceErrorV1::Conflict);
        }
        let actual_removals = select_person_source_removals(
            &mut transaction,
            &input.logical_owner_id,
            input.account_public_id,
            &durable_seen,
        )
        .await?;
        if actual_removals != input.expected_removals {
            return Err(MailAddressBookPersistenceErrorV1::Conflict);
        }
        for removal in &input.expected_removals {
            let prior_revision = removal
                .source_revision
                .checked_sub(1)
                .ok_or(MailAddressBookPersistenceErrorV1::InvalidInput)?;
            let affected = sqlx::query(
                "UPDATE makosh_data.mail_address_book_person_sources \
                 SET active=FALSE,source_revision=$4,last_terminal_run_id=$5,updated_at_unix_millis=$6 \
                 WHERE logical_owner_id=$1 AND account_public_id=$2 AND provider_source_contact_public_id=$3 \
                   AND active=TRUE AND source_revision=$7",
            )
            .bind(&input.logical_owner_id)
            .bind(input.account_public_id.as_slice())
            .bind(removal.provider_source_contact_public_id.as_slice())
            .bind(i64::try_from(removal.source_revision).map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?)
            .bind(input.run_id.as_slice())
            .bind(input.completed_at_unix_millis)
            .bind(i64::try_from(prior_revision).map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?)
            .execute(&mut *transaction)
            .await
            .map_err(storage)?
            .rows_affected();
            if affected != 1 {
                return Err(MailAddressBookPersistenceErrorV1::Conflict);
            }
        }
        for page in &input.removal_pages {
            for output in &page.outputs {
                sqlx::query(
                    "INSERT INTO makosh_data.mail_address_book_person_source_fetch_outbox \
                     (logical_owner_id,message_id,envelope_sha256,envelope_bytes,account_public_id,run_id,page_sequence,semantic_order_key,created_at_unix_millis,published_at_unix_millis) \
                     VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,NULL)",
                )
                .bind(&input.logical_owner_id)
                .bind(output.record.message_id.as_slice())
                .bind(output.record.envelope_sha256.as_slice())
                .bind(&output.record.envelope_bytes)
                .bind(input.account_public_id.as_slice())
                .bind(input.run_id.as_slice())
                .bind(i64::try_from(page.page_sequence).map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?)
                .bind(&output.semantic_order_key)
                .bind(input.completed_at_unix_millis)
                .execute(&mut *transaction)
                .await
                .map_err(storage)?;
            }
        }
        let terminalized = sqlx::query(
            "UPDATE makosh_data.mail_address_book_person_source_runs \
             SET state=3,terminal_snapshot_succeeded=TRUE,updated_at_unix_millis=$4, \
             terminal_command_id=$5,terminal_envelope_sha256=$6,terminal_envelope_bytes=$7, \
             terminal_fingerprint=$8,terminal_plan_sha256=$9 \
             WHERE logical_owner_id=$1 AND account_public_id=$2 AND run_id=$3 AND state=2",
        )
        .bind(&input.logical_owner_id)
        .bind(input.account_public_id.as_slice())
        .bind(input.run_id.as_slice())
        .bind(input.completed_at_unix_millis)
        .bind(input.terminal_command.message_id.as_slice())
        .bind(input.terminal_command.envelope_sha256.as_slice())
        .bind(&input.terminal_command.envelope_bytes)
        .bind(validation.terminal_fingerprint.as_slice())
        .bind(plan_sha256.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage)?
        .rows_affected();
        if terminalized != 1 {
            return Err(MailAddressBookPersistenceErrorV1::Conflict);
        }
        transaction.commit().await.map_err(storage)?;
        Ok(actual_removals)
    }

    pub async fn load_completed_person_source_removals(
        &self,
        logical_owner_id: &str,
        account_public_id: [u8; 16],
        run_id: [u8; 16],
    ) -> Result<Vec<MailPersonSourceRemovalStateV1>, MailAddressBookPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        if !nonzero(&account_public_id) || !nonzero(&run_id) {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage)?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let run = sqlx::query(
            "SELECT state,terminal_snapshot_succeeded FROM makosh_data.mail_address_book_person_source_runs \
             WHERE logical_owner_id=$1 AND account_public_id=$2 AND run_id=$3",
        )
        .bind(logical_owner_id)
        .bind(account_public_id.as_slice())
        .bind(run_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage)?
        .ok_or(MailAddressBookPersistenceErrorV1::NotFound)?;
        if run.try_get::<i16, _>("state").map_err(storage)? != 3
            || !run
                .try_get::<bool, _>("terminal_snapshot_succeeded")
                .map_err(storage)?
        {
            return Err(MailAddressBookPersistenceErrorV1::Conflict);
        }
        let rows = sqlx::query(
            "SELECT a.integration_public_id,s.provider_source_contact_public_id,s.source_revision \
             FROM makosh_data.mail_address_book_person_sources s \
             JOIN makosh_data.mail_address_book_person_source_accounts a \
               ON a.logical_owner_id=s.logical_owner_id AND a.account_public_id=s.account_public_id \
             WHERE s.logical_owner_id=$1 AND s.account_public_id=$2 AND s.last_terminal_run_id=$3 AND s.active=FALSE \
             ORDER BY s.provider_source_contact_public_id",
        )
        .bind(logical_owner_id)
        .bind(account_public_id.as_slice())
        .bind(run_id.as_slice())
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage)?;
        let mut removals = Vec::with_capacity(rows.len());
        for row in rows {
            removals.push(MailPersonSourceRemovalStateV1 {
                integration_public_id: bytes::<16>(&row, "integration_public_id")?,
                account_public_id,
                provider_source_contact_public_id: bytes::<16>(
                    &row,
                    "provider_source_contact_public_id",
                )?,
                source_revision: row
                    .try_get::<i64, _>("source_revision")
                    .map_err(storage)?
                    .try_into()
                    .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?,
            });
        }
        transaction.rollback().await.map_err(storage)?;
        Ok(removals)
    }
}

fn exact_person_source_contract(
    envelope: &DurableEnvelopeV1,
    expected: MailPersonSourceContractV1,
) -> bool {
    let Some(actual) = envelope.contract.as_ref() else {
        return false;
    };
    let expected = expected.reference();
    actual.owner == expected.owner
        && actual.name == expected.name
        && actual.major == expected.major
        && actual.revision == expected.revision
        && actual.schema_sha256 == expected.schema_sha256
}

fn exact_id16(value: &[u8]) -> Result<[u8; 16], MailAddressBookPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)
}

fn exact_module_authority_and_time_v1(
    envelope: &DurableEnvelopeV1,
    expected_module_id: &str,
    expected_seconds: i64,
    expected_nanos: i32,
) -> bool {
    let Some(source) = envelope.source.as_ref() else {
        return false;
    };
    let Some(actor) = envelope.actor.as_ref() else {
        return false;
    };
    let Some(fence) = envelope.source_fence.as_ref() else {
        return false;
    };
    source.module_id == expected_module_id
        && source.runtime_generation > 0
        && actor.kind == ActorKindV1::Module as i32
        && actor.actor_id == expected_module_id.as_bytes()
        && fence.kind == FenceKindV1::RuntimeLease as i32
        && fence.scope_id == expected_module_id.as_bytes()
        && fence.epoch == source.runtime_generation
        && envelope.recorded_at.as_ref().is_some_and(|recorded_at| {
            recorded_at.seconds == expected_seconds && recorded_at.nanos == expected_nanos
        })
}

fn exact_timestamp_unix_millis_v1(seconds: i64, nanos: i32, expected_unix_millis: i64) -> bool {
    timestamp_unix_millis_v1(seconds, nanos) == Some(expected_unix_millis)
}

fn timestamp_unix_millis_v1(seconds: i64, nanos: i32) -> Option<i64> {
    if !(0..1_000_000_000).contains(&nanos) || nanos % 1_000_000 != 0 {
        return None;
    }
    seconds
        .checked_mul(1_000)
        .and_then(|value| value.checked_add(i64::from(nanos / 1_000_000)))
}

fn validate_fetch_command_envelope_v1(
    input: &MailPersonSourceAtomicFetchCommitV1,
) -> Result<FetchMailPersonSourcePageCommandV1, MailAddressBookPersistenceErrorV1> {
    let envelope = input.command.decode()?;
    let payload = FetchMailPersonSourcePageCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
    validate_fetch_mail_person_source_page_v1(&payload)
        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
    let Some(Semantics::Command(command)) = envelope.semantics.as_ref() else {
        return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
    };
    let recorded_at = envelope
        .recorded_at
        .as_ref()
        .ok_or(MailAddressBookPersistenceErrorV1::InvalidInput)?;
    let source_module = "makosh-mail-persons-sync-runtime";
    let exact = payload.encode_to_vec() == envelope.payload
        && exact_person_source_contract(&envelope, MailPersonSourceContractV1::FetchPageCommand)
        && exact_id16(&payload.command_id)? == input.command.message_id
        && payload.command_id == command.command_id
        && command.target_capability == MAIL_PERSON_SOURCE_CAPABILITY_ID_V1
        && payload.logical_owner_id == input.logical_owner_id
        && exact_id16(&payload.account_public_id)? == input.account_public_id
        && exact_id16(&payload.run_id)? == input.run_id
        && payload.page_sequence == input.page_sequence
        && envelope.message_id == payload.command_id
        && envelope.partition_key == payload.run_id
        && envelope.correlation_id == payload.run_id
        && envelope.causation_message_id.is_empty()
        && exact_module_authority_and_time_v1(
            &envelope,
            source_module,
            recorded_at.seconds,
            recorded_at.nanos,
        );
    if exact {
        Ok(payload)
    } else {
        Err(MailAddressBookPersistenceErrorV1::InvalidInput)
    }
}

fn validate_fetch_command_freshness_v1(
    input: &MailPersonSourceAtomicFetchCommitV1,
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    let envelope = input.command.decode()?;
    let Some(Semantics::Command(command)) = envelope.semantics.as_ref() else {
        return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
    };
    let recorded_at = envelope
        .recorded_at
        .as_ref()
        .ok_or(MailAddressBookPersistenceErrorV1::InvalidInput)?;
    let deadline = command
        .deadline
        .as_ref()
        .ok_or(MailAddressBookPersistenceErrorV1::InvalidInput)?;
    let fresh = timestamp_unix_millis_v1(recorded_at.seconds, recorded_at.nanos)
        .is_some_and(|recorded| recorded <= input.processed_at_unix_millis)
        && timestamp_unix_millis_v1(deadline.seconds, deadline.nanos)
            .is_some_and(|deadline| input.processed_at_unix_millis < deadline);
    if fresh {
        Ok(())
    } else {
        Err(MailAddressBookPersistenceErrorV1::InvalidInput)
    }
}

fn validate_atomic_fetch_request(
    input: &MailPersonSourceAtomicFetchCommitV1,
) -> Result<[u8; 32], MailAddressBookPersistenceErrorV1> {
    validate_owner(&input.logical_owner_id)?;
    validate_fetch_command_envelope_v1(input)?;
    if !nonzero(&input.account_public_id)
        || !nonzero(&input.run_id)
        || !(1..=4_096).contains(&input.page_sequence)
        || input.processed_at_unix_millis <= 0
        || input
            .expected_provider_cursor
            .as_ref()
            .is_some_and(|cursor| cursor.is_empty() || cursor.len() > 4_096)
        || input
            .next_provider_cursor
            .as_ref()
            .is_some_and(|cursor| cursor.is_empty() || cursor.len() > 4_096)
        || input.has_more != input.next_provider_cursor.is_some()
        || (input.has_more && !input.public_has_more)
    {
        return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
    }
    let mut digest = Sha256::new();
    digest.update(b"makosh.mail.person-source.atomic-fetch-request.v1");
    digest.update((input.logical_owner_id.len() as u64).to_be_bytes());
    digest.update(input.logical_owner_id.as_bytes());
    digest.update(input.account_public_id);
    digest.update(input.run_id);
    digest.update(input.page_sequence.to_be_bytes());
    update_optional_bytes_digest(&mut digest, input.expected_provider_cursor.as_deref());
    update_optional_bytes_digest(&mut digest, input.next_provider_cursor.as_deref());
    digest.update([u8::from(input.public_has_more)]);
    digest.update([u8::from(input.has_more)]);
    digest.update(input.command.message_id);
    digest.update(input.command.envelope_sha256);
    Ok(digest.finalize().into())
}

fn validate_atomic_fetch_plan(
    input: &MailPersonSourceAtomicFetchCommitV1,
    observations: &[MailPersonSourceObservationV1],
) -> Result<[u8; 32], MailAddressBookPersistenceErrorV1> {
    if observations.len() > 500 {
        return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
    }
    let request_sha256 = validate_atomic_fetch_request(input)?;
    let mut provider_keys = BTreeSet::new();
    let mut proposed_ids = BTreeSet::new();
    let mut digest = Sha256::new();
    digest.update(b"makosh.mail.person-source.atomic-fetch-plan.v2");
    digest.update(request_sha256);
    digest.update((observations.len() as u64).to_be_bytes());
    for observation in observations {
        validate_observation(observation)?;
        if observation.logical_owner_id != input.logical_owner_id
            || observation.account_public_id != input.account_public_id
            || observation.observed_at_unix_millis > input.processed_at_unix_millis
            || !provider_keys.insert(observation.provider_record_key.clone())
            || !proposed_ids.insert(observation.proposed_source_public_id)
        {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        digest.update((observation.provider_record_key.len() as u64).to_be_bytes());
        digest.update(&observation.provider_record_key);
        match &observation.provider_record_etag {
            Some(value) => {
                digest.update([1]);
                digest.update((value.len() as u64).to_be_bytes());
                digest.update(value);
            }
            None => digest.update([0]),
        }
        digest.update(observation.proposed_source_public_id);
        digest.update(observation.claims_digest);
        digest.update(observation.observed_at_unix_millis.to_be_bytes());
    }
    Ok(digest.finalize().into())
}

fn update_optional_bytes_digest(digest: &mut Sha256, value: Option<&[u8]>) {
    match value {
        Some(value) => {
            digest.update([1]);
            digest.update((value.len() as u64).to_be_bytes());
            digest.update(value);
        }
        None => digest.update([0]),
    }
}

fn validate_observation(
    input: &MailPersonSourceObservationV1,
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    validate_owner(&input.logical_owner_id)?;
    if !nonzero(&input.account_public_id)
        || input.provider_record_key.is_empty()
        || input.provider_record_key.len() > 512
        || input
            .provider_record_etag
            .as_ref()
            .is_some_and(|value| value.is_empty() || value.len() > 512)
        || !nonzero(&input.proposed_source_public_id)
        || !nonzero(&input.claims_digest)
        || input.observed_at_unix_millis <= 0
    {
        Err(MailAddressBookPersistenceErrorV1::InvalidInput)
    } else {
        Ok(())
    }
}

fn validate_fetch_output_envelopes_v1(
    input: &MailPersonSourceAtomicFetchCommitV1,
    integration_public_id: [u8; 16],
    observations: &[MailPersonSourceObservationV1],
    changes: &[MailPersonSourceObservationOutcomeV1],
    outputs: &[MailPersonSourceFetchOutputV1],
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    if observations.len() != changes.len() || outputs.is_empty() || outputs.len() > 501 {
        return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
    }
    let changed = observations
        .iter()
        .zip(changes)
        .filter(|(_, change)| change.change_kind != MailPersonSourceChangeKindV1::Unchanged)
        .collect::<Vec<_>>();
    if outputs.len() != changed.len() + 1 {
        return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
    }
    let mut prior: Option<&[u8]> = None;
    let mut message_ids = BTreeSet::new();
    for (index, output) in outputs.iter().enumerate() {
        output.record.validate()?;
        let ordinal = u16::try_from(index + 1)
            .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
        if output.semantic_order_key
            != mail_person_source_semantic_order_key_v1(input.page_sequence, ordinal)?
            || prior.is_some_and(|value| value >= output.semantic_order_key.as_slice())
            || !message_ids.insert(output.record.message_id)
        {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        prior = Some(&output.semantic_order_key);
    }
    for ((observation, change), output) in changed.into_iter().zip(outputs) {
        validate_source_change_output_envelope_v1(
            input,
            integration_public_id,
            observation,
            change,
            &output.record,
        )?;
    }
    validate_page_completed_output_envelope_v1(
        input,
        changes,
        outputs
            .last()
            .ok_or(MailAddressBookPersistenceErrorV1::InvalidInput)?,
    )?;
    Ok(())
}

fn validate_source_change_output_envelope_v1(
    input: &MailPersonSourceAtomicFetchCommitV1,
    integration_public_id: [u8; 16],
    observation: &MailPersonSourceObservationV1,
    change: &MailPersonSourceObservationOutcomeV1,
    record: &MailPersonSourceEnvelopeRecordV1,
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    let envelope = record.decode()?;
    let (contract, owner, run_id, page_sequence, source, provenance, observation_id) =
        match change.change_kind {
            MailPersonSourceChangeKindV1::Observed => {
                let payload = MailPersonSourceObservedV1::decode(envelope.payload.as_slice())
                    .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
                validate_mail_person_source_observed_v1(&payload)
                    .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
                if payload.encode_to_vec() != envelope.payload {
                    return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
                }
                (
                    MailPersonSourceContractV1::SourceObserved,
                    payload.logical_owner_id,
                    payload.run_id,
                    payload.page_sequence,
                    payload.source,
                    payload.provenance,
                    payload.observation_id,
                )
            }
            MailPersonSourceChangeKindV1::Updated => {
                let payload = MailPersonSourceUpdatedV1::decode(envelope.payload.as_slice())
                    .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
                validate_mail_person_source_updated_v1(&payload)
                    .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
                if payload.encode_to_vec() != envelope.payload {
                    return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
                }
                (
                    MailPersonSourceContractV1::SourceUpdated,
                    payload.logical_owner_id,
                    payload.run_id,
                    payload.page_sequence,
                    payload.source,
                    payload.provenance,
                    payload.observation_id,
                )
            }
            MailPersonSourceChangeKindV1::Unchanged => {
                return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
            }
        };
    let source = source.ok_or(MailAddressBookPersistenceErrorV1::InvalidInput)?;
    let provenance = provenance.ok_or(MailAddressBookPersistenceErrorV1::InvalidInput)?;
    let observed_at = provenance
        .observed_at
        .as_ref()
        .ok_or(MailAddressBookPersistenceErrorV1::InvalidInput)?;
    let Some(Semantics::Observation(metadata)) = envelope.semantics.as_ref() else {
        return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
    };
    let exact = exact_person_source_contract(&envelope, contract)
        && owner == input.logical_owner_id
        && exact_id16(&run_id)? == input.run_id
        && page_sequence == input.page_sequence
        && exact_id16(&source.integration_public_id)? == integration_public_id
        && exact_id16(&source.account_public_id)? == input.account_public_id
        && exact_id16(&source.provider_source_contact_public_id)?
            == change.provider_source_contact_public_id
        && provenance.source_revision == change.source_revision
        && provenance.source_digest == observation.claims_digest
        && exact_id16(&observation_id)? == record.message_id
        && metadata.observation_id == observation_id
        && metadata.observed_at.as_ref() == Some(observed_at)
        && metadata.occurred_at.as_ref() == Some(observed_at)
        && metadata.source_cursor_sha256 == provenance.source_digest
        && metadata.source_sequence == Some(input.page_sequence)
        && exact_timestamp_unix_millis_v1(
            observed_at.seconds,
            observed_at.nanos,
            observation.observed_at_unix_millis,
        )
        && exact_module_authority_and_time_v1(
            &envelope,
            MAIL_RUNTIME_MODULE_ID_V1,
            observed_at.seconds,
            observed_at.nanos,
        )
        && envelope.partition_key == run_id
        && envelope.correlation_id == run_id
        && envelope.causation_message_id == input.command.message_id;
    if exact {
        Ok(())
    } else {
        Err(MailAddressBookPersistenceErrorV1::InvalidInput)
    }
}

fn validate_page_completed_output_envelope_v1(
    input: &MailPersonSourceAtomicFetchCommitV1,
    changes: &[MailPersonSourceObservationOutcomeV1],
    output: &MailPersonSourceFetchOutputV1,
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    let envelope = output.record.decode()?;
    let payload = MailPersonSourcePageCompletedV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
    validate_mail_person_source_page_completed_v1(&payload)
        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
    let Some(Semantics::Result(metadata)) = envelope.semantics.as_ref() else {
        return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
    };
    let completed_at = payload
        .completed_at
        .as_ref()
        .ok_or(MailAddressBookPersistenceErrorV1::InvalidInput)?;
    let observed = changes
        .iter()
        .filter(|change| change.change_kind == MailPersonSourceChangeKindV1::Observed)
        .count() as u32;
    let updated = changes
        .iter()
        .filter(|change| change.change_kind == MailPersonSourceChangeKindV1::Updated)
        .count() as u32;
    let exact = payload.encode_to_vec() == envelope.payload
        && exact_person_source_contract(&envelope, MailPersonSourceContractV1::PageCompleted)
        && payload.logical_owner_id == input.logical_owner_id
        && exact_id16(&payload.account_public_id)? == input.account_public_id
        && exact_id16(&payload.run_id)? == input.run_id
        && payload.page_sequence == input.page_sequence
        && payload.observed_sources == observed
        && payload.updated_sources == updated
        && payload.removed_sources == 0
        && payload.has_more == input.public_has_more
        && exact_id16(&payload.command_id)? == input.command.message_id
        && metadata.command_id == payload.command_id
        && metadata.command_message_id == input.command.message_id
        && metadata.outcome == ResultOutcomeV1::Succeeded as i32
        && metadata.completed_at.as_ref() == Some(completed_at)
        && exact_timestamp_unix_millis_v1(
            completed_at.seconds,
            completed_at.nanos,
            input.processed_at_unix_millis,
        )
        && exact_module_authority_and_time_v1(
            &envelope,
            MAIL_RUNTIME_MODULE_ID_V1,
            completed_at.seconds,
            completed_at.nanos,
        )
        && envelope.partition_key == payload.run_id
        && envelope.correlation_id == payload.run_id
        && envelope.causation_message_id == input.command.message_id;
    if exact {
        Ok(())
    } else {
        Err(MailAddressBookPersistenceErrorV1::InvalidInput)
    }
}

async fn load_atomic_fetch_outputs(
    transaction: &mut Transaction<'_, Postgres>,
    input: &MailPersonSourceAtomicFetchCommitV1,
) -> Result<Vec<MailPersonSourceFetchOutputV1>, MailAddressBookPersistenceErrorV1> {
    let rows = sqlx::query(
        "SELECT message_id,envelope_sha256,envelope_bytes,semantic_order_key \
         FROM makosh_data.mail_address_book_person_source_fetch_outbox \
         WHERE logical_owner_id=$1 AND account_public_id=$2 AND run_id=$3 AND page_sequence=$4 \
         ORDER BY semantic_order_key,message_id",
    )
    .bind(&input.logical_owner_id)
    .bind(input.account_public_id.as_slice())
    .bind(input.run_id.as_slice())
    .bind(
        i64::try_from(input.page_sequence)
            .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?,
    )
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage)?;
    let mut outputs = Vec::with_capacity(rows.len());
    for row in rows {
        let record = MailPersonSourceEnvelopeRecordV1 {
            message_id: bytes::<16>(&row, "message_id")?,
            envelope_sha256: bytes::<32>(&row, "envelope_sha256")?,
            envelope_bytes: row.try_get("envelope_bytes").map_err(storage)?,
        };
        record.validate()?;
        outputs.push(MailPersonSourceFetchOutputV1 {
            semantic_order_key: row.try_get("semantic_order_key").map_err(storage)?,
            record,
        });
    }
    validate_replayed_fetch_outputs_v1(input, &outputs)?;
    Ok(outputs)
}

fn validate_replayed_fetch_outputs_v1(
    input: &MailPersonSourceAtomicFetchCommitV1,
    outputs: &[MailPersonSourceFetchOutputV1],
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    if outputs.is_empty() || outputs.len() > 501 {
        return Err(MailAddressBookPersistenceErrorV1::InvalidRow);
    }
    for (index, output) in outputs.iter().enumerate() {
        if output.semantic_order_key
            != mail_person_source_semantic_order_key_v1(
                input.page_sequence,
                u16::try_from(index + 1)
                    .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?,
            )?
        {
            return Err(MailAddressBookPersistenceErrorV1::InvalidRow);
        }
        let envelope = output.record.decode()?;
        if index + 1 == outputs.len() {
            let payload = MailPersonSourcePageCompletedV1::decode(envelope.payload.as_slice())
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?;
            validate_mail_person_source_page_completed_v1(&payload)
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?;
            let completed_at = payload
                .completed_at
                .as_ref()
                .ok_or(MailAddressBookPersistenceErrorV1::InvalidRow)?;
            let Some(Semantics::Result(result)) = envelope.semantics.as_ref() else {
                return Err(MailAddressBookPersistenceErrorV1::InvalidRow);
            };
            if payload.encode_to_vec() != envelope.payload
                || !exact_person_source_contract(
                    &envelope,
                    MailPersonSourceContractV1::PageCompleted,
                )
                || payload.logical_owner_id != input.logical_owner_id
                || exact_id16(&payload.account_public_id)? != input.account_public_id
                || exact_id16(&payload.run_id)? != input.run_id
                || payload.page_sequence != input.page_sequence
                || payload.has_more != input.public_has_more
                || exact_id16(&payload.command_id)? != input.command.message_id
                || result.command_id != payload.command_id
                || result.command_message_id != input.command.message_id
                || result.outcome != ResultOutcomeV1::Succeeded as i32
                || result.completed_at.as_ref() != Some(completed_at)
                || !exact_timestamp_unix_millis_v1(
                    completed_at.seconds,
                    completed_at.nanos,
                    input.processed_at_unix_millis,
                )
                || !exact_module_authority_and_time_v1(
                    &envelope,
                    MAIL_RUNTIME_MODULE_ID_V1,
                    completed_at.seconds,
                    completed_at.nanos,
                )
            {
                return Err(MailAddressBookPersistenceErrorV1::InvalidRow);
            }
        } else {
            validate_replayed_source_output_identity_v1(input, &envelope)?;
        }
    }
    Ok(())
}

fn validate_replayed_source_output_identity_v1(
    input: &MailPersonSourceAtomicFetchCommitV1,
    envelope: &DurableEnvelopeV1,
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    let (owner, run_id, page_sequence, account_id, observed_at) =
        if exact_person_source_contract(envelope, MailPersonSourceContractV1::SourceObserved) {
            let payload = MailPersonSourceObservedV1::decode(envelope.payload.as_slice())
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?;
            validate_mail_person_source_observed_v1(&payload)
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?;
            if payload.encode_to_vec() != envelope.payload {
                return Err(MailAddressBookPersistenceErrorV1::InvalidRow);
            }
            (
                payload.logical_owner_id,
                payload.run_id,
                payload.page_sequence,
                payload.source.map(|source| source.account_public_id),
                payload
                    .provenance
                    .and_then(|provenance| provenance.observed_at),
            )
        } else if exact_person_source_contract(envelope, MailPersonSourceContractV1::SourceUpdated)
        {
            let payload = MailPersonSourceUpdatedV1::decode(envelope.payload.as_slice())
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?;
            validate_mail_person_source_updated_v1(&payload)
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?;
            if payload.encode_to_vec() != envelope.payload {
                return Err(MailAddressBookPersistenceErrorV1::InvalidRow);
            }
            (
                payload.logical_owner_id,
                payload.run_id,
                payload.page_sequence,
                payload.source.map(|source| source.account_public_id),
                payload
                    .provenance
                    .and_then(|provenance| provenance.observed_at),
            )
        } else {
            return Err(MailAddressBookPersistenceErrorV1::InvalidRow);
        };
    let observed_at = observed_at.ok_or(MailAddressBookPersistenceErrorV1::InvalidRow)?;
    let Some(Semantics::Observation(observation)) = envelope.semantics.as_ref() else {
        return Err(MailAddressBookPersistenceErrorV1::InvalidRow);
    };
    if owner == input.logical_owner_id
        && exact_id16(&run_id)? == input.run_id
        && page_sequence == input.page_sequence
        && account_id
            .as_deref()
            .is_some_and(|account| account == input.account_public_id)
        && envelope.causation_message_id == input.command.message_id
        && observation.observed_at.as_ref() == Some(&observed_at)
        && observation.occurred_at.as_ref() == Some(&observed_at)
        && exact_module_authority_and_time_v1(
            envelope,
            MAIL_RUNTIME_MODULE_ID_V1,
            observed_at.seconds,
            observed_at.nanos,
        )
    {
        Ok(())
    } else {
        Err(MailAddressBookPersistenceErrorV1::InvalidRow)
    }
}

async fn observe_person_source_contact_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    input: &MailPersonSourceObservationV1,
) -> Result<MailPersonSourceObservationOutcomeV1, MailAddressBookPersistenceErrorV1> {
    let existing = sqlx::query(
        "SELECT provider_source_contact_public_id,claims_digest,source_revision,active,updated_at_unix_millis \
         FROM makosh_data.mail_address_book_person_sources \
         WHERE logical_owner_id=$1 AND account_public_id=$2 AND provider_record_key=$3 FOR UPDATE",
    )
    .bind(&input.logical_owner_id)
    .bind(input.account_public_id.as_slice())
    .bind(&input.provider_record_key)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage)?;
    if let Some(row) = existing {
        let public_id = bytes::<16>(&row, "provider_source_contact_public_id")?;
        let stored_digest = bytes::<32>(&row, "claims_digest")?;
        let revision = row
            .try_get::<i64, _>("source_revision")
            .map_err(storage)?
            .try_into()
            .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?;
        let active = row.try_get::<bool, _>("active").map_err(storage)?;
        let updated_at = row
            .try_get::<i64, _>("updated_at_unix_millis")
            .map_err(storage)?;
        if input.observed_at_unix_millis < updated_at {
            return Err(MailAddressBookPersistenceErrorV1::Conflict);
        }
        if active && stored_digest == input.claims_digest {
            sqlx::query(
                "UPDATE makosh_data.mail_address_book_person_sources SET provider_record_etag=$4,updated_at_unix_millis=$5 \
                 WHERE logical_owner_id=$1 AND account_public_id=$2 AND provider_record_key=$3",
            )
            .bind(&input.logical_owner_id)
            .bind(input.account_public_id.as_slice())
            .bind(&input.provider_record_key)
            .bind(input.provider_record_etag.as_deref())
            .bind(input.observed_at_unix_millis)
            .execute(&mut **transaction)
            .await
            .map_err(storage)?;
            Ok(MailPersonSourceObservationOutcomeV1 {
                provider_source_contact_public_id: public_id,
                source_revision: revision,
                change_kind: MailPersonSourceChangeKindV1::Unchanged,
            })
        } else {
            let next_revision = revision
                .checked_add(1)
                .ok_or(MailAddressBookPersistenceErrorV1::Conflict)?;
            sqlx::query(
                "UPDATE makosh_data.mail_address_book_person_sources SET provider_record_etag=$4,claims_digest=$5,source_revision=$6,active=TRUE,last_terminal_run_id=NULL,updated_at_unix_millis=$7 \
                 WHERE logical_owner_id=$1 AND account_public_id=$2 AND provider_record_key=$3",
            )
            .bind(&input.logical_owner_id)
            .bind(input.account_public_id.as_slice())
            .bind(&input.provider_record_key)
            .bind(input.provider_record_etag.as_deref())
            .bind(input.claims_digest.as_slice())
            .bind(
                i64::try_from(next_revision)
                    .map_err(|_| MailAddressBookPersistenceErrorV1::Conflict)?,
            )
            .bind(input.observed_at_unix_millis)
            .execute(&mut **transaction)
            .await
            .map_err(storage)?;
            Ok(MailPersonSourceObservationOutcomeV1 {
                provider_source_contact_public_id: public_id,
                source_revision: next_revision,
                change_kind: if active {
                    MailPersonSourceChangeKindV1::Updated
                } else {
                    MailPersonSourceChangeKindV1::Observed
                },
            })
        }
    } else {
        sqlx::query(
            "INSERT INTO makosh_data.mail_address_book_person_sources \
             (logical_owner_id,account_public_id,provider_record_key,provider_record_etag,provider_source_contact_public_id,claims_digest,source_revision,active,last_terminal_run_id,updated_at_unix_millis) \
             VALUES ($1,$2,$3,$4,$5,$6,1,TRUE,NULL,$7)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.account_public_id.as_slice())
        .bind(&input.provider_record_key)
        .bind(input.provider_record_etag.as_deref())
        .bind(input.proposed_source_public_id.as_slice())
        .bind(input.claims_digest.as_slice())
        .bind(input.observed_at_unix_millis)
        .execute(&mut **transaction)
        .await
        .map_err(storage)?;
        Ok(MailPersonSourceObservationOutcomeV1 {
            provider_source_contact_public_id: input.proposed_source_public_id,
            source_revision: 1,
            change_kind: MailPersonSourceChangeKindV1::Observed,
        })
    }
}

fn validate_snapshot_identity(
    logical_owner_id: &str,
    account_public_id: [u8; 16],
    seen_public_source_ids: &[[u8; 16]],
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    validate_owner(logical_owner_id)?;
    let seen = seen_public_source_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if !nonzero(&account_public_id)
        || seen.len() != seen_public_source_ids.len()
        || seen.iter().any(|source_id| !nonzero(source_id))
    {
        Err(MailAddressBookPersistenceErrorV1::InvalidInput)
    } else {
        Ok(())
    }
}

struct MailPersonSourceSnapshotValidationV1 {
    plan_sha256: [u8; 32],
    terminal_fingerprint: [u8; 32],
    terminal_page_sequence: u64,
}

fn validate_snapshot_terminal_envelope_v1(
    input: &MailPersonSourceSnapshotCommitV1,
) -> Result<(u64, [u8; 32]), MailAddressBookPersistenceErrorV1> {
    let envelope = input.terminal_command.decode()?;
    let payload = MailPersonSourcePageCompletedV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
    validate_mail_person_source_page_completed_v1(&payload)
        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
    let Some(Semantics::Result(metadata)) = envelope.semantics.as_ref() else {
        return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
    };
    let completed_at = payload
        .completed_at
        .as_ref()
        .ok_or(MailAddressBookPersistenceErrorV1::InvalidInput)?;
    let exact = payload.encode_to_vec() == envelope.payload
        && exact_person_source_contract(&envelope, MailPersonSourceContractV1::PageCompleted)
        && payload.logical_owner_id == input.logical_owner_id
        && exact_id16(&payload.account_public_id)? == input.account_public_id
        && exact_id16(&payload.run_id)? == input.run_id
        && payload.has_more == !input.removal_pages.is_empty()
        && metadata.command_id == payload.command_id
        && metadata.command_message_id == payload.command_id
        && metadata.outcome == ResultOutcomeV1::Succeeded as i32
        && metadata.completed_at.as_ref() == Some(completed_at)
        && exact_module_authority_and_time_v1(
            &envelope,
            MAIL_RUNTIME_MODULE_ID_V1,
            completed_at.seconds,
            completed_at.nanos,
        )
        && envelope.partition_key == payload.run_id
        && envelope.correlation_id == payload.run_id
        && envelope.causation_message_id == payload.command_id;
    if !exact {
        return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
    }
    let mut fingerprint = Sha256::new();
    fingerprint.update(b"makosh.mail.person-source.terminal-fingerprint.v1");
    fingerprint.update((input.terminal_command.envelope_bytes.len() as u64).to_be_bytes());
    fingerprint.update(&input.terminal_command.envelope_bytes);
    Ok((payload.page_sequence, fingerprint.finalize().into()))
}

fn validate_synthetic_fetch_continuation_page_v1(
    logical_owner_id: &str,
    account_public_id: [u8; 16],
    run_id: [u8; 16],
    page_sequence: u64,
    command_id: [u8; 16],
    record: &MailPersonSourceEnvelopeRecordV1,
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    let envelope = record.decode()?;
    let payload = MailPersonSourcePageCompletedV1::decode(envelope.payload.as_slice())
        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?;
    validate_mail_person_source_page_completed_v1(&payload)
        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?;
    let Some(Semantics::Result(result)) = envelope.semantics.as_ref() else {
        return Err(MailAddressBookPersistenceErrorV1::InvalidRow);
    };
    let completed_at = payload
        .completed_at
        .as_ref()
        .ok_or(MailAddressBookPersistenceErrorV1::InvalidRow)?;
    let exact = payload.encode_to_vec() == envelope.payload
        && exact_person_source_contract(&envelope, MailPersonSourceContractV1::PageCompleted)
        && payload.logical_owner_id == logical_owner_id
        && exact_id16(&payload.account_public_id)? == account_public_id
        && exact_id16(&payload.run_id)? == run_id
        && payload.page_sequence == page_sequence
        && payload.observed_sources == 0
        && payload.updated_sources == 0
        && payload.removed_sources > 0
        && exact_id16(&payload.command_id)? == command_id
        && result.command_id == payload.command_id
        && result.command_message_id == payload.command_id
        && result.outcome == ResultOutcomeV1::Succeeded as i32
        && result.completed_at.as_ref() == Some(completed_at)
        && exact_module_authority_and_time_v1(
            &envelope,
            MAIL_RUNTIME_MODULE_ID_V1,
            completed_at.seconds,
            completed_at.nanos,
        )
        && envelope.partition_key == payload.run_id
        && envelope.correlation_id == payload.run_id
        && envelope.causation_message_id == payload.command_id;
    if exact {
        Ok(())
    } else {
        Err(MailAddressBookPersistenceErrorV1::InvalidRow)
    }
}

fn validate_removal_output_envelopes_v1(
    input: &MailPersonSourceSnapshotCommitV1,
    terminal_page_sequence: u64,
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    let expected_chunks = input.expected_removals.chunks(500).collect::<Vec<_>>();
    for (page_index, (expected, page)) in expected_chunks
        .into_iter()
        .zip(&input.removal_pages)
        .enumerate()
    {
        let expected_page_sequence = terminal_page_sequence
            .checked_add(
                u64::try_from(page_index + 1)
                    .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?,
            )
            .ok_or(MailAddressBookPersistenceErrorV1::InvalidInput)?;
        if page.page_sequence != expected_page_sequence || page.outputs.len() != expected.len() + 1
        {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let completion = page
            .outputs
            .last()
            .ok_or(MailAddressBookPersistenceErrorV1::InvalidInput)?;
        let completion_envelope = completion.record.decode()?;
        let completion_payload =
            MailPersonSourcePageCompletedV1::decode(completion_envelope.payload.as_slice())
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
        validate_mail_person_source_page_completed_v1(&completion_payload)
            .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
        let Some(Semantics::Result(result)) = completion_envelope.semantics.as_ref() else {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        };
        let completion_time = completion_payload
            .completed_at
            .as_ref()
            .ok_or(MailAddressBookPersistenceErrorV1::InvalidInput)?;
        let command_id = exact_id16(&completion_payload.command_id)?;
        if completion_payload.encode_to_vec() != completion_envelope.payload
            || !exact_person_source_contract(
                &completion_envelope,
                MailPersonSourceContractV1::PageCompleted,
            )
            || completion_payload.logical_owner_id != input.logical_owner_id
            || exact_id16(&completion_payload.account_public_id)? != input.account_public_id
            || exact_id16(&completion_payload.run_id)? != input.run_id
            || completion_payload.page_sequence != page.page_sequence
            || completion_payload.observed_sources != 0
            || completion_payload.updated_sources != 0
            || completion_payload.removed_sources
                != u32::try_from(expected.len())
                    .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?
            || completion_payload.has_more != (page_index + 1 < input.removal_pages.len())
            || result.command_id != completion_payload.command_id
            || result.command_message_id != completion_payload.command_id
            || result.outcome != ResultOutcomeV1::Succeeded as i32
            || result.completed_at.as_ref() != Some(completion_time)
            || !exact_timestamp_unix_millis_v1(
                completion_time.seconds,
                completion_time.nanos,
                input.completed_at_unix_millis,
            )
            || !exact_module_authority_and_time_v1(
                &completion_envelope,
                MAIL_RUNTIME_MODULE_ID_V1,
                completion_time.seconds,
                completion_time.nanos,
            )
            || completion_envelope.partition_key != completion_payload.run_id
            || completion_envelope.correlation_id != completion_payload.run_id
            || completion_envelope.causation_message_id != completion_payload.command_id
        {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let mut page_digest = Sha256::new();
        for (removal, output) in expected.iter().zip(&page.outputs) {
            let envelope = output.record.decode()?;
            let payload = MailPersonSourceRemovedV1::decode(envelope.payload.as_slice())
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
            validate_mail_person_source_removed_v1(&payload)
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
            let source = payload
                .source
                .as_ref()
                .ok_or(MailAddressBookPersistenceErrorV1::InvalidInput)?;
            let provenance = payload
                .provenance
                .as_ref()
                .ok_or(MailAddressBookPersistenceErrorV1::InvalidInput)?;
            let observed_at = provenance
                .observed_at
                .as_ref()
                .ok_or(MailAddressBookPersistenceErrorV1::InvalidInput)?;
            let Some(Semantics::Observation(observation)) = envelope.semantics.as_ref() else {
                return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
            };
            if payload.encode_to_vec() != envelope.payload
                || !exact_person_source_contract(
                    &envelope,
                    MailPersonSourceContractV1::SourceRemoved,
                )
                || payload.logical_owner_id != input.logical_owner_id
                || exact_id16(&payload.run_id)? != input.run_id
                || payload.page_sequence != page.page_sequence
                || exact_id16(&source.integration_public_id)? != removal.integration_public_id
                || exact_id16(&source.account_public_id)? != input.account_public_id
                || exact_id16(&source.provider_source_contact_public_id)?
                    != removal.provider_source_contact_public_id
                || provenance.source_revision != removal.source_revision
                || provenance.source_digest
                    != mail_person_source_tombstone_digest_v1(source)
                        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?
                || exact_id16(&payload.observation_id)? != output.record.message_id
                || observation.observation_id != payload.observation_id
                || observation.observed_at.as_ref() != Some(observed_at)
                || observation.occurred_at.as_ref() != Some(observed_at)
                || observation.source_cursor_sha256 != provenance.source_digest
                || observation.source_sequence != Some(page.page_sequence)
                || !exact_timestamp_unix_millis_v1(
                    observed_at.seconds,
                    observed_at.nanos,
                    input.completed_at_unix_millis,
                )
                || !exact_module_authority_and_time_v1(
                    &envelope,
                    MAIL_RUNTIME_MODULE_ID_V1,
                    observed_at.seconds,
                    observed_at.nanos,
                )
                || envelope.partition_key != payload.run_id
                || envelope.correlation_id != payload.run_id
                || envelope.causation_message_id != command_id
            {
                return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
            }
            page_digest.update(output.record.envelope_sha256);
        }
        let page_digest: [u8; 32] = page_digest.finalize().into();
        if completion_payload.page_digest != page_digest {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
    }
    Ok(())
}

fn validate_snapshot_commit(
    input: &MailPersonSourceSnapshotCommitV1,
) -> Result<MailPersonSourceSnapshotValidationV1, MailAddressBookPersistenceErrorV1> {
    validate_snapshot_identity(
        &input.logical_owner_id,
        input.account_public_id,
        &input.seen_public_source_ids,
    )?;
    let (terminal_page_sequence, terminal_fingerprint) =
        validate_snapshot_terminal_envelope_v1(input)?;
    if !nonzero(&input.run_id)
        || input.completed_at_unix_millis <= 0
        || input
            .seen_public_source_ids
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
    {
        return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
    }
    let mut prior_source_id: Option<[u8; 16]> = None;
    for removal in &input.expected_removals {
        if removal.account_public_id != input.account_public_id
            || !nonzero(&removal.integration_public_id)
            || !nonzero(&removal.provider_source_contact_public_id)
            || removal.source_revision < 2
            || prior_source_id
                .is_some_and(|prior| prior >= removal.provider_source_contact_public_id)
        {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        prior_source_id = Some(removal.provider_source_contact_public_id);
    }
    if input.expected_removals.is_empty() != input.removal_pages.is_empty() {
        return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
    }
    let expected_chunks = input.expected_removals.chunks(500).collect::<Vec<_>>();
    if expected_chunks.len() != input.removal_pages.len() {
        return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
    }
    let mut prior_page_sequence = Some(terminal_page_sequence);
    let mut semantic_keys = BTreeSet::new();
    let mut message_ids = BTreeSet::new();
    for (expected, page) in expected_chunks.into_iter().zip(&input.removal_pages) {
        if !(1..=4_096).contains(&page.page_sequence)
            || prior_page_sequence.is_some_and(|prior| page.page_sequence != prior + 1)
            || page.source_ids
                != expected
                    .iter()
                    .map(|value| value.provider_source_contact_public_id)
                    .collect::<Vec<_>>()
            || page.outputs.len() != page.source_ids.len() + 1
        {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        prior_page_sequence = Some(page.page_sequence);
        let mut prior_key: Option<&[u8]> = None;
        for (index, output) in page.outputs.iter().enumerate() {
            output.record.validate()?;
            let ordinal = u16::try_from(index + 1)
                .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)?;
            if output.semantic_order_key
                != mail_person_source_semantic_order_key_v1(page.page_sequence, ordinal)?
                || prior_key.is_some_and(|prior| prior >= output.semantic_order_key.as_slice())
                || !semantic_keys.insert(output.semantic_order_key.clone())
                || !message_ids.insert(output.record.message_id)
            {
                return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
            }
            prior_key = Some(&output.semantic_order_key);
        }
    }
    validate_removal_output_envelopes_v1(input, terminal_page_sequence)?;
    let mut digest = Sha256::new();
    digest.update(b"makosh.mail.person-source.terminal-snapshot-plan.v1");
    digest.update((input.logical_owner_id.len() as u64).to_be_bytes());
    digest.update(input.logical_owner_id.as_bytes());
    digest.update(input.account_public_id);
    digest.update(input.run_id);
    digest.update((input.seen_public_source_ids.len() as u64).to_be_bytes());
    for source_id in &input.seen_public_source_ids {
        digest.update(source_id);
    }
    digest.update((input.expected_removals.len() as u64).to_be_bytes());
    for removal in &input.expected_removals {
        digest.update(removal.integration_public_id);
        digest.update(removal.account_public_id);
        digest.update(removal.provider_source_contact_public_id);
        digest.update(removal.source_revision.to_be_bytes());
    }
    digest.update((input.removal_pages.len() as u64).to_be_bytes());
    for page in &input.removal_pages {
        digest.update(page.page_sequence.to_be_bytes());
        digest.update((page.outputs.len() as u64).to_be_bytes());
        for output in &page.outputs {
            digest.update((output.semantic_order_key.len() as u64).to_be_bytes());
            digest.update(&output.semantic_order_key);
            digest.update(output.record.message_id);
            digest.update(output.record.envelope_sha256);
            digest.update((output.record.envelope_bytes.len() as u64).to_be_bytes());
            digest.update(&output.record.envelope_bytes);
        }
    }
    digest.update(input.completed_at_unix_millis.to_be_bytes());
    Ok(MailPersonSourceSnapshotValidationV1 {
        plan_sha256: digest.finalize().into(),
        terminal_fingerprint,
        terminal_page_sequence,
    })
}

async fn select_person_source_removals(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
    account_public_id: [u8; 16],
    seen_public_source_ids: &[[u8; 16]],
) -> Result<Vec<MailPersonSourceRemovalStateV1>, MailAddressBookPersistenceErrorV1> {
    let seen = seen_public_source_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let rows = sqlx::query(
        "SELECT a.integration_public_id,s.provider_source_contact_public_id,s.source_revision \
         FROM makosh_data.mail_address_book_person_sources s \
         JOIN makosh_data.mail_address_book_person_source_accounts a \
           ON a.logical_owner_id=s.logical_owner_id AND a.account_public_id=s.account_public_id \
         WHERE s.logical_owner_id=$1 AND s.account_public_id=$2 AND s.active=TRUE \
         ORDER BY s.provider_source_contact_public_id FOR UPDATE OF s",
    )
    .bind(logical_owner_id)
    .bind(account_public_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage)?;
    let mut removals = Vec::new();
    for row in rows {
        let source_id = bytes::<16>(&row, "provider_source_contact_public_id")?;
        if seen.contains(&source_id) {
            continue;
        }
        let revision: u64 = row
            .try_get::<i64, _>("source_revision")
            .map_err(storage)?
            .try_into()
            .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)?;
        removals.push(MailPersonSourceRemovalStateV1 {
            integration_public_id: bytes::<16>(&row, "integration_public_id")?,
            account_public_id,
            provider_source_contact_public_id: source_id,
            source_revision: revision
                .checked_add(1)
                .ok_or(MailAddressBookPersistenceErrorV1::Conflict)?,
        });
    }
    Ok(removals)
}

async fn set_owner(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    owner: &str,
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    sqlx::query("SELECT set_config('makosh.logical_owner_id',$1,true)")
        .bind(owner)
        .execute(&mut **transaction)
        .await
        .map(|_| ())
        .map_err(storage)
}

fn validate_owner(owner: &str) -> Result<(), MailAddressBookPersistenceErrorV1> {
    if !owner.is_empty()
        && owner.len() <= 128
        && owner.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        })
    {
        Ok(())
    } else {
        Err(MailAddressBookPersistenceErrorV1::InvalidInput)
    }
}
fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|b| *b != 0)
}
fn bytes<const N: usize>(
    row: &sqlx::postgres::PgRow,
    name: &str,
) -> Result<[u8; N], MailAddressBookPersistenceErrorV1> {
    row.try_get::<Vec<u8>, _>(name)
        .map_err(storage)?
        .try_into()
        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidRow)
}
fn storage<T>(_: T) -> MailAddressBookPersistenceErrorV1 {
    MailAddressBookPersistenceErrorV1::StorageUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_events_protocol::v1::{ActorKindV1, FenceKindV1};
    use makosh_mail_address_book_contract::{
        MAIL_RUNTIME_MODULE_ID_V1, MailAddressBookEnvelopeContextV1,
        MailAddressBookResultEnvelopeContextV1, build_fetch_mail_person_source_page_command_v1,
        build_mail_person_source_observed_v1, build_mail_person_source_page_completed_v1,
        build_mail_person_source_removed_v1, mail_person_source_claims_digest_v1,
        mail_person_source_tombstone_digest_v1,
        wire_person_source::{
            MailPersonSourceClaimsV1, MailPersonSourceIdentityV1, MailPersonSourceProvenanceV1,
            MailPersonSourceRemovedV1,
        },
    };

    fn fetch_input() -> MailPersonSourceAtomicFetchCommitV1 {
        let command = build_fetch_mail_person_source_page_command_v1(
            FetchMailPersonSourcePageCommandV1 {
                command_id: [1; 16].to_vec(),
                run_id: [2; 16].to_vec(),
                logical_owner_id: "owner-a".to_owned(),
                account_public_id: [3; 16].to_vec(),
                page_sequence: 1,
                page_size: 500,
            },
            2,
            &MailAddressBookEnvelopeContextV1 {
                module_id: "makosh-mail-persons-sync-runtime".to_owned(),
                runtime_instance_id: "exact-envelope-test".to_owned(),
                runtime_generation: 1,
                recorded_at_unix_seconds: 1,
                recorded_at_nanos: 0,
            },
        )
        .expect("canonical fetch");
        MailPersonSourceAtomicFetchCommitV1 {
            logical_owner_id: "owner-a".to_owned(),
            account_public_id: [3; 16],
            run_id: [2; 16],
            page_sequence: 1,
            expected_provider_cursor: None,
            next_provider_cursor: None,
            public_has_more: false,
            has_more: false,
            command: MailPersonSourceEnvelopeRecordV1::from_outbox(&command),
            processed_at_unix_millis: 1_000,
        }
    }

    #[test]
    fn fetch_command_exact_binds_public_identity() {
        let input = fetch_input();
        validate_fetch_command_envelope_v1(&input).expect("exact public fetch");
        for mut changed in [
            {
                let mut value = input.clone();
                value.logical_owner_id = "owner-b".to_owned();
                value
            },
            {
                let mut value = input.clone();
                value.account_public_id = [4; 16];
                value
            },
            {
                let mut value = input.clone();
                value.run_id = [5; 16];
                value
            },
            {
                let mut value = input.clone();
                value.page_sequence = 2;
                value
            },
        ] {
            assert_eq!(
                validate_fetch_command_envelope_v1(&changed),
                Err(MailAddressBookPersistenceErrorV1::InvalidInput)
            );
            changed.processed_at_unix_millis += 1;
        }
    }

    #[test]
    fn fetch_command_exact_binds_authority_and_processing_freshness() {
        let input = fetch_input();
        validate_fetch_command_freshness_v1(&input).expect("fresh canonical command");
        for mutated in [
            mutate_record(&input.command, |envelope| {
                envelope.actor.as_mut().expect("actor").kind = ActorKindV1::System as i32;
            }),
            mutate_record(&input.command, |envelope| {
                envelope.source_fence.as_mut().expect("fence").kind =
                    FenceKindV1::GrantEpoch as i32;
            }),
        ] {
            let mut changed = input.clone();
            changed.command = mutated;
            assert_eq!(
                validate_fetch_command_envelope_v1(&changed),
                Err(MailAddressBookPersistenceErrorV1::InvalidInput)
            );
        }
        let mut zero_generation = input.command.decode().expect("canonical fetch");
        zero_generation
            .source
            .as_mut()
            .expect("source")
            .runtime_generation = 0;
        zero_generation.source_fence.as_mut().expect("fence").epoch = 0;
        let recorded_at = zero_generation.recorded_at.as_ref().expect("recorded at");
        assert!(!exact_module_authority_and_time_v1(
            &zero_generation,
            "makosh-mail-persons-sync-runtime",
            recorded_at.seconds,
            recorded_at.nanos,
        ));

        let mut recorded_in_future = input.clone();
        recorded_in_future.processed_at_unix_millis = 999;
        assert_eq!(
            validate_fetch_command_freshness_v1(&recorded_in_future),
            Err(MailAddressBookPersistenceErrorV1::InvalidInput)
        );
        let mut expired = input;
        expired.processed_at_unix_millis = 2_000;
        assert_eq!(
            validate_fetch_command_freshness_v1(&expired),
            Err(MailAddressBookPersistenceErrorV1::InvalidInput)
        );
    }

    #[test]
    fn atomic_fetch_distinguishes_provider_cursor_from_public_removal_continuation() {
        let mut synthetic_removal_continuation = fetch_input();
        synthetic_removal_continuation.public_has_more = true;
        validate_atomic_fetch_request(&synthetic_removal_continuation)
            .expect("public continuation after provider exhaustion");

        let mut impossible_provider_continuation = fetch_input();
        impossible_provider_continuation.next_provider_cursor = Some(vec![1]);
        impossible_provider_continuation.has_more = true;
        assert_eq!(
            validate_atomic_fetch_request(&impossible_provider_continuation),
            Err(MailAddressBookPersistenceErrorV1::InvalidInput),
        );
    }

    #[test]
    fn canonical_decode_rejects_unknown_private_payload_material() {
        let mut input = fetch_input();
        let mut envelope = input.command.decode().expect("canonical envelope");
        envelope.payload.extend_from_slice(&[0x98, 0x06, 0x01]);
        let accepted = OutboxRecordV1::accept(envelope.encode_to_vec())
            .expect("generic envelope accepts forward protobuf field");
        input.command = MailPersonSourceEnvelopeRecordV1::from_outbox(&accepted);
        assert_eq!(
            validate_fetch_command_envelope_v1(&input),
            Err(MailAddressBookPersistenceErrorV1::InvalidInput)
        );
    }

    fn output_fixture() -> (
        MailPersonSourceAtomicFetchCommitV1,
        [u8; 16],
        MailPersonSourceObservationV1,
        MailPersonSourceObservationOutcomeV1,
        Vec<MailPersonSourceFetchOutputV1>,
    ) {
        let input = fetch_input();
        let integration_public_id = [4; 16];
        let source = MailPersonSourceIdentityV1 {
            integration_public_id: integration_public_id.to_vec(),
            account_public_id: input.account_public_id.to_vec(),
            provider_source_contact_public_id: [5; 16].to_vec(),
        };
        let claims = MailPersonSourceClaimsV1 {
            display_name: Some("Public Person".to_owned()),
            normalized_emails: vec!["public@example.test".to_owned()],
            normalized_phones: Vec::new(),
        };
        let claims_digest =
            mail_person_source_claims_digest_v1(&source, &claims).expect("canonical claims");
        let observed_at = input
            .command
            .decode()
            .expect("fetch envelope")
            .recorded_at
            .expect("fetch timestamp");
        let observation = MailPersonSourceObservationV1 {
            logical_owner_id: input.logical_owner_id.clone(),
            account_public_id: input.account_public_id,
            provider_record_key: vec![1],
            provider_record_etag: Some(vec![2]),
            proposed_source_public_id: [5; 16],
            claims_digest,
            observed_at_unix_millis: 1_000,
        };
        let change = MailPersonSourceObservationOutcomeV1 {
            provider_source_contact_public_id: [5; 16],
            source_revision: 1,
            change_kind: MailPersonSourceChangeKindV1::Observed,
        };
        let context = MailAddressBookEnvelopeContextV1 {
            module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
            runtime_instance_id: "output-authority-test".to_owned(),
            runtime_generation: 7,
            recorded_at_unix_seconds: 1,
            recorded_at_nanos: 0,
        };
        let observed = build_mail_person_source_observed_v1(
            input.command.message_id,
            MailPersonSourceObservedV1 {
                observation_id: [6; 16].to_vec(),
                run_id: input.run_id.to_vec(),
                logical_owner_id: input.logical_owner_id.clone(),
                page_sequence: input.page_sequence,
                source: Some(source),
                claims: Some(claims),
                provenance: Some(MailPersonSourceProvenanceV1 {
                    source_revision: 1,
                    source_digest: claims_digest.to_vec(),
                    observed_at: Some(observed_at),
                }),
            },
            &context,
        )
        .expect("observed output");
        let completed = build_mail_person_source_page_completed_v1(
            input.command.message_id,
            MailPersonSourcePageCompletedV1 {
                command_id: input.command.message_id.to_vec(),
                run_id: input.run_id.to_vec(),
                logical_owner_id: input.logical_owner_id.clone(),
                account_public_id: input.account_public_id.to_vec(),
                page_sequence: input.page_sequence,
                observed_sources: 1,
                updated_sources: 0,
                removed_sources: 0,
                has_more: false,
                page_digest: observed.envelope_sha256().to_vec(),
                completed_at: Some(observed_at),
            },
            &MailAddressBookResultEnvelopeContextV1 {
                runtime_instance_id: "output-authority-test".to_owned(),
                runtime_generation: 7,
                completed_at_unix_seconds: 1,
                completed_at_nanos: 0,
                execution_attempt: 1,
            },
        )
        .expect("page completion");
        let outputs = vec![
            MailPersonSourceFetchOutputV1 {
                semantic_order_key: mail_person_source_semantic_order_key_v1(1, 1)
                    .expect("source order"),
                record: MailPersonSourceEnvelopeRecordV1::from_outbox(&observed),
            },
            MailPersonSourceFetchOutputV1 {
                semantic_order_key: mail_person_source_semantic_order_key_v1(1, 2)
                    .expect("completion order"),
                record: MailPersonSourceEnvelopeRecordV1::from_outbox(&completed),
            },
        ];
        (input, integration_public_id, observation, change, outputs)
    }

    fn mutate_record(
        record: &MailPersonSourceEnvelopeRecordV1,
        mutate: impl FnOnce(&mut DurableEnvelopeV1),
    ) -> MailPersonSourceEnvelopeRecordV1 {
        let mut envelope = record.decode().expect("canonical output");
        mutate(&mut envelope);
        let accepted = OutboxRecordV1::accept(envelope.encode_to_vec()).expect("valid mutation");
        MailPersonSourceEnvelopeRecordV1::from_outbox(&accepted)
    }

    #[test]
    fn producer_outputs_exact_bind_mail_authority_fence_generation_and_times() {
        let (input, integration, observation, change, outputs) = output_fixture();
        validate_fetch_output_envelopes_v1(
            &input,
            integration,
            std::slice::from_ref(&observation),
            std::slice::from_ref(&change),
            &outputs,
        )
        .expect("canonical outputs");

        let source_mutations = [
            mutate_record(&outputs[0].record, |envelope| {
                envelope.source.as_mut().expect("source").module_id = "other-mail-runtime".into();
            }),
            mutate_record(&outputs[0].record, |envelope| {
                envelope.source.as_mut().expect("source").runtime_generation += 1;
            }),
            mutate_record(&outputs[0].record, |envelope| {
                envelope.actor.as_mut().expect("actor").kind = ActorKindV1::System as i32;
            }),
            mutate_record(&outputs[0].record, |envelope| {
                envelope.actor.as_mut().expect("actor").actor_id = b"other-mail-runtime".to_vec();
            }),
            mutate_record(&outputs[0].record, |envelope| {
                envelope.source_fence.as_mut().expect("fence").kind =
                    FenceKindV1::GrantEpoch as i32;
            }),
            mutate_record(&outputs[0].record, |envelope| {
                envelope.source_fence.as_mut().expect("fence").scope_id =
                    b"other-mail-runtime".to_vec();
            }),
            mutate_record(&outputs[0].record, |envelope| {
                envelope.source_fence.as_mut().expect("fence").epoch += 1;
            }),
            mutate_record(&outputs[0].record, |envelope| {
                envelope.recorded_at.as_mut().expect("recorded at").seconds += 1;
            }),
            mutate_record(&outputs[0].record, |envelope| {
                let Some(Semantics::Observation(metadata)) = envelope.semantics.as_mut() else {
                    panic!("observation semantics");
                };
                metadata.observed_at.as_mut().expect("observed at").seconds += 1;
            }),
            mutate_record(&outputs[0].record, |envelope| {
                let Some(Semantics::Observation(metadata)) = envelope.semantics.as_mut() else {
                    panic!("observation semantics");
                };
                metadata.occurred_at.as_mut().expect("occurred at").seconds += 1;
            }),
        ];
        for mutated in source_mutations {
            let mut changed_outputs = outputs.clone();
            changed_outputs[0].record = mutated;
            assert_eq!(
                validate_fetch_output_envelopes_v1(
                    &input,
                    integration,
                    std::slice::from_ref(&observation),
                    std::slice::from_ref(&change),
                    &changed_outputs,
                ),
                Err(MailAddressBookPersistenceErrorV1::InvalidInput)
            );
        }

        let result_mutations = [
            mutate_record(&outputs[1].record, |envelope| {
                envelope.source.as_mut().expect("source").module_id = "other-mail-runtime".into();
            }),
            mutate_record(&outputs[1].record, |envelope| {
                envelope.actor.as_mut().expect("actor").kind = ActorKindV1::System as i32;
            }),
            mutate_record(&outputs[1].record, |envelope| {
                envelope.source_fence.as_mut().expect("fence").kind =
                    FenceKindV1::GrantEpoch as i32;
            }),
            mutate_record(&outputs[1].record, |envelope| {
                envelope.source_fence.as_mut().expect("fence").epoch += 1;
            }),
            mutate_record(&outputs[1].record, |envelope| {
                envelope.recorded_at.as_mut().expect("recorded at").seconds += 1;
            }),
            mutate_record(&outputs[1].record, |envelope| {
                let Some(Semantics::Result(metadata)) = envelope.semantics.as_mut() else {
                    panic!("result semantics");
                };
                metadata
                    .completed_at
                    .as_mut()
                    .expect("completed at")
                    .seconds += 1;
            }),
            mutate_record(&outputs[1].record, |envelope| {
                envelope.recorded_at.as_mut().expect("recorded at").seconds += 1;
                let Some(Semantics::Result(metadata)) = envelope.semantics.as_mut() else {
                    panic!("result semantics");
                };
                metadata
                    .completed_at
                    .as_mut()
                    .expect("completed at")
                    .seconds += 1;
                let mut payload =
                    MailPersonSourcePageCompletedV1::decode(envelope.payload.as_slice())
                        .expect("page completion payload");
                payload
                    .completed_at
                    .as_mut()
                    .expect("payload completed at")
                    .seconds += 1;
                envelope.payload = payload.encode_to_vec();
            }),
        ];
        for mutated in result_mutations {
            let mut changed_outputs = outputs.clone();
            changed_outputs[1].record = mutated;
            assert_eq!(
                validate_fetch_output_envelopes_v1(
                    &input,
                    integration,
                    std::slice::from_ref(&observation),
                    std::slice::from_ref(&change),
                    &changed_outputs,
                ),
                Err(MailAddressBookPersistenceErrorV1::InvalidInput)
            );
        }
    }

    fn snapshot_fixture() -> MailPersonSourceSnapshotCommitV1 {
        let (input, integration_public_id, _, _, mut outputs) = output_fixture();
        outputs[1].record = mutate_record(&outputs[1].record, |envelope| {
            let mut payload = MailPersonSourcePageCompletedV1::decode(envelope.payload.as_slice())
                .expect("terminal page payload");
            payload.has_more = true;
            envelope.payload = payload.encode_to_vec();
        });
        let source = MailPersonSourceIdentityV1 {
            integration_public_id: integration_public_id.to_vec(),
            account_public_id: input.account_public_id.to_vec(),
            provider_source_contact_public_id: [9; 16].to_vec(),
        };
        let timestamp = input
            .command
            .decode()
            .expect("fetch envelope")
            .recorded_at
            .expect("fetch timestamp");
        let tombstone =
            mail_person_source_tombstone_digest_v1(&source).expect("canonical tombstone");
        let command_id = [8; 16];
        let context = MailAddressBookEnvelopeContextV1 {
            module_id: MAIL_RUNTIME_MODULE_ID_V1.to_owned(),
            runtime_instance_id: "snapshot-time-test".to_owned(),
            runtime_generation: 7,
            recorded_at_unix_seconds: 1,
            recorded_at_nanos: 0,
        };
        let removed = build_mail_person_source_removed_v1(
            command_id,
            MailPersonSourceRemovedV1 {
                observation_id: [10; 16].to_vec(),
                run_id: input.run_id.to_vec(),
                logical_owner_id: input.logical_owner_id.clone(),
                page_sequence: 2,
                source: Some(source),
                provenance: Some(MailPersonSourceProvenanceV1 {
                    source_revision: 2,
                    source_digest: tombstone.to_vec(),
                    observed_at: Some(timestamp),
                }),
            },
            &context,
        )
        .expect("removed output");
        let completion = build_mail_person_source_page_completed_v1(
            command_id,
            MailPersonSourcePageCompletedV1 {
                command_id: command_id.to_vec(),
                run_id: input.run_id.to_vec(),
                logical_owner_id: input.logical_owner_id.clone(),
                account_public_id: input.account_public_id.to_vec(),
                page_sequence: 2,
                observed_sources: 0,
                updated_sources: 0,
                removed_sources: 1,
                has_more: false,
                page_digest: Sha256::digest(removed.envelope_sha256()).to_vec(),
                completed_at: Some(timestamp),
            },
            &MailAddressBookResultEnvelopeContextV1 {
                runtime_instance_id: context.runtime_instance_id.clone(),
                runtime_generation: context.runtime_generation,
                completed_at_unix_seconds: context.recorded_at_unix_seconds,
                completed_at_nanos: context.recorded_at_nanos,
                execution_attempt: 1,
            },
        )
        .expect("removal completion");
        MailPersonSourceSnapshotCommitV1 {
            logical_owner_id: input.logical_owner_id,
            account_public_id: input.account_public_id,
            run_id: input.run_id,
            seen_public_source_ids: Vec::new(),
            expected_removals: vec![MailPersonSourceRemovalStateV1 {
                integration_public_id,
                account_public_id: input.account_public_id,
                provider_source_contact_public_id: [9; 16],
                source_revision: 2,
            }],
            removal_pages: vec![MailPersonSourceRemovalPageCommitV1 {
                page_sequence: 2,
                source_ids: vec![[9; 16]],
                outputs: vec![
                    MailPersonSourceFetchOutputV1 {
                        semantic_order_key: mail_person_source_semantic_order_key_v1(2, 1)
                            .expect("source key"),
                        record: MailPersonSourceEnvelopeRecordV1::from_outbox(&removed),
                    },
                    MailPersonSourceFetchOutputV1 {
                        semantic_order_key: mail_person_source_semantic_order_key_v1(2, 2)
                            .expect("completion key"),
                        record: MailPersonSourceEnvelopeRecordV1::from_outbox(&completion),
                    },
                ],
            }],
            terminal_command: outputs[1].record.clone(),
            completed_at_unix_millis: 1_000,
        }
    }

    #[test]
    fn synthetic_removal_outputs_bind_all_timestamps_to_snapshot_completion() {
        let snapshot = snapshot_fixture();
        let terminal = snapshot
            .terminal_command
            .decode()
            .expect("terminal envelope");
        let terminal_payload = MailPersonSourcePageCompletedV1::decode(terminal.payload.as_slice())
            .expect("terminal page payload");
        assert!(
            terminal_payload.has_more,
            "the provider terminal page must continue into planned synthetic removals"
        );
        let terminal_page_sequence = validate_snapshot_terminal_envelope_v1(&snapshot)
            .expect("canonical terminal")
            .0;
        validate_removal_output_envelopes_v1(&snapshot, terminal_page_sequence)
            .expect("canonical removal outputs");
        validate_snapshot_commit(&snapshot).expect("canonical snapshot");

        let mut shifted_source = snapshot.clone();
        shifted_source.removal_pages[0].outputs[0].record =
            mutate_record(&snapshot.removal_pages[0].outputs[0].record, |envelope| {
                envelope.recorded_at.as_mut().expect("recorded at").seconds += 1;
                let Some(Semantics::Observation(metadata)) = envelope.semantics.as_mut() else {
                    panic!("observation semantics");
                };
                metadata.observed_at.as_mut().expect("observed at").seconds += 1;
                metadata.occurred_at.as_mut().expect("occurred at").seconds += 1;
                let mut payload = MailPersonSourceRemovedV1::decode(envelope.payload.as_slice())
                    .expect("removed payload");
                payload
                    .provenance
                    .as_mut()
                    .expect("provenance")
                    .observed_at
                    .as_mut()
                    .expect("payload observed at")
                    .seconds += 1;
                envelope.payload = payload.encode_to_vec();
            });
        let shifted_digest = shifted_source.removal_pages[0].outputs[0]
            .record
            .envelope_sha256;
        shifted_source.removal_pages[0].outputs[1].record =
            mutate_record(&snapshot.removal_pages[0].outputs[1].record, |envelope| {
                let mut payload =
                    MailPersonSourcePageCompletedV1::decode(envelope.payload.as_slice())
                        .expect("completion payload");
                payload.page_digest = Sha256::digest(shifted_digest).to_vec();
                envelope.payload = payload.encode_to_vec();
            });
        assert!(matches!(
            validate_snapshot_commit(&shifted_source),
            Err(MailAddressBookPersistenceErrorV1::InvalidInput)
        ));

        let mut shifted_completion = snapshot.clone();
        shifted_completion.removal_pages[0].outputs[1].record =
            mutate_record(&snapshot.removal_pages[0].outputs[1].record, |envelope| {
                envelope.recorded_at.as_mut().expect("recorded at").seconds += 1;
                let Some(Semantics::Result(metadata)) = envelope.semantics.as_mut() else {
                    panic!("result semantics");
                };
                metadata
                    .completed_at
                    .as_mut()
                    .expect("completed at")
                    .seconds += 1;
                let mut payload =
                    MailPersonSourcePageCompletedV1::decode(envelope.payload.as_slice())
                        .expect("completion payload");
                payload
                    .completed_at
                    .as_mut()
                    .expect("payload completed at")
                    .seconds += 1;
                envelope.payload = payload.encode_to_vec();
            });
        assert!(matches!(
            validate_snapshot_commit(&shifted_completion),
            Err(MailAddressBookPersistenceErrorV1::InvalidInput)
        ));
    }
}
