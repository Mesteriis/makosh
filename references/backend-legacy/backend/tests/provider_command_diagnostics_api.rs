use axum::body::to_bytes;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::commands::NewCommunicationProviderCommand;
use makosh_communications_postgres::provider_commands::CommunicationProviderCommandStore;
use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;

use makosh_backend_testkit::app::{TestApp, get, post_json};
use makosh_backend_testkit::composition::router_for_context;
use makosh_backend_testkit::context::TestContext;
use serde_json::{Value, json};
use tower::ServiceExt;

async fn test_app() -> TestApp {
    let context = TestContext::new().await;
    let router = router_for_context(&context);
    TestApp::new(context, router)
}

#[tokio::test]
async fn mail_provider_command_diagnostics_are_filtered_and_payload_safe() {
    let app = test_app().await;
    let pool = app.context().pool().clone();
    let account_id = "mail-command-diagnostics";
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Gmail,
            "Diagnostics account",
            "diagnostics@example.test",
        ))
        .await
        .expect("provider account");
    let store = CommunicationProviderCommandStore::new(pool.clone());
    store
        .enqueue(
            &NewCommunicationProviderCommand::new(
                "diagnostic-command-queued",
                account_id,
                "mail",
                "mark_read",
                "diagnostic-command-queued",
                "test-actor",
            )
            .target_ref(json!({"message_id": "message-safe-ref"}))
            .payload(json!({"private_body": "must never leave the backend"})),
        )
        .await
        .expect("queued command");
    store
        .enqueue(&NewCommunicationProviderCommand::new(
            "diagnostic-command-dead",
            account_id,
            "mail",
            "archive",
            "diagnostic-command-dead",
            "test-actor",
        ))
        .await
        .expect("dead-letter command seed");
    sqlx::query(
        r#"
        UPDATE communication_provider_commands
        SET status = 'dead_letter', retry_count = 3,
            last_error = 'provider timeout\nprivate transport detail',
            dead_lettered_at = now()
        WHERE command_id = 'diagnostic-command-dead'
        "#,
    )
    .execute(&pool)
    .await
    .expect("dead-letter transition");

    let response = app
        .clone_router()
        .oneshot(get(&format!(
            "/api/v1/communications/provider-commands/diagnostics?account_id={account_id}&status=dead_letter&limit=10"
        )))
        .await
        .expect("diagnostics response");
    assert!(response.status().is_success());
    let body: Value = serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("diagnostics body"),
    )
    .expect("diagnostics json");

    assert_eq!(body["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(body["items"][0]["command_id"], "diagnostic-command-dead");
    assert_eq!(body["items"][0]["status"], "dead_letter");
    assert_eq!(
        body["items"][0]["last_error"],
        "provider timeout private transport detail"
    );
    assert!(body["items"][0].get("payload").is_none());
    assert!(body["items"][0].get("target_ref").is_none());
    assert!(body["counts"].as_array().is_some_and(|counts| {
        counts
            .iter()
            .any(|item| item["status"] == "queued" && item["count"] == 1)
    }));
    assert!(body["counts"].as_array().is_some_and(|counts| {
        counts
            .iter()
            .any(|item| item["status"] == "dead_letter" && item["count"] == 1)
    }));
}

#[tokio::test]
async fn mail_provider_dead_letter_can_be_requeued_without_exposing_payloads() {
    let app = test_app().await;
    let pool = app.context().pool().clone();
    let account_id = "mail-command-retry";
    CommunicationProviderAccountStore::new(pool.clone())
        .upsert(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Gmail,
            "Retry account",
            "retry@example.test",
        ))
        .await
        .expect("provider account");
    let store = CommunicationProviderCommandStore::new(pool.clone());
    store
        .enqueue(
            &NewCommunicationProviderCommand::new(
                "diagnostic-command-retry",
                account_id,
                "mail",
                "mark_read",
                "diagnostic-command-retry",
                "test-actor",
            )
            .payload(json!({"private_body": "must not leave the backend"})),
        )
        .await
        .expect("queued command");
    sqlx::query(
        r#"
        UPDATE communication_provider_commands
        SET status = 'dead_letter', retry_count = 3,
            last_error = 'provider timeout', dead_lettered_at = now()
        WHERE command_id = 'diagnostic-command-retry'
        "#,
    )
    .execute(&pool)
    .await
    .expect("dead-letter transition");

    let response = app
        .clone_router()
        .oneshot(post_json(
            "/api/v1/communications/provider-commands/diagnostic-command-retry/retry",
            json!({}),
        ))
        .await
        .expect("retry response");
    assert!(response.status().is_success());
    let body: Value = serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("retry body"),
    )
    .expect("retry json");
    assert_eq!(body["command_id"], "diagnostic-command-retry");
    assert_eq!(body["status"], "retrying");
    assert_eq!(body["retry_count"], 0);
    assert!(body["next_attempt_at"].is_string());
    assert!(body.get("payload").is_none());
    assert!(body.get("target_ref").is_none());

    let retried = store
        .list(account_id, "mail", 10)
        .await
        .expect("requeued command");
    assert_eq!(retried[0].status, "retrying");
    assert_eq!(retried[0].retry_count, 0);
    assert!(retried[0].dead_lettered_at.is_none());
}
