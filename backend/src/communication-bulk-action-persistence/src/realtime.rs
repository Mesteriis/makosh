use sqlx::{Postgres, Row, Transaction};

use crate::{
    BulkDeliveryBatchStateV1, BulkDeliveryPersistenceErrorV1, CommunicationBulkActionPersistenceV1,
    valid_bounded_identity,
};

const MAX_REPLAY_WINDOW_V1: u16 = 1_024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BulkDeliveryClientRealtimeTransitionV1 {
    pub sequence: u64,
    pub batch_id: [u8; 16],
    pub state: BulkDeliveryBatchStateV1,
    pub state_revision: u64,
    pub occurred_at_unix_seconds: i64,
}

impl CommunicationBulkActionPersistenceV1 {
    pub async fn client_realtime_window(
        &self,
        logical_owner_id: &str,
        after_sequence: Option<u64>,
        limit: u16,
    ) -> Result<Vec<BulkDeliveryClientRealtimeTransitionV1>, BulkDeliveryPersistenceErrorV1> {
        if !valid_bounded_identity(logical_owner_id)
            || !(1..=MAX_REPLAY_WINDOW_V1).contains(&limit)
            || after_sequence == Some(0)
        {
            return Err(BulkDeliveryPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let rows = if let Some(after_sequence) = after_sequence {
            let after_sequence = i64::try_from(after_sequence)
                .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidInput)?;
            sqlx::query(
                "SELECT realtime_sequence, batch_id, state, state_revision,
                        occurred_at_unix_seconds
                 FROM makosh_data.communication_bulk_action_realtime
                 WHERE logical_owner_id = $1 AND realtime_sequence > $2
                 ORDER BY realtime_sequence
                 LIMIT $3",
            )
            .bind(logical_owner_id)
            .bind(after_sequence)
            .bind(i64::from(limit))
            .fetch_all(&mut *transaction)
            .await
        } else {
            sqlx::query(
                "SELECT realtime_sequence, batch_id, state, state_revision,
                        occurred_at_unix_seconds
                 FROM (
                   SELECT realtime_sequence, batch_id, state, state_revision,
                          occurred_at_unix_seconds
                   FROM makosh_data.communication_bulk_action_realtime
                   WHERE logical_owner_id = $1
                   ORDER BY realtime_sequence DESC
                   LIMIT $2
                 ) replay
                 ORDER BY realtime_sequence",
            )
            .bind(logical_owner_id)
            .bind(i64::from(limit))
            .fetch_all(&mut *transaction)
            .await
        }
        .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)?;
        let transitions = rows.into_iter().map(transition_from_row).collect();
        transaction
            .commit()
            .await
            .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)?;
        transitions
    }
}

pub(crate) async fn insert_batch_transition(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    batch_id: &[u8; 16],
    occurred_at_unix_seconds: i64,
) -> Result<(), BulkDeliveryPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.communication_bulk_action_realtime (
           logical_owner_id, batch_id, state_revision, state,
           occurred_at_unix_seconds
         )
         SELECT batches.logical_owner_id, batches.batch_id,
                batches.state_revision,
                CASE
                  WHEN COUNT(*) FILTER (WHERE targets.state = 3)
                       + COUNT(*) FILTER (WHERE targets.state = 5)
                       < COUNT(*) THEN 1
                  WHEN COUNT(*) FILTER (WHERE targets.state = 3)
                       = COUNT(*) THEN 2
                  WHEN COUNT(*) FILTER (WHERE targets.state = 5)
                       = COUNT(*) THEN 4
                  ELSE 3
                END,
                $1
         FROM makosh_data.communication_bulk_action_batches AS batches
         JOIN makosh_data.communication_bulk_action_targets AS targets
           ON targets.logical_owner_id = batches.logical_owner_id
          AND targets.batch_id = batches.batch_id
         WHERE batches.logical_owner_id = $2 AND batches.batch_id = $3
         GROUP BY batches.logical_owner_id, batches.batch_id,
                  batches.state_revision",
    )
    .bind(occurred_at_unix_seconds)
    .bind(logical_owner_id)
    .bind(batch_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(|_| BulkDeliveryPersistenceErrorV1::StorageUnavailable)?
    .rows_affected();
    if inserted == 1 {
        Ok(())
    } else {
        Err(BulkDeliveryPersistenceErrorV1::InvalidRow)
    }
}

fn transition_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<BulkDeliveryClientRealtimeTransitionV1, BulkDeliveryPersistenceErrorV1> {
    let sequence = positive_u64(
        row.try_get("realtime_sequence")
            .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidRow)?,
    )?;
    let batch_id = row
        .try_get::<Vec<u8>, _>("batch_id")
        .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidRow)?
        .try_into()
        .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidRow)?;
    let state = match row
        .try_get::<i16, _>("state")
        .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidRow)?
    {
        1 => BulkDeliveryBatchStateV1::Accepted,
        2 => BulkDeliveryBatchStateV1::Completed,
        3 => BulkDeliveryBatchStateV1::CompletedWithErrors,
        4 => BulkDeliveryBatchStateV1::Rejected,
        _ => return Err(BulkDeliveryPersistenceErrorV1::InvalidRow),
    };
    let state_revision = positive_u64(
        row.try_get("state_revision")
            .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidRow)?,
    )?;
    let occurred_at_unix_seconds = row
        .try_get("occurred_at_unix_seconds")
        .map_err(|_| BulkDeliveryPersistenceErrorV1::InvalidRow)?;
    if occurred_at_unix_seconds <= 0 {
        return Err(BulkDeliveryPersistenceErrorV1::InvalidRow);
    }
    Ok(BulkDeliveryClientRealtimeTransitionV1 {
        sequence,
        batch_id,
        state,
        state_revision,
        occurred_at_unix_seconds,
    })
}

fn positive_u64(value: i64) -> Result<u64, BulkDeliveryPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(BulkDeliveryPersistenceErrorV1::InvalidRow)
}
