//! Durable owner-local replay ledger for client-safe delivery-intent state changes.

use sqlx::Row;

use crate::{
    CommunicationDeliveryIntentPersistenceV1, DeliveryIntentPersistenceErrorV1,
    DeliveryIntentStateV1, intents::state_from_code, valid_bounded_identity,
};

const MAX_REPLAY_WINDOW: u16 = 1_024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryIntentClientRealtimeTransitionV1 {
    pub sequence: u64,
    pub intent_id: [u8; 16],
    pub state: DeliveryIntentStateV1,
    pub state_revision: u64,
    pub rejection_code: Option<u16>,
    pub occurred_at_unix_seconds: i64,
}

impl CommunicationDeliveryIntentPersistenceV1 {
    pub async fn client_realtime_window(
        &self,
        logical_owner_id: &str,
        after_sequence: Option<u64>,
        limit: u16,
    ) -> Result<Vec<DeliveryIntentClientRealtimeTransitionV1>, DeliveryIntentPersistenceErrorV1>
    {
        if !valid_bounded_identity(logical_owner_id)
            || !(1..=MAX_REPLAY_WINDOW).contains(&limit)
            || after_sequence == Some(0)
        {
            return Err(DeliveryIntentPersistenceErrorV1::InvalidInput);
        }
        let limit = i64::from(limit);
        let rows = if let Some(after_sequence) = after_sequence {
            let after_sequence = i64::try_from(after_sequence)
                .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidInput)?;
            sqlx::query(
                "SELECT realtime_sequence, intent_id, state, state_revision,
                        rejection_code, occurred_at_unix_seconds
                 FROM makosh_data.communication_delivery_intent_transitions
                 WHERE logical_owner_id = $1 AND realtime_sequence > $2
                 ORDER BY realtime_sequence ASC
                 LIMIT $3",
            )
            .bind(logical_owner_id)
            .bind(after_sequence)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                "SELECT realtime_sequence, intent_id, state, state_revision,
                        rejection_code, occurred_at_unix_seconds
                 FROM (
                   SELECT realtime_sequence, intent_id, state, state_revision,
                          rejection_code, occurred_at_unix_seconds
                   FROM makosh_data.communication_delivery_intent_transitions
                   WHERE logical_owner_id = $1
                   ORDER BY realtime_sequence DESC
                   LIMIT $2
                 ) replay
                 ORDER BY realtime_sequence ASC",
            )
            .bind(logical_owner_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|_| DeliveryIntentPersistenceErrorV1::StorageUnavailable)?;
        rows.into_iter().map(transition_from_row).collect()
    }
}

fn transition_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<DeliveryIntentClientRealtimeTransitionV1, DeliveryIntentPersistenceErrorV1> {
    let sequence = positive_u64(row.try_get("realtime_sequence").map_err(row_error)?)?;
    let intent_id = row
        .try_get::<Vec<u8>, _>("intent_id")
        .map_err(row_error)?
        .try_into()
        .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
    let state = state_from_code(row.try_get("state").map_err(row_error)?)?;
    let state_revision = positive_u64(row.try_get("state_revision").map_err(row_error)?)?;
    let rejection_code = row
        .try_get::<Option<i16>, _>("rejection_code")
        .map_err(row_error)?
        .map(u16::try_from)
        .transpose()
        .map_err(|_| DeliveryIntentPersistenceErrorV1::InvalidRow)?;
    let occurred_at_unix_seconds = row.try_get("occurred_at_unix_seconds").map_err(row_error)?;
    if occurred_at_unix_seconds <= 0 {
        return Err(DeliveryIntentPersistenceErrorV1::InvalidRow);
    }
    Ok(DeliveryIntentClientRealtimeTransitionV1 {
        sequence,
        intent_id,
        state,
        state_revision,
        rejection_code,
        occurred_at_unix_seconds,
    })
}

fn positive_u64(value: i64) -> Result<u64, DeliveryIntentPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(DeliveryIntentPersistenceErrorV1::InvalidRow)
}

fn row_error(_: sqlx::Error) -> DeliveryIntentPersistenceErrorV1 {
    DeliveryIntentPersistenceErrorV1::InvalidRow
}
