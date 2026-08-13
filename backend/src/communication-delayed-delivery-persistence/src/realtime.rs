use makosh_communication_delayed_delivery_core::DelayedDeliveryStateV1;
use sqlx::{Postgres, Row, Transaction};

use crate::{
    CommunicationDelayedDeliveryPersistenceV1, DelayedDeliveryPersistenceErrorV1,
    operations::{state_from_code, valid_owner},
};

const MAX_REPLAY_WINDOW_V1: u16 = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DelayedDeliveryClientRealtimeTransitionV1 {
    pub sequence: u64,
    pub delayed_operation_id: [u8; 16],
    pub state: DelayedDeliveryStateV1,
    pub state_revision: u64,
    pub occurred_at_unix_millis: u64,
}

impl CommunicationDelayedDeliveryPersistenceV1 {
    pub async fn client_realtime_window(
        &self,
        logical_owner_id: &str,
        after_sequence: Option<u64>,
        limit: u16,
    ) -> Result<Vec<DelayedDeliveryClientRealtimeTransitionV1>, DelayedDeliveryPersistenceErrorV1>
    {
        if !valid_owner(logical_owner_id)
            || !(1..=MAX_REPLAY_WINDOW_V1).contains(&limit)
            || after_sequence == Some(0)
        {
            return Err(DelayedDeliveryPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let rows = if let Some(after_sequence) = after_sequence {
            let after_sequence = i64::try_from(after_sequence)
                .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidInput)?;
            sqlx::query(
                "SELECT realtime_sequence, delayed_operation_id, state,
                        state_revision, occurred_at_unix_millis
                 FROM makosh_data.communication_delayed_delivery_realtime
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
                "SELECT realtime_sequence, delayed_operation_id, state,
                        state_revision, occurred_at_unix_millis
                 FROM (
                   SELECT realtime_sequence, delayed_operation_id, state,
                          state_revision, occurred_at_unix_millis
                   FROM makosh_data.communication_delayed_delivery_realtime
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
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        let transitions = rows.into_iter().map(transition_from_row).collect();
        transaction
            .commit()
            .await
            .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?;
        transitions
    }
}

pub(crate) async fn insert_operation_transition(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    delayed_operation_id: &[u8; 16],
    occurred_at_unix_millis: i64,
) -> Result<(), DelayedDeliveryPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.communication_delayed_delivery_realtime (
           logical_owner_id, delayed_operation_id, state_revision, state,
           occurred_at_unix_millis
         )
         SELECT logical_owner_id, delayed_operation_id, state_revision, state, $1
         FROM makosh_data.communication_delayed_delivery_operations
         WHERE logical_owner_id = $2 AND delayed_operation_id = $3",
    )
    .bind(occurred_at_unix_millis)
    .bind(logical_owner_id)
    .bind(delayed_operation_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?
    .rows_affected();
    if inserted == 1 {
        Ok(())
    } else {
        Err(DelayedDeliveryPersistenceErrorV1::InvalidRow)
    }
}

fn transition_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<DelayedDeliveryClientRealtimeTransitionV1, DelayedDeliveryPersistenceErrorV1> {
    Ok(DelayedDeliveryClientRealtimeTransitionV1 {
        sequence: positive_u64(
            row.try_get("realtime_sequence")
                .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)?,
        )?,
        delayed_operation_id: id16(
            row.try_get("delayed_operation_id")
                .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)?,
        )?,
        state: state_from_code(
            row.try_get("state")
                .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)?,
        )?,
        state_revision: positive_u64(
            row.try_get("state_revision")
                .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)?,
        )?,
        occurred_at_unix_millis: positive_u64(
            row.try_get("occurred_at_unix_millis")
                .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)?,
        )?,
    })
}

fn positive_u64(value: i64) -> Result<u64, DelayedDeliveryPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(DelayedDeliveryPersistenceErrorV1::InvalidRow)
}

fn id16(value: Vec<u8>) -> Result<[u8; 16], DelayedDeliveryPersistenceErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::InvalidRow)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(DelayedDeliveryPersistenceErrorV1::InvalidRow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_inputs_are_bounded_and_zero_cursor_is_not_a_checkpoint() {
        assert!(valid_owner("owner-1"));
        assert!(!valid_owner(""));
        assert_eq!(MAX_REPLAY_WINDOW_V1, 256);
        assert!(positive_u64(0).is_err());
    }
}
