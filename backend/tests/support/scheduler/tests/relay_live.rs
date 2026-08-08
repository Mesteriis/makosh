//! Disposable PostgreSQL proof that Scheduler dispatches survive broker outages.

use makosh_clock_protocol::UtcMillisV1;
use makosh_events_protocol::{
    delivery::{
        ExactOutboxPublisherPortV1, OutboxPublishReceiptV1, OutboxRecordV1, OutboxRelayErrorV1,
        OutboxRelayOutcomeV1, relay_once,
    },
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
};
use makosh_scheduler_persistence::{
    SchedulerDispatchClaimV1, SchedulerPostgresStoreV1, SchedulerRunClaimV1,
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
async fn unavailable_broker_keeps_scheduler_dispatch_pending_until_acknowledged() {
    let pool = PgPoolOptions::new()
        .connect(&required(URL))
        .await
        .expect("connect PostgreSQL");
    install_schema(&pool).await;
    let mut store = install_pending_dispatch(&pool).await;

    assert_eq!(
        relay_once(&mut store, &UnavailablePublisher).await,
        Err(OutboxRelayErrorV1::PublisherUnavailable)
    );
    assert_eq!(
        states(&pool).await,
        ("pending".into(), "pending_dispatch".into())
    );

    assert_eq!(
        relay_once(&mut store, &AcknowledgingPublisher).await,
        Ok(OutboxRelayOutcomeV1::Published {
            outbox_id: "scheduler_dispatch_33333333333333333333333333333333".into(),
            duplicate: false,
        })
    );
    assert_eq!(
        states(&pool).await,
        ("published".into(), "dispatched".into())
    );
    assert!(store.next_pending_dispatch().await.expect("read").is_none());
}

struct UnavailablePublisher;

impl ExactOutboxPublisherPortV1 for UnavailablePublisher {
    #[allow(clippy::manual_async_fn)] // The outbox publisher port requires a Send future.
    fn publish_exact(
        &self,
        _: &OutboxRecordV1,
    ) -> impl std::future::Future<Output = Result<OutboxPublishReceiptV1, OutboxRelayErrorV1>> + Send
    {
        async { Err(OutboxRelayErrorV1::PublisherUnavailable) }
    }
}

struct AcknowledgingPublisher;

impl ExactOutboxPublisherPortV1 for AcknowledgingPublisher {
    #[allow(clippy::manual_async_fn)] // The outbox publisher port requires a Send future.
    fn publish_exact(
        &self,
        _: &OutboxRecordV1,
    ) -> impl std::future::Future<Output = Result<OutboxPublishReceiptV1, OutboxRelayErrorV1>> + Send
    {
        async { OutboxPublishReceiptV1::new("MAKOSH_COMMAND_V1", 1, false) }
    }
}

async fn install_schema(pool: &PgPool) {
    sqlx::raw_sql("DROP SCHEMA IF EXISTS makosh_platform CASCADE; CREATE SCHEMA makosh_platform;")
        .execute(pool)
        .await
        .expect("fresh relay schema");
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

async fn install_pending_dispatch(pool: &PgPool) -> SchedulerPostgresStoreV1 {
    let store = SchedulerPostgresStoreV1::new(pool.clone());
    let policy = policy();
    let key = ConcurrencyKeyV1::new("mailbox:opaque_relay".into()).expect("key");
    store
        .ensure_concurrency_slot(&key, &policy, UtcMillisV1::new(CLAIMED_AT))
        .await
        .expect("slot");
    install_schedule(pool, &key, &policy).await;
    let claim = claim(key, &policy);
    let record = OutboxRecordV1::accept(envelope(claim.dispatch_message_id()).encode_to_vec())
        .expect("envelope");
    let dispatch = SchedulerDispatchClaimV1::new(claim, record).expect("dispatch binding");
    store
        .claim_due_with_dispatch(&dispatch)
        .await
        .expect("pending dispatch");
    store
}

async fn install_schedule(pool: &PgPool, key: &ConcurrencyKeyV1, policy: &SchedulePolicyV1) {
    sqlx::query("INSERT INTO makosh_platform.scheduler_schedules (schedule_id, schedule_revision, job_owner, job_name, job_major, contract_name, contract_revision, contract_schema_sha256, scope_id, concurrency_key, max_parallelism, enabled, policy_bytes, next_due_at_unix_ms, updated_at_unix_ms) VALUES ($1, 1, 'platform', 'maintenance', 1, 'platform.maintenance', 1, $2, 'scope:technical', $3, 1, TRUE, $4, $5, $5)")
        .bind(vec![11_u8; 16]).bind(vec![7_u8; 32]).bind(key.value()).bind(policy.canonical_bytes()).bind(CLAIMED_AT)
        .execute(pool).await.expect("schedule");
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

async fn states(pool: &PgPool) -> (String, String) {
    let dispatch =
        sqlx::query("SELECT state FROM makosh_platform.scheduler_dispatches WHERE message_id = $1")
            .bind(vec![51_u8; 16])
            .fetch_one(pool)
            .await
            .expect("dispatch row")
            .get("state");
    let run = sqlx::query("SELECT state FROM makosh_platform.scheduler_runs WHERE run_id = $1")
        .bind(vec![31_u8; 16])
        .fetch_one(pool)
        .await
        .expect("run row")
        .get("state");
    (dispatch, run)
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("Scheduler test requires {name}"))
}
