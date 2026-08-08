use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{TimeZone, Utc};
use makosh_events_api::{EventEnvelope, NewEventEnvelope, StoredEventEnvelope};
use makosh_events_postgres::cursors::ProjectionCursorStore;
use makosh_events_postgres::store::EventStore;
use makosh_hub_backend::engines::timeline::{TimelineEngine, models::TimelineEventDraft};
use makosh_hub_backend::platform::storage::database::Database;
use serde_json::json;

#[test]
fn timeline_engine_bounds_entity_timeline_limits() {
    assert_eq!(TimelineEngine::bounded_entity_limit(0), 1);
    assert_eq!(TimelineEngine::bounded_entity_limit(25), 25);
    assert_eq!(TimelineEngine::bounded_entity_limit(250), 100);
}

#[test]
fn timeline_engine_rejects_unsourced_timeline_event() {
    let draft = TimelineEventDraft {
        entity_kind: "persona",
        entity_id: "persona:v1:human:alice",
        event_type: "first_message",
        title: "First message",
        occurred_at: Utc::now(),
        source: " ",
    };

    let error = TimelineEngine::validate_event(&draft).expect_err("event should be rejected");

    assert_eq!(error.to_string(), "timeline event source must not be empty");
}

#[test]
fn timeline_engine_accepts_source_backed_timeline_event() {
    let draft = TimelineEventDraft {
        entity_kind: "persona",
        entity_id: "persona:v1:human:alice",
        event_type: "first_message",
        title: "First message",
        occurred_at: Utc::now(),
        source: "communication_messages:message-1",
    };

    TimelineEngine::validate_event(&draft).expect("source-backed event should be valid");
}

#[test]
fn timeline_engine_builds_period_summary_for_source_backed_events() {
    let period_start = Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap();
    let period_end = Utc.with_ymd_and_hms(2026, 6, 30, 23, 59, 59).unwrap();
    let events = vec![
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "message",
            title: "Message from Alice",
            occurred_at: Utc.with_ymd_and_hms(2026, 6, 3, 12, 0, 0).unwrap(),
            source: "communication_messages:message-1",
        },
        TimelineEventDraft {
            entity_kind: "project",
            entity_id: "project:makosh",
            event_type: "decision",
            title: "Decision accepted",
            occurred_at: Utc.with_ymd_and_hms(2026, 6, 5, 9, 0, 0).unwrap(),
            source: "decisions:decision-1",
        },
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "message",
            title: "Message from Alice before period",
            occurred_at: Utc.with_ymd_and_hms(2026, 5, 31, 23, 0, 0).unwrap(),
            source: "communication_messages:message-0",
        },
    ];

    let summary = TimelineEngine::period_summary(&events, period_start, period_end)
        .expect("period summary should be valid");

    assert_eq!(summary.period_start, period_start);
    assert_eq!(summary.period_end, period_end);
    assert_eq!(summary.total_events, 2);
    assert_eq!(summary.by_entity_kind.get("persona"), Some(&1));
    assert_eq!(summary.by_entity_kind.get("project"), Some(&1));
    assert_eq!(summary.by_event_type.get("message"), Some(&1));
    assert_eq!(summary.by_event_type.get("decision"), Some(&1));
}

#[test]
fn timeline_engine_rejects_invalid_period_summary_range() {
    let period_start = Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap();
    let period_end = Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap();

    let error = TimelineEngine::period_summary(&[], period_start, period_end)
        .expect_err("period start must not be after period end");

    assert_eq!(
        error.to_string(),
        "timeline period start must not be after period end"
    );
}

#[test]
fn timeline_engine_builds_recency_signal_for_source_backed_entity_events() {
    let as_of = Utc.with_ymd_and_hms(2026, 6, 10, 12, 0, 0).unwrap();
    let last_event_at = Utc.with_ymd_and_hms(2026, 6, 8, 12, 0, 0).unwrap();
    let events = vec![
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "message",
            title: "Earlier message",
            occurred_at: Utc.with_ymd_and_hms(2026, 6, 2, 12, 0, 0).unwrap(),
            source: "communication_messages:message-1",
        },
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "decision",
            title: "Latest reviewed decision",
            occurred_at: last_event_at,
            source: "decisions:decision-1",
        },
        TimelineEventDraft {
            entity_kind: "project",
            entity_id: "project:makosh",
            event_type: "status_change",
            title: "Project update",
            occurred_at: Utc.with_ymd_and_hms(2026, 6, 9, 12, 0, 0).unwrap(),
            source: "projects:project-1",
        },
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "future_message",
            title: "Future message",
            occurred_at: Utc.with_ymd_and_hms(2026, 6, 11, 12, 0, 0).unwrap(),
            source: "communication_messages:message-2",
        },
    ];

    let signal =
        TimelineEngine::recency_signal(&events, "persona", "persona:v1:human:alice", as_of)
            .expect("recency signal should be valid");

    assert_eq!(signal.entity_kind, "persona");
    assert_eq!(signal.entity_id, "persona:v1:human:alice");
    assert_eq!(signal.last_event_at, Some(last_event_at));
    assert_eq!(signal.last_event_type.as_deref(), Some("decision"));
    assert_eq!(
        signal.last_event_source.as_deref(),
        Some("decisions:decision-1")
    );
    assert_eq!(signal.age_seconds, Some(172_800));
}

#[test]
fn timeline_engine_detects_source_backed_entity_timeline_gaps() {
    let period_start = Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap();
    let period_end = Utc.with_ymd_and_hms(2026, 6, 30, 23, 59, 59).unwrap();
    let gap_start = Utc.with_ymd_and_hms(2026, 6, 2, 12, 0, 0).unwrap();
    let gap_end = Utc.with_ymd_and_hms(2026, 6, 10, 12, 0, 0).unwrap();
    let events = vec![
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "message",
            title: "Later message",
            occurred_at: gap_end,
            source: "communication_messages:message-2",
        },
        TimelineEventDraft {
            entity_kind: "project",
            entity_id: "project:makosh",
            event_type: "status_change",
            title: "Project update",
            occurred_at: Utc.with_ymd_and_hms(2026, 6, 6, 12, 0, 0).unwrap(),
            source: "projects:project-1",
        },
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "message",
            title: "Earlier message",
            occurred_at: gap_start,
            source: "communication_messages:message-1",
        },
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "decision",
            title: "Decision after gap",
            occurred_at: Utc.with_ymd_and_hms(2026, 6, 11, 12, 0, 0).unwrap(),
            source: "decisions:decision-1",
        },
    ];

    let gaps = TimelineEngine::timeline_gaps(
        &events,
        "persona",
        "persona:v1:human:alice",
        period_start,
        period_end,
        259_200,
    )
    .expect("timeline gaps should be valid");

    assert_eq!(gaps.len(), 1);
    let gap = &gaps[0];
    assert_eq!(gap.entity_kind, "persona");
    assert_eq!(gap.entity_id, "persona:v1:human:alice");
    assert_eq!(gap.gap_start, gap_start);
    assert_eq!(gap.gap_end, gap_end);
    assert_eq!(gap.gap_seconds, 691_200);
    assert_eq!(
        gap.previous_event_source.as_deref(),
        Some("communication_messages:message-1")
    );
    assert_eq!(
        gap.next_event_source.as_deref(),
        Some("communication_messages:message-2")
    );
}

#[test]
fn timeline_engine_rejects_invalid_gap_threshold() {
    let period_start = Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap();
    let period_end = Utc.with_ymd_and_hms(2026, 6, 30, 23, 59, 59).unwrap();

    let error = TimelineEngine::timeline_gaps(
        &[],
        "persona",
        "persona:v1:human:alice",
        period_start,
        period_end,
        0,
    )
    .expect_err("timeline gap threshold must be positive");

    assert_eq!(
        error.to_string(),
        "timeline gap threshold must be greater than zero"
    );
}

#[test]
fn timeline_engine_builds_source_backed_change_diff_for_entity_snapshots() {
    let shared_event_at = Utc.with_ymd_and_hms(2026, 6, 2, 12, 0, 0).unwrap();
    let removed_event_at = Utc.with_ymd_and_hms(2026, 6, 4, 12, 0, 0).unwrap();
    let added_event_at = Utc.with_ymd_and_hms(2026, 6, 8, 12, 0, 0).unwrap();
    let previous_events = vec![
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "message",
            title: "Shared message",
            occurred_at: shared_event_at,
            source: "communication_messages:message-1",
        },
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "decision",
            title: "Removed decision",
            occurred_at: removed_event_at,
            source: "decisions:decision-1",
        },
    ];
    let current_events = vec![
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "message",
            title: "Shared message",
            occurred_at: shared_event_at,
            source: "communication_messages:message-1",
        },
        TimelineEventDraft {
            entity_kind: "project",
            entity_id: "project:makosh",
            event_type: "status_change",
            title: "Project update",
            occurred_at: Utc.with_ymd_and_hms(2026, 6, 7, 12, 0, 0).unwrap(),
            source: "projects:project-1",
        },
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "decision",
            title: "Added decision",
            occurred_at: added_event_at,
            source: "decisions:decision-2",
        },
    ];

    let diff = TimelineEngine::change_diff(
        &previous_events,
        &current_events,
        "persona",
        "persona:v1:human:alice",
    )
    .expect("change diff should be valid");

    assert_eq!(diff.entity_kind, "persona");
    assert_eq!(diff.entity_id, "persona:v1:human:alice");
    assert_eq!(diff.added.len(), 1);
    assert_eq!(diff.removed.len(), 1);
    assert_eq!(diff.added[0].source, "decisions:decision-2");
    assert_eq!(diff.added[0].event_type, "decision");
    assert_eq!(diff.added[0].occurred_at, added_event_at);
    assert_eq!(diff.removed[0].source, "decisions:decision-1");
    assert_eq!(diff.removed[0].event_type, "decision");
    assert_eq!(diff.removed[0].occurred_at, removed_event_at);
}

#[test]
fn timeline_engine_builds_cross_domain_timeline_for_source_backed_events() {
    let period_start = Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap();
    let period_end = Utc.with_ymd_and_hms(2026, 6, 30, 23, 59, 59).unwrap();
    let message_at = Utc.with_ymd_and_hms(2026, 6, 3, 12, 0, 0).unwrap();
    let decision_at = Utc.with_ymd_and_hms(2026, 6, 5, 9, 0, 0).unwrap();
    let events = vec![
        TimelineEventDraft {
            entity_kind: "project",
            entity_id: "project:makosh",
            event_type: "decision",
            title: "Decision accepted",
            occurred_at: decision_at,
            source: "decisions:decision-1",
        },
        TimelineEventDraft {
            entity_kind: "persona",
            entity_id: "persona:v1:human:alice",
            event_type: "message",
            title: "Message from Alice",
            occurred_at: message_at,
            source: "communication_messages:message-1",
        },
        TimelineEventDraft {
            entity_kind: "document",
            entity_id: "document:outside-period",
            event_type: "imported",
            title: "Imported document",
            occurred_at: Utc.with_ymd_and_hms(2026, 7, 1, 9, 0, 0).unwrap(),
            source: "documents:document-1",
        },
    ];

    let timeline = TimelineEngine::cross_domain_timeline(&events, period_start, period_end, 10)
        .expect("cross-domain timeline should be valid");

    assert_eq!(timeline.len(), 2);
    assert_eq!(timeline[0].entity_kind, "persona");
    assert_eq!(timeline[0].entity_id, "persona:v1:human:alice");
    assert_eq!(timeline[0].event_type, "message");
    assert_eq!(timeline[0].title, "Message from Alice");
    assert_eq!(timeline[0].occurred_at, message_at);
    assert_eq!(timeline[0].source, "communication_messages:message-1");
    assert_eq!(timeline[1].entity_kind, "project");
    assert_eq!(timeline[1].entity_id, "project:makosh");
    assert_eq!(timeline[1].event_type, "decision");
    assert_eq!(timeline[1].title, "Decision accepted");
    assert_eq!(timeline[1].occurred_at, decision_at);
    assert_eq!(timeline[1].source, "decisions:decision-1");
}

#[test]
fn timeline_engine_replays_canonical_event_log_batch_into_cross_domain_timeline() {
    let period_start = Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap();
    let period_end = Utc.with_ymd_and_hms(2026, 6, 30, 23, 59, 59).unwrap();
    let recorded_at = Utc.with_ymd_and_hms(2026, 6, 10, 12, 0, 0).unwrap();
    let message_at = Utc.with_ymd_and_hms(2026, 6, 3, 12, 0, 0).unwrap();
    let decision_at = Utc.with_ymd_and_hms(2026, 6, 5, 9, 0, 0).unwrap();
    let stored_events = vec![
        StoredEventEnvelope {
            position: 1,
            event: EventEnvelope {
                event_id: "evt-decision-1".to_owned(),
                event_type: "decision_recorded".to_owned(),
                schema_version: 1,
                occurred_at: decision_at,
                recorded_at,
                source: json!({"kind": "decisions", "source_id": "decision-1"}),
                actor: None,
                subject: json!({"kind": "project", "entity_id": "project:makosh"}),
                payload: json!({"title": "Decision accepted"}),
                provenance: json!({"confidence": 1.0}),
                causation_id: None,
                correlation_id: Some("timeline-replay-test".to_owned()),
            },
        },
        StoredEventEnvelope {
            position: 2,
            event: EventEnvelope {
                event_id: "evt-message-1".to_owned(),
                event_type: "message_received".to_owned(),
                schema_version: 1,
                occurred_at: message_at,
                recorded_at,
                source: json!({"kind": "communication_messages", "source_id": "message-1"}),
                actor: None,
                subject: json!({"kind": "persona", "entity_id": "persona:v1:human:alice"}),
                payload: json!({"title": "Message from Alice"}),
                provenance: json!({"confidence": 1.0}),
                causation_id: None,
                correlation_id: Some("timeline-replay-test".to_owned()),
            },
        },
        StoredEventEnvelope {
            position: 3,
            event: EventEnvelope {
                event_id: "evt-document-1".to_owned(),
                event_type: "document_uploaded".to_owned(),
                schema_version: 1,
                occurred_at: Utc.with_ymd_and_hms(2026, 7, 1, 9, 0, 0).unwrap(),
                recorded_at,
                source: json!({"kind": "documents", "source_id": "document-1"}),
                actor: None,
                subject: json!({"kind": "document", "entity_id": "document:outside-period"}),
                payload: json!({"title": "Imported document"}),
                provenance: json!({"confidence": 1.0}),
                causation_id: None,
                correlation_id: Some("timeline-replay-test".to_owned()),
            },
        },
    ];

    let replay = TimelineEngine::replay_event_log(&stored_events, period_start, period_end, 10)
        .expect("event log replay should be valid");

    assert_eq!(replay.last_replayed_position, 3);
    assert_eq!(replay.entries.len(), 2);
    assert_eq!(replay.entries[0].entity_kind, "persona");
    assert_eq!(replay.entries[0].entity_id, "persona:v1:human:alice");
    assert_eq!(replay.entries[0].event_type, "message_received");
    assert_eq!(replay.entries[0].title, "Message from Alice");
    assert_eq!(replay.entries[0].occurred_at, message_at);
    assert_eq!(replay.entries[0].source, "communication_messages:message-1");
    assert_eq!(replay.entries[1].entity_kind, "project");
    assert_eq!(replay.entries[1].entity_id, "project:makosh");
    assert_eq!(replay.entries[1].event_type, "decision_recorded");
    assert_eq!(replay.entries[1].title, "Decision accepted");
    assert_eq!(replay.entries[1].occurred_at, decision_at);
    assert_eq!(replay.entries[1].source, "decisions:decision-1");
}

#[tokio::test]
async fn timeline_engine_projection_reads_event_log_and_advances_cursor_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let events = EventStore::new(pool.clone());
    let cursors = ProjectionCursorStore::new(pool.clone());
    let suffix = unique_suffix();
    let projection_name = format!("timeline_projection_{suffix}");
    cursors
        .save_position(&projection_name, latest_event_position(&pool).await)
        .await
        .expect("initialize cursor");

    let period_start = Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap();
    let period_end = Utc.with_ymd_and_hms(2026, 6, 30, 23, 59, 59).unwrap();
    let message_at = Utc.with_ymd_and_hms(2026, 6, 3, 12, 0, 0).unwrap();
    let outside_period_at = Utc.with_ymd_and_hms(2026, 7, 1, 9, 0, 0).unwrap();

    let first_position = append_timeline_projection_event(
        &events,
        &suffix,
        TimelineProjectionTestEvent {
            logical_name: "message",
            event_type: "message_received",
            occurred_at: message_at,
            source: json!({"kind": "communication_messages", "source_id": format!("message-{suffix}")}),
            subject: json!({"kind": "persona", "entity_id": "persona:v1:human:alice"}),
            title: "Message from Alice",
        },
    )
    .await;
    let second_position = append_timeline_projection_event(
        &events,
        &suffix,
        TimelineProjectionTestEvent {
            logical_name: "document",
            event_type: "document_uploaded",
            occurred_at: outside_period_at,
            source: json!({"kind": "documents", "source_id": format!("document-{suffix}")}),
            subject: json!({"kind": "document", "entity_id": "document:outside-period"}),
            title: "Imported document",
        },
    )
    .await;

    let run = TimelineEngine::run_event_log_projection(
        &events,
        &cursors,
        &projection_name,
        period_start,
        period_end,
        10,
        10,
    )
    .await
    .expect("timeline projection run");

    assert_eq!(run.processed_count, 2);
    assert_eq!(run.last_processed_position, second_position);
    assert_eq!(run.entries.len(), 1);
    assert_eq!(run.entries[0].entity_kind, "persona");
    assert_eq!(run.entries[0].entity_id, "persona:v1:human:alice");
    assert_eq!(run.entries[0].event_type, "message_received");
    assert_eq!(run.entries[0].title, "Message from Alice");
    assert_eq!(run.entries[0].occurred_at, message_at);
    assert_eq!(
        run.entries[0].source,
        format!("communication_messages:message-{suffix}")
    );
    assert_eq!(
        cursors
            .last_processed_position(&projection_name)
            .await
            .expect("cursor after run"),
        second_position
    );
    assert!(first_position < second_position);
}

async fn append_timeline_projection_event(
    events: &EventStore,
    suffix: &str,
    event: TimelineProjectionTestEvent,
) -> i64 {
    let event_id = format!("evt_timeline_projection_{}_{}", event.logical_name, suffix);
    let envelope = NewEventEnvelope::builder(
        &event_id,
        event.event_type,
        event.occurred_at,
        event.source,
        event.subject,
    )
    .payload(json!({"title": event.title}))
    .provenance(json!({"confidence": 1.0}))
    .correlation_id(format!("timeline-projection-{suffix}"))
    .build()
    .expect("valid event");

    events.append(&envelope).await.expect("append event")
}

struct TimelineProjectionTestEvent {
    logical_name: &'static str,
    event_type: &'static str,
    occurred_at: chrono::DateTime<Utc>,
    source: serde_json::Value,
    subject: serde_json::Value,
    title: &'static str,
}

fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
        .to_string()
}

async fn latest_event_position(pool: &sqlx::PgPool) -> i64 {
    sqlx::query_scalar::<_, Option<i64>>("SELECT max(position) FROM event_log")
        .fetch_one(pool)
        .await
        .expect("latest event position")
        .unwrap_or(0)
}
