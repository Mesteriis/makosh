//! Disposable PostgreSQL proof for fenced, durable Scheduler retry state.

use makosh_clock_protocol::UtcMillisV1;
use makosh_scheduler_persistence::{
    RetryFailureOutcomeV1, SchedulerPostgresStoreV1, SchedulerRunClaimErrorV1, SchedulerRunClaimV1,
    scheduler_storage_bundle_v1,
};
use makosh_scheduler_protocol::{
    ConcurrencyKeyV1, JobRunIdV1, MisfirePolicyV1, OverlapPolicyV1, RetryPolicyV1, ScheduleIdV1,
    SchedulePolicyV1, ScheduleRevisionV1, ScheduleRunLeaseV1, ScheduleTriggerV1,
};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};

const URL: &str = "MAKOSH_SCHEDULER_POSTGRES_URL";
const CLAIMED_AT: i64 = 1_000;

#[tokio::test]
#[ignore = "requires the disposable Scheduler PostgreSQL contour"]
async fn failed_run_keeps_identity_and_persists_its_next_retry() {
    let pool = PgPoolOptions::new()
        .connect(&required(URL))
        .await
        .expect("connect PostgreSQL");
    install_schema(&pool).await;
    let store = SchedulerPostgresStoreV1::new(pool.clone());
    let policy = retry_policy();
    let key = ConcurrencyKeyV1::new("mailbox:opaque_retry".into()).expect("key");
    store
        .ensure_concurrency_slot(&key, &policy, UtcMillisV1::new(CLAIMED_AT))
        .await
        .expect("slot");
    install_schedule(&pool, &key, &policy).await;
    let initial_claim = claim(key.clone(), &policy, 1, CLAIMED_AT);
    store
        .claim_due(&initial_claim)
        .await
        .expect("claim retryable run");
    assert_eq!(
        store
            .fail_claim(&initial_claim, UtcMillisV1::new(CLAIMED_AT + 1_000))
            .await,
        Ok(RetryFailureOutcomeV1::RetryAt(UtcMillisV1::new(
            CLAIMED_AT + 2_000
        )))
    );
    assert_eq!(retry_due_at(&pool).await, CLAIMED_AT + 2_000);
    let retry_claim = claim(key, &policy, 2, CLAIMED_AT + 2_000);
    store
        .claim_retry(&retry_claim)
        .await
        .expect("resume retry under a new lease epoch");
    assert_eq!(
        run_epoch_and_state(&pool).await,
        (2, "pending_dispatch".into())
    );
    assert_eq!(
        store
            .finish_claim(&initial_claim, UtcMillisV1::new(CLAIMED_AT + 2_001))
            .await,
        Err(SchedulerRunClaimErrorV1::Denied)
    );
    assert_eq!(active_runs(&pool).await, 1);
    store
        .finish_claim(&retry_claim, UtcMillisV1::new(CLAIMED_AT + 2_002))
        .await
        .expect("only the replacement lease can complete the retry");
    assert_eq!(active_runs(&pool).await, 0);
}

async fn install_schema(pool: &PgPool) {
    sqlx::raw_sql("DROP SCHEMA IF EXISTS makosh_platform CASCADE; CREATE SCHEMA makosh_platform;")
        .execute(pool)
        .await
        .expect("fresh retry schema");
    for step in scheduler_storage_bundle_v1().steps {
        sqlx::raw_sql(sqlx::AssertSqlSafe(
            std::str::from_utf8(&step.forward_sql_utf8)
                .expect("migration UTF-8")
                .to_owned(),
        ))
        .execute(pool)
        .await
        .expect("migration");
    }
}

async fn install_schedule(pool: &PgPool, key: &ConcurrencyKeyV1, policy: &SchedulePolicyV1) {
    sqlx::query("INSERT INTO makosh_platform.scheduler_schedules (schedule_id, schedule_revision, job_owner, job_name, job_major, contract_name, contract_revision, contract_schema_sha256, scope_id, concurrency_key, max_parallelism, enabled, policy_bytes, next_due_at_unix_ms, updated_at_unix_ms) VALUES ($1, 1, 'platform', 'maintenance', 1, 'platform.maintenance', 1, $2, 'scope:technical', $3, 1, TRUE, $4, $5, $5)")
        .bind(vec![11_u8; 16]).bind(vec![7_u8; 32]).bind(key.value()).bind(policy.canonical_bytes()).bind(CLAIMED_AT)
        .execute(pool).await.expect("schedule");
}

fn claim(
    key: ConcurrencyKeyV1,
    policy: &SchedulePolicyV1,
    lease_epoch: u64,
    claimed_at: i64,
) -> SchedulerRunClaimV1 {
    let lease = ScheduleRunLeaseV1::new(
        JobRunIdV1::new([51; 16]).expect("run"),
        ScheduleIdV1::new([11; 16]).expect("schedule"),
        ScheduleRevisionV1::new(1).expect("revision"),
        lease_epoch,
        UtcMillisV1::new(claimed_at + 10_000),
    )
    .expect("lease");
    SchedulerRunClaimV1::new(
        lease,
        UtcMillisV1::new(CLAIMED_AT),
        UtcMillisV1::new(claimed_at),
        UtcMillisV1::new(CLAIMED_AT + 60_000),
        key,
        policy,
        [lease_epoch as u8; 16],
        [51; 32],
    )
    .expect("claim")
}

fn retry_policy() -> SchedulePolicyV1 {
    SchedulePolicyV1::new(
        ScheduleTriggerV1::FixedInterval {
            interval_millis: 60_000,
        },
        OverlapPolicyV1::Forbid,
        MisfirePolicyV1::Skip,
        RetryPolicyV1::new(2, 1_000).expect("retry"),
        30_000,
        0,
    )
    .expect("policy")
}

async fn retry_due_at(pool: &PgPool) -> i64 {
    sqlx::query("SELECT next_attempt_at_unix_ms FROM makosh_platform.scheduler_run_retries WHERE run_id = $1")
        .bind(vec![51_u8; 16]).fetch_one(pool).await.expect("retry state").get("next_attempt_at_unix_ms")
}

async fn run_epoch_and_state(pool: &PgPool) -> (i64, String) {
    let row = sqlx::query(
        "SELECT lease_epoch, state FROM makosh_platform.scheduler_runs WHERE run_id = $1",
    )
    .bind(vec![51_u8; 16])
    .fetch_one(pool)
    .await
    .expect("retry run");
    (row.get("lease_epoch"), row.get("state"))
}

async fn active_runs(pool: &PgPool) -> i32 {
    sqlx::query(
        "SELECT active_runs FROM makosh_platform.scheduler_concurrency WHERE concurrency_key = $1",
    )
    .bind("mailbox:opaque_retry")
    .fetch_one(pool)
    .await
    .expect("retry slot")
    .get("active_runs")
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("Scheduler test requires {name}"))
}
