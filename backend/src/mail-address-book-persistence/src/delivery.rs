use makosh_events_protocol::{
    delivery::OutboxRecordV1, v1::durable_envelope_v1::Semantics,
    validation::envelope::decode_envelope_v1,
};
use makosh_mail_address_book_contract::{
    MailAddressBookContractV1, validate_mail_address_book_entry_upsert_rejected_v1,
    validate_mail_address_book_entry_upserted_v1,
    wire::{MailAddressBookEntryUpsertRejectedV1, MailAddressBookEntryUpsertedV1},
};
use prost::Message;
use sqlx::Row;

use crate::{
    MailAddressBookPersistenceErrorV1, MailAddressBookPersistenceV1,
    MailAddressBookUpsertAdmissionV1, PendingMailAddressBookUpsertV1,
};

const MAX_BATCH: i64 = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAddressBookCommandInboxOutcomeV1 {
    Accepted,
    DuplicateAccepted,
    DuplicateDispatching,
    DuplicateCompleted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAddressBookDispatchOutcomeV1 {
    Started,
    AlreadyDispatching,
    AlreadyCompleted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAddressBookResultStoreOutcomeV1 {
    Stored,
    AlreadyStored,
}

impl MailAddressBookPersistenceV1 {
    pub async fn accept_upsert_command(
        &self,
        admission: &MailAddressBookUpsertAdmissionV1,
        accepted_at_unix_seconds: i64,
    ) -> Result<MailAddressBookCommandInboxOutcomeV1, MailAddressBookPersistenceErrorV1> {
        validate_admission(admission, accepted_at_unix_seconds)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.mail_address_book_upsert_inbox
                (command_message_id, command_envelope_sha256, command_id, run_id,
                 logical_owner_id, account_id, contact_snapshot_reference_id,
                 contact_snapshot_sha256, expected_contact_revision,
                 contact_snapshot_declared_bytes, contact_snapshot_custody_source_proof,
                 state, execution_attempt, accepted_at_unix_seconds)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 0, 1, $12)
             ON CONFLICT DO NOTHING",
        )
        .bind(admission.command_message_id.as_slice())
        .bind(admission.command_envelope_sha256.as_slice())
        .bind(admission.command_id.as_slice())
        .bind(admission.run_id.as_slice())
        .bind(&admission.logical_owner_id)
        .bind(&admission.account_id)
        .bind(admission.contact_snapshot_reference_id.as_slice())
        .bind(admission.contact_snapshot_sha256.as_slice())
        .bind(as_i64(admission.expected_contact_revision)?)
        .bind(as_i64(admission.contact_snapshot_declared_bytes)?)
        .bind(&admission.contact_snapshot_custody_source_proof)
        .bind(accepted_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
        if inserted.rows_affected() == 1 {
            transaction.commit().await.map_err(storage_error)?;
            return Ok(MailAddressBookCommandInboxOutcomeV1::Accepted);
        }
        let rows = sqlx::query(
            "SELECT * FROM makosh_data.mail_address_book_upsert_inbox
             WHERE command_message_id = $1 OR command_id = $2
             FOR UPDATE",
        )
        .bind(admission.command_message_id.as_slice())
        .bind(admission.command_id.as_slice())
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage_error)?;
        if rows.len() != 1 {
            return Err(MailAddressBookPersistenceErrorV1::Conflict);
        }
        verify_admission_row(&rows[0], admission)?;
        let state: i16 = rows[0].try_get("state").map_err(row_error)?;
        transaction.commit().await.map_err(storage_error)?;
        match state {
            0 => Ok(MailAddressBookCommandInboxOutcomeV1::DuplicateAccepted),
            1 => Ok(MailAddressBookCommandInboxOutcomeV1::DuplicateDispatching),
            2 => Ok(MailAddressBookCommandInboxOutcomeV1::DuplicateCompleted),
            _ => Err(MailAddressBookPersistenceErrorV1::InvalidRow),
        }
    }

    pub async fn pending_upserts(
        &self,
        limit: i64,
    ) -> Result<Vec<PendingMailAddressBookUpsertV1>, MailAddressBookPersistenceErrorV1> {
        jobs_with_state(&self.pool, 0, limit).await
    }

    pub async fn uncertain_upserts(
        &self,
        limit: i64,
    ) -> Result<Vec<PendingMailAddressBookUpsertV1>, MailAddressBookPersistenceErrorV1> {
        jobs_with_state(&self.pool, 1, limit).await
    }

    pub async fn mark_dispatch_started(
        &self,
        command_id: [u8; 16],
        dispatch_started_at_unix_seconds: i64,
    ) -> Result<MailAddressBookDispatchOutcomeV1, MailAddressBookPersistenceErrorV1> {
        if zero(&command_id) || dispatch_started_at_unix_seconds <= 0 {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let row = sqlx::query(
            "SELECT state FROM makosh_data.mail_address_book_upsert_inbox
             WHERE command_id = $1 FOR UPDATE",
        )
        .bind(command_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_error)?
        .ok_or(MailAddressBookPersistenceErrorV1::NotFound)?;
        let state: i16 = row.try_get("state").map_err(row_error)?;
        let outcome = match state {
            0 => {
                let updated = sqlx::query(
                    "UPDATE makosh_data.mail_address_book_upsert_inbox
                     SET state = 1, dispatch_started_at_unix_seconds = $2
                     WHERE command_id = $1 AND state = 0",
                )
                .bind(command_id.as_slice())
                .bind(dispatch_started_at_unix_seconds)
                .execute(&mut *transaction)
                .await
                .map_err(storage_error)?;
                if updated.rows_affected() != 1 {
                    return Err(MailAddressBookPersistenceErrorV1::Conflict);
                }
                MailAddressBookDispatchOutcomeV1::Started
            }
            1 => MailAddressBookDispatchOutcomeV1::AlreadyDispatching,
            2 => MailAddressBookDispatchOutcomeV1::AlreadyCompleted,
            _ => return Err(MailAddressBookPersistenceErrorV1::InvalidRow),
        };
        transaction.commit().await.map_err(storage_error)?;
        Ok(outcome)
    }

    pub async fn complete_upsert_command(
        &self,
        command_id: [u8; 16],
        result: &OutboxRecordV1,
        created_at_unix_seconds: i64,
    ) -> Result<MailAddressBookResultStoreOutcomeV1, MailAddressBookPersistenceErrorV1> {
        if zero(&command_id) || created_at_unix_seconds <= 0 {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let inbox = sqlx::query(
            "SELECT * FROM makosh_data.mail_address_book_upsert_inbox
             WHERE command_id = $1 FOR UPDATE",
        )
        .bind(command_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_error)?
        .ok_or(MailAddressBookPersistenceErrorV1::NotFound)?;
        let pending = job_from_row(inbox)?;
        verify_result_record(result, &pending.admission)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.mail_address_book_upsert_result_outbox
                (message_id, envelope_sha256, exact_envelope_bytes, command_id,
                 command_message_id, created_at_unix_seconds)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT DO NOTHING",
        )
        .bind(result.message_id().as_slice())
        .bind(result.envelope_sha256().as_slice())
        .bind(result.exact_bytes())
        .bind(command_id.as_slice())
        .bind(pending.admission.command_message_id.as_slice())
        .bind(created_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
        let outcome = if inserted.rows_affected() == 1 {
            MailAddressBookResultStoreOutcomeV1::Stored
        } else {
            let existing = sqlx::query(
                "SELECT message_id, envelope_sha256, exact_envelope_bytes,
                        command_message_id
                 FROM makosh_data.mail_address_book_upsert_result_outbox
                 WHERE command_id = $1",
            )
            .bind(command_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(storage_error)?
            .ok_or(MailAddressBookPersistenceErrorV1::Conflict)?;
            verify_result_row(&existing, pending.admission.command_message_id, result)?;
            MailAddressBookResultStoreOutcomeV1::AlreadyStored
        };
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_address_book_upsert_inbox
             SET state = 2 WHERE command_id = $1 AND state <> 2",
        )
        .bind(command_id.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
        if updated.rows_affected() > 1 {
            return Err(MailAddressBookPersistenceErrorV1::InvalidRow);
        }
        transaction.commit().await.map_err(storage_error)?;
        Ok(outcome)
    }

    pub async fn pending_results(
        &self,
        limit: i64,
    ) -> Result<Vec<OutboxRecordV1>, MailAddressBookPersistenceErrorV1> {
        validate_limit(limit)?;
        let rows = sqlx::query(
            "SELECT result.message_id, result.envelope_sha256, result.exact_envelope_bytes,
                    inbox.command_message_id, inbox.command_envelope_sha256,
                    inbox.command_id, inbox.run_id, inbox.logical_owner_id,
                    inbox.account_id, inbox.contact_snapshot_reference_id,
                    inbox.contact_snapshot_sha256, inbox.expected_contact_revision,
                    inbox.contact_snapshot_declared_bytes,
                    inbox.contact_snapshot_custody_source_proof
             FROM makosh_data.mail_address_book_upsert_result_outbox result
             JOIN makosh_data.mail_address_book_upsert_inbox inbox
               ON inbox.command_message_id = result.command_message_id
              AND inbox.command_id = result.command_id
             WHERE result.published_at_unix_seconds IS NULL
             ORDER BY result.created_at_unix_seconds, result.message_id
             LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?;
        rows.into_iter().map(result_from_row).collect()
    }

    pub async fn mark_result_published(
        &self,
        message_id: [u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<bool, MailAddressBookPersistenceErrorV1> {
        if zero(&message_id) || published_at_unix_seconds <= 0 {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "UPDATE makosh_data.mail_address_book_upsert_result_outbox
             SET published_at_unix_seconds = $1
             WHERE message_id = $2 AND published_at_unix_seconds IS NULL",
        )
        .bind(published_at_unix_seconds)
        .bind(message_id.as_slice())
        .execute(&self.pool)
        .await
        .map(|result| result.rows_affected() == 1)
        .map_err(storage_error)
    }
}

async fn jobs_with_state(
    pool: &sqlx::PgPool,
    state: i16,
    limit: i64,
) -> Result<Vec<PendingMailAddressBookUpsertV1>, MailAddressBookPersistenceErrorV1> {
    validate_limit(limit)?;
    let rows = sqlx::query(
        "SELECT * FROM makosh_data.mail_address_book_upsert_inbox
         WHERE state = $1
         ORDER BY accepted_at_unix_seconds, command_message_id
         LIMIT $2",
    )
    .bind(state)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(storage_error)?;
    rows.into_iter().map(job_from_row).collect()
}

fn job_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<PendingMailAddressBookUpsertV1, MailAddressBookPersistenceErrorV1> {
    let execution_attempt: i32 = row.try_get("execution_attempt").map_err(row_error)?;
    let pending = PendingMailAddressBookUpsertV1 {
        admission: admission_from_row(&row)?,
        target_snapshot_receipt: target_snapshot_receipt_from_row(&row)?,
        execution_attempt: execution_attempt.try_into().map_err(row_error)?,
    };
    validate_admission(&pending.admission, 1)?;
    if pending.execution_attempt == 0 {
        return Err(MailAddressBookPersistenceErrorV1::InvalidRow);
    }
    Ok(pending)
}

fn target_snapshot_receipt_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<Option<crate::MailAddressBookTargetSnapshotReceiptV1>, MailAddressBookPersistenceErrorV1>
{
    let reference_id = optional_row_id::<16>(row, "target_contact_snapshot_reference_id")?;
    let receipt_sha256 = optional_row_id::<32>(row, "target_contact_snapshot_receipt_sha256")?;
    let recorded_at: Option<i64> = row
        .try_get("snapshot_custody_recorded_at_unix_seconds")
        .map_err(row_error)?;
    match (reference_id, receipt_sha256, recorded_at) {
        (None, None, None) => Ok(None),
        (Some(reference_id), Some(receipt_sha256), Some(recorded_at)) if recorded_at > 0 => {
            Ok(Some(crate::MailAddressBookTargetSnapshotReceiptV1 {
                reference_id,
                receipt_sha256,
            }))
        }
        _ => Err(MailAddressBookPersistenceErrorV1::InvalidRow),
    }
}

fn verify_admission_row(
    row: &sqlx::postgres::PgRow,
    admission: &MailAddressBookUpsertAdmissionV1,
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    let persisted = admission_from_row(row)?;
    if &persisted == admission {
        Ok(())
    } else {
        Err(MailAddressBookPersistenceErrorV1::Conflict)
    }
}

fn verify_result_record(
    record: &OutboxRecordV1,
    admission: &MailAddressBookUpsertAdmissionV1,
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes()).map_err(row_error)?;
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(MailAddressBookPersistenceErrorV1::WrongContract)?;
    let upserted = MailAddressBookContractV1::EntryUpserted.reference();
    let rejected = MailAddressBookContractV1::EntryUpsertRejected.reference();
    let is_upserted = contract.owner == upserted.owner
        && contract.name == upserted.name
        && contract.major == upserted.major
        && contract.revision == upserted.revision
        && contract.schema_sha256 == upserted.schema_sha256;
    let is_rejected = contract.owner == rejected.owner
        && contract.name == rejected.name
        && contract.major == rejected.major
        && contract.revision == rejected.revision
        && contract.schema_sha256 == rejected.schema_sha256;
    if (!is_upserted && !is_rejected)
        || envelope.partition_key != admission.run_id
        || envelope.correlation_id != admission.run_id
        || envelope.causation_message_id != admission.command_message_id
    {
        return Err(MailAddressBookPersistenceErrorV1::WrongContract);
    }
    let Some(Semantics::Result(metadata)) = envelope.semantics.as_ref() else {
        return Err(MailAddressBookPersistenceErrorV1::WrongContract);
    };
    if metadata.command_id != admission.command_id
        || metadata.command_message_id != admission.command_message_id
    {
        return Err(MailAddressBookPersistenceErrorV1::WrongContract);
    }
    if is_upserted {
        let payload = MailAddressBookEntryUpsertedV1::decode(envelope.payload.as_slice())
            .map_err(row_error)?;
        validate_mail_address_book_entry_upserted_v1(&payload)
            .map_err(|_| MailAddressBookPersistenceErrorV1::WrongContract)?;
        if payload.command_id != admission.command_id || payload.run_id != admission.run_id {
            return Err(MailAddressBookPersistenceErrorV1::WrongContract);
        }
    } else {
        let payload = MailAddressBookEntryUpsertRejectedV1::decode(envelope.payload.as_slice())
            .map_err(row_error)?;
        validate_mail_address_book_entry_upsert_rejected_v1(&payload)
            .map_err(|_| MailAddressBookPersistenceErrorV1::WrongContract)?;
        if payload.command_id != admission.command_id || payload.run_id != admission.run_id {
            return Err(MailAddressBookPersistenceErrorV1::WrongContract);
        }
    }
    Ok(())
}

fn verify_result_row(
    row: &sqlx::postgres::PgRow,
    command_message_id: [u8; 16],
    result: &OutboxRecordV1,
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    let stored_message_id: [u8; 16] = row_id(row, "message_id")?;
    let stored_hash: [u8; 32] = row_id(row, "envelope_sha256")?;
    let stored_command: [u8; 16] = row_id(row, "command_message_id")?;
    let stored_bytes: Vec<u8> = row.try_get("exact_envelope_bytes").map_err(row_error)?;
    if stored_message_id != *result.message_id()
        || stored_hash != *result.envelope_sha256()
        || stored_command != command_message_id
        || stored_bytes.as_slice() != result.exact_bytes()
    {
        return Err(MailAddressBookPersistenceErrorV1::Conflict);
    }
    Ok(())
}

fn result_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<OutboxRecordV1, MailAddressBookPersistenceErrorV1> {
    let message_id: [u8; 16] = row_id(&row, "message_id")?;
    let envelope_sha256: [u8; 32] = row_id(&row, "envelope_sha256")?;
    let exact_bytes: Vec<u8> = row.try_get("exact_envelope_bytes").map_err(row_error)?;
    let record = OutboxRecordV1::accept(exact_bytes).map_err(row_error)?;
    if record.message_id() != &message_id || record.envelope_sha256() != &envelope_sha256 {
        return Err(MailAddressBookPersistenceErrorV1::HashMismatch);
    }
    let admission = admission_from_row(&row)?;
    verify_result_record(&record, &admission)?;
    Ok(record)
}

fn admission_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<MailAddressBookUpsertAdmissionV1, MailAddressBookPersistenceErrorV1> {
    let expected_contact_revision: i64 = row
        .try_get("expected_contact_revision")
        .map_err(row_error)?;
    let declared_bytes: i64 = row
        .try_get("contact_snapshot_declared_bytes")
        .map_err(row_error)?;
    let admission = MailAddressBookUpsertAdmissionV1 {
        command_message_id: row_id(row, "command_message_id")?,
        command_envelope_sha256: row_id(row, "command_envelope_sha256")?,
        command_id: row_id(row, "command_id")?,
        run_id: row_id(row, "run_id")?,
        logical_owner_id: row.try_get("logical_owner_id").map_err(row_error)?,
        account_id: row.try_get("account_id").map_err(row_error)?,
        contact_snapshot_reference_id: row_id(row, "contact_snapshot_reference_id")?,
        contact_snapshot_sha256: row_id(row, "contact_snapshot_sha256")?,
        expected_contact_revision: expected_contact_revision.try_into().map_err(row_error)?,
        contact_snapshot_declared_bytes: declared_bytes.try_into().map_err(row_error)?,
        contact_snapshot_custody_source_proof: row
            .try_get("contact_snapshot_custody_source_proof")
            .map_err(row_error)?,
    };
    validate_admission(&admission, 1)?;
    Ok(admission)
}

fn validate_admission(
    admission: &MailAddressBookUpsertAdmissionV1,
    timestamp: i64,
) -> Result<(), MailAddressBookPersistenceErrorV1> {
    if zero(&admission.command_message_id)
        || zero(&admission.command_envelope_sha256)
        || zero(&admission.command_id)
        || zero(&admission.run_id)
        || !valid_identity(&admission.logical_owner_id, 128)
        || !valid_identity(&admission.account_id, 256)
        || zero(&admission.contact_snapshot_reference_id)
        || zero(&admission.contact_snapshot_sha256)
        || admission.expected_contact_revision == 0
        || !(1..=32 * 1024).contains(&admission.contact_snapshot_declared_bytes)
        || admission.contact_snapshot_custody_source_proof.is_empty()
        || admission.contact_snapshot_custody_source_proof.len() > 4096
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

fn as_i64(value: u64) -> Result<i64, MailAddressBookPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| MailAddressBookPersistenceErrorV1::InvalidInput)
}

fn row_id<const WIDTH: usize>(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<[u8; WIDTH], MailAddressBookPersistenceErrorV1> {
    let value: Vec<u8> = row.try_get(column).map_err(row_error)?;
    let value: [u8; WIDTH] = value.try_into().map_err(row_error)?;
    (!zero(&value))
        .then_some(value)
        .ok_or(MailAddressBookPersistenceErrorV1::InvalidRow)
}

fn optional_row_id<const WIDTH: usize>(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<Option<[u8; WIDTH]>, MailAddressBookPersistenceErrorV1> {
    row.try_get::<Option<Vec<u8>>, _>(column)
        .map_err(row_error)?
        .map(|value| {
            let value: [u8; WIDTH] = value.try_into().map_err(row_error)?;
            (!zero(&value))
                .then_some(value)
                .ok_or(MailAddressBookPersistenceErrorV1::InvalidRow)
        })
        .transpose()
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

#[cfg(test)]
mod tests {
    use makosh_mail_address_book_contract::{
        MailAddressBookResultEnvelopeContextV1, build_mail_address_book_entry_upserted_result_v1,
        wire::MailAddressBookEntryUpsertedV1,
    };

    use super::*;

    fn admission() -> MailAddressBookUpsertAdmissionV1 {
        MailAddressBookUpsertAdmissionV1 {
            command_message_id: [1; 16],
            command_envelope_sha256: [2; 32],
            command_id: [3; 16],
            run_id: [4; 16],
            logical_owner_id: "owner".to_owned(),
            account_id: "mail-account".to_owned(),
            contact_snapshot_reference_id: [5; 16],
            contact_snapshot_sha256: [6; 32],
            expected_contact_revision: 7,
            contact_snapshot_declared_bytes: 128,
            contact_snapshot_custody_source_proof: vec![8; 32],
        }
    }

    #[test]
    fn admission_is_bounded_and_rejects_foreign_identity_shapes() {
        let valid = admission();
        assert_eq!(validate_admission(&valid, 1), Ok(()));

        let mut invalid = valid.clone();
        invalid.account_id = "mail/account".to_owned();
        assert_eq!(
            validate_admission(&invalid, 1),
            Err(MailAddressBookPersistenceErrorV1::InvalidInput)
        );

        let mut oversized = valid;
        oversized.contact_snapshot_custody_source_proof = vec![9; 4097];
        assert_eq!(
            validate_admission(&oversized, 1),
            Err(MailAddressBookPersistenceErrorV1::InvalidInput)
        );
    }

    #[test]
    fn terminal_result_must_be_correlated_to_the_exact_admitted_command() {
        let admission = admission();
        let result = build_mail_address_book_entry_upserted_result_v1(
            admission.command_message_id,
            MailAddressBookEntryUpsertedV1 {
                command_id: admission.command_id.to_vec(),
                run_id: admission.run_id.to_vec(),
                provider_entry_id: "people/c1".to_owned(),
                provider_etag: "etag-1".to_owned(),
                applied_contact_revision: admission.expected_contact_revision,
                provider_kind: makosh_mail_address_book_contract::wire::MailAddressBookProviderKindV1::MailAddressBookProviderKindGooglePeople as i32,
            },
            &MailAddressBookResultEnvelopeContextV1 {
                runtime_instance_id: "mail-runtime-instance".to_owned(),
                runtime_generation: 1,
                completed_at_unix_seconds: 10,
                completed_at_nanos: 0,
                execution_attempt: 1,
            },
        )
        .expect("terminal result");
        assert_eq!(verify_result_record(&result, &admission), Ok(()));

        let mut different_command = admission;
        different_command.command_message_id = [9; 16];
        assert_eq!(
            verify_result_record(&result, &different_command),
            Err(MailAddressBookPersistenceErrorV1::WrongContract)
        );
    }
}
