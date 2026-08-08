//! Shared disposable Scheduler receipt persistence fixture.

use makosh_clock_protocol::UtcMillisV1;
use makosh_scheduler_persistence::{
    SchedulerPostgresStoreV1, SchedulerRunClaimV1, SchedulerRunTerminalResultV1,
    scheduler_storage_bundle_v1,
};
use makosh_scheduler_protocol::{
    ConcurrencyKeyV1, JobRunIdV1, MisfirePolicyV1, OverlapPolicyV1, RetryPolicyV1, ScheduleIdV1,
    SchedulePolicyV1, ScheduleRevisionV1, ScheduleRunLeaseV1, ScheduleTriggerV1,
    v1::{JobLeaseV1, JobRunOutcomeV1, JobRunReceiptV1},
};
use sqlx::{PgPool, postgres::PgPoolOptions, query, query_scalar};

pub const CLAIMED_AT: i64 = 1_000;
const URL: &str = "MAKOSH_SCHEDULER_POSTGRES_URL";

pub async fn pending_published_dispatch() -> (PgPool, SchedulerPostgresStoreV1, SchedulerRunClaimV1)
{
    let pool = PgPoolOptions::new()
        .connect(&required(URL))
        .await
        .expect("connect PostgreSQL");
    install_schema(&pool).await;
    let policy = policy();
    let key = ConcurrencyKeyV1::new("mailbox:opaque_receipt".into()).expect("key");
    let store = SchedulerPostgresStoreV1::new(pool.clone());
    store
        .ensure_concurrency_slot(&key, &policy, UtcMillisV1::new(CLAIMED_AT))
        .await
        .expect("slot");
    install_schedule(&pool, &key, &policy).await;
    let claim = claim(key, &policy);
    store.claim_due(&claim).await.expect("claim");
    publish_dispatch(&pool, &claim).await;
    (pool, store, claim)
}

pub fn receipt(claim: &SchedulerRunClaimV1) -> JobRunReceiptV1 {
    JobRunReceiptV1 {
        job_run_id: claim.run_id().bytes().to_vec(),
        command_message_id: claim.dispatch_message_id().to_vec(),
        lease: Some(JobLeaseV1 {
            run_id: claim.run_id().bytes().to_vec(),
            epoch: claim.lease_epoch(),
            expires_at_unix_millis: claim.lease_expires_at().value(),
        }),
        outcome: JobRunOutcomeV1::Accepted as i32,
        observed_at_unix_millis: CLAIMED_AT + 1,
    }
}

pub fn terminal_result(claim: &SchedulerRunClaimV1) -> SchedulerRunTerminalResultV1 {
    terminal_result_for(claim, JobRunOutcomeV1::Succeeded)
}

pub fn retryable_failure_result(claim: &SchedulerRunClaimV1) -> SchedulerRunTerminalResultV1 {
    terminal_result_for(claim, JobRunOutcomeV1::RetryableFailed)
}

pub fn failed_result(claim: &SchedulerRunClaimV1) -> SchedulerRunTerminalResultV1 {
    terminal_result_for(claim, JobRunOutcomeV1::Failed)
}

pub async fn run_state(pool: &PgPool) -> String {
    query_scalar("SELECT state FROM makosh_platform.scheduler_runs WHERE run_id = $1")
        .bind(vec![31_u8; 16])
        .fetch_one(pool)
        .await
        .expect("run state")
}

pub async fn acceptance_count(pool: &PgPool) -> i64 {
    query_scalar("SELECT COUNT(*) FROM makosh_platform.scheduler_run_acceptances")
        .fetch_one(pool)
        .await
        .expect("acceptance count")
}

pub async fn active_runs(pool: &PgPool) -> i32 {
    query_scalar("SELECT active_runs FROM makosh_platform.scheduler_concurrency")
        .fetch_one(pool)
        .await
        .expect("active runs")
}

pub async fn retry_due_at(pool: &PgPool) -> Option<i64> {
    query_scalar("SELECT next_attempt_at_unix_ms FROM makosh_platform.scheduler_run_retries")
        .fetch_one(pool)
        .await
        .expect("retry due")
}

pub fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} must be set"))
}

async fn install_schema(pool: &PgPool) {
    sqlx::raw_sql("DROP SCHEMA IF EXISTS makosh_platform CASCADE; CREATE SCHEMA makosh_platform;")
        .execute(pool)
        .await
        .expect("fresh receipt schema");
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
    query("INSERT INTO makosh_platform.scheduler_schedules (schedule_id, schedule_revision, job_owner, job_name, job_major, contract_name, contract_revision, contract_schema_sha256, scope_id, concurrency_key, max_parallelism, enabled, policy_bytes, next_due_at_unix_ms, updated_at_unix_ms) VALUES ($1, 1, 'platform', 'maintenance', 1, 'platform.maintenance', 1, $2, 'scope:technical', $3, 1, TRUE, $4, $5, $5)")
        .bind(vec![11_u8; 16]).bind(vec![7_u8; 32]).bind(key.value()).bind(policy.canonical_bytes()).bind(CLAIMED_AT)
        .execute(pool).await.expect("schedule");
}

async fn publish_dispatch(pool: &PgPool, claim: &SchedulerRunClaimV1) {
    query("INSERT INTO makosh_platform.scheduler_dispatches (run_id, lease_epoch, message_id, envelope_sha256, exact_envelope_bytes, state, created_at_unix_ms) VALUES ($1, $2, $3, $4, $5, 'published', $6)")
        .bind(claim.run_id().bytes().to_vec()).bind(claim.lease_epoch() as i64).bind(claim.dispatch_message_id().to_vec()).bind(vec![1_u8; 32]).bind(vec![2_u8]).bind(CLAIMED_AT)
        .execute(pool).await.expect("published dispatch");
    query("UPDATE makosh_platform.scheduler_runs SET state = 'dispatched' WHERE run_id = $1")
        .bind(claim.run_id().bytes().to_vec())
        .execute(pool)
        .await
        .expect("dispatched run");
}

fn claim(key: ConcurrencyKeyV1, policy: &SchedulePolicyV1) -> SchedulerRunClaimV1 {
    let lease = ScheduleRunLeaseV1::new(
        JobRunIdV1::new([31; 16]).expect("run"),
        ScheduleIdV1::new([11; 16]).expect("schedule"),
        ScheduleRevisionV1::new(1).expect("revision"),
        1,
        UtcMillisV1::new(CLAIMED_AT + 10_000),
    )
    .expect("lease");
    SchedulerRunClaimV1::new(
        lease,
        UtcMillisV1::new(CLAIMED_AT),
        UtcMillisV1::new(CLAIMED_AT),
        UtcMillisV1::new(CLAIMED_AT + 60_000),
        key,
        policy,
        [51; 16],
        [51; 32],
    )
    .expect("claim")
}

fn policy() -> SchedulePolicyV1 {
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

fn terminal_result_for(
    claim: &SchedulerRunClaimV1,
    outcome: JobRunOutcomeV1,
) -> SchedulerRunTerminalResultV1 {
    let mut value = receipt(claim);
    value.outcome = outcome as i32;
    SchedulerRunTerminalResultV1::try_from(&value).expect("terminal receipt")
}
