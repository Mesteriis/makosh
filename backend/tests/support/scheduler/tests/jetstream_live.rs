//! Disposable PostgreSQL and JetStream proof for Scheduler exact-byte delivery.

use std::time::Duration;

use futures_util::StreamExt;
use makosh_clock_protocol::UtcMillisV1;
use makosh_events_jetstream::{
    ConsumerBudgetV1, ConsumerSpecV1, DurableSubjectV1, EventHubTopologyPlanV1, JetStreamClient,
    NatsPasswordCredentialV1, RuntimeNatsIdentity, RuntimeOutboxPublisherV1,
    RuntimePublishPermitV1, StreamBudgetV1, StreamKindV1, StreamSpecV1,
};
use makosh_events_protocol::{
    delivery::{OutboxRecordV1, relay_once},
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
const ENDPOINT: &str = "MAKOSH_SCHEDULER_NATS_ENDPOINT";
const POSTGRES_URL: &str = "MAKOSH_SCHEDULER_POSTGRES_URL";

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires the disposable Scheduler PostgreSQL and JetStream contour"]
async fn scheduler_relay_publishes_persisted_bytes_once_to_jetstream() {
    let endpoint = required(ENDPOINT);
    configure_event_hub(&endpoint).await;
    let pool = PgPoolOptions::new()
        .connect(&required(POSTGRES_URL))
        .await
        .expect("connect PostgreSQL");
    install_schema(&pool).await;
    let (mut store, expected) = install_pending_dispatch(&pool).await;
    let runtime = JetStreamClient::connect_runtime(
        &endpoint,
        RuntimeNatsIdentity::new("scheduler_runtime", 1, 1).expect("identity"),
        NatsPasswordCredentialV1::new("scheduler_runtime", "scheduler-runtime-test-only")
            .expect("credential"),
    )
    .await
    .expect("runtime connection");
    let permit = RuntimePublishPermitV1::new(
        "scheduler_registration",
        "scheduler_runtime",
        1,
        1,
        vec![
            DurableSubjectV1::new(StreamKindV1::Command, "platform", "maintenance", 1)
                .expect("command subject"),
        ],
    )
    .expect("permit");
    let publisher = RuntimeOutboxPublisherV1::new(&runtime, &permit);

    relay_once(&mut store, &publisher)
        .await
        .expect("publish saved dispatch");
    assert_eq!(dispatch_state(&pool).await, "published");
    assert_exact_delivery(&endpoint, &expected).await;
}

async fn configure_event_hub(endpoint: &str) {
    let hub = JetStreamClient::connect_event_hub(
        endpoint,
        NatsPasswordCredentialV1::new("event_hub", "event-hub-test-only").expect("credential"),
    )
    .await
    .expect("event hub connection");
    let stream = StreamSpecV1::new(
        StreamKindV1::Command,
        StreamBudgetV1::new(1_048_576, Duration::from_secs(3600), 1).expect("budget"),
    );
    let consumer = ConsumerSpecV1::new(
        StreamKindV1::Command,
        "scheduler_delivery",
        "makosh.command.v1.platform.maintenance.v1",
        ConsumerBudgetV1::new(16, 3, Duration::from_secs(2)).expect("budget"),
    )
    .expect("consumer");
    hub.reconcile(&EventHubTopologyPlanV1::new(vec![stream], vec![consumer]).expect("topology"))
        .await
        .expect("topology reconciliation");
}

async fn install_schema(pool: &PgPool) {
    sqlx::raw_sql("DROP SCHEMA IF EXISTS makosh_platform CASCADE; CREATE SCHEMA makosh_platform;")
        .execute(pool)
        .await
        .expect("fresh scheduler schema");
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

async fn install_pending_dispatch(pool: &PgPool) -> (SchedulerPostgresStoreV1, Vec<u8>) {
    let store = SchedulerPostgresStoreV1::new(pool.clone());
    let policy = policy();
    let key = ConcurrencyKeyV1::new("mailbox:opaque_delivery".into()).expect("key");
    store
        .ensure_concurrency_slot(&key, &policy, UtcMillisV1::new(CLAIMED_AT))
        .await
        .expect("slot");
    install_schedule(pool, &key, &policy).await;
    let claim = claim(key, &policy);
    let expected = envelope(claim.dispatch_message_id()).encode_to_vec();
    let record = OutboxRecordV1::accept(expected.clone()).expect("durable envelope");
    store
        .claim_due_with_dispatch(
            &SchedulerDispatchClaimV1::new(claim, record).expect("dispatch binding"),
        )
        .await
        .expect("pending dispatch");
    (store, expected)
}

async fn install_schedule(pool: &PgPool, key: &ConcurrencyKeyV1, policy: &SchedulePolicyV1) {
    sqlx::query("INSERT INTO makosh_platform.scheduler_schedules (schedule_id, schedule_revision, job_owner, job_name, job_major, contract_name, contract_revision, contract_schema_sha256, scope_id, concurrency_key, max_parallelism, enabled, policy_bytes, next_due_at_unix_ms, updated_at_unix_ms) VALUES ($1, 1, 'platform', 'maintenance', 1, 'platform.maintenance', 1, $2, 'scope:technical', $3, 1, TRUE, $4, $5, $5)")
        .bind(vec![11_u8; 16]).bind(vec![7_u8; 32]).bind(key.value()).bind(policy.canonical_bytes()).bind(CLAIMED_AT)
        .execute(pool).await.expect("schedule");
}

fn claim(key: ConcurrencyKeyV1, policy: &SchedulePolicyV1) -> SchedulerRunClaimV1 {
    let lease = ScheduleRunLeaseV1::new(
        JobRunIdV1::new([41; 16]).expect("run"),
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

async fn dispatch_state(pool: &PgPool) -> String {
    sqlx::query("SELECT state FROM makosh_platform.scheduler_dispatches WHERE message_id = $1")
        .bind(vec![51_u8; 16])
        .fetch_one(pool)
        .await
        .expect("dispatch row")
        .get("state")
}

async fn assert_exact_delivery(endpoint: &str, expected: &[u8]) {
    let client = async_nats::ConnectOptions::new()
        .user_and_password("event_hub".into(), "event-hub-test-only".into())
        .connect(endpoint)
        .await
        .expect("consumer connection");
    let context = async_nats::jetstream::new(client);
    let stream = context
        .get_stream("MAKOSH_COMMAND_V1")
        .await
        .expect("command stream");
    let consumer: async_nats::jetstream::consumer::PullConsumer = stream
        .get_consumer("scheduler_delivery")
        .await
        .expect("consumer");
    let mut messages = consumer
        .fetch()
        .max_messages(1)
        .messages()
        .await
        .expect("fetch");
    let message = tokio::time::timeout(Duration::from_secs(2), messages.next())
        .await
        .expect("delivery timeout")
        .expect("delivery missing")
        .expect("delivery error");
    assert_eq!(message.payload.as_ref(), expected);
    message.ack().await.expect("acknowledge delivery");
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("Scheduler test requires {name}"))
}
