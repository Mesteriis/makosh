//! Disposable PostgreSQL proof that a due claim and its exact dispatch are one durable unit.

use makosh_clock_protocol::UtcMillisV1;
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
};
use makosh_scheduler_persistence::{
    SchedulerDispatchAdmissionV1, SchedulerDispatchClaimErrorV1, SchedulerDispatchClaimV1,
    SchedulerMaterializationSourceV1, SchedulerPostgresStoreV1, SchedulerRunClaimV1,
    scheduler_storage_bundle_v1,
};
use makosh_scheduler_protocol::{
    ConcurrencyKeyV1, JobRunIdV1, MisfirePolicyV1, OverlapPolicyV1, RetryPolicyV1, ScheduleIdV1,
    SchedulePolicyV1, ScheduleRevisionV1, ScheduleRunLeaseV1, ScheduleTriggerV1,
};
use prost::Message;
use prost_types::Timestamp;
use sqlx::{PgPool, Row, postgres::PgPoolOptions};

const CLAIMED_AT: i64 = 1_000;
const URL: &str = "MAKOSH_SCHEDULER_POSTGRES_URL";

#[tokio::test]
#[ignore = "requires the disposable Scheduler PostgreSQL contour"]
async fn due_claim_persists_the_exact_dispatch_in_its_transaction() {
    let pool = PgPoolOptions::new()
        .connect(&required(URL))
        .await
        .expect("connect PostgreSQL");
    install_schema(&pool).await;
    let policy = policy();
    let key = ConcurrencyKeyV1::new("mailbox:opaque_dispatch".into()).expect("key");
    let store = SchedulerPostgresStoreV1::new(pool.clone());
    store
        .ensure_concurrency_slot(&key, &policy, UtcMillisV1::new(CLAIMED_AT))
        .await
        .expect("slot");
    install_schedule(&pool, &key, &policy).await;

    let claim = claim(key, &policy);
    let record = OutboxRecordV1::accept(envelope(claim.dispatch_message_id()).encode_to_vec())
        .expect("valid durable command");
    store
        .claim_due_with_dispatch(
            &SchedulerDispatchClaimV1::new(claim.clone(), record.clone()).expect("binding"),
        )
        .await
        .expect("atomic due claim and dispatch");

    assert_eq!(
        dispatch_row(&pool).await,
        ("pending".into(), record.exact_bytes().to_vec())
    );
    assert_eq!(run_state(&pool).await, "pending_dispatch");
    assert_eq!(active_runs(&pool).await, 1);
}

#[tokio::test]
#[ignore = "requires the disposable Scheduler PostgreSQL contour"]
async fn due_materialization_is_admitted_exactly_once_before_advancing_schedule() {
    let pool = PgPoolOptions::new()
        .connect(&required(URL))
        .await
        .expect("connect PostgreSQL");
    install_schema(&pool).await;
    let policy = policy();
    let key = ConcurrencyKeyV1::new("mailbox:opaque_materialization".into()).expect("key");
    let store = SchedulerPostgresStoreV1::new(pool.clone());
    store
        .ensure_concurrency_slot(&key, &policy, UtcMillisV1::new(CLAIMED_AT))
        .await
        .expect("slot");
    install_schedule(&pool, &key, &policy).await;
    let source =
        SchedulerMaterializationSourceV1::new("scheduler".into(), [8; 16], 1).expect("source");
    let denied = SchedulerDispatchAdmissionV1::new(["makosh.command.v1.other.job.v1".into()])
        .expect("admission");
    assert_eq!(
        store
            .materialize_due(UtcMillisV1::new(CLAIMED_AT), 1, &source, &denied)
            .await
            .expect("materialize denial")
            .denied(),
        1
    );
    assert_eq!(dispatch_count(&pool).await, 0);
    assert_eq!(next_due(&pool).await, CLAIMED_AT);

    let admitted =
        SchedulerDispatchAdmissionV1::new(["makosh.command.v1.platform.maintenance.v1".into()])
            .expect("admission");
    let outcome = store
        .materialize_due(UtcMillisV1::new(CLAIMED_AT), 1, &source, &admitted)
        .await
        .expect("materialize due fire");
    assert_eq!(outcome.dispatched(), 1);
    assert_eq!(dispatch_count(&pool).await, 1);
    assert_eq!(next_due(&pool).await, CLAIMED_AT + 60_000);
    assert_eq!(
        store
            .materialize_due(UtcMillisV1::new(CLAIMED_AT), 1, &source, &admitted)
            .await
            .expect("repeat materialization")
            .dispatched(),
        0
    );
}

#[test]
fn dispatch_rejects_an_envelope_for_another_run() {
    let policy = policy();
    let key = ConcurrencyKeyV1::new("mailbox:opaque_dispatch".into()).expect("key");
    let claim = claim(key, &policy);
    let record = OutboxRecordV1::accept(envelope([99; 16]).encode_to_vec()).expect("envelope");
    assert_eq!(
        SchedulerDispatchClaimV1::new(claim, record),
        Err(SchedulerDispatchClaimErrorV1::MessageIdMismatch)
    );
}

async fn install_schema(pool: &PgPool) {
    sqlx::raw_sql("DROP SCHEMA IF EXISTS makosh_platform CASCADE; CREATE SCHEMA makosh_platform;")
        .execute(pool)
        .await
        .expect("fresh dispatch schema");
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

fn claim(key: ConcurrencyKeyV1, policy: &SchedulePolicyV1) -> SchedulerRunClaimV1 {
    let lease = ScheduleRunLeaseV1::new(
        JobRunIdV1::new([51; 16]).expect("run"),
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
        RetryPolicyV1::new(1, 1_000).expect("retry"),
        30_000,
        0,
    )
    .expect("policy")
}

fn envelope(message_id: [u8; 16]) -> DurableEnvelopeV1 {
    DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(ContractRefV1 {
            owner: "platform".into(),
            name: "maintenance".into(),
            major: 1,
            revision: 1,
            schema_sha256: vec![7; 32],
        }),
        source: Some(SourceRefV1 {
            module_id: "scheduler".into(),
            runtime_instance_id: vec![8; 16],
            runtime_generation: 1,
        }),
        recorded_at: Some(Timestamp {
            seconds: 1,
            nanos: 0,
        }),
        partition_key: b"scheduler".to_vec(),
        causation_message_id: Vec::new(),
        correlation_id: vec![2; 16],
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::System as i32,
            actor_id: b"scheduler".to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: b"scheduler".to_vec(),
            epoch: 1,
        }),
        semantics: Some(Semantics::Command(CommandMetadataV1 {
            command_id: vec![4; 16],
            target_capability: "job_execute".into(),
            idempotency_key: vec![5],
            deadline: Some(Timestamp {
                seconds: 2,
                nanos: 0,
            }),
            logical_attempt: 1,
        })),
        payload: vec![1],
    }
}

async fn dispatch_row(pool: &PgPool) -> (String, Vec<u8>) {
    let row = sqlx::query(
        "SELECT state, exact_envelope_bytes FROM makosh_platform.scheduler_dispatches WHERE message_id = $1",
    )
    .bind(vec![51_u8; 16])
    .fetch_one(pool)
    .await
    .expect("dispatch row");
    (row.get("state"), row.get("exact_envelope_bytes"))
}

async fn run_state(pool: &PgPool) -> String {
    sqlx::query("SELECT state FROM makosh_platform.scheduler_runs WHERE run_id = $1")
        .bind(vec![51_u8; 16])
        .fetch_one(pool)
        .await
        .expect("run row")
        .get("state")
}

async fn active_runs(pool: &PgPool) -> i32 {
    sqlx::query(
        "SELECT active_runs FROM makosh_platform.scheduler_concurrency WHERE concurrency_key = $1",
    )
    .bind("mailbox:opaque_dispatch")
    .fetch_one(pool)
    .await
    .expect("slot")
    .get("active_runs")
}

async fn dispatch_count(pool: &PgPool) -> i64 {
    sqlx::query("SELECT COUNT(*) AS count FROM makosh_platform.scheduler_dispatches")
        .fetch_one(pool)
        .await
        .expect("dispatch count")
        .get("count")
}

async fn next_due(pool: &PgPool) -> i64 {
    sqlx::query("SELECT next_due_at_unix_ms FROM makosh_platform.scheduler_schedules WHERE schedule_id = $1")
        .bind(vec![11_u8; 16])
        .fetch_one(pool)
        .await
        .expect("schedule")
        .get("next_due_at_unix_ms")
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("Scheduler test requires {name}"))
}
