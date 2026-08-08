use chrono::{Duration, Utc};
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::commands::NewCommunicationProviderCommand;
use makosh_communications_postgres::provider_commands::CommunicationProviderCommandStore;
use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;

use makosh_backend_testkit::app::TestApp;
use makosh_backend_testkit::composition::router_for_context;
use makosh_backend_testkit::context::TestContext;
use serde_json::{Value, json};

const PRIVATE_TARGET: &str = "private-target-message";
const PRIVATE_PROVIDER_ID: &str = "private-provider-id";
const PRIVATE_PAYLOAD: &str = "private-command-payload";
const PRIVATE_ERROR: &str = "private-provider-error";
const PRIVATE_RESULT: &str = "private-provider-result";

async fn test_app() -> TestApp {
    let context = TestContext::new().await;
    let router = router_for_context(&context);
    TestApp::new(context, router)
}

#[tokio::test]
async fn provider_command_lifecycle_events_are_idempotent_and_payload_safe() {
    let app = test_app().await;
    let pool = app.context().pool().clone();
    let account_id = "mail-command-lifecycle-events";
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Gmail,
            "Lifecycle event account",
            "lifecycle@example.test",
        ))
        .await
        .expect("provider account");
    let store = CommunicationProviderCommandStore::new(pool.clone());
    let command = NewCommunicationProviderCommand::new(
        "provider-command-lifecycle",
        account_id,
        "mail",
        "mark_read",
        "provider-command-lifecycle-idempotency",
        "test-automation-actor",
    )
    .provider_conversation_id("private-provider-conversation")
    .provider_message_id(PRIVATE_PROVIDER_ID)
    .target_ref(json!({"message_id": PRIVATE_TARGET}))
    .payload(json!({"body": PRIVATE_PAYLOAD}));

    let stored = store.enqueue(&command).await.expect("enqueue command");
    let duplicate = store
        .enqueue(&NewCommunicationProviderCommand {
            command_id: "provider-command-lifecycle-duplicate".to_owned(),
            ..command.clone()
        })
        .await
        .expect("idempotent enqueue");
    assert_eq!(duplicate.command_id, stored.command_id);

    let executing_at = Utc::now() + Duration::seconds(1);
    let claimed = store
        .claim_due(account_id, "mail", executing_at, 10)
        .await
        .expect("claim command");
    assert_eq!(claimed.len(), 1);

    let failed_at = executing_at + Duration::seconds(1);
    let failed = store
        .mark_terminal_failed(
            &stored.command_id,
            "mail",
            failed_at,
            PRIVATE_ERROR,
            json!({"detail": PRIVATE_RESULT}),
        )
        .await
        .expect("terminal failure")
        .expect("failed command");
    assert_eq!(failed.status, "dead_letter");

    let retry_at = failed_at + Duration::seconds(1);
    store
        .manual_retry(&stored.command_id, "mail", retry_at)
        .await
        .expect("manual retry")
        .expect("retried command");
    let reclaimed = store
        .claim_due(account_id, "mail", retry_at, 10)
        .await
        .expect("reclaim command");
    assert_eq!(reclaimed.len(), 1);

    let completed_at = retry_at + Duration::seconds(1);
    let completed = store
        .mark_completed(
            &stored.command_id,
            "mail",
            completed_at,
            json!({
                "provider_message_id": PRIVATE_PROVIDER_ID,
                "detail": PRIVATE_RESULT,
            }),
        )
        .await
        .expect("complete command")
        .expect("completed command");
    assert_eq!(completed.status, "completed");
    assert!(
        store
            .mark_completed(
                &stored.command_id,
                "mail",
                completed_at + Duration::seconds(1),
                json!({"detail": "duplicate-completion"}),
            )
            .await
            .expect("idempotent completion")
            .is_none()
    );

    let events = sqlx::query_as::<_, (String, i32, Value, Value, Value, Value)>(
        r#"
        SELECT event_type, schema_version, source, subject, payload, provenance
        FROM event_log
        WHERE subject->>'kind' = 'communication_provider_command'
          AND subject->>'id' = $1
        ORDER BY position ASC
        "#,
    )
    .bind(&stored.command_id)
    .fetch_all(&pool)
    .await
    .expect("provider command events");

    let event_types = events
        .iter()
        .map(|event| event.0.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        event_types,
        vec![
            "communication.provider_command.requested.v1",
            "communication.provider_command.executing.v1",
            "communication.provider_command.failed.v1",
            "communication.provider_command.retry_requested.v1",
            "communication.provider_command.executing.v1",
            "communication.provider_command.completed.v1",
        ]
    );
    assert!(events.iter().all(|event| event.1 == 1));

    let serialized_events = serde_json::to_string(&events).expect("serialize events");
    for private_value in [
        PRIVATE_TARGET,
        PRIVATE_PROVIDER_ID,
        PRIVATE_PAYLOAD,
        PRIVATE_ERROR,
        PRIVATE_RESULT,
        "private-provider-conversation",
        "duplicate-completion",
    ] {
        assert!(
            !serialized_events.contains(private_value),
            "event log leaked private value: {private_value}"
        );
    }

    let statuses = events
        .iter()
        .map(|event| event.4["status"].as_str().expect("event status"))
        .collect::<Vec<_>>();
    assert_eq!(
        statuses,
        vec![
            "queued",
            "executing",
            "dead_letter",
            "retrying",
            "executing",
            "completed",
        ]
    );
}
