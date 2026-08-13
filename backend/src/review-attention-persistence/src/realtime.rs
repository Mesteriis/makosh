use makosh_review_attention_core::{
    ReviewDispositionV1, ReviewImportanceV1, ReviewTimestampV1, STABLE_ID_BYTES_V1,
};
use sqlx::{Postgres, Row, Transaction};

use crate::repository::{
    ReviewAttentionPersistenceErrorV1, ReviewAttentionPersistenceV1, disposition, disposition_code,
    id16, importance, importance_code, optional_timestamp, positive_u64, timestamp,
};

pub const REVIEW_ATTENTION_REALTIME_REPLAY_LIMIT_V1: u16 = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewAttentionRealtimeTransitionV1 {
    pub sequence: u64,
    pub attention_id: [u8; STABLE_ID_BYTES_V1],
    pub revision: u64,
    pub disposition: ReviewDispositionV1,
    pub pinned: bool,
    pub importance: ReviewImportanceV1,
    pub snoozed_until: Option<ReviewTimestampV1>,
    pub occurred_at: ReviewTimestampV1,
}

impl ReviewAttentionPersistenceV1 {
    pub async fn realtime_window(
        &self,
        logical_owner_id: &str,
        after_sequence: Option<u64>,
        limit: u16,
    ) -> Result<Vec<ReviewAttentionRealtimeTransitionV1>, ReviewAttentionPersistenceErrorV1> {
        if logical_owner_id.is_empty()
            || after_sequence == Some(0)
            || limit == 0
            || limit > REVIEW_ATTENTION_REALTIME_REPLAY_LIMIT_V1
        {
            return Err(ReviewAttentionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let rows = sqlx::query(
            "SELECT realtime_sequence, attention_id, state_revision, disposition,
                    pinned, importance, snoozed_until_unix_seconds,
                    snoozed_until_nanos, occurred_at_unix_seconds, occurred_at_nanos
             FROM makosh_data.review_attention_realtime
             WHERE logical_owner_id = $1
               AND ($2::BIGINT IS NULL OR realtime_sequence > $2)
             ORDER BY realtime_sequence ASC
             LIMIT $3",
        )
        .bind(logical_owner_id)
        .bind(after_sequence.map(signed).transpose()?)
        .bind(i64::from(limit))
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| ReviewAttentionPersistenceErrorV1::StorageUnavailable)?;
        let transitions = rows
            .iter()
            .map(transition_from_row)
            .collect::<Result<Vec<_>, _>>()?;
        transaction
            .commit()
            .await
            .map_err(|_| ReviewAttentionPersistenceErrorV1::StorageUnavailable)?;
        Ok(transitions)
    }
}

pub(crate) async fn insert_realtime_transition(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    attention: &makosh_review_attention_core::ReviewAttentionV1,
) -> Result<(), ReviewAttentionPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.review_attention_realtime (
           logical_owner_id, attention_id, state_revision, disposition,
           pinned, importance, snoozed_until_unix_seconds,
           snoozed_until_nanos, occurred_at_unix_seconds, occurred_at_nanos
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(logical_owner_id)
    .bind(attention.attention_id.as_slice())
    .bind(signed(attention.revision)?)
    .bind(disposition_code(attention.disposition))
    .bind(attention.pinned)
    .bind(importance_code(attention.importance))
    .bind(attention.snoozed_until.map(|value| value.unix_seconds))
    .bind(attention.snoozed_until.map(|value| value.nanos))
    .bind(attention.updated_at.unix_seconds)
    .bind(attention.updated_at.nanos)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(|_| ReviewAttentionPersistenceErrorV1::StorageUnavailable)
}

fn transition_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<ReviewAttentionRealtimeTransitionV1, ReviewAttentionPersistenceErrorV1> {
    Ok(ReviewAttentionRealtimeTransitionV1 {
        sequence: positive_u64(row.try_get("realtime_sequence").map_err(row_error)?)?,
        attention_id: id16(row.try_get("attention_id").map_err(row_error)?)?,
        revision: positive_u64(row.try_get("state_revision").map_err(row_error)?)?,
        disposition: disposition(row.try_get("disposition").map_err(row_error)?)?,
        pinned: row.try_get("pinned").map_err(row_error)?,
        importance: importance(row.try_get("importance").map_err(row_error)?)?,
        snoozed_until: optional_timestamp(
            row.try_get("snoozed_until_unix_seconds")
                .map_err(row_error)?,
            row.try_get("snoozed_until_nanos").map_err(row_error)?,
        )?,
        occurred_at: timestamp(
            row.try_get("occurred_at_unix_seconds").map_err(row_error)?,
            row.try_get("occurred_at_nanos").map_err(row_error)?,
        )?,
    })
}

fn signed(value: u64) -> Result<i64, ReviewAttentionPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| ReviewAttentionPersistenceErrorV1::InvalidInput)
}

fn row_error(_: sqlx::Error) -> ReviewAttentionPersistenceErrorV1 {
    ReviewAttentionPersistenceErrorV1::InvalidRow
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_window_is_explicitly_bounded() {
        assert_eq!(REVIEW_ATTENTION_REALTIME_REPLAY_LIMIT_V1, 256);
        assert_eq!(signed(0), Ok(0));
        assert!(signed(u64::MAX).is_err());
    }
}
