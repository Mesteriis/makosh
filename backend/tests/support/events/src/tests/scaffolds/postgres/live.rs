//! Disposable PostgreSQL proof for owner-local durable delivery scaffolds.

use std::time::Duration;

use async_nats::jetstream::consumer::PullConsumer;
use futures_util::StreamExt;
use makosh_events_jetstream::{
    ConsumerBudgetV1, ConsumerSpecV1, DurableSubjectV1, JetStreamClient, NatsPasswordCredentialV1,
    RuntimeNatsIdentity, RuntimeOutboxPublisherV1, RuntimePublishPermitV1,
    RuntimeSubscribePermitV1, StreamKindV1,
};
use makosh_events_protocol::delivery::{
    ExactOutboxPublisherPortV1, InboxDecisionV1, OutboxPublishReceiptV1, OutboxRecordV1,
    OutboxRelayErrorV1, OutboxRelayOutcomeV1, OwnerOutboxStorePortV1, relay_once,
};
use prost::Message;
use sqlx::postgres::PgPoolOptions;

use super::super::OwnerDeliveryScaffoldV1;
use super::store::PostgresOwnerDeliveryStore;
use crate::tests::jetstream_live::event_envelope;

const POSTGRES_URL: &str = "MAKOSH_EVENTS_POSTGRES_URL";
const NATS_ENDPOINT: &str = "MAKOSH_NATS_TEST_ENDPOINT";
const RUNTIME_USER: &str = "MAKOSH_NATS_RUNTIME_USERNAME";
const RUNTIME_PASSWORD: &str = "MAKOSH_NATS_RUNTIME_PASSWORD";
const UNAVAILABLE_NATS_ENDPOINT: &str = "nats://127.0.0.1:43224";

#[tokio::test]
#[ignore = "requires the disposable PostgreSQL owner-delivery contour"]
async fn each_owner_scaffold_keeps_outbox_and_inbox_in_its_own_schema() {
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&required(POSTGRES_URL))
        .await
        .expect("connect disposable PostgreSQL");
    for scaffold in OwnerDeliveryScaffoldV1::ALL {
        verify_owner(&pool, scaffold).await;
    }
}

#[tokio::test]
#[ignore = "requires the disposable PostgreSQL and authenticated JetStream contour"]
async fn postgres_outbox_relays_exact_bytes_to_jetstream_without_domain_behavior() {
    let endpoint = required(NATS_ENDPOINT);
    crate::tests::jetstream_live::configure_event_hub(&endpoint).await;
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&required(POSTGRES_URL))
        .await
        .expect("connect disposable PostgreSQL");
    let expected = relay_from_postgres(&pool, &endpoint).await;
    crate::tests::jetstream_live::assert_exact_runtime_delivery(&endpoint, &expected).await;
}

#[tokio::test]
#[ignore = "requires the disposable PostgreSQL and authenticated JetStream contour"]
async fn owner_inbox_commits_before_jetstream_ack_and_deduplicates_redelivery() {
    let endpoint = required(NATS_ENDPOINT);
    crate::tests::jetstream_live::configure_event_hub(&endpoint).await;
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&required(POSTGRES_URL))
        .await
        .expect("connect disposable PostgreSQL");
    let mut store = PostgresOwnerDeliveryStore::new(pool, OwnerDeliveryScaffoldV1::ALL[0]);
    store
        .install()
        .await
        .expect("install owner delivery scaffold");
    let expected = relay_record(10);
    store
        .enqueue("outbox_inbox_ack_1", &expected)
        .await
        .expect("enqueue exact bytes");

    let (connection, publish_permit) = connect_runtime(&endpoint).await;
    let publisher = RuntimeOutboxPublisherV1::new(&connection, &publish_permit);
    assert!(matches!(
        relay_once(&mut store, &publisher).await,
        Ok(OutboxRelayOutcomeV1::Published { .. })
    ));
    let consumer = connection
        .open_pull_consumer(&consumer_permit())
        .await
        .expect("open exact owner delivery consumer");

    persist_initial_delivery_without_ack(&store, &consumer, &expected).await;

    tokio::time::sleep(Duration::from_secs(3)).await;
    persist_duplicate_redelivery_and_ack(&store, &consumer, &expected).await;
}

async fn persist_initial_delivery_without_ack(
    store: &PostgresOwnerDeliveryStore,
    consumer: &PullConsumer,
    expected: &OutboxRecordV1,
) {
    let mut messages = consumer
        .fetch()
        .max_messages(1)
        .messages()
        .await
        .expect("fetch first owner delivery");
    let delivery = tokio::time::timeout(Duration::from_secs(2), messages.next())
        .await
        .expect("first delivery timeout")
        .expect("first delivery missing")
        .expect("first delivery error");
    let record = OutboxRecordV1::accept(delivery.payload.to_vec()).expect("valid delivery bytes");
    assert_eq!(record.exact_bytes(), expected.exact_bytes());
    assert_eq!(
        store.accept_inbox(&record).await,
        Ok(InboxDecisionV1::Accept)
    );
}

async fn persist_duplicate_redelivery_and_ack(
    store: &PostgresOwnerDeliveryStore,
    consumer: &PullConsumer,
    expected: &OutboxRecordV1,
) {
    let mut messages = consumer
        .fetch()
        .max_messages(1)
        .messages()
        .await
        .expect("fetch redelivery after missing acknowledgement");
    let delivery = tokio::time::timeout(Duration::from_secs(2), messages.next())
        .await
        .expect("redelivery timeout")
        .expect("redelivery missing")
        .expect("redelivery error");
    let record = OutboxRecordV1::accept(delivery.payload.to_vec()).expect("valid retry bytes");
    assert_eq!(record.exact_bytes(), expected.exact_bytes());
    assert_eq!(
        store.accept_inbox(&record).await,
        Ok(InboxDecisionV1::Duplicate)
    );
    delivery
        .ack()
        .await
        .expect("acknowledge only after durable inbox decision");
}

#[tokio::test]
#[ignore = "requires the disposable PostgreSQL and authenticated JetStream contour"]
async fn nats_outage_keeps_owner_outbox_pending_until_reconnect() {
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&required(POSTGRES_URL))
        .await
        .expect("connect disposable PostgreSQL");
    let mut store = PostgresOwnerDeliveryStore::new(pool, OwnerDeliveryScaffoldV1::ALL[0]);
    store
        .install()
        .await
        .expect("install owner delivery scaffold");
    let record = relay_record(9);
    store
        .enqueue("outbox_outage_1", &record)
        .await
        .expect("enqueue exact bytes");

    assert_eq!(
        relay_once(&mut store, &UnavailableNatsPublisher).await,
        Err(OutboxRelayErrorV1::PublisherUnavailable)
    );
    assert!(
        store
            .next_pending()
            .await
            .expect("read pending outbox")
            .is_some()
    );

    let (connection, permit) = connect_runtime(&required(NATS_ENDPOINT)).await;
    let publisher = RuntimeOutboxPublisherV1::new(&connection, &permit);
    assert_eq!(
        relay_once(&mut store, &publisher).await,
        Ok(OutboxRelayOutcomeV1::Published {
            outbox_id: "outbox_outage_1".to_owned(),
            duplicate: false,
        })
    );
}

async fn verify_owner(pool: &sqlx::PgPool, scaffold: OwnerDeliveryScaffoldV1) {
    let mut store = PostgresOwnerDeliveryStore::new(pool.clone(), scaffold);
    store.install().await.expect("install owner scaffold");
    let initial = record("initial");
    store
        .enqueue("outbox_1", &initial)
        .await
        .expect("enqueue exact bytes");
    let entry = store
        .next_pending()
        .await
        .expect("read pending outbox")
        .expect("one pending outbox entry");
    assert_eq!(entry.record().exact_bytes(), initial.exact_bytes());
    store
        .mark_published(
            &entry,
            &OutboxPublishReceiptV1::new("MAKOSH_EVENT_V1", 1, false).expect("receipt"),
        )
        .await
        .expect("mark broker acknowledgement");
    store
        .mark_published(
            &entry,
            &OutboxPublishReceiptV1::new("MAKOSH_EVENT_V1", 1, false).expect("same receipt"),
        )
        .await
        .expect("accept the same broker acknowledgement again");
    assert!(
        store
            .next_pending()
            .await
            .expect("read empty outbox")
            .is_none()
    );
    assert_eq!(
        store.accept_inbox(&initial).await,
        Ok(InboxDecisionV1::Accept)
    );
    assert_eq!(
        store.accept_inbox(&initial).await,
        Ok(InboxDecisionV1::Duplicate)
    );
    assert_eq!(
        store.accept_inbox(&record("conflict")).await,
        Ok(InboxDecisionV1::HashConflict)
    );
}

async fn relay_from_postgres(pool: &sqlx::PgPool, endpoint: &str) -> Vec<u8> {
    let mut store = PostgresOwnerDeliveryStore::new(pool.clone(), OwnerDeliveryScaffoldV1::ALL[0]);
    store
        .install()
        .await
        .expect("install owner delivery scaffold");
    let record = relay_record(8);
    store
        .enqueue("outbox_relay_1", &record)
        .await
        .expect("enqueue exact bytes");
    let (connection, permit) = connect_runtime(endpoint).await;
    let publisher = RuntimeOutboxPublisherV1::new(&connection, &permit);
    assert_eq!(
        relay_once(&mut store, &publisher).await,
        Ok(OutboxRelayOutcomeV1::Published {
            outbox_id: "outbox_relay_1".to_owned(),
            duplicate: false,
        })
    );
    assert_eq!(
        relay_once(&mut store, &publisher).await,
        Ok(OutboxRelayOutcomeV1::Idle)
    );
    record.exact_bytes().to_vec()
}

async fn connect_runtime(
    endpoint: &str,
) -> (
    makosh_events_jetstream::RuntimeJetStreamConnection,
    RuntimePublishPermitV1,
) {
    let connection = JetStreamClient::connect_runtime(
        endpoint,
        RuntimeNatsIdentity::new("notes_runtime", 1, 1).expect("runtime identity"),
        NatsPasswordCredentialV1::new(required(RUNTIME_USER), required(RUNTIME_PASSWORD))
            .expect("runtime credential"),
    )
    .await
    .expect("connect runtime");
    let permit = RuntimePublishPermitV1::new(
        "registration_notes",
        "notes_runtime",
        1,
        1,
        vec![
            DurableSubjectV1::new(StreamKindV1::Event, "notes", "changed", 1)
                .expect("exact publish subject"),
        ],
    )
    .expect("runtime publish permit");
    (connection, permit)
}

fn consumer_permit() -> RuntimeSubscribePermitV1 {
    RuntimeSubscribePermitV1::new(
        "registration_notes",
        "notes_runtime",
        1,
        1,
        ConsumerSpecV1::new(
            StreamKindV1::Event,
            "notes_projection",
            "makosh.event.v1.notes.changed.v1",
            ConsumerBudgetV1::new(16, 3, Duration::from_secs(2)).expect("consumer budget"),
        )
        .expect("consumer spec"),
    )
    .expect("runtime subscribe permit")
}

struct UnavailableNatsPublisher;

impl ExactOutboxPublisherPortV1 for UnavailableNatsPublisher {
    async fn publish_exact(
        &self,
        _: &OutboxRecordV1,
    ) -> Result<OutboxPublishReceiptV1, OutboxRelayErrorV1> {
        let result = JetStreamClient::connect_runtime(
            UNAVAILABLE_NATS_ENDPOINT,
            RuntimeNatsIdentity::new("notes_runtime", 1, 1).expect("runtime identity"),
            NatsPasswordCredentialV1::new(required(RUNTIME_USER), required(RUNTIME_PASSWORD))
                .expect("runtime credential"),
        )
        .await;
        assert!(
            result.is_err(),
            "unavailable endpoint unexpectedly accepted NATS"
        );
        Err(OutboxRelayErrorV1::PublisherUnavailable)
    }
}

fn record(payload: &str) -> OutboxRecordV1 {
    OutboxRecordV1::accept(event_envelope(payload).encode_to_vec()).expect("valid durable envelope")
}

fn relay_record(message_id: u8) -> OutboxRecordV1 {
    let mut envelope = event_envelope("changed");
    envelope.message_id = vec![message_id; 16];
    envelope.correlation_id = vec![message_id; 16];
    OutboxRecordV1::accept(envelope.encode_to_vec()).expect("valid relay durable envelope")
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} is required"))
}
