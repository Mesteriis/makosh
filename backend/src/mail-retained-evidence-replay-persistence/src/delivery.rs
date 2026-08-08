//! Mail-owned durable replay command delivery and terminal result outbox.

use makosh_events_protocol::{
    delivery::OutboxRecordV1, v1::durable_envelope_v1::Semantics,
    validation::envelope::decode_envelope_v1,
};
use makosh_mail_retained_evidence_replay_contract::{
    mail_replay_result_contract_reference_v1, validate_mail_replay_result_v1,
    wire::ReplayMailEvidenceResultV1,
};
use prost::Message;
use sqlx::Row;

use crate::{MailRetainedEvidenceReplayPersistenceV1, RetainedMailReplayErrorV1};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailReplayCommandAdmissionV1 {
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub operation_id: [u8; 16],
    pub logical_owner_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailReplayCommandInboxOutcomeV1 {
    Accepted,
    DuplicateAccepted,
    DuplicateCompleted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailReplayResultStoreOutcomeV1 {
    Stored,
    AlreadyStored,
}

impl MailRetainedEvidenceReplayPersistenceV1 {
    pub async fn accept_replay_command(
        &self,
        admission: &MailReplayCommandAdmissionV1,
        accepted_at_unix_seconds: i64,
    ) -> Result<MailReplayCommandInboxOutcomeV1, RetainedMailReplayErrorV1> {
        validate_admission(admission, accepted_at_unix_seconds)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.mail_retained_evidence_replay_command_inbox
                (message_id, envelope_sha256, operation_id, logical_owner_id, state,
                 accepted_at_unix_seconds)
             VALUES ($1, $2, $3, $4, 0, $5)
             ON CONFLICT DO NOTHING",
        )
        .bind(admission.command_message_id.as_slice())
        .bind(admission.command_envelope_sha256.as_slice())
        .bind(admission.operation_id.as_slice())
        .bind(&admission.logical_owner_id)
        .bind(accepted_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
        if inserted.rows_affected() == 1 {
            transaction.commit().await.map_err(storage_error)?;
            return Ok(MailReplayCommandInboxOutcomeV1::Accepted);
        }
        let row = sqlx::query(
            "SELECT message_id, envelope_sha256, operation_id, logical_owner_id, state
             FROM makosh_data.mail_retained_evidence_replay_command_inbox
             WHERE message_id = $1 OR operation_id = $2
             FOR UPDATE",
        )
        .bind(admission.command_message_id.as_slice())
        .bind(admission.operation_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_error)?
        .ok_or(RetainedMailReplayErrorV1::Conflict)?;
        verify_command_row(&row, admission)?;
        let state: i16 = row.try_get("state").map_err(row_error)?;
        transaction.commit().await.map_err(storage_error)?;
        match state {
            0 => Ok(MailReplayCommandInboxOutcomeV1::DuplicateAccepted),
            1 => Ok(MailReplayCommandInboxOutcomeV1::DuplicateCompleted),
            _ => Err(RetainedMailReplayErrorV1::InvalidRow),
        }
    }

    pub async fn complete_replay_command(
        &self,
        admission: &MailReplayCommandAdmissionV1,
        result: &OutboxRecordV1,
        created_at_unix_seconds: i64,
    ) -> Result<MailReplayResultStoreOutcomeV1, RetainedMailReplayErrorV1> {
        validate_admission(admission, created_at_unix_seconds)?;
        verify_result_record(result, admission)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let inbox = sqlx::query(
            "SELECT message_id, envelope_sha256, operation_id, logical_owner_id, state
             FROM makosh_data.mail_retained_evidence_replay_command_inbox
             WHERE message_id = $1
             FOR UPDATE",
        )
        .bind(admission.command_message_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_error)?
        .ok_or(RetainedMailReplayErrorV1::NotFound)?;
        verify_command_row(&inbox, admission)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.mail_retained_evidence_replay_result_outbox
                (message_id, envelope_sha256, exact_envelope_bytes, operation_id,
                 command_message_id, created_at_unix_seconds)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT DO NOTHING",
        )
        .bind(result.message_id().as_slice())
        .bind(result.envelope_sha256().as_slice())
        .bind(result.exact_bytes())
        .bind(admission.operation_id.as_slice())
        .bind(admission.command_message_id.as_slice())
        .bind(created_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
        let outcome = if inserted.rows_affected() == 1 {
            MailReplayResultStoreOutcomeV1::Stored
        } else {
            let existing = sqlx::query(
                "SELECT message_id, envelope_sha256, exact_envelope_bytes, command_message_id
                 FROM makosh_data.mail_retained_evidence_replay_result_outbox
                 WHERE operation_id = $1",
            )
            .bind(admission.operation_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(storage_error)?
            .ok_or(RetainedMailReplayErrorV1::Conflict)?;
            verify_result_row(&existing, admission.command_message_id, result)?;
            MailReplayResultStoreOutcomeV1::AlreadyStored
        };
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_retained_evidence_replay_command_inbox
             SET state = 1
             WHERE message_id = $1 AND state = 0",
        )
        .bind(admission.command_message_id.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
        if updated.rows_affected() > 1 {
            return Err(RetainedMailReplayErrorV1::InvalidRow);
        }
        transaction.commit().await.map_err(storage_error)?;
        Ok(outcome)
    }

    pub async fn pending_replay_results(
        &self,
        limit: i64,
    ) -> Result<Vec<OutboxRecordV1>, RetainedMailReplayErrorV1> {
        if !(1..=256).contains(&limit) {
            return Err(RetainedMailReplayErrorV1::InvalidInput);
        }
        let rows = sqlx::query(
            "SELECT result.message_id, result.envelope_sha256, result.exact_envelope_bytes,
                    result.operation_id, result.command_message_id,
                    inbox.envelope_sha256 AS command_envelope_sha256,
                    inbox.logical_owner_id
             FROM makosh_data.mail_retained_evidence_replay_result_outbox result
             JOIN makosh_data.mail_retained_evidence_replay_command_inbox inbox
               ON inbox.message_id = result.command_message_id
              AND inbox.operation_id = result.operation_id
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

    pub async fn mark_replay_result_published(
        &self,
        message_id: [u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<bool, RetainedMailReplayErrorV1> {
        if zero(&message_id) || published_at_unix_seconds <= 0 {
            return Err(RetainedMailReplayErrorV1::InvalidInput);
        }
        sqlx::query(
            "UPDATE makosh_data.mail_retained_evidence_replay_result_outbox
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

fn validate_admission(
    admission: &MailReplayCommandAdmissionV1,
    timestamp: i64,
) -> Result<(), RetainedMailReplayErrorV1> {
    if zero(&admission.command_message_id)
        || zero(&admission.command_envelope_sha256)
        || zero(&admission.operation_id)
        || !valid_identity(&admission.logical_owner_id)
        || timestamp <= 0
    {
        return Err(RetainedMailReplayErrorV1::InvalidInput);
    }
    Ok(())
}

fn verify_command_row(
    row: &sqlx::postgres::PgRow,
    admission: &MailReplayCommandAdmissionV1,
) -> Result<(), RetainedMailReplayErrorV1> {
    let message_id = row_id::<16>(row, "message_id")?;
    let envelope_sha256 = row_id::<32>(row, "envelope_sha256")?;
    let operation_id = row_id::<16>(row, "operation_id")?;
    let logical_owner_id: String = row.try_get("logical_owner_id").map_err(row_error)?;
    if message_id != admission.command_message_id
        || envelope_sha256 != admission.command_envelope_sha256
        || operation_id != admission.operation_id
        || logical_owner_id != admission.logical_owner_id
    {
        return Err(RetainedMailReplayErrorV1::Conflict);
    }
    Ok(())
}

fn verify_result_record(
    record: &OutboxRecordV1,
    admission: &MailReplayCommandAdmissionV1,
) -> Result<(), RetainedMailReplayErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes()).map_err(row_error)?;
    let contract = mail_replay_result_contract_reference_v1();
    let actual = envelope
        .contract
        .as_ref()
        .ok_or(RetainedMailReplayErrorV1::WrongContract)?;
    if actual.owner != contract.owner
        || actual.name != contract.name
        || actual.major != contract.major
        || actual.revision != contract.revision
        || actual.schema_sha256 != contract.schema_sha256
        || envelope.partition_key != admission.operation_id
        || envelope.correlation_id != admission.operation_id
        || envelope.causation_message_id != admission.command_message_id
    {
        return Err(RetainedMailReplayErrorV1::WrongContract);
    }
    let Some(Semantics::Result(metadata)) = envelope.semantics.as_ref() else {
        return Err(RetainedMailReplayErrorV1::WrongContract);
    };
    if metadata.command_id != admission.operation_id
        || metadata.command_message_id != admission.command_message_id
    {
        return Err(RetainedMailReplayErrorV1::WrongContract);
    }
    let payload =
        ReplayMailEvidenceResultV1::decode(envelope.payload.as_slice()).map_err(row_error)?;
    validate_mail_replay_result_v1(&payload)
        .map_err(|_| RetainedMailReplayErrorV1::WrongContract)?;
    if payload.operation_id != admission.operation_id {
        return Err(RetainedMailReplayErrorV1::WrongContract);
    }
    Ok(())
}

fn verify_result_row(
    row: &sqlx::postgres::PgRow,
    command_message_id: [u8; 16],
    result: &OutboxRecordV1,
) -> Result<(), RetainedMailReplayErrorV1> {
    let stored_message_id = row_id::<16>(row, "message_id")?;
    let stored_hash = row_id::<32>(row, "envelope_sha256")?;
    let stored_command = row_id::<16>(row, "command_message_id")?;
    let stored_bytes: Vec<u8> = row.try_get("exact_envelope_bytes").map_err(row_error)?;
    if stored_message_id != *result.message_id()
        || stored_hash != *result.envelope_sha256()
        || stored_command != command_message_id
        || stored_bytes.as_slice() != result.exact_bytes()
    {
        return Err(RetainedMailReplayErrorV1::Conflict);
    }
    Ok(())
}

fn result_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<OutboxRecordV1, RetainedMailReplayErrorV1> {
    let message_id = row_id::<16>(&row, "message_id")?;
    let envelope_sha256 = row_id::<32>(&row, "envelope_sha256")?;
    let exact_bytes: Vec<u8> = row.try_get("exact_envelope_bytes").map_err(row_error)?;
    let record = OutboxRecordV1::accept(exact_bytes).map_err(row_error)?;
    if record.message_id() != &message_id || record.envelope_sha256() != &envelope_sha256 {
        return Err(RetainedMailReplayErrorV1::HashMismatch);
    }
    let admission = MailReplayCommandAdmissionV1 {
        command_message_id: row_id::<16>(&row, "command_message_id")?,
        command_envelope_sha256: row_id::<32>(&row, "command_envelope_sha256")?,
        operation_id: row_id::<16>(&row, "operation_id")?,
        logical_owner_id: row.try_get("logical_owner_id").map_err(row_error)?,
    };
    verify_result_record(&record, &admission)?;
    Ok(record)
}

fn row_id<const WIDTH: usize>(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<[u8; WIDTH], RetainedMailReplayErrorV1> {
    let value: Vec<u8> = row.try_get(column).map_err(row_error)?;
    let value: [u8; WIDTH] = value.try_into().map_err(row_error)?;
    (!zero(&value))
        .then_some(value)
        .ok_or(RetainedMailReplayErrorV1::InvalidRow)
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
}

fn zero(value: &[u8]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

fn storage_error<T>(_: T) -> RetainedMailReplayErrorV1 {
    RetainedMailReplayErrorV1::StorageUnavailable
}

fn row_error<T>(_: T) -> RetainedMailReplayErrorV1 {
    RetainedMailReplayErrorV1::InvalidRow
}
