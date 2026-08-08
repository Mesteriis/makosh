use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use serde_json::json;

use makosh_events_api::{EventEnvelope, NewEventEnvelope, StoredEventEnvelope};
use makosh_events_postgres::consumers::EventConsumerStore;
use makosh_events_postgres::cursors::ProjectionCursorStore;
use makosh_events_postgres::store::EventStore;
use makosh_hub_backend::platform::events::trace_context::TraceContext;
use makosh_hub_backend::platform::storage::database::Database;

#[test]
fn new_event_envelope_rejects_empty_event_type() {
    let error = NewEventEnvelope::builder(
        "evt_test_empty_type",
        " ",
        Utc::now(),
        json!({"kind": "test", "source_id": "source-empty-type"}),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .build()
    .expect_err("empty event type must fail");

    assert_eq!(error.to_string(), "event_type must not be empty");
}

#[test]
fn new_event_envelope_rejects_non_object_source() {
    let error = NewEventEnvelope::builder(
        "evt_test_bad_source",
        "system_test_event",
        Utc::now(),
        json!("not-an-object"),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .build()
    .expect_err("non-object source must fail");

    assert_eq!(error.to_string(), "source must be a JSON object");
}

#[test]
fn new_event_envelope_normalizes_missing_correlation_id_to_event_id() {
    let event = NewEventEnvelope::builder(
        " evt_test_missing_correlation ",
        "system_test_event",
        Utc::now(),
        json!({"kind": "test", "source_id": "source-missing-correlation"}),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .correlation_id(" ")
    .build()
    .expect("valid event");

    assert_eq!(event.event_id, "evt_test_missing_correlation");
    assert_eq!(
        event.correlation_id.as_deref(),
        Some("evt_test_missing_correlation")
    );
}

#[test]
fn trace_context_builds_root_and_child_contexts() {
    let root = TraceContext::root("trace-root");
    assert_eq!(root.correlation_id, "trace-root");
    assert_eq!(root.causation_id, None);

    let parent = EventEnvelope {
        event_id: "evt_parent".to_owned(),
        event_type: "system_test_event".to_owned(),
        schema_version: 1,
        occurred_at: Utc::now(),
        recorded_at: Utc::now(),
        source: json!({"kind": "test"}),
        actor: None,
        subject: json!({"kind": "system", "entity_id": "backend"}),
        payload: json!({}),
        provenance: json!({}),
        causation_id: None,
        correlation_id: Some("trace-parent".to_owned()),
    };

    let child = TraceContext::child_of(&parent);
    assert_eq!(child.correlation_id, "trace-parent");
    assert_eq!(child.causation_id.as_deref(), Some("evt_parent"));
}

#[tokio::test]
async fn event_store_appends_and_loads_event_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = EventStore::new(database.pool().expect("configured pool").clone());

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let event_id = format!("evt_test_{suffix}");
    let occurred_at: DateTime<Utc> = Utc::now();

    let event = NewEventEnvelope::builder(
        &event_id,
        "system_test_event",
        occurred_at,
        json!({
            "kind": "test",
            "provider": "integration",
            "source_id": event_id,
            "import_batch_id": "event-log-test"
        }),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .payload(json!({"test": true}))
    .provenance(json!({"confidence": 1.0}))
    .correlation_id("corr_event_log_test")
    .build()
    .expect("valid event");

    store.append(&event).await.expect("append event");

    let loaded = store
        .get_by_id(&event_id)
        .await
        .expect("load event")
        .expect("event exists");

    assert_eq!(
        loaded,
        EventEnvelope {
            event_id: event_id.clone(),
            event_type: "system_test_event".to_owned(),
            schema_version: 1,
            occurred_at,
            recorded_at: loaded.recorded_at,
            source: json!({
                "kind": "test",
                "provider": "integration",
                "source_id": loaded.event_id,
                "import_batch_id": "event-log-test"
            }),
            actor: None,
            subject: json!({"kind": "system", "entity_id": "backend"}),
            payload: json!({"test": true}),
            provenance: json!({"confidence": 1.0}),
            causation_id: None,
            correlation_id: Some("corr_event_log_test".to_owned()),
        }
    );

    let duplicate_source_event = NewEventEnvelope::builder(
        format!("{event_id}_duplicate"),
        "system_test_event",
        occurred_at,
        json!({
            "kind": "test",
            "provider": "integration",
            "source_id": loaded.event_id,
            "import_batch_id": "event-log-test"
        }),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .build()
    .expect("valid duplicate source event");

    assert!(
        store.append(&duplicate_source_event).await.is_err(),
        "same event_type and source identity must be idempotent"
    );

    let mutation = sqlx::query("UPDATE event_log SET payload = '{}'::jsonb WHERE event_id = $1")
        .bind(&loaded.event_id)
        .execute(database.pool().expect("configured pool"))
        .await;

    assert!(mutation.is_err(), "event_log must be append-only");
}

#[tokio::test]
async fn event_store_replays_events_after_position_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = EventStore::new(database.pool().expect("configured pool").clone());

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let occurred_at = Utc::now();
    let first_id = format!("evt_replay_first_{suffix}");
    let second_id = format!("evt_replay_second_{suffix}");

    let first = NewEventEnvelope::builder(
        &first_id,
        "system_replay_test_event",
        occurred_at,
        json!({"kind": "test", "provider": "integration", "source_id": first_id}),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .build()
    .expect("valid first event");
    let first_position = store.append(&first).await.expect("append first event");

    let second = NewEventEnvelope::builder(
        &second_id,
        "system_replay_test_event",
        occurred_at,
        json!({"kind": "test", "provider": "integration", "source_id": second_id}),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .build()
    .expect("valid second event");
    let second_position = store.append(&second).await.expect("append second event");

    let replayed = store
        .list_after_position(first_position, 10)
        .await
        .expect("replay events");

    assert_eq!(
        replayed,
        vec![StoredEventEnvelope {
            position: second_position,
            event: EventEnvelope {
                event_id: second_id,
                event_type: "system_replay_test_event".to_owned(),
                schema_version: 1,
                occurred_at,
                recorded_at: replayed[0].event.recorded_at,
                source: json!({
                    "kind": "test",
                    "provider": "integration",
                    "source_id": replayed[0].event.event_id
                }),
                actor: None,
                subject: json!({"kind": "system", "entity_id": "backend"}),
                payload: json!({}),
                provenance: json!({}),
                causation_id: None,
                correlation_id: Some(replayed[0].event.event_id.clone()),
            },
        }]
    );
}

#[tokio::test]
async fn event_store_reconstructs_trace_edges_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = EventStore::new(database.pool().expect("configured pool").clone());

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let occurred_at = Utc::now();
    let trace_id = format!("trace_event_log_{suffix}");
    let root_id = format!("evt_trace_root_{suffix}");
    let child_id = format!("evt_trace_child_{suffix}");
    let grandchild_id = format!("evt_trace_grandchild_{suffix}");

    let root = NewEventEnvelope::builder(
        &root_id,
        "system_trace_test_event",
        occurred_at,
        json!({"kind": "test", "provider": "integration", "source_id": root_id}),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .correlation_id(&trace_id)
    .build()
    .expect("valid root event");
    store.append(&root).await.expect("append root event");

    let child = NewEventEnvelope::builder(
        &child_id,
        "system_trace_test_event",
        occurred_at,
        json!({"kind": "test", "provider": "integration", "source_id": child_id}),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .causation_id(&root_id)
    .correlation_id(&trace_id)
    .build()
    .expect("valid child event");
    store.append(&child).await.expect("append child event");

    let grandchild = NewEventEnvelope::builder(
        &grandchild_id,
        "system_trace_test_event",
        occurred_at,
        json!({"kind": "test", "provider": "integration", "source_id": grandchild_id}),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .causation_id(&child_id)
    .correlation_id(&trace_id)
    .build()
    .expect("valid grandchild event");
    store
        .append(&grandchild)
        .await
        .expect("append grandchild event");

    let trace = store
        .trace_by_event_id(&grandchild_id, 100)
        .await
        .expect("trace query")
        .expect("trace exists");

    assert_eq!(trace.correlation_id, trace_id);
    assert_eq!(trace.root_event_ids, vec![root_id.clone()]);
    assert_eq!(trace.events.len(), 3);
    assert_eq!(trace.missing_parent_ids, Vec::<String>::new());
    assert_eq!(trace.orphan_event_ids, Vec::<String>::new());
    assert_eq!(
        trace.edges,
        vec![
            makosh_events_postgres::trace::EventTraceEdge {
                parent_event_id: root_id.clone(),
                child_event_id: child_id.clone(),
            },
            makosh_events_postgres::trace::EventTraceEdge {
                parent_event_id: child_id.clone(),
                child_event_id: grandchild_id,
            },
        ]
    );

    let children = store
        .list_children(&root_id, 10)
        .await
        .expect("children query");
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].event.event_id, child_id);
}

#[tokio::test]
async fn event_store_reports_missing_trace_parent_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = EventStore::new(database.pool().expect("configured pool").clone());

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let trace_id = format!("trace_missing_parent_{suffix}");
    let event_id = format!("evt_missing_parent_{suffix}");
    let missing_parent_id = format!("evt_missing_parent_root_{suffix}");

    let event = NewEventEnvelope::builder(
        &event_id,
        "system_trace_test_event",
        Utc::now(),
        json!({"kind": "test", "provider": "integration", "source_id": event_id}),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .causation_id(&missing_parent_id)
    .correlation_id(&trace_id)
    .build()
    .expect("valid event");
    store.append(&event).await.expect("append event");

    let trace = store
        .trace_by_correlation_id(&trace_id, 100)
        .await
        .expect("trace query");

    assert_eq!(trace.root_event_ids, Vec::<String>::new());
    assert_eq!(trace.orphan_event_ids, vec![event_id]);
    assert_eq!(trace.missing_parent_ids, vec![missing_parent_id]);
}

#[tokio::test]
async fn event_store_trace_includes_consumer_and_dlq_annotations_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = EventStore::new(pool.clone());
    let consumers = EventConsumerStore::new(pool);

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let event_id = format!("evt_trace_annotations_{suffix}");

    let event = NewEventEnvelope::builder(
        &event_id,
        "system_trace_annotation_test_event",
        Utc::now(),
        json!({"kind": "test", "provider": "integration", "source_id": event_id}),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .build()
    .expect("valid event");
    store.append(&event).await.expect("append event");
    let stored = store
        .list_by_correlation_id(&event_id, 10)
        .await
        .expect("load stored event")
        .into_iter()
        .next()
        .expect("stored event exists");

    consumers
        .record_processed("trace-processed-consumer", &stored)
        .await
        .expect("record processed annotation");
    consumers
        .record_failure(
            "trace-failed-consumer",
            &stored,
            "projection failed",
            Utc::now(),
        )
        .await
        .expect("record failure annotation");
    consumers
        .dead_letter(
            "trace-dlq-consumer",
            &stored,
            3,
            "projection permanently failed",
        )
        .await
        .expect("record dead letter annotation");

    let trace = store
        .trace_by_event_id(&event_id, 100)
        .await
        .expect("trace query")
        .expect("trace exists");

    assert_eq!(trace.consumer_annotations.len(), 2);
    assert!(trace.consumer_annotations.iter().any(|annotation| {
        annotation.event_id == event_id
            && annotation.consumer_name == "trace-processed-consumer"
            && annotation.status == "processed"
            && annotation.processed_at.is_some()
            && annotation.attempts.is_none()
    }));
    assert!(trace.consumer_annotations.iter().any(|annotation| {
        annotation.event_id == event_id
            && annotation.consumer_name == "trace-failed-consumer"
            && annotation.status == "failed"
            && annotation.processed_at.is_none()
            && annotation.attempts == Some(1)
    }));
    assert_eq!(trace.dead_letters.len(), 1);
    assert_eq!(trace.dead_letters[0].event_id, event_id);
    assert_eq!(
        trace.dead_letters[0].consumer_name.as_deref(),
        Some("trace-dlq-consumer")
    );
    assert_eq!(
        trace.dead_letters[0].reason,
        "projection permanently failed"
    );
    assert!(trace.dead_letters[0].failed_at.is_some());
}

#[tokio::test]
async fn projection_cursor_store_tracks_monotonic_positions_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let cursors = ProjectionCursorStore::new(database.pool().expect("configured pool").clone());

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let projection_name = format!("projection_cursor_test_{suffix}");

    assert_eq!(
        cursors
            .last_processed_position(&projection_name)
            .await
            .expect("initial cursor"),
        0
    );

    cursors
        .save_position(&projection_name, 10)
        .await
        .expect("save initial position");
    cursors
        .save_position(&projection_name, 7)
        .await
        .expect("lower position must not regress cursor");

    assert_eq!(
        cursors
            .last_processed_position(&projection_name)
            .await
            .expect("updated cursor"),
        10
    );
}

#[tokio::test]
async fn projection_cursor_store_can_explicitly_rewind_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let cursors = ProjectionCursorStore::new(database.pool().expect("configured pool").clone());

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let projection_name = format!("projection_cursor_rewind_test_{suffix}");

    cursors
        .save_position(&projection_name, 25)
        .await
        .expect("save initial position");

    assert_eq!(
        cursors
            .rewind_position(&projection_name, 9)
            .await
            .expect("rewind projection cursor"),
        9
    );
    assert_eq!(
        cursors
            .last_processed_position(&projection_name)
            .await
            .expect("rewound cursor"),
        9
    );
}
