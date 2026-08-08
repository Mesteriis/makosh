//! Disposable PostgreSQL proof for Scheduler worker lease renewal and expiry fencing.

use makosh_clock_protocol::UtcMillisV1;
use makosh_scheduler_persistence::{
    SchedulerPostgresStoreV1, SchedulerRunClaimErrorV1, SchedulerRunClaimV1,
    scheduler_storage_bundle_v1,
};
use makosh_scheduler_protocol::{
    ConcurrencyKeyV1, JobRunIdV1, MisfirePolicyV1, OverlapPolicyV1, RetryPolicyV1, ScheduleIdV1,
    SchedulePolicyV1, ScheduleRevisionV1, ScheduleRunLeaseV1, ScheduleTriggerV1,
};
use sqlx::{PgPool, postgres::PgPoolOptions};

const URL: &str = "MAKOSH_SCHEDULER_POSTGRES_URL";
const CLAIMED_AT: i64 = 1_000;

#[test]
fn initial_lease_cannot_outlive_the_run_deadline() {
    let policy = policy();
    let lease = ScheduleRunLeaseV1::new(
        JobRunIdV1::new([60; 16]).expect("run"),
        ScheduleIdV1::new([60; 16]).expect("schedule"),
        ScheduleRevisionV1::new(1).expect("revision"),
        1,
        UtcMillisV1::new(CLAIMED_AT + 30_001),
    )
    .expect("lease");
    assert!(
        SchedulerRunClaimV1::new(
            lease,
            UtcMillisV1::new(CLAIMED_AT),
            UtcMillisV1::new(CLAIMED_AT),
            UtcMillisV1::new(CLAIMED_AT + 60_000),
            ConcurrencyKeyV1::new("mailbox:opaque_renewal".into()).expect("key"),
            &policy,
            [60; 16],
            [60; 32],
        )
        .is_err()
    );
}

#[tokio::test]
#[ignore = "requires the disposable Scheduler PostgreSQL contour"]
async fn renewed_lease_blocks_the_next_same_key_run_and_expired_worker_is_fenced() {
    let pool = PgPoolOptions::new()
        .connect(&required(URL))
        .await
        .expect("connect PostgreSQL");
    install_schema(&pool).await;
    let store = SchedulerPostgresStoreV1::new(pool.clone());
    let policy = policy();
    let key = ConcurrencyKeyV1::new("mailbox:opaque_renewal".into()).expect("key");
    store
        .ensure_concurrency_slot(&key, &policy, UtcMillisV1::new(CLAIMED_AT))
        .await
        .expect("slot");
    install_schedule(&pool, 61, &key, &policy).await;
    install_schedule(&pool, 62, &key, &policy).await;

    let first = claim(61, 61, key.clone(), &policy, CLAIMED_AT, 11_000);
    store.claim_due(&first).await.expect("first claim");
    store
        .renew_claim_lease(&first, UtcMillisV1::new(5_000), UtcMillisV1::new(15_000))
        .await
        .expect("timely renewal below the original deadline");

    let next = claim(62, 62, key, &policy, 12_000, 22_000);
    assert_eq!(
        store.claim_due(&next).await,
        Err(SchedulerRunClaimErrorV1::ConcurrencyExhausted)
    );
    assert_eq!(
        store.finish_claim(&first, UtcMillisV1::new(15_000)).await,
        Err(SchedulerRunClaimErrorV1::Denied)
    );
    assert_eq!(
        store.fail_claim(&first, UtcMillisV1::new(15_000)).await,
        Err(SchedulerRunClaimErrorV1::Denied)
    );
    assert_eq!(
        store
            .renew_claim_lease(&first, UtcMillisV1::new(15_000), UtcMillisV1::new(16_000),)
            .await,
        Err(SchedulerRunClaimErrorV1::Denied)
    );
    store
        .claim_due(&claim(
            63,
            62,
            next.concurrency_key().clone(),
            &policy,
            15_001,
            25_001,
        ))
        .await
        .expect("expired run releases its key only for a new fenced run");
}

async fn install_schema(pool: &PgPool) {
    sqlx::raw_sql("DROP SCHEMA IF EXISTS makosh_platform CASCADE; CREATE SCHEMA makosh_platform;")
        .execute(pool)
        .await
        .expect("fresh lease schema");
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

async fn install_schedule(
    pool: &PgPool,
    id: u8,
    key: &ConcurrencyKeyV1,
    policy: &SchedulePolicyV1,
) {
    sqlx::query("INSERT INTO makosh_platform.scheduler_schedules (schedule_id, schedule_revision, job_owner, job_name, job_major, contract_name, contract_revision, contract_schema_sha256, scope_id, concurrency_key, max_parallelism, enabled, policy_bytes, next_due_at_unix_ms, updated_at_unix_ms) VALUES ($1, 1, 'platform', 'maintenance', 1, 'platform.maintenance', 1, $2, 'scope:technical', $3, 1, TRUE, $4, $5, $5)")
        .bind(vec![id; 16]).bind(vec![7_u8; 32]).bind(key.value()).bind(policy.canonical_bytes()).bind(CLAIMED_AT)
        .execute(pool).await.expect("schedule");
}

fn claim(
    run: u8,
    schedule: u8,
    key: ConcurrencyKeyV1,
    policy: &SchedulePolicyV1,
    claimed_at: i64,
    expires_at: i64,
) -> SchedulerRunClaimV1 {
    let lease = ScheduleRunLeaseV1::new(
        JobRunIdV1::new([run; 16]).expect("run"),
        ScheduleIdV1::new([schedule; 16]).expect("schedule"),
        ScheduleRevisionV1::new(1).expect("revision"),
        1,
        UtcMillisV1::new(expires_at),
    )
    .expect("lease");
    SchedulerRunClaimV1::new(
        lease,
        UtcMillisV1::new(CLAIMED_AT),
        UtcMillisV1::new(claimed_at),
        UtcMillisV1::new(claimed_at + 60_000),
        key,
        policy,
        [run; 16],
        [run; 32],
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
        RetryPolicyV1::new(1, 1_000).expect("retry"),
        30_000,
        0,
    )
    .expect("policy")
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("Scheduler test requires {name}"))
}
