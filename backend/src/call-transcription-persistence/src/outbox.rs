use sqlx::{Postgres, Row, Transaction};

use crate::{
    CallTranscriptionPersistenceErrorV1, CallTranscriptionPersistenceV1, DurableOutboxRecordV1,
    UnpublishedCallTranscriptionEventV1,
    model::{CALL_TRANSCRIPTION_OUTBOX_LIMIT_V1, valid_id16, valid_owner, valid_timestamp_millis},
    repository::{id16, id32, row_error, storage_error},
};

pub(crate) async fn insert_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    record: &DurableOutboxRecordV1,
    created_at_unix_millis: i64,
) -> Result<(), CallTranscriptionPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.call_transcription_outbox
         (logical_owner_id,message_id,envelope_sha256,envelope_bytes,created_at_unix_millis)
         VALUES ($1,$2,$3,$4,$5)",
    )
    .bind(logical_owner_id)
    .bind(record.message_id.as_slice())
    .bind(record.envelope_sha256.as_slice())
    .bind(&record.envelope_bytes)
    .bind(created_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?;
    Ok(())
}

impl CallTranscriptionPersistenceV1 {
    pub async fn unpublished_outbox(
        &self,
        logical_owner_id: &str,
        limit: u32,
    ) -> Result<Vec<UnpublishedCallTranscriptionEventV1>, CallTranscriptionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !(1..=CALL_TRANSCRIPTION_OUTBOX_LIMIT_V1).contains(&limit)
        {
            return Err(CallTranscriptionPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "SELECT message_id,envelope_sha256,envelope_bytes FROM
             makosh_data.call_transcription_outbox WHERE logical_owner_id=$1
               AND published_at_unix_millis IS NULL
             ORDER BY created_at_unix_millis,message_id LIMIT $2",
        )
        .bind(logical_owner_id)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?
        .iter()
        .map(|row| {
            Ok(UnpublishedCallTranscriptionEventV1 {
                message_id: id16(row.try_get("message_id").map_err(row_error)?)?,
                envelope_sha256: id32(row.try_get("envelope_sha256").map_err(row_error)?)?,
                envelope_bytes: row.try_get("envelope_bytes").map_err(row_error)?,
            })
        })
        .collect()
    }

    pub async fn mark_outbox_published(
        &self,
        logical_owner_id: &str,
        message_id: [u8; 16],
        envelope_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), CallTranscriptionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_id16(&message_id)
            || envelope_sha256 == [0; 32]
            || !valid_timestamp_millis(published_at_unix_millis)
        {
            return Err(CallTranscriptionPersistenceErrorV1::InvalidInput);
        }
        let changed = sqlx::query(
            "UPDATE makosh_data.call_transcription_outbox SET published_at_unix_millis=$1
             WHERE logical_owner_id=$2 AND message_id=$3 AND envelope_sha256=$4
               AND published_at_unix_millis IS NULL AND created_at_unix_millis<=$1",
        )
        .bind(published_at_unix_millis)
        .bind(logical_owner_id)
        .bind(message_id.as_slice())
        .bind(envelope_sha256.as_slice())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if changed == 1 {
            Ok(())
        } else {
            Err(CallTranscriptionPersistenceErrorV1::RevisionConflict)
        }
    }
}
