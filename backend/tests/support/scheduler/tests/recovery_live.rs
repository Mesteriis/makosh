//! Disposable PostgreSQL evidence for exact Scheduler replay preparation.

use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{
        ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
        SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
};
use makosh_scheduler_persistence::{
    SchedulerPostgresStoreV1, SchedulerRecoveryErrorV1, scheduler_storage_bundle_v1,
};
use prost::Message;
use prost_types::Timestamp;
use sqlx::{PgPool, Postgres, Row, pool::PoolConnection, postgres::PgPoolOptions};

const URL: &str = "MAKOSH_SCHEDULER_POSTGRES_URL";
const TEST_SCHEMA_LOCK: i64 = 0x4845_524d_4553_5302;

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires the disposable Scheduler PostgreSQL contour"]
async fn recovery_requeues_only_unaccepted_exact_dispatches_and_preserves_inbox() {
    let (pool, _schema_guard) = connect_and_install().await;
    let accepted = record([31; 16]);
    let replay = record([32; 16]);
    insert_run_dispatch(&pool, &accepted, "succeeded", 1).await;
    insert_run_dispatch(&pool, &replay, "dispatched", 2).await;
    insert_inbox(&pool, &accepted).await;

    let report = SchedulerPostgresStoreV1::new(pool.clone())
        .prepare_event_hub_replay()
        .await
        .expect("prepare replay");
    assert_eq!(report.requeued_dispatches(), 1);
    assert_eq!(report.preserved_acceptances(), 1);
    assert_eq!(report.preserved_results(), 1);
    assert_dispatch(
        &pool,
        &accepted,
        "published",
        Some("MAKOSH_COMMANDS"),
        &accepted,
    )
    .await;
    assert_dispatch(&pool, &replay, "pending", None, &replay).await;
    assert_eq!(
        run_state(&pool, replay.message_id()).await,
        "pending_dispatch"
    );

    let repeated = SchedulerPostgresStoreV1::new(pool.clone())
        .prepare_event_hub_replay()
        .await
        .expect("idempotent replay preparation");
    assert_eq!(repeated.requeued_dispatches(), 0);
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires the disposable Scheduler PostgreSQL contour"]
async fn recovery_rejects_corrupt_exact_dispatch_state_without_mutation() {
    let (pool, _schema_guard) = connect_and_install().await;
    let replay = record([41; 16]);
    insert_run_dispatch(&pool, &replay, "dispatched", 1).await;
    sqlx::query(
        "UPDATE makosh_platform.scheduler_dispatches SET envelope_sha256 = $2 WHERE message_id = $1",
    )
    .bind(replay.message_id().to_vec())
    .bind(vec![0_u8; 32])
    .execute(&pool)
    .await
    .expect("corrupt replay digest");

    assert_eq!(
        SchedulerPostgresStoreV1::new(pool.clone())
            .prepare_event_hub_replay()
            .await,
        Err(SchedulerRecoveryErrorV1::InvalidDurableState)
    );
    assert_eq!(
        dispatch_state(&pool, replay.message_id()).await,
        "published"
    );
    assert_eq!(run_state(&pool, replay.message_id()).await, "dispatched");
}

async fn connect_and_install() -> (PgPool, PoolConnection<Postgres>) {
    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(&required(URL))
        .await
        .expect("connect disposable Scheduler PostgreSQL");
    let mut schema_guard = pool.acquire().await.expect("acquire schema guard");
    sqlx::query("SELECT pg_advisory_lock($1)")
        .bind(TEST_SCHEMA_LOCK)
        .execute(&mut *schema_guard)
        .await
        .expect("lock Scheduler recovery schema");
    sqlx::raw_sql("DROP SCHEMA IF EXISTS makosh_platform CASCADE; CREATE SCHEMA makosh_platform;")
        .execute(&mut *schema_guard)
        .await
        .expect("fresh Scheduler schema");
    for step in scheduler_storage_bundle_v1().steps {
        sqlx::raw_sql(sqlx::AssertSqlSafe(
            std::str::from_utf8(&step.forward_sql_utf8)
                .expect("migration UTF-8")
                .to_owned(),
        ))
        .execute(&mut *schema_guard)
        .await
        .expect("Scheduler migration");
    }
    (pool, schema_guard)
}

async fn insert_run_dispatch(
    pool: &PgPool,
    record: &OutboxRecordV1,
    run_state: &str,
    created_at: i64,
) {
    let id = record.message_id().to_vec();
    sqlx::query("INSERT INTO makosh_platform.scheduler_runs (run_id, schedule_id, schedule_revision, scheduled_for_unix_ms, lease_epoch, lease_expires_at_unix_ms, state, attempt_count, dispatch_message_id, fire_key, concurrency_key, created_at_unix_ms) VALUES ($1, $2, 1, 1, 1, 60000, $3, 0, $1, $4, $5, $6)")
        .bind(&id)
        .bind(vec![created_at as u8; 16])
        .bind(run_state)
        .bind(vec![created_at as u8; 32])
        .bind(format!("scope:{created_at}"))
        .bind(created_at)
        .execute(pool)
        .await
        .expect("insert recovery run");
    sqlx::query("INSERT INTO makosh_platform.scheduler_dispatches (run_id, lease_epoch, message_id, envelope_sha256, exact_envelope_bytes, state, published_stream, published_sequence, created_at_unix_ms) VALUES ($1, 1, $1, $2, $3, 'published', 'MAKOSH_COMMANDS', $4, $4)")
        .bind(id)
        .bind(record.envelope_sha256().to_vec())
        .bind(record.exact_bytes())
        .bind(created_at)
        .execute(pool)
        .await
        .expect("insert recovery dispatch");
}

async fn insert_inbox(pool: &PgPool, record: &OutboxRecordV1) {
    let id = record.message_id().to_vec();
    sqlx::query("INSERT INTO makosh_platform.scheduler_run_acceptances (command_message_id, run_id, lease_epoch, observed_at_unix_ms) VALUES ($1, $1, 1, 2)")
        .bind(&id)
        .execute(pool)
        .await
        .expect("insert acceptance inbox");
    sqlx::query("INSERT INTO makosh_platform.scheduler_run_results (command_message_id, run_id, lease_epoch, outcome, observed_at_unix_ms) VALUES ($1, $1, 1, 'succeeded', 3)")
        .bind(id)
        .execute(pool)
        .await
        .expect("insert result inbox");
}

async fn assert_dispatch(
    pool: &PgPool,
    key: &OutboxRecordV1,
    state: &str,
    stream: Option<&str>,
    exact: &OutboxRecordV1,
) {
    let row = sqlx::query("SELECT state, published_stream, published_sequence, envelope_sha256, exact_envelope_bytes FROM makosh_platform.scheduler_dispatches WHERE message_id = $1")
        .bind(key.message_id().to_vec())
        .fetch_one(pool)
        .await
        .expect("read dispatch");
    assert_eq!(row.get::<String, _>("state"), state);
    assert_eq!(
        row.get::<Option<String>, _>("published_stream").as_deref(),
        stream
    );
    assert_eq!(
        row.get::<Option<i64>, _>("published_sequence"),
        stream.map(|_| 1)
    );
    assert_eq!(
        row.get::<Vec<u8>, _>("envelope_sha256"),
        exact.envelope_sha256()
    );
    assert_eq!(
        row.get::<Vec<u8>, _>("exact_envelope_bytes"),
        exact.exact_bytes()
    );
}

async fn dispatch_state(pool: &PgPool, message_id: &[u8; 16]) -> String {
    sqlx::query_scalar(
        "SELECT state FROM makosh_platform.scheduler_dispatches WHERE message_id = $1",
    )
    .bind(message_id.to_vec())
    .fetch_one(pool)
    .await
    .expect("read dispatch state")
}

async fn run_state(pool: &PgPool, run_id: &[u8; 16]) -> String {
    sqlx::query_scalar("SELECT state FROM makosh_platform.scheduler_runs WHERE run_id = $1")
        .bind(run_id.to_vec())
        .fetch_one(pool)
        .await
        .expect("read run state")
}

fn record(message_id: [u8; 16]) -> OutboxRecordV1 {
    OutboxRecordV1::accept(envelope(message_id).encode_to_vec()).expect("valid exact envelope")
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
        correlation_id: message_id.to_vec(),
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
            command_id: message_id.to_vec(),
            target_capability: "job_execute".into(),
            idempotency_key: message_id.to_vec(),
            deadline: Some(Timestamp {
                seconds: 60,
                nanos: 0,
            }),
            logical_attempt: 1,
        })),
        payload: vec![1],
    }
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("Scheduler test requires {name}"))
}
