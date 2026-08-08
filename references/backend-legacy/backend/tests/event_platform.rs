use chrono::Utc;
use futures::StreamExt;
use makosh_backend_testkit::context::TestContext;
use serde_json::json;
use sqlx::Row;
use tokio::time::{Duration, timeout};

use makosh_events_api::NewEventEnvelope;
use makosh_events_nats::jetstream::NatsJetStreamEventBus;
use makosh_events_postgres::store::EventStore;
use makosh_hub_backend::platform::events::bus::InMemoryEventBus;
use makosh_hub_backend::platform::events::dispatcher::EventOutboxDispatcher;

#[tokio::test]
async fn append_for_dispatch_records_event_and_pending_outbox_subject() {
    let ctx = TestContext::new().await;
    let store = EventStore::new(ctx.pool().clone());
    let occurred_at = Utc::now();
    let event = NewEventEnvelope::builder(
        format!("evt_outbox_{}", occurred_at.timestamp_nanos_opt().unwrap()),
        "signal.accepted.telegram.message",
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "accepted-message-1"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "accepted-message-1"
        }),
    )
    .build()
    .expect("valid event");

    let position = store
        .append_for_dispatch(&event)
        .await
        .expect("append event for dispatch");

    assert!(position > 0);

    let outbox = store
        .pending_outbox_batch(10)
        .await
        .expect("load pending outbox");

    assert_eq!(outbox.len(), 1);
    assert_eq!(outbox[0].event_id, event.event_id);
    assert_eq!(outbox[0].subject, "signal.accepted.telegram.message");
    assert_eq!(outbox[0].status, "pending");
    assert_eq!(outbox[0].attempts, 0);
}

#[tokio::test]
async fn in_memory_event_bus_delivers_events_to_subscribers() {
    let bus = InMemoryEventBus::new();
    let mut subscriber = bus.subscribe();
    let occurred_at = Utc::now();
    let event = NewEventEnvelope::builder(
        format!(
            "evt_memory_bus_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        "signal.raw.fixture.message.observed",
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "fixture",
            "source_id": "fixture-message-1"
        }),
        json!({
            "kind": "signal",
            "source_code": "fixture",
            "entity_id": "fixture-message-1"
        }),
    )
    .build()
    .expect("valid event");

    assert_eq!(bus.broadcast(event.clone()), 1);

    let received = subscriber.recv().await.expect("receive event");
    assert_eq!(received.event_id, event.event_id);
    assert_eq!(received.event_type, event.event_type);
}

#[tokio::test]
async fn event_outbox_dispatcher_publishes_pending_events_to_nats() {
    let ctx = TestContext::new().await;
    let store = EventStore::new(ctx.pool().clone());
    let nats_server_url = ctx.nats_server_url().await;
    let bus = NatsJetStreamEventBus::connect(&nats_server_url)
        .await
        .expect("connect JetStream bus");
    let dispatcher = EventOutboxDispatcher::new(store.clone(), bus);
    let client = async_nats::connect(&nats_server_url)
        .await
        .expect("connect NATS client");
    let event_subject = format!(
        "signal.accepted.telegram.message.test.{}",
        Utc::now().timestamp_nanos_opt().unwrap()
    );
    let mut subscriber = client
        .subscribe(event_subject.clone())
        .await
        .expect("subscribe to accepted telegram signal");

    let occurred_at = Utc::now();
    let event = NewEventEnvelope::builder(
        format!(
            "evt_dispatch_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        event_subject.clone(),
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "accepted-message-2"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "accepted-message-2"
        }),
    )
    .payload(json!({
        "message_id": "accepted-message-2"
    }))
    .build()
    .expect("valid event");

    store
        .append_for_dispatch(&event)
        .await
        .expect("append event for dispatch");

    let report = dispatcher
        .dispatch_pending_once()
        .await
        .expect("dispatch outbox");

    assert_eq!(report.recovered, 0);
    assert_eq!(report.claimed, 1);
    assert_eq!(report.published, 1);
    assert_eq!(report.retried, 0);

    let message = timeout(Duration::from_secs(5), subscriber.next())
        .await
        .expect("message receive timeout")
        .expect("subscription yields message");
    let published_event: makosh_events_api::EventEnvelope =
        serde_json::from_slice(&message.payload).expect("decode published event");
    assert_eq!(published_event.event_id, event.event_id);
    assert_eq!(published_event.event_type, event.event_type);

    let row = sqlx::query(
        r#"
        SELECT status, attempts, published_at IS NOT NULL AS published
        FROM event_outbox
        WHERE event_id = $1
        "#,
    )
    .bind(&event.event_id)
    .fetch_one(ctx.pool())
    .await
    .expect("load event outbox row");

    let status: String = row.try_get("status").expect("status");
    let attempts: i32 = row.try_get("attempts").expect("attempts");
    let published: bool = row.try_get("published").expect("published flag");

    assert_eq!(status, "published");
    assert_eq!(attempts, 1);
    assert!(published);
}

#[tokio::test]
async fn event_outbox_dispatcher_broadcasts_published_events_to_realtime_bus() {
    let ctx = TestContext::new().await;
    let store = EventStore::new(ctx.pool().clone());
    let nats_server_url = ctx.nats_server_url().await;
    let jetstream_bus = NatsJetStreamEventBus::connect(&nats_server_url)
        .await
        .expect("connect JetStream bus");
    let realtime_bus = InMemoryEventBus::new();
    let mut subscriber = realtime_bus.subscribe();
    let dispatcher = EventOutboxDispatcher::new(store.clone(), jetstream_bus)
        .with_realtime_bus(realtime_bus.clone());

    let event_subject = format!(
        "signal.accepted.telegram.message.test.{}",
        Utc::now().timestamp_nanos_opt().unwrap()
    );
    let occurred_at = Utc::now();
    let event = NewEventEnvelope::builder(
        format!(
            "evt_realtime_dispatch_{}",
            occurred_at.timestamp_nanos_opt().unwrap()
        ),
        event_subject,
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "telegram",
            "source_id": "accepted-message-realtime"
        }),
        json!({
            "kind": "signal",
            "source_code": "telegram",
            "entity_id": "accepted-message-realtime"
        }),
    )
    .payload(json!({
        "message_id": "accepted-message-realtime"
    }))
    .build()
    .expect("valid event");

    store
        .append_for_dispatch(&event)
        .await
        .expect("append event for dispatch");

    let report = dispatcher
        .dispatch_pending_once()
        .await
        .expect("dispatch outbox");

    assert_eq!(report.claimed, 1);
    assert_eq!(report.published, 1);

    let received = timeout(Duration::from_secs(5), subscriber.recv())
        .await
        .expect("receive timeout")
        .expect("receive realtime event");
    assert_eq!(received.event_id, event.event_id);
    assert_eq!(received.event_type, event.event_type);
    assert_eq!(
        received.payload["message_id"],
        json!("accepted-message-realtime")
    );
}

#[tokio::test]
async fn event_outbox_dispatcher_recovers_stale_dispatching_items() {
    let ctx = TestContext::new().await;
    let store = EventStore::new(ctx.pool().clone());
    let nats_server_url = ctx.nats_server_url().await;
    let bus = NatsJetStreamEventBus::connect(&nats_server_url)
        .await
        .expect("connect JetStream bus");
    let dispatcher = EventOutboxDispatcher::new(store.clone(), bus);
    let client = async_nats::connect(&nats_server_url)
        .await
        .expect("connect NATS client");
    let mut subscriber = client
        .subscribe("signal.accepted.mail.message")
        .await
        .expect("subscribe to accepted mail signal");

    let occurred_at = Utc::now();
    let event = NewEventEnvelope::builder(
        format!("evt_recover_{}", occurred_at.timestamp_nanos_opt().unwrap()),
        "signal.accepted.mail.message",
        occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": "mail",
            "source_id": "accepted-mail-1"
        }),
        json!({
            "kind": "signal",
            "source_code": "mail",
            "entity_id": "accepted-mail-1"
        }),
    )
    .build()
    .expect("valid event");

    store
        .append_for_dispatch(&event)
        .await
        .expect("append event for dispatch");
    let claimed = store
        .claim_pending_outbox_batch(10)
        .await
        .expect("claim event outbox item");
    assert_eq!(claimed.len(), 1);

    sqlx::query(
        r#"
        UPDATE event_outbox
        SET updated_at = now() - interval '5 minutes'
        WHERE event_id = $1
        "#,
    )
    .bind(&event.event_id)
    .execute(ctx.pool())
    .await
    .expect("mark event outbox item stale");

    let report = dispatcher
        .dispatch_pending_once()
        .await
        .expect("dispatch recovered outbox");

    assert_eq!(report.recovered, 1);
    assert_eq!(report.claimed, 1);
    assert_eq!(report.published, 1);
    assert_eq!(report.retried, 0);

    let message = timeout(Duration::from_secs(5), subscriber.next())
        .await
        .expect("message receive timeout")
        .expect("subscription yields message");
    let published_event: makosh_events_api::EventEnvelope =
        serde_json::from_slice(&message.payload).expect("decode published event");
    assert_eq!(published_event.event_id, event.event_id);
}
