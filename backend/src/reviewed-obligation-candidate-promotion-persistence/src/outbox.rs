use makosh_events_protocol::delivery::OutboxRecordV1;
use sqlx::{Postgres, Row, Transaction};

use crate::{
    ReviewedObligationCandidatePromotionPersistenceErrorV1,
    ReviewedObligationCandidatePromotionPersistenceV1,
    model::{
        MAX_EVENT_BYTES_V1, MAX_OUTBOX_BATCH_V1, UnpublishedPromotionEventV1, nonzero, valid_owner,
        valid_timestamp,
    },
};

pub(crate) async fn insert_exact_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    record: &OutboxRecordV1,
    created_at_unix_millis: i64,
) -> Result<(), ReviewedObligationCandidatePromotionPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.reviewed_obligation_candidate_promotion_outbox (
           logical_owner_id, message_id, envelope_sha256, envelope_bytes,
           created_at_unix_millis
         ) VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (logical_owner_id, message_id) DO NOTHING",
    )
    .bind(logical_owner_id)
    .bind(record.message_id().as_slice())
    .bind(record.envelope_sha256().as_slice())
    .bind(record.exact_bytes())
    .bind(created_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?
    .rows_affected();
    if inserted == 1 {
        return Ok(());
    }
    verify_exact_outbox(transaction, logical_owner_id, record).await
}

pub(crate) async fn verify_exact_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    record: &OutboxRecordV1,
) -> Result<(), ReviewedObligationCandidatePromotionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT envelope_sha256, envelope_bytes
         FROM makosh_data.reviewed_obligation_candidate_promotion_outbox
         WHERE logical_owner_id = $1 AND message_id = $2",
    )
    .bind(logical_owner_id)
    .bind(record.message_id().as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_error)?
    .ok_or(ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidRow)?;
    let envelope_sha256: Vec<u8> = row
        .try_get("envelope_sha256")
        .map_err(|_| ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidRow)?;
    let envelope_bytes: Vec<u8> = row
        .try_get("envelope_bytes")
        .map_err(|_| ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidRow)?;
    if envelope_sha256.as_slice() != record.envelope_sha256()
        || envelope_bytes.as_slice() != record.exact_bytes()
    {
        return Err(ReviewedObligationCandidatePromotionPersistenceErrorV1::OutboxConflict);
    }
    Ok(())
}

impl ReviewedObligationCandidatePromotionPersistenceV1 {
    pub async fn unpublished_events(
        &self,
        logical_owner_id: &str,
        limit: u16,
    ) -> Result<
        Vec<UnpublishedPromotionEventV1>,
        ReviewedObligationCandidatePromotionPersistenceErrorV1,
    > {
        if !valid_owner(logical_owner_id) || !(1..=MAX_OUTBOX_BATCH_V1).contains(&limit) {
            return Err(ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let events = sqlx::query(
            "SELECT message_id, envelope_sha256, envelope_bytes
             FROM makosh_data.reviewed_obligation_candidate_promotion_outbox
             WHERE logical_owner_id = $1 AND published_at_unix_millis IS NULL
             ORDER BY created_at_unix_millis, message_id
             LIMIT $2",
        )
        .bind(logical_owner_id)
        .bind(i64::from(limit))
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(event_from_row)
        .collect::<Result<Vec<_>, _>>()?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(events)
    }

    pub async fn mark_event_published(
        &self,
        logical_owner_id: &str,
        message_id: &[u8; 16],
        envelope_sha256: &[u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), ReviewedObligationCandidatePromotionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !nonzero(message_id)
            || !nonzero(envelope_sha256)
            || !valid_timestamp(published_at_unix_millis)
        {
            return Err(ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let updated = sqlx::query(
            "UPDATE makosh_data.reviewed_obligation_candidate_promotion_outbox
             SET published_at_unix_millis = $1
             WHERE logical_owner_id = $2 AND message_id = $3
               AND envelope_sha256 = $4
               AND published_at_unix_millis IS NULL
               AND created_at_unix_millis <= $1",
        )
        .bind(published_at_unix_millis)
        .bind(logical_owner_id)
        .bind(message_id.as_slice())
        .bind(envelope_sha256.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if updated == 1 {
            transaction.commit().await.map_err(storage_error)?;
            Ok(())
        } else {
            Err(ReviewedObligationCandidatePromotionPersistenceErrorV1::OutboxConflict)
        }
    }
}

fn event_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<UnpublishedPromotionEventV1, ReviewedObligationCandidatePromotionPersistenceErrorV1> {
    let event = UnpublishedPromotionEventV1 {
        message_id: row
            .try_get::<Vec<u8>, _>("message_id")
            .map_err(|_| ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidRow)?
            .try_into()
            .map_err(|_| ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidRow)?,
        envelope_sha256: row
            .try_get::<Vec<u8>, _>("envelope_sha256")
            .map_err(|_| ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidRow)?
            .try_into()
            .map_err(|_| ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidRow)?,
        envelope_bytes: row
            .try_get("envelope_bytes")
            .map_err(|_| ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidRow)?,
    };
    if !nonzero(&event.message_id)
        || !nonzero(&event.envelope_sha256)
        || event.envelope_bytes.is_empty()
        || event.envelope_bytes.len() > MAX_EVENT_BYTES_V1
    {
        return Err(ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidRow);
    }
    Ok(event)
}

fn storage_error(_: sqlx::Error) -> ReviewedObligationCandidatePromotionPersistenceErrorV1 {
    ReviewedObligationCandidatePromotionPersistenceErrorV1::StorageUnavailable
}
