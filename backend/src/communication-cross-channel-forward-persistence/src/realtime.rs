use makosh_communication_cross_channel_forward_core::CrossChannelForwardStateV1;
use sqlx::{Postgres, Row, Transaction};

use crate::{
    CommunicationCrossChannelForwardPersistenceV1, CrossChannelForwardPersistenceErrorV1,
    operations::{id16, positive_u64, state_from_code},
    valid_bounded_identity, valid_timestamp,
};

const MAX_REPLAY_WINDOW_V1: u16 = 1_024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CrossChannelForwardClientRealtimeTransitionV1 {
    pub sequence: u64,
    pub forward_id: [u8; 16],
    pub state: CrossChannelForwardStateV1,
    pub state_revision: u64,
    pub error_code: Option<u16>,
    pub occurred_at_unix_millis: i64,
}

impl CommunicationCrossChannelForwardPersistenceV1 {
    pub async fn client_realtime_window(
        &self,
        logical_owner_id: &str,
        after_sequence: Option<u64>,
        limit: u16,
    ) -> Result<
        Vec<CrossChannelForwardClientRealtimeTransitionV1>,
        CrossChannelForwardPersistenceErrorV1,
    > {
        if !valid_bounded_identity(logical_owner_id)
            || !(1..=MAX_REPLAY_WINDOW_V1).contains(&limit)
            || after_sequence == Some(0)
        {
            return Err(CrossChannelForwardPersistenceErrorV1::InvalidInput);
        }
        let rows = if let Some(after_sequence) = after_sequence {
            let after_sequence = i64::try_from(after_sequence)
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidInput)?;
            sqlx::query(
                "SELECT realtime_sequence, forward_id, state, state_revision,
                        error_code, occurred_at_unix_millis
                 FROM makosh_data.communication_cross_channel_forward_realtime
                 WHERE logical_owner_id = $1 AND realtime_sequence > $2
                 ORDER BY realtime_sequence
                 LIMIT $3",
            )
            .bind(logical_owner_id)
            .bind(after_sequence)
            .bind(i64::from(limit))
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                "SELECT realtime_sequence, forward_id, state, state_revision,
                        error_code, occurred_at_unix_millis
                 FROM (
                   SELECT realtime_sequence, forward_id, state, state_revision,
                          error_code, occurred_at_unix_millis
                   FROM makosh_data.communication_cross_channel_forward_realtime
                   WHERE logical_owner_id = $1
                   ORDER BY realtime_sequence DESC
                   LIMIT $2
                 ) replay
                 ORDER BY realtime_sequence",
            )
            .bind(logical_owner_id)
            .bind(i64::from(limit))
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?;
        rows.into_iter().map(transition_from_row).collect()
    }
}

pub(crate) async fn insert_forward_transition(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    forward_id: &[u8; 16],
    occurred_at_unix_millis: i64,
) -> Result<(), CrossChannelForwardPersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.communication_cross_channel_forward_realtime (
           logical_owner_id, forward_id, state_revision, state,
           error_code, occurred_at_unix_millis
         )
         SELECT logical_owner_id, forward_id, state_revision, state,
                error_code, $1
         FROM makosh_data.communication_cross_channel_forward_operations
         WHERE logical_owner_id = $2 AND forward_id = $3",
    )
    .bind(occurred_at_unix_millis)
    .bind(logical_owner_id)
    .bind(forward_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(|_| CrossChannelForwardPersistenceErrorV1::StorageUnavailable)?
    .rows_affected();
    if inserted == 1 {
        Ok(())
    } else {
        Err(CrossChannelForwardPersistenceErrorV1::InvalidRow)
    }
}

fn transition_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<CrossChannelForwardClientRealtimeTransitionV1, CrossChannelForwardPersistenceErrorV1> {
    let occurred_at_unix_millis: i64 = row
        .try_get("occurred_at_unix_millis")
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    if !valid_timestamp(occurred_at_unix_millis) {
        return Err(CrossChannelForwardPersistenceErrorV1::InvalidRow);
    }
    let error_code: Option<i16> = row
        .try_get("error_code")
        .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?;
    Ok(CrossChannelForwardClientRealtimeTransitionV1 {
        sequence: positive_u64(
            row.try_get("realtime_sequence")
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
        )?,
        forward_id: id16(
            row.try_get("forward_id")
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
        )?,
        state: state_from_code(
            row.try_get("state")
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
        )?,
        state_revision: positive_u64(
            row.try_get("state_revision")
                .map_err(|_| CrossChannelForwardPersistenceErrorV1::InvalidRow)?,
        )?,
        error_code: error_code
            .map(|value| {
                u16::try_from(value)
                    .ok()
                    .filter(|value| (1..=7).contains(value))
                    .ok_or(CrossChannelForwardPersistenceErrorV1::InvalidRow)
            })
            .transpose()?,
        occurred_at_unix_millis,
    })
}
