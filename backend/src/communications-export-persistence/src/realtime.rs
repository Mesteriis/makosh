use sqlx::{Postgres, Row, Transaction};

use crate::{
    CommunicationsExportPersistenceErrorV1, CommunicationsExportPersistenceV1, valid_timestamp,
};

pub const COMMUNICATIONS_EXPORT_REALTIME_LIMIT_V1: u16 = 1_024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommunicationsExportRealtimeTransitionV1 {
    pub sequence: u64,
    pub export_id: [u8; 16],
    pub state: u8,
    pub requested_items: u32,
    pub completed_items: u32,
    pub artifact_bytes: u64,
    pub rejection_code: Option<u16>,
    pub occurred_at_unix_millis: i64,
}

impl CommunicationsExportPersistenceV1 {
    pub async fn client_realtime_window(
        &self,
        logical_owner_id: &str,
        after_sequence: Option<u64>,
        limit: u16,
    ) -> Result<Vec<CommunicationsExportRealtimeTransitionV1>, CommunicationsExportPersistenceErrorV1>
    {
        if logical_owner_id.is_empty()
            || logical_owner_id.len() > 128
            || after_sequence == Some(0)
            || !(1..=COMMUNICATIONS_EXPORT_REALTIME_LIMIT_V1).contains(&limit)
        {
            return Err(CommunicationsExportPersistenceErrorV1::InvalidInput);
        }
        let rows = if let Some(after) = after_sequence {
            sqlx::query(
                "SELECT realtime_sequence, export_id, state, requested_items,
                        completed_items, artifact_bytes, rejection_code,
                        occurred_at_unix_millis
                 FROM makosh_data.communications_export_client_realtime
                 WHERE logical_owner_id = $1 AND realtime_sequence > $2
                 ORDER BY realtime_sequence
                 LIMIT $3",
            )
            .bind(logical_owner_id)
            .bind(
                i64::try_from(after)
                    .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidInput)?,
            )
            .bind(i64::from(limit))
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                "SELECT realtime_sequence, export_id, state, requested_items,
                        completed_items, artifact_bytes, rejection_code,
                        occurred_at_unix_millis
                 FROM (
                   SELECT realtime_sequence, export_id, state, requested_items,
                          completed_items, artifact_bytes, rejection_code,
                          occurred_at_unix_millis
                   FROM makosh_data.communications_export_client_realtime
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
        .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
        rows.into_iter().map(transition_from_row).collect()
    }
}

pub(crate) async fn insert_realtime_transition(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    export_id: &[u8; 16],
    occurred_at_unix_seconds: i64,
) -> Result<(), CommunicationsExportPersistenceErrorV1> {
    if !valid_timestamp(occurred_at_unix_seconds) {
        return Err(CommunicationsExportPersistenceErrorV1::InvalidInput);
    }
    let occurred_at_unix_millis = occurred_at_unix_seconds
        .checked_mul(1_000)
        .ok_or(CommunicationsExportPersistenceErrorV1::InvalidInput)?;
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.communications_export_client_realtime (
           logical_owner_id, export_id, state, requested_items, completed_items,
           artifact_bytes, rejection_code, occurred_at_unix_millis
         )
         SELECT logical_owner_id, export_id, state, requested_items, completed_items,
                COALESCE(artifact_declared_bytes, 0), rejection_code, $1
         FROM makosh_data.communications_export_jobs
         WHERE logical_owner_id = $2 AND export_id = $3
         ON CONFLICT (logical_owner_id, export_id, state) DO NOTHING",
    )
    .bind(occurred_at_unix_millis)
    .bind(logical_owner_id)
    .bind(export_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(|_| CommunicationsExportPersistenceErrorV1::StorageUnavailable)?;
    if inserted.rows_affected() == 1 {
        Ok(())
    } else {
        Err(CommunicationsExportPersistenceErrorV1::InvalidRow)
    }
}

fn transition_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<CommunicationsExportRealtimeTransitionV1, CommunicationsExportPersistenceErrorV1> {
    let sequence: i64 = row.try_get("realtime_sequence").map_err(invalid_row)?;
    let export_id: Vec<u8> = row.try_get("export_id").map_err(invalid_row)?;
    let state: i16 = row.try_get("state").map_err(invalid_row)?;
    let requested_items: i32 = row.try_get("requested_items").map_err(invalid_row)?;
    let completed_items: i32 = row.try_get("completed_items").map_err(invalid_row)?;
    let artifact_bytes: i64 = row.try_get("artifact_bytes").map_err(invalid_row)?;
    let rejection_code: Option<i16> = row.try_get("rejection_code").map_err(invalid_row)?;
    let occurred_at_unix_millis: i64 = row
        .try_get("occurred_at_unix_millis")
        .map_err(invalid_row)?;
    if sequence <= 0
        || !(1..=4).contains(&state)
        || requested_items <= 0
        || completed_items < 0
        || completed_items > requested_items
        || artifact_bytes < 0
        || !valid_timestamp(occurred_at_unix_millis)
    {
        return Err(CommunicationsExportPersistenceErrorV1::InvalidRow);
    }
    Ok(CommunicationsExportRealtimeTransitionV1 {
        sequence: u64::try_from(sequence)
            .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        export_id: export_id
            .try_into()
            .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        state: u8::try_from(state)
            .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        requested_items: u32::try_from(requested_items)
            .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        completed_items: u32::try_from(completed_items)
            .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        artifact_bytes: u64::try_from(artifact_bytes)
            .map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)?,
        rejection_code: rejection_code
            .map(|value| {
                u16::try_from(value).map_err(|_| CommunicationsExportPersistenceErrorV1::InvalidRow)
            })
            .transpose()?,
        occurred_at_unix_millis,
    })
}

fn invalid_row(_: sqlx::Error) -> CommunicationsExportPersistenceErrorV1 {
    CommunicationsExportPersistenceErrorV1::InvalidRow
}
