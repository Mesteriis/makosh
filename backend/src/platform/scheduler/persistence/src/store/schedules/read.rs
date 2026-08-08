use makosh_clock_protocol::UtcMillisV1;
use sqlx::query;

use super::record::PersistedScheduleRowV1;
use super::request::{SchedulerDueScheduleV1, SchedulerScheduleStoreErrorV1};
use crate::store::runs::SchedulerPostgresStoreV1;

const MAX_DUE_SCHEDULES: u16 = 64;

impl SchedulerPostgresStoreV1 {
    /// Reads a bounded due snapshot; concurrent claims remain fenced by PostgreSQL updates.
    pub async fn due_schedules(
        &self,
        now: UtcMillisV1,
        limit: u16,
    ) -> Result<Vec<SchedulerDueScheduleV1>, SchedulerScheduleStoreErrorV1> {
        if limit == 0 || limit > MAX_DUE_SCHEDULES {
            return Err(SchedulerScheduleStoreErrorV1::InvalidLimit);
        }
        let rows = query(
            "SELECT schedule_id, schedule_revision, job_owner, job_name, job_major, contract_name, contract_revision, contract_schema_sha256, scope_id, concurrency_key, max_parallelism, enabled, policy_bytes, next_due_at_unix_ms FROM makosh_platform.scheduler_schedules WHERE enabled = TRUE AND next_due_at_unix_ms <= $1 ORDER BY next_due_at_unix_ms, schedule_id LIMIT $2",
        )
        .bind(now.value())
        .bind(i64::from(limit))
        .fetch_all(self.pool())
        .await
        .map_err(|_| SchedulerScheduleStoreErrorV1::Unavailable)?;
        rows.into_iter()
            .map(PersistedScheduleRowV1::from_row)
            .map(|result| result.and_then(PersistedScheduleRowV1::into_due))
            .collect()
    }
}
