use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::{Row, query};
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::integrations::telegram::client::participants::reconcile_leave_commands_from_exhaustive_absence;
use makosh_hub_backend::platform::storage::database::Database;

const LOCAL_API_TOKEN: &str = "telegram-participant-absence-test-secret";

#[tokio::test]
async fn telegram_exhaustive_roster_absence_reconciles_self_leave_command() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database.clone(),
    );
    let pool = database.pool().expect("configured pool").clone();
    let observed_at = Utc::now();

    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": "acct-1",
            "provider_kind": "telegram_user",
            "display_name": "Telegram Exhaustive Leave",
            "external_account_id": "telegram:12345",
            "tdlib_data_path": "docker/data/telegram/test-exhaustive-absence",
            "transcription_enabled": false
        }),
    )
    .await;

    query(
        r#"
        INSERT INTO telegram_provider_write_commands (
            command_id,
            account_id,
            command_kind,
            idempotency_key,
            provider_chat_id,
            provider_message_id,
            target_ref,
            payload,
            capability_state,
            action_class,
            confirmation_decision,
            status,
            retry_count,
            max_retries,
            result_payload,
            audit_metadata,
            actor_id,
            happened_at,
            reconciliation_status
        )
        VALUES (
            'cmd-leave-exhaustive-1',
            'acct-1',
            'leave',
            'idem-leave-exhaustive-1',
            'provider-chat-1',
            NULL,
            '{}'::jsonb,
            '{}'::jsonb,
            'available',
            'provider_write',
            'confirmed',
            'executing',
            0,
            3,
            '{}'::jsonb,
            '{}'::jsonb,
            'makosh-frontend',
            NOW() - INTERVAL '1 minute',
            'awaiting_provider'
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert leave command");

    let commands = reconcile_leave_commands_from_exhaustive_absence(
        &pool,
        "acct-1",
        "provider-chat-1",
        "user:12345",
        observed_at,
        "tdlib.getSupergroupMembers.exhaustive_absence",
    )
    .await
    .expect("reconcile exhaustive absence");

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command_id, "cmd-leave-exhaustive-1");
    assert_eq!(commands[0].status, "completed");
    assert_eq!(commands[0].reconciliation_status, "observed");
    assert_eq!(
        commands[0].provider_state["membership_state"],
        "absent_exhaustive"
    );
    assert_eq!(commands[0].provider_state["status"], Value::Null);
    assert_eq!(commands[0].provider_state["role"], Value::Null);
    assert_eq!(
        commands[0].provider_state["observed_via"],
        "tdlib.getSupergroupMembers.exhaustive_absence"
    );

    let row = query(
        r#"
        SELECT status, reconciliation_status, provider_state, result_payload, completed_at
        FROM telegram_provider_write_commands
        WHERE command_id = 'cmd-leave-exhaustive-1'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("reconciled leave command row");
    let provider_state: Value = row.try_get("provider_state").expect("provider state");
    let result_payload: Value = row.try_get("result_payload").expect("result payload");
    assert_eq!(
        row.try_get::<String, _>("status").expect("status"),
        "completed"
    );
    assert_eq!(
        row.try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "observed"
    );
    assert_eq!(provider_state["membership_state"], "absent_exhaustive");
    assert_eq!(
        result_payload["source"],
        "tdlib.getSupergroupMembers.exhaustive_absence"
    );
    assert_eq!(result_payload["provider_member_id"], "user:12345");
    assert!(
        row.try_get::<Option<chrono::DateTime<Utc>>, _>("completed_at")
            .expect("completed at")
            .is_some()
    );
}

async fn post_ok<S>(app: S, uri: &str, body: Value) -> Value
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header(header::CONTENT_TYPE, "application/json")
                .header("X-Макошь-Secret", LOCAL_API_TOKEN)
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&body).expect("json body")
}
