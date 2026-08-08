use makosh_backend_testkit::context::TestContext;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::json;

use makosh_events_api::NewEventEnvelope;
use makosh_events_postgres::cursors::ProjectionCursorStore;
use makosh_events_postgres::store::EventStore;
use makosh_hub_backend::platform::projections::{
    ProjectionBatchOutcome, ProjectionHandlerError, run_projection_batch,
};
use makosh_hub_backend::platform::storage::database::Database;

#[tokio::test]
async fn projection_runner_processes_batch_and_advances_cursor_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let events = EventStore::new(pool.clone());
    let cursors = ProjectionCursorStore::new(pool.clone());

    let suffix = unique_suffix();
    let projection_name = format!("projection_runner_success_{suffix}");
    cursors
        .save_position(&projection_name, latest_event_position(&pool).await)
        .await
        .expect("initialize cursor");
    let first_position = append_projection_test_event(&events, &suffix, "first").await;
    let second_position = append_projection_test_event(&events, &suffix, "second").await;
    let handled_event_ids = Arc::new(Mutex::new(Vec::new()));

    let outcome = run_projection_batch(&events, &cursors, &projection_name, 10, {
        let handled_event_ids = Arc::clone(&handled_event_ids);
        move |event| {
            let handled_event_ids = Arc::clone(&handled_event_ids);
            async move {
                handled_event_ids
                    .lock()
                    .expect("handled ids lock")
                    .push(event.event.event_id);
                Ok(())
            }
        }
    })
    .await
    .expect("projection run");

    assert_eq!(
        outcome,
        ProjectionBatchOutcome {
            processed_count: 2,
            last_processed_position: second_position,
        }
    );
    assert_eq!(
        cursors
            .last_processed_position(&projection_name)
            .await
            .expect("cursor"),
        second_position
    );
    assert_eq!(handled_event_ids.lock().expect("handled ids lock").len(), 2);
    assert!(first_position < second_position);
}

#[tokio::test]
async fn projection_runner_stops_on_handler_error_without_advancing_failed_event_against_postgres()
{
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let events = EventStore::new(pool.clone());
    let cursors = ProjectionCursorStore::new(pool.clone());

    let suffix = unique_suffix();
    let projection_name = format!("projection_runner_failure_{suffix}");
    cursors
        .save_position(&projection_name, latest_event_position(&pool).await)
        .await
        .expect("initialize cursor");
    let first_position = append_projection_test_event(&events, &suffix, "first").await;
    let second_position = append_projection_test_event(&events, &suffix, "second").await;

    let result = run_projection_batch(
        &events,
        &cursors,
        &projection_name,
        10,
        |event| async move {
            if event.position == second_position {
                return Err(ProjectionHandlerError::new("handler failed"));
            }

            Ok(())
        },
    )
    .await;

    assert!(result.is_err(), "handler failure must fail the batch");
    assert_eq!(
        cursors
            .last_processed_position(&projection_name)
            .await
            .expect("cursor after failure"),
        first_position
    );

    let retry = run_projection_batch(&events, &cursors, &projection_name, 10, |_| async {
        Ok(())
    })
    .await
    .expect("retry projection run");

    assert_eq!(
        retry,
        ProjectionBatchOutcome {
            processed_count: 1,
            last_processed_position: second_position,
        }
    );
}

async fn append_projection_test_event(
    events: &EventStore,
    suffix: &str,
    logical_name: &str,
) -> i64 {
    let event_id = format!("evt_projection_runner_{logical_name}_{suffix}");
    let event = NewEventEnvelope::builder(
        &event_id,
        "system_projection_runner_test_event",
        Utc::now(),
        json!({
            "kind": "test",
            "provider": "integration",
            "source_id": event_id,
        }),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .build()
    .expect("valid event");

    events.append(&event).await.expect("append event")
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
