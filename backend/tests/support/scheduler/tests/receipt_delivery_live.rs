//! Disposable PostgreSQL and JetStream proof for Scheduler receipt persistence.

#[allow(dead_code)]
#[path = "support/receipt_fixture.rs"]
mod fixture;

use std::time::Duration;

use makosh_events_jetstream::{
    ConsumerBudgetV1, ConsumerSpecV1, DurableSubjectV1, EventHubTopologyPlanV1, JetStreamClient,
    NatsPasswordCredentialV1, RuntimeNatsIdentity, RuntimePublishPermitV1,
    RuntimeSchedulerReceiptPortV1, RuntimeSubscribePermitV1, StreamBudgetV1, StreamKindV1,
    StreamSpecV1,
};
use makosh_events_protocol::v1::{
    AckDispositionV1, AckMetadataV1, AckStageV1, ActorKindV1, ActorRefV1, ContractRefV1,
    DurableEnvelopeV1, FenceKindV1, ResultMetadataV1, ResultOutcomeV1, SourceFenceV1, SourceRefV1,
    durable_envelope_v1::Semantics,
};
use makosh_scheduler_persistence::{
    SchedulerReceiptConsumeOutcomeV1, SchedulerReceiptConsumerV1, SchedulerRunClaimV1,
};
use makosh_scheduler_protocol::v1::{JobRunOutcomeV1, JobRunReceiptV1};
use prost::Message;
use prost_types::Timestamp;

use fixture::{active_runs, pending_published_dispatch, receipt, required, run_state};

const NATS_ENDPOINT: &str = "MAKOSH_SCHEDULER_NATS_ENDPOINT";

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires the disposable Scheduler PostgreSQL and JetStream contour"]
async fn receipt_delivery_commits_before_jetstream_ack() {
    let endpoint = required(NATS_ENDPOINT);
    configure_receipt_topology(&endpoint).await;
    let (pool, store, claim) = pending_published_dispatch().await;
    let (owner, permit) = connect_owner_runtime(&endpoint).await;
    let (scheduler, acceptance_permit, terminal_permit) =
        connect_scheduler_runtime(&endpoint).await;
    let mut acceptance = SchedulerReceiptConsumerV1::new(
        RuntimeSchedulerReceiptPortV1::new(&scheduler, acceptance_permit),
        &store,
    );
    let mut terminal = SchedulerReceiptConsumerV1::new(
        RuntimeSchedulerReceiptPortV1::new(&scheduler, terminal_permit),
        &store,
    );

    commit_acceptance(&mut acceptance, &pool, &claim, &owner, &permit).await;
    commit_success(&mut terminal, &pool, &claim, &owner, &permit).await;
}

async fn commit_acceptance<P>(
    consumer: &mut SchedulerReceiptConsumerV1<'_, P>,
    pool: &sqlx::PgPool,
    claim: &SchedulerRunClaimV1,
    owner: &makosh_events_jetstream::RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
) where
    P: makosh_scheduler_protocol::SchedulerReceiptDeliveryPortV1,
{
    let accepted = receipt(claim);
    publish_owner_receipt(owner, permit, acceptance_envelope(&accepted)).await;
    assert_eq!(
        consumer.consume_one().await,
        Ok(SchedulerReceiptConsumeOutcomeV1::AcceptanceApplied)
    );
    assert_eq!(run_state(pool).await, "running");
}

async fn commit_success<P>(
    consumer: &mut SchedulerReceiptConsumerV1<'_, P>,
    pool: &sqlx::PgPool,
    claim: &SchedulerRunClaimV1,
    owner: &makosh_events_jetstream::RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
) where
    P: makosh_scheduler_protocol::SchedulerReceiptDeliveryPortV1,
{
    let mut terminal = receipt(claim);
    terminal.outcome = JobRunOutcomeV1::Succeeded as i32;
    publish_owner_receipt(owner, permit, terminal_envelope(&terminal)).await;
    assert_eq!(
        consumer.consume_one().await,
        Ok(SchedulerReceiptConsumeOutcomeV1::TerminalApplied)
    );
    assert_eq!(run_state(pool).await, "finished");
    assert_eq!(active_runs(pool).await, 0);
}

async fn connect_scheduler_runtime(
    endpoint: &str,
) -> (
    makosh_events_jetstream::RuntimeJetStreamConnection,
    RuntimeSubscribePermitV1,
    RuntimeSubscribePermitV1,
) {
    let runtime = JetStreamClient::connect_runtime(
        endpoint,
        RuntimeNatsIdentity::new("scheduler_runtime", 1, 1).expect("scheduler identity"),
        NatsPasswordCredentialV1::new("scheduler_runtime", "scheduler-runtime-test-only")
            .expect("scheduler credential"),
    )
    .await
    .expect("scheduler runtime connection");
    let budget = ConsumerBudgetV1::new(16, 3, Duration::from_secs(2)).expect("budget");
    let acceptance = RuntimeSubscribePermitV1::new(
        "scheduler_registration",
        "scheduler_runtime",
        1,
        1,
        receipt_consumer(StreamKindV1::Ack, "scheduler_receipt_acceptance", budget),
    )
    .expect("acceptance permit");
    let terminal = RuntimeSubscribePermitV1::new(
        "scheduler_registration",
        "scheduler_runtime",
        1,
        1,
        receipt_consumer(StreamKindV1::Result, "scheduler_receipt_terminal", budget),
    )
    .expect("terminal permit");
    (runtime, acceptance, terminal)
}

async fn connect_owner_runtime(
    endpoint: &str,
) -> (
    makosh_events_jetstream::RuntimeJetStreamConnection,
    RuntimePublishPermitV1,
) {
    let owner = JetStreamClient::connect_runtime(
        endpoint,
        RuntimeNatsIdentity::new("mail_runtime", 1, 1).expect("owner identity"),
        NatsPasswordCredentialV1::new("mail_runtime", "mail-runtime-test-only")
            .expect("owner credential"),
    )
    .await
    .expect("owner runtime connection");
    let permit = RuntimePublishPermitV1::new(
        "mail_registration",
        "mail_runtime",
        1,
        1,
        vec![
            DurableSubjectV1::new(StreamKindV1::Ack, "mail", "job_receipt", 1)
                .expect("acceptance subject"),
            DurableSubjectV1::new(StreamKindV1::Result, "mail", "job_receipt", 1)
                .expect("terminal subject"),
        ],
    )
    .expect("owner permit");
    (owner, permit)
}

async fn publish_owner_receipt(
    owner: &makosh_events_jetstream::RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    envelope: DurableEnvelopeV1,
) {
    owner
        .publish_exact(permit, &envelope.encode_to_vec())
        .await
        .expect("publish durable receipt");
}

async fn configure_receipt_topology(endpoint: &str) {
    let hub = JetStreamClient::connect_event_hub(
        endpoint,
        NatsPasswordCredentialV1::new("event_hub", "event-hub-test-only").expect("credential"),
    )
    .await
    .expect("event hub connection");
    let budget = StreamBudgetV1::new(1_048_576, Duration::from_secs(3600), 1).expect("budget");
    let consumer_budget = ConsumerBudgetV1::new(16, 3, Duration::from_secs(2)).expect("budget");
    let topology = EventHubTopologyPlanV1::new(
        vec![
            StreamSpecV1::new(StreamKindV1::Ack, budget),
            StreamSpecV1::new(StreamKindV1::Result, budget),
        ],
        vec![
            receipt_consumer(
                StreamKindV1::Ack,
                "scheduler_receipt_acceptance",
                consumer_budget,
            ),
            receipt_consumer(
                StreamKindV1::Result,
                "scheduler_receipt_terminal",
                consumer_budget,
            ),
        ],
    )
    .expect("receipt topology");
    hub.reconcile(&topology)
        .await
        .expect("topology reconciliation");
}

fn receipt_consumer(kind: StreamKindV1, name: &str, budget: ConsumerBudgetV1) -> ConsumerSpecV1 {
    let subject = format!("makosh.{}.v1.mail.job_receipt.v1", kind.subject_token());
    ConsumerSpecV1::new(kind, name, subject, budget).expect("receipt consumer")
}

fn acceptance_envelope(receipt: &JobRunReceiptV1) -> DurableEnvelopeV1 {
    receipt_envelope(
        receipt,
        vec![61; 16],
        Semantics::Ack(AckMetadataV1 {
            acknowledged_message_id: receipt.command_message_id.clone(),
            stage: AckStageV1::DurableAcceptance as i32,
            disposition: AckDispositionV1::Applied as i32,
            acknowledged_at: Some(timestamp(receipt.observed_at_unix_millis)),
        }),
    )
}

fn terminal_envelope(receipt: &JobRunReceiptV1) -> DurableEnvelopeV1 {
    receipt_envelope(
        receipt,
        vec![62; 16],
        Semantics::Result(ResultMetadataV1 {
            command_id: receipt.job_run_id.clone(),
            command_message_id: receipt.command_message_id.clone(),
            outcome: ResultOutcomeV1::Succeeded as i32,
            completed_at: Some(timestamp(receipt.observed_at_unix_millis)),
            execution_attempt: 1,
        }),
    )
}

fn receipt_envelope(
    receipt: &JobRunReceiptV1,
    message_id: Vec<u8>,
    semantics: Semantics,
) -> DurableEnvelopeV1 {
    DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id,
        contract: Some(ContractRefV1 {
            owner: "mail".into(),
            name: "job_receipt".into(),
            major: 1,
            revision: 1,
            schema_sha256: vec![7; 32],
        }),
        source: Some(SourceRefV1 {
            module_id: "mail_runtime".into(),
            runtime_instance_id: vec![8; 16],
            runtime_generation: 1,
        }),
        recorded_at: Some(timestamp(receipt.observed_at_unix_millis)),
        partition_key: b"scheduler".to_vec(),
        causation_message_id: receipt.command_message_id.clone(),
        correlation_id: receipt.job_run_id.clone(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: b"mail_runtime".to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: b"mail_runtime".to_vec(),
            epoch: 1,
        }),
        semantics: Some(semantics),
        payload: receipt.encode_to_vec(),
    }
}

fn timestamp(value: i64) -> Timestamp {
    Timestamp {
        seconds: value.div_euclid(1_000),
        nanos: i32::try_from(value.rem_euclid(1_000) * 1_000_000).expect("millisecond nanos"),
    }
}
