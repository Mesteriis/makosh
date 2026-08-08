use makosh_communication_task_candidate_core::{
    CommunicationTaskCandidateRejectionCodeV1, CommunicationTaskCandidateStateV1,
};
use sqlx::{Postgres, Row, Transaction};

use crate::{
    CommunicationTaskCandidatePersistenceErrorV1,
    model::{COMMUNICATION_TASK_CANDIDATE_REALTIME_LIMIT_V1, valid_identity, valid_timestamp},
    repository::state_from_code,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommunicationTaskCandidateRealtimeTransitionV1 {
    pub sequence: u64,
    pub run_id: [u8; 16],
    pub state: CommunicationTaskCandidateStateV1,
    pub state_revision: u64,
    pub rejection: Option<CommunicationTaskCandidateRejectionCodeV1>,
    pub occurred_at_unix_millis: i64,
}

impl crate::CommunicationTaskCandidatePersistenceV1 {
    pub async fn client_realtime_window(
        &self,
        logical_owner_id: &str,
        after_sequence: Option<u64>,
        limit: u16,
    ) -> Result<
        Vec<CommunicationTaskCandidateRealtimeTransitionV1>,
        CommunicationTaskCandidatePersistenceErrorV1,
    > {
        if !valid_identity(logical_owner_id)
            || after_sequence == Some(0)
            || !(1..=COMMUNICATION_TASK_CANDIDATE_REALTIME_LIMIT_V1).contains(&limit)
        {
            return Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidInput);
        }
        let rows = if let Some(after) = after_sequence {
            let after = i64::try_from(after)
                .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::InvalidInput)?;
            sqlx::query(
                "SELECT realtime_sequence, run_id, state, state_revision,
                        rejection_code, occurred_at_unix_millis
                 FROM makosh_data.communication_task_candidate_extraction_realtime
                 WHERE logical_owner_id = $1 AND realtime_sequence > $2
                 ORDER BY realtime_sequence
                 LIMIT $3",
            )
            .bind(logical_owner_id)
            .bind(after)
            .bind(i64::from(limit))
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                "SELECT realtime_sequence, run_id, state, state_revision,
                        rejection_code, occurred_at_unix_millis
                 FROM (
                   SELECT realtime_sequence, run_id, state, state_revision,
                          rejection_code, occurred_at_unix_millis
                   FROM makosh_data.communication_task_candidate_extraction_realtime
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
        .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::StorageUnavailable)?;
        rows.into_iter().map(transition_from_row).collect()
    }
}

pub(crate) async fn insert_realtime_transition(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: &[u8; 16],
    occurred_at_unix_millis: i64,
) -> Result<(), CommunicationTaskCandidatePersistenceErrorV1> {
    let inserted = sqlx::query(
        "INSERT INTO makosh_data.communication_task_candidate_extraction_realtime (
           logical_owner_id, run_id, state, state_revision,
           rejection_code, occurred_at_unix_millis
         )
         SELECT logical_owner_id, run_id, state, state_revision,
                rejection_code, $1
         FROM makosh_data.communication_task_candidate_extraction_runs
         WHERE logical_owner_id = $2 AND run_id = $3",
    )
    .bind(occurred_at_unix_millis)
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .execute(&mut **transaction)
    .await
    .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::StorageUnavailable)?
    .rows_affected();
    if inserted == 1 {
        Ok(())
    } else {
        Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)
    }
}

fn transition_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<
    CommunicationTaskCandidateRealtimeTransitionV1,
    CommunicationTaskCandidatePersistenceErrorV1,
> {
    let sequence: i64 = row
        .try_get("realtime_sequence")
        .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)?;
    let run_id: Vec<u8> = row
        .try_get("run_id")
        .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)?;
    let state_revision: i64 = row
        .try_get("state_revision")
        .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)?;
    let rejection: Option<i16> = row
        .try_get("rejection_code")
        .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)?;
    let occurred_at_unix_millis: i64 = row
        .try_get("occurred_at_unix_millis")
        .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)?;
    if sequence <= 0 || state_revision <= 0 || !valid_timestamp(occurred_at_unix_millis) {
        return Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidRow);
    }
    Ok(CommunicationTaskCandidateRealtimeTransitionV1 {
        sequence: u64::try_from(sequence)
            .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)?,
        run_id: run_id
            .try_into()
            .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)?,
        state: state_from_code(
            row.try_get("state")
                .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)?,
        )?,
        state_revision: u64::try_from(state_revision)
            .map_err(|_| CommunicationTaskCandidatePersistenceErrorV1::InvalidRow)?,
        rejection: rejection.map(rejection_from_code).transpose()?,
        occurred_at_unix_millis,
    })
}

fn rejection_from_code(
    value: i16,
) -> Result<CommunicationTaskCandidateRejectionCodeV1, CommunicationTaskCandidatePersistenceErrorV1>
{
    match value {
        1 => Ok(CommunicationTaskCandidateRejectionCodeV1::InvalidRequest),
        2 => Ok(CommunicationTaskCandidateRejectionCodeV1::SourceRejected),
        3 => Ok(CommunicationTaskCandidateRejectionCodeV1::ExtractionRejected),
        4 => Ok(CommunicationTaskCandidateRejectionCodeV1::Policy),
        _ => Err(CommunicationTaskCandidatePersistenceErrorV1::InvalidRow),
    }
}
