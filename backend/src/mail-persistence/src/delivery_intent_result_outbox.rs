//! Mail-owned terminal delivery-result outbox.

use makosh_events_protocol::delivery::OutboxRecordV1;
use sqlx::Row;

use crate::{MailDeliveryIntentStoreV1, MailDurablePersistenceError};

impl MailDeliveryIntentStoreV1 {
    pub async fn pending_result_outbox(
        &self,
        limit: i64,
    ) -> Result<Vec<OutboxRecordV1>, MailDurablePersistenceError> {
        if !(1..=256).contains(&limit) {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let rows = sqlx::query(
            "SELECT message_id, envelope_sha256, exact_envelope_bytes
             FROM makosh_data.mail_delivery_intent_result_outbox
             WHERE published_at_unix_seconds IS NULL
             ORDER BY created_at_unix_seconds, message_id
             LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        rows.into_iter()
            .map(|row| {
                let message_id = required_id::<16>(&row, "message_id")?;
                let envelope_sha256 = required_id::<32>(&row, "envelope_sha256")?;
                let exact_bytes: Vec<u8> = row
                    .try_get("exact_envelope_bytes")
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                let record = OutboxRecordV1::accept(exact_bytes)
                    .map_err(|_| MailDurablePersistenceError::InvalidRow)?;
                if record.message_id() != &message_id
                    || record.envelope_sha256() != &envelope_sha256
                {
                    return Err(MailDurablePersistenceError::InvalidRow);
                }
                Ok(record)
            })
            .collect()
    }

    pub async fn mark_result_outbox_published(
        &self,
        message_id: &[u8; 16],
        published_at_unix_seconds: i64,
    ) -> Result<(), MailDurablePersistenceError> {
        if message_id.iter().all(|byte| *byte == 0) || published_at_unix_seconds <= 0 {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.mail_delivery_intent_result_outbox
             SET published_at_unix_seconds = $1
             WHERE message_id = $2
               AND published_at_unix_seconds IS NULL",
        )
        .bind(published_at_unix_seconds)
        .bind(message_id.as_slice())
        .execute(&self.pool)
        .await
        .map_err(|_| MailDurablePersistenceError::Database)?;
        if updated.rows_affected() > 1 {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        Ok(())
    }
}

pub(crate) async fn insert_result_outbox(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    intent_id: [u8; 16],
    record: &OutboxRecordV1,
    created_at_unix_seconds: i64,
) -> Result<(), MailDurablePersistenceError> {
    sqlx::query(
        "INSERT INTO makosh_data.mail_delivery_intent_result_outbox
            (message_id, envelope_sha256, exact_envelope_bytes, intent_id,
             created_at_unix_seconds)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(record.message_id().as_slice())
    .bind(record.envelope_sha256().as_slice())
    .bind(record.exact_bytes())
    .bind(intent_id.as_slice())
    .bind(created_at_unix_seconds)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(|_| MailDurablePersistenceError::Database)
}

fn required_id<const WIDTH: usize>(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<[u8; WIDTH], MailDurablePersistenceError> {
    row.try_get::<Vec<u8>, _>(column)
        .map_err(|_| MailDurablePersistenceError::InvalidRow)?
        .try_into()
        .map_err(|_| MailDurablePersistenceError::InvalidRow)
}
