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
        .map_err(unavailable)?;
        rows.into_iter()
            .map(PersistedScheduleRowV1::from_row)
            .map(|result| result.and_then(PersistedScheduleRowV1::into_due))
            .collect()
    }
}

fn unavailable(error: sqlx::Error) -> SchedulerScheduleStoreErrorV1 {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        if let Some(database) = error.as_database_error() {
            let sqlstate = database
                .code()
                .filter(|value| {
                    value.len() <= 5 && value.bytes().all(|byte| byte.is_ascii_alphanumeric())
                })
                .unwrap_or_else(|| "unknown".into());
            let message = bounded_diagnostic(database.message());
            eprintln!(
                "developer_scheduler_persistence_failure=due_schedules:sqlstate_{sqlstate}:{message}"
            );
        } else {
            let class = match error {
                sqlx::Error::PoolTimedOut => "pool_timeout",
                sqlx::Error::PoolClosed => "pool_closed",
                sqlx::Error::WorkerCrashed => "worker_crashed",
                sqlx::Error::Io(_) => "io",
                sqlx::Error::Tls(_) => "tls",
                sqlx::Error::Protocol(_) => "protocol",
                _ => "driver",
            };
            eprintln!("developer_scheduler_persistence_failure=due_schedules:{class}");
        }
    }
    SchedulerScheduleStoreErrorV1::Unavailable
}

fn bounded_diagnostic(value: &str) -> String {
    value
        .chars()
        .take(160)
        .map(|character| {
            if character.is_ascii_alphanumeric()
                || matches!(character, ' ' | '_' | '-' | '.' | ':' | '"')
            {
                character
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::bounded_diagnostic;

    #[test]
    fn database_diagnostics_are_bounded_and_single_line() {
        let diagnostic = bounded_diagnostic(&format!("line\n{}", "x".repeat(256)));
        assert!(diagnostic.len() <= 160);
        assert!(!diagnostic.contains('\n'));
    }
}
