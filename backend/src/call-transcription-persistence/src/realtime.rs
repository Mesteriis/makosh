use makosh_call_transcription_core::CallTranscriptionStatusV1;
use sqlx::{Postgres, Row, Transaction};

use crate::{
    CallTranscriptionPersistenceErrorV1, CallTranscriptionPersistenceV1,
    CallTranscriptionRealtimeTransitionV1,
    model::{CALL_TRANSCRIPTION_REALTIME_LIMIT_V1, valid_owner},
    repository::{
        id16, rejection_code, rejection_from_code, row_error, signed, state_code, state_from_code,
        storage_error,
    },
};

pub(crate) async fn append_realtime(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    status: &CallTranscriptionStatusV1,
    occurred_at_unix_millis: i64,
) -> Result<(), CallTranscriptionPersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.call_transcription_realtime
         (logical_owner_id,run_id,state,state_revision,rejection_code,occurred_at_unix_millis)
         VALUES ($1,$2,$3,$4,$5,$6)",
    )
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .bind(state_code(status.state))
    .bind(signed(status.state_revision)?)
    .bind(status.rejection.map(rejection_code))
    .bind(occurred_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?;
    Ok(())
}

impl CallTranscriptionPersistenceV1 {
    pub async fn realtime_after(
        &self,
        logical_owner_id: &str,
        after_sequence: u64,
        limit: u32,
    ) -> Result<Vec<CallTranscriptionRealtimeTransitionV1>, CallTranscriptionPersistenceErrorV1>
    {
        if !valid_owner(logical_owner_id)
            || !(1..=CALL_TRANSCRIPTION_REALTIME_LIMIT_V1).contains(&limit)
        {
            return Err(CallTranscriptionPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "SELECT realtime_sequence,run_id,state,state_revision,rejection_code,
             occurred_at_unix_millis FROM makosh_data.call_transcription_realtime
             WHERE logical_owner_id=$1 AND realtime_sequence>$2
             ORDER BY realtime_sequence LIMIT $3",
        )
        .bind(logical_owner_id)
        .bind(signed(after_sequence)?)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?
        .iter()
        .map(|row| {
            Ok(CallTranscriptionRealtimeTransitionV1 {
                sequence: u64::try_from(
                    row.try_get::<i64, _>("realtime_sequence")
                        .map_err(row_error)?,
                )
                .map_err(row_error)?,
                run_id: id16(row.try_get("run_id").map_err(row_error)?)?,
                state: state_from_code(row.try_get("state").map_err(row_error)?)?,
                state_revision: u64::try_from(
                    row.try_get::<i64, _>("state_revision").map_err(row_error)?,
                )
                .map_err(row_error)?,
                rejection: row
                    .try_get::<Option<i16>, _>("rejection_code")
                    .map_err(row_error)?
                    .map(rejection_from_code)
                    .transpose()?,
                occurred_at_unix_millis: row
                    .try_get("occurred_at_unix_millis")
                    .map_err(row_error)?,
            })
        })
        .collect()
    }
}
