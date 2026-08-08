use chrono::Utc;
use makosh_backend_testkit::app::TestApp;
use makosh_backend_testkit::composition::router_for_context;
use makosh_backend_testkit::context::TestContext;
use makosh_events_api::NewEventEnvelope;
use makosh_events_postgres::store::EventStore;
use makosh_hub_backend::domains::signal_hub::ai::dispatch_ai_helper_signal;
use serde_json::json;

async fn test_app() -> TestApp {
    let context = TestContext::new().await;
    let router = router_for_context(&context);
    TestApp::new(context, router)
}

#[tokio::test]
async fn repeated_ai_signal_resolves_legacy_source_idempotency_event() {
    let app = test_app().await;
    let pool = app.context().pool().clone();
    let event_store = EventStore::new(pool.clone());
    let source_id = "mail-ai-idempotency-message";
    let event_type = "signal.raw.ai.mail_intelligence.observed";
    let legacy_event = NewEventEnvelope::builder(
        "legacy-random-ai-event-id",
        event_type,
        Utc::now(),
        json!({
            "kind": "signal_source",
            "source_code": "ai",
            "source_id": source_id,
        }),
        json!({
            "kind": "communication_message",
            "message_id": source_id,
        }),
    )
    .payload(json!({"status": "processed", "body_included": false}))
    .provenance(json!({"source": "mail_ai_pipeline", "privacy": "body_redacted"}))
    .build()
    .expect("legacy raw event");
    event_store
        .append_for_dispatch(&legacy_event)
        .await
        .expect("seed legacy raw event");

    let accepted = dispatch_ai_helper_signal(
        pool.clone(),
        "mail_intelligence",
        source_id,
        json!({
            "kind": "communication_message",
            "message_id": source_id,
        }),
        json!({"status": "processed", "body_included": false}),
        json!({"source": "mail_ai_pipeline", "privacy": "body_redacted"}),
        Some("mail-ai-idempotency-observation"),
    )
    .await
    .expect("idempotent AI signal dispatch")
    .expect("accepted AI signal");

    assert_eq!(accepted.event_type, "signal.accepted.ai.mail_intelligence");
    let raw_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM event_log WHERE event_type = $1 AND source->>'source_id' = $2",
    )
    .bind(event_type)
    .bind(source_id)
    .fetch_one(&pool)
    .await
    .expect("raw event count");
    assert_eq!(raw_count, 1);
}

#[tokio::test]
async fn repeated_new_ai_signal_reuses_deterministic_event_identity() {
    let app = test_app().await;
    let pool = app.context().pool().clone();
    let source_id = "mail-ai-deterministic-message";

    let first = dispatch_test_mail_ai_signal(pool.clone(), source_id)
        .await
        .expect("first accepted AI signal");
    let second = dispatch_test_mail_ai_signal(pool.clone(), source_id)
        .await
        .expect("second accepted AI signal");

    assert_eq!(first.event_id, second.event_id);
    let raw_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM event_log WHERE event_type = 'signal.raw.ai.mail_intelligence.observed' AND source->>'source_id' = $1",
    )
    .bind(source_id)
    .fetch_one(&pool)
    .await
    .expect("raw event count");
    assert_eq!(raw_count, 1);
}

async fn dispatch_test_mail_ai_signal(
    pool: sqlx::PgPool,
    source_id: &str,
) -> Option<makosh_events_api::EventEnvelope> {
    dispatch_ai_helper_signal(
        pool,
        "mail_intelligence",
        source_id,
        json!({
            "kind": "communication_message",
            "message_id": source_id,
        }),
        json!({"status": "processed", "body_included": false}),
        json!({"source": "mail_ai_pipeline", "privacy": "body_redacted"}),
        Some("mail-ai-deterministic-observation"),
    )
    .await
    .expect("AI signal dispatch")
}
