use sqlx::Row;

use crate::{
    MailContactsSyncPersistenceErrorV1, MailContactsSyncPersistenceV1,
    model::{
        MAIL_CONTACTS_SYNC_REALTIME_LIMIT_V1, MailContactsSyncRealtimeTransitionV1, nonzero,
        valid_identity,
    },
    repository::{rejection_from_code, state_from_code},
};

impl MailContactsSyncPersistenceV1 {
    pub async fn client_realtime_window(
        &self,
        logical_owner_id: &str,
        after_sequence: Option<u64>,
        limit: u16,
    ) -> Result<Vec<MailContactsSyncRealtimeTransitionV1>, MailContactsSyncPersistenceErrorV1> {
        if !valid_identity(logical_owner_id)
            || after_sequence == Some(0)
            || !(1..=MAIL_CONTACTS_SYNC_REALTIME_LIMIT_V1).contains(&limit)
        {
            return Err(MailContactsSyncPersistenceErrorV1::InvalidInput);
        }
        let rows = if let Some(after) = after_sequence {
            let after = i64::try_from(after)
                .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidInput)?;
            sqlx::query(
                "SELECT realtime_sequence, run_id, state, state_revision,
                        rejection_code, occurred_at_unix_millis
                 FROM makosh_data.mail_contacts_sync_realtime
                 WHERE logical_owner_id = $1 AND realtime_sequence > $2
                 ORDER BY realtime_sequence LIMIT $3",
            )
            .bind(logical_owner_id)
            .bind(after)
            .bind(i64::from(limit))
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                "SELECT realtime_sequence, run_id, state, state_revision,
                        rejection_code, occurred_at_unix_millis FROM (
                   SELECT realtime_sequence, run_id, state, state_revision,
                          rejection_code, occurred_at_unix_millis
                   FROM makosh_data.mail_contacts_sync_realtime
                   WHERE logical_owner_id = $1
                   ORDER BY realtime_sequence DESC LIMIT $2
                 ) replay ORDER BY realtime_sequence",
            )
            .bind(logical_owner_id)
            .bind(i64::from(limit))
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|_| MailContactsSyncPersistenceErrorV1::StorageUnavailable)?;
        rows.into_iter().map(realtime_from_row).collect()
    }
}

fn realtime_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<MailContactsSyncRealtimeTransitionV1, MailContactsSyncPersistenceErrorV1> {
    let sequence: i64 = row
        .try_get("realtime_sequence")
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
    let run_id: Vec<u8> = row
        .try_get("run_id")
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
    let state_revision: i64 = row
        .try_get("state_revision")
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
    let occurred_at_unix_millis: i64 = row
        .try_get("occurred_at_unix_millis")
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
    let run_id: [u8; 16] = run_id
        .try_into()
        .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?;
    if sequence <= 0 || state_revision <= 0 || occurred_at_unix_millis <= 0 || !nonzero(&run_id) {
        return Err(MailContactsSyncPersistenceErrorV1::InvalidRow);
    }
    Ok(MailContactsSyncRealtimeTransitionV1 {
        sequence: u64::try_from(sequence)
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?,
        run_id,
        state: state_from_code(
            row.try_get("state")
                .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?,
        )?,
        state_revision: u64::try_from(state_revision)
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?,
        rejection: row
            .try_get::<Option<i16>, _>("rejection_code")
            .map_err(|_| MailContactsSyncPersistenceErrorV1::InvalidRow)?
            .map(rejection_from_code)
            .transpose()?,
        occurred_at_unix_millis,
    })
}
