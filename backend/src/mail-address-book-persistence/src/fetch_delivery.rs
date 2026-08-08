use std::collections::BTreeSet;

use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ResultOutcomeV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_mail_address_book_contract::{
    MailAddressBookContractV1, validate_mail_address_book_entry_observed_v1,
    validate_mail_address_book_page_completed_v1, validate_mail_address_book_page_rejected_v1,
    wire::{
        MailAddressBookEntryObservedV1, MailAddressBookPageCompletedV1,
        MailAddressBookPageRejectedV1,
    },
};
use prost::Message;
use sqlx::Row;

use crate::{
    MailAddressBookFetchAdmissionV1, MailAddressBookPersistenceErrorV1,
    MailAddressBookPersistenceV1, PendingMailAddressBookFetchV1,
};

const MAX_BATCH: i64 = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAddressBookFetchInboxOutcomeV1 {
    Accepted,
    DuplicateAccepted,
    DuplicateCompleted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAddressBookFetchStoreOutcomeV1 {
    Stored,
    AlreadyStored,
}

impl MailAddressBookPersistenceV1 {
    pub async fn accept_fetch_command(
        &self,
        admission: &MailAddressBookFetchAdmissionV1,
        accepted_at_unix_seconds: i64,
    ) -> Result<MailAddressBookFetchInboxOutcomeV1, MailAddressBookPersistenceErrorV1> {
        validate_fetch_admission(admission, accepted_at_unix_seconds)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.mail_address_book_fetch_inbox
                (command_message_id, command_envelope_sha256, command_id, run_id,
                 logical_owner_id, account_id, page_sequence, continuation_cursor,
                 page_size, state, execution_attempt, accepted_at_unix_seconds)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,0,1,$10)
             ON CONFLICT DO NOTHING",
        )
        .bind(admission.command_message_id.as_slice())
        .bind(admission.command_envelope_sha256.as_slice())
        .bind(admission.command_id.as_slice())
        .bind(admission.run_id.as_slice())
        .bind(&admission.logical_owner_id)
        .bind(&admission.account_id)
        .bind(as_i64(admission.page_sequence)?)
        .bind(admission.continuation_cursor.as_deref())
        .bind(i32::try_from(admission.page_size).map_err(input_error)?)
        .bind(accepted_at_unix_seconds)
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        if inserted.rows_affected() == 1 {
            return Ok(MailAddressBookFetchInboxOutcomeV1::Accepted);
        }
        let rows = sqlx::query(
            "SELECT * FROM makosh_data.mail_address_book_fetch_inbox
             WHERE command_message_id=$1 OR command_id=$2",
        )
        .bind(admission.command_message_id.as_slice())
        .bind(admission.command_id.as_slice())
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?;
        if rows.len() != 1 || fetch_admission_from_row(&rows[0])? != *admission {
            return Err(MailAddressBookPersistenceErrorV1::Conflict);
        }
        match rows[0].try_get::<i16, _>("state").map_err(row_error)? {
            0 => Ok(MailAddressBookFetchInboxOutcomeV1::DuplicateAccepted),
            1 => Ok(MailAddressBookFetchInboxOutcomeV1::DuplicateCompleted),
            _ => Err(MailAddressBookPersistenceErrorV1::InvalidRow),
        }
    }

    pub async fn pending_fetches(
        &self,
        limit: i64,
    ) -> Result<Vec<PendingMailAddressBookFetchV1>, MailAddressBookPersistenceErrorV1> {
        validate_limit(limit)?;
        let rows = sqlx::query(
            "SELECT * FROM makosh_data.mail_address_book_fetch_inbox
             WHERE state=0 ORDER BY accepted_at_unix_seconds,command_message_id LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?;
        rows.into_iter().map(fetch_from_row).collect()
    }

    pub async fn complete_fetch_command(
        &self,
        command_id: [u8; 16],
        records: &[OutboxRecordV1],
        completed_at_unix_seconds: i64,
    ) -> Result<MailAddressBookFetchStoreOutcomeV1, MailAddressBookPersistenceErrorV1> {
        if zero(&command_id) || records.is_empty() || completed_at_unix_seconds <= 0 {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let row = sqlx::query(
            "SELECT * FROM makosh_data.mail_address_book_fetch_inbox
             WHERE command_id=$1 FOR UPDATE",
        )
        .bind(command_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_error)?
        .ok_or(MailAddressBookPersistenceErrorV1::NotFound)?;
        let pending = fetch_from_row(row)?;
        verify_fetch_records(records, &pending.admission)?;
        if pending_fetch_state(&mut transaction, command_id).await? == 1 {
            verify_stored_records(&mut transaction, command_id, records).await?;
            transaction.commit().await.map_err(storage_error)?;
            return Ok(MailAddressBookFetchStoreOutcomeV1::AlreadyStored);
        }
        for (ordinal, record) in records.iter().enumerate() {
            sqlx::query(
                "INSERT INTO makosh_data.mail_address_book_fetch_outbox
                    (message_id,envelope_sha256,exact_envelope_bytes,command_id,
                     command_message_id,ordinal,created_at_unix_seconds)
                 VALUES ($1,$2,$3,$4,$5,$6,$7)",
            )
            .bind(record.message_id().as_slice())
            .bind(record.envelope_sha256().as_slice())
            .bind(record.exact_bytes())
            .bind(command_id.as_slice())
            .bind(pending.admission.command_message_id.as_slice())
            .bind(i32::try_from(ordinal).map_err(input_error)?)
            .bind(completed_at_unix_seconds)
            .execute(&mut *transaction)
            .await
            .map_err(storage_error)?;
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_address_book_fetch_inbox
             SET state=1,completed_at_unix_seconds=$2 WHERE command_id=$1 AND state=0",
        )
        .bind(command_id.as_slice())
        .bind(completed_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
        if updated.rows_affected() != 1 {
            return Err(MailAddressBookPersistenceErrorV1::Conflict);
        }
        transaction.commit().await.map_err(storage_error)?;
        Ok(MailAddressBookFetchStoreOutcomeV1::Stored)
    }

    pub async fn pending_fetch_events(
        &self,
        limit: i64,
    ) -> Result<Vec<OutboxRecordV1>, MailAddressBookPersistenceErrorV1> {
        validate_limit(limit)?;
        let rows = sqlx::query(
            "SELECT message_id,envelope_sha256,exact_envelope_bytes
             FROM makosh_data.mail_address_book_fetch_outbox
             WHERE published_at_unix_seconds IS NULL
             ORDER BY created_at_unix_seconds,command_id,ordinal LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?;
        rows.into_iter().map(outbox_from_row).collect()
    }

    pub async fn mark_fetch_event_published(
        &self,
        message_id: [u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<bool, MailAddressBookPersistenceErrorV1> {
        if zero(&message_id) || published_at_unix_seconds <= 0 {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "UPDATE makosh_data.mail_address_book_fetch_outbox
             SET published_at_unix_seconds=$1
             WHERE message_id=$2 AND published_at_unix_seconds IS NULL",
        )
        .bind(published_at_unix_seconds)
        .bind(message_id.as_slice())
        .execute(&self.pool)
        .await
        .map(|result| result.rows_affected() == 1)
        .map_err(storage_error)
    }
}

async fn pending_fetch_state(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    command_id: [u8; 16],
) -> Result<i16, MailAddressBookPersistenceErrorV1> {
    sqlx::query_scalar(
        "SELECT state FROM makosh_data.mail_address_book_fetch_inbox WHERE command_id=$1",
    )
    .bind(command_id.as_slice())
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage_error)
}

async fn verify_stored_records(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    command_id: [u8; 16],
    records: &[OutboxRecordV1],
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    let rows = sqlx::query(
        "SELECT exact_envelope_bytes FROM makosh_data.mail_address_book_fetch_outbox
         WHERE command_id=$1 ORDER BY ordinal",
    )
    .bind(command_id.as_slice())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage_error)?;
    if rows.len() != records.len()
        || rows.iter().zip(records).any(|(row, record)| {
            row.try_get::<Vec<u8>, _>("exact_envelope_bytes")
                .map_or(true, |bytes| bytes.as_slice() != record.exact_bytes())
        })
    {
        return Err(MailAddressBookPersistenceErrorV1::Conflict);
    }
    Ok(())
}

fn verify_fetch_records(
    records: &[OutboxRecordV1],
    admission: &MailAddressBookFetchAdmissionV1,
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    if records.len() > usize::try_from(admission.page_size).map_err(input_error)? + 1 {
        return Err(MailAddressBookPersistenceErrorV1::WrongContract);
    }
    let mut ids = BTreeSet::new();
    for (index, record) in records.iter().enumerate() {
        if !ids.insert(*record.message_id()) {
            return Err(MailAddressBookPersistenceErrorV1::Conflict);
        }
        let envelope = decode_envelope_v1(record.exact_bytes()).map_err(row_error)?;
        let contract = envelope
            .contract
            .as_ref()
            .ok_or(MailAddressBookPersistenceErrorV1::WrongContract)?;
        if envelope.partition_key != admission.run_id
            || envelope.correlation_id != admission.run_id
            || envelope.causation_message_id != admission.command_message_id
        {
            return Err(MailAddressBookPersistenceErrorV1::WrongContract);
        }
        let last = index + 1 == records.len();
        if !last {
            exact_contract(contract, MailAddressBookContractV1::EntryObserved)?;
            let payload = MailAddressBookEntryObservedV1::decode(envelope.payload.as_slice())
                .map_err(row_error)?;
            validate_mail_address_book_entry_observed_v1(&payload)
                .map_err(|_| MailAddressBookPersistenceErrorV1::WrongContract)?;
            if payload.run_id != admission.run_id
                || payload.logical_owner_id != admission.logical_owner_id
                || payload.account_id != admission.account_id
                || payload.page_sequence != admission.page_sequence
                || !matches!(envelope.semantics, Some(Semantics::Observation(_)))
            {
                return Err(MailAddressBookPersistenceErrorV1::WrongContract);
            }
        } else if contract.name == MailAddressBookContractV1::PageCompleted.name() {
            exact_contract(contract, MailAddressBookContractV1::PageCompleted)?;
            let payload = MailAddressBookPageCompletedV1::decode(envelope.payload.as_slice())
                .map_err(row_error)?;
            validate_mail_address_book_page_completed_v1(&payload)
                .map_err(|_| MailAddressBookPersistenceErrorV1::WrongContract)?;
            verify_terminal(
                &envelope,
                &payload.command_id,
                &payload.run_id,
                admission,
                ResultOutcomeV1::Succeeded,
            )?;
            if payload.page_sequence != admission.page_sequence
                || usize::try_from(payload.observed_entries).map_err(input_error)?
                    != records.len() - 1
            {
                return Err(MailAddressBookPersistenceErrorV1::WrongContract);
            }
        } else {
            exact_contract(contract, MailAddressBookContractV1::PageRejected)?;
            let payload = MailAddressBookPageRejectedV1::decode(envelope.payload.as_slice())
                .map_err(row_error)?;
            validate_mail_address_book_page_rejected_v1(&payload)
                .map_err(|_| MailAddressBookPersistenceErrorV1::WrongContract)?;
            verify_terminal(
                &envelope,
                &payload.command_id,
                &payload.run_id,
                admission,
                ResultOutcomeV1::Rejected,
            )?;
            if records.len() != 1 {
                return Err(MailAddressBookPersistenceErrorV1::WrongContract);
            }
        }
    }
    Ok(())
}

fn verify_terminal(
    envelope: &makosh_events_protocol::v1::DurableEnvelopeV1,
    command_id: &[u8],
    run_id: &[u8],
    admission: &MailAddressBookFetchAdmissionV1,
    outcome: ResultOutcomeV1,
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    let Some(Semantics::Result(result)) = envelope.semantics.as_ref() else {
        return Err(MailAddressBookPersistenceErrorV1::WrongContract);
    };
    if command_id != admission.command_id
        || run_id != admission.run_id
        || result.command_id != admission.command_id
        || result.command_message_id != admission.command_message_id
        || result.outcome != outcome as i32
    {
        return Err(MailAddressBookPersistenceErrorV1::WrongContract);
    }
    Ok(())
}

fn exact_contract(
    actual: &makosh_events_protocol::v1::ContractRefV1,
    expected: MailAddressBookContractV1,
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    let expected = expected.reference();
    if actual.owner == expected.owner
        && actual.name == expected.name
        && actual.major == expected.major
        && actual.revision == expected.revision
        && actual.schema_sha256 == expected.schema_sha256
    {
        Ok(())
    } else {
        Err(MailAddressBookPersistenceErrorV1::WrongContract)
    }
}

fn fetch_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<PendingMailAddressBookFetchV1, MailAddressBookPersistenceErrorV1> {
    let execution_attempt: i32 = row.try_get("execution_attempt").map_err(row_error)?;
    Ok(PendingMailAddressBookFetchV1 {
        admission: fetch_admission_from_row(&row)?,
        execution_attempt: execution_attempt.try_into().map_err(row_error)?,
    })
}

fn fetch_admission_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<MailAddressBookFetchAdmissionV1, MailAddressBookPersistenceErrorV1> {
    let admission = MailAddressBookFetchAdmissionV1 {
        command_message_id: id(row, "command_message_id")?,
        command_envelope_sha256: id(row, "command_envelope_sha256")?,
        command_id: id(row, "command_id")?,
        run_id: id(row, "run_id")?,
        logical_owner_id: row.try_get("logical_owner_id").map_err(row_error)?,
        account_id: row.try_get("account_id").map_err(row_error)?,
        page_sequence: row
            .try_get::<i64, _>("page_sequence")
            .map_err(row_error)?
            .try_into()
            .map_err(row_error)?,
        continuation_cursor: row.try_get("continuation_cursor").map_err(row_error)?,
        page_size: row
            .try_get::<i32, _>("page_size")
            .map_err(row_error)?
            .try_into()
            .map_err(row_error)?,
    };
    validate_fetch_admission(&admission, 1)?;
    Ok(admission)
}

fn outbox_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<OutboxRecordV1, MailAddressBookPersistenceErrorV1> {
    let message_id: [u8; 16] = id(&row, "message_id")?;
    let hash: [u8; 32] = id(&row, "envelope_sha256")?;
    let bytes: Vec<u8> = row.try_get("exact_envelope_bytes").map_err(row_error)?;
    let record = OutboxRecordV1::accept(bytes).map_err(row_error)?;
    if record.message_id() != &message_id || record.envelope_sha256() != &hash {
        return Err(MailAddressBookPersistenceErrorV1::HashMismatch);
    }
    Ok(record)
}

fn validate_fetch_admission(
    admission: &MailAddressBookFetchAdmissionV1,
    timestamp: i64,
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    if zero(&admission.command_message_id)
        || zero(&admission.command_envelope_sha256)
        || zero(&admission.command_id)
        || zero(&admission.run_id)
        || !valid_identity(&admission.logical_owner_id, 128)
        || !valid_identity(&admission.account_id, 256)
        || admission.page_sequence == 0
        || admission
            .continuation_cursor
            .as_ref()
            .is_some_and(|value| value.is_empty() || value.len() > 4096)
        || !(1..=500).contains(&admission.page_size)
        || timestamp <= 0
    {
        return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_limit(limit: i64) -> Result<(), MailAddressBookPersistenceErrorV1> {
    if (1..=MAX_BATCH).contains(&limit) {
        Ok(())
    } else {
        Err(MailAddressBookPersistenceErrorV1::InvalidInput)
    }
}

fn valid_identity(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
}

fn id<const WIDTH: usize>(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<[u8; WIDTH], MailAddressBookPersistenceErrorV1> {
    let value: [u8; WIDTH] = row
        .try_get::<Vec<u8>, _>(column)
        .map_err(row_error)?
        .try_into()
        .map_err(row_error)?;
    (!zero(&value))
        .then_some(value)
        .ok_or(MailAddressBookPersistenceErrorV1::InvalidRow)
}

fn as_i64(value: u64) -> Result<i64, MailAddressBookPersistenceErrorV1> {
    value.try_into().map_err(input_error)
}
fn zero(value: &[u8]) -> bool {
    value.iter().all(|byte| *byte == 0)
}
fn storage_error<T>(_: T) -> MailAddressBookPersistenceErrorV1 {
    MailAddressBookPersistenceErrorV1::StorageUnavailable
}
fn row_error<T>(_: T) -> MailAddressBookPersistenceErrorV1 {
    MailAddressBookPersistenceErrorV1::InvalidRow
}
fn input_error<T>(_: T) -> MailAddressBookPersistenceErrorV1 {
    MailAddressBookPersistenceErrorV1::InvalidInput
}
