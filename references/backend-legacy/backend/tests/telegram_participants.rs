use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::integrations::telegram::client::participants::{
    reconcile_join_commands_from_provider_roster, reconcile_leave_commands_from_provider_roster,
    telegram_self_provider_member_id,
};

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::storage::database::Database;

const LOCAL_API_TOKEN: &str = "telegram-participants-test-secret";

#[tokio::test]
async fn telegram_members_route_prefers_provider_roster_over_message_heuristic() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-participants-{suffix}");
    let provider_chat_id = format!("participants-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram Participants",
            "external_account_id": format!("tg-participants-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    for (index, (sender_id, sender_display_name)) in [
        (format!("sender-a-{suffix}"), "Alice"),
        (format!("sender-b-{suffix}"), "Bob"),
        (format!("sender-a-{suffix}"), "Alice"),
    ]
    .into_iter()
    .enumerate()
    {
        post_ok(
            app.clone(),
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": format!("message-{suffix}-{index}"),
                "chat_kind": "group",
                "chat_title": "Provider Roster Room",
                "sender_id": sender_id,
                "sender_display_name": sender_display_name,
                "text": format!("message {index}"),
                "import_batch_id": format!("telegram-participants-seed-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
        )
        .await;
    }

    let telegram_chat_id = first_chat_id(app.clone(), &account_id).await;
    let fallback_response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/conversations/{telegram_chat_id}/members?limit=10"
        )))
        .await
        .expect("fallback members response");
    assert_eq!(fallback_response.status(), StatusCode::OK);
    let fallback_body = json_body(fallback_response).await;
    assert_eq!(fallback_body["items"].as_array().expect("items").len(), 2);
    assert_eq!(fallback_body["items"][0]["source"], "message_heuristic");
    assert_eq!(fallback_body["items"][0]["sender_display_name"], "Alice");
    assert_eq!(fallback_body["items"][0]["message_count"], 2);

    insert_provider_participant(
        &pool,
        &telegram_chat_id,
        &account_id,
        &provider_chat_id,
        &suffix,
    )
    .await;

    let provider_response = app
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/conversations/{telegram_chat_id}/members?query=owner&role=owner&limit=10"
        )))
        .await
        .expect("provider members response");
    assert_eq!(provider_response.status(), StatusCode::OK);
    let provider_body = json_body(provider_response).await;
    let items = provider_body["items"].as_array().expect("provider items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["source"], "tdlib");
    assert_eq!(items[0]["provider_member_id"], "user:42");
    assert_eq!(items[0]["sender_id"], "user:42");
    assert_eq!(items[0]["sender_display_name"], "Owner User");
    assert_eq!(items[0]["role"], "owner");
    assert_eq!(items[0]["status"], "creator");
    assert_eq!(items[0]["is_admin"], true);
    assert_eq!(items[0]["is_owner"], true);
    assert_eq!(items[0]["permissions"]["can_invite_users"], true);
    assert_eq!(items[0]["message_count"], 0);
}

#[tokio::test]
async fn telegram_join_leave_routes_enqueue_provider_write_commands() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-join-leave-{suffix}");
    let provider_chat_id = format!("join-leave-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    seed_tdlib_qr_account(
        &pool,
        &account_id,
        "Telegram Join Leave",
        &format!("tg-join-leave-{suffix}"),
    )
    .await;
    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": format!("join-leave-message-{suffix}"),
            "chat_kind": "group",
            "chat_title": "Join Leave Room",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Sender",
            "text": "seed chat",
            "import_batch_id": format!("telegram-join-leave-seed-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let telegram_chat_id = first_chat_id(app.clone(), &account_id).await;
    let join_body = command_response(
        app.clone(),
        "/api/v1/integrations/telegram/provider-commands/conversations/join",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id
        }),
    )
    .await;
    assert_eq!(join_body["action"], "join");
    assert_eq!(join_body["status"], "queued");
    assert_eq!(join_body["telegram_chat_id"], Value::Null);

    let leave_body = command_response(
        app.clone(),
        &format!(
            "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/leave"
        ),
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id
        }),
    )
    .await;
    assert_eq!(leave_body["action"], "leave");
    assert_eq!(leave_body["status"], "queued");
    assert_eq!(leave_body["telegram_chat_id"], json!(telegram_chat_id));

    assert_command_row(
        &pool,
        join_body["command_id"].as_str().expect("join command id"),
        "join",
        &provider_chat_id,
        None,
    )
    .await;
    assert_command_row(
        &pool,
        leave_body["command_id"].as_str().expect("leave command id"),
        "leave",
        &provider_chat_id,
        Some(&telegram_chat_id),
    )
    .await;
}

#[tokio::test]
async fn telegram_roster_sync_reconciles_join_only_after_self_member_is_observed() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-join-reconcile-{suffix}");
    let provider_chat_id = format!("join-reconcile-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    seed_tdlib_qr_account(
        &pool,
        &account_id,
        "Telegram Join Reconcile",
        "telegram:12345",
    )
    .await;
    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": format!("join-reconcile-message-{suffix}"),
            "chat_kind": "group",
            "chat_title": "Join Reconcile Room",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Sender",
            "text": "seed chat",
            "import_batch_id": format!("telegram-join-reconcile-seed-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let join_body = command_response(
        app.clone(),
        "/api/v1/integrations/telegram/provider-commands/conversations/join",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id
        }),
    )
    .await;
    let command_id = join_body["command_id"].as_str().expect("command id");
    assert_eq!(
        telegram_self_provider_member_id("telegram:12345").as_deref(),
        Some("user:12345")
    );
    assert_eq!(
        telegram_self_provider_member_id(&format!("fixture-{suffix}")),
        None
    );

    let observed_at = sqlx::query_scalar("SELECT now()")
        .fetch_one(&pool)
        .await
        .expect("observed at");
    let commands = reconcile_join_commands_from_provider_roster(
        &pool,
        &account_id,
        &provider_chat_id,
        "user:12345",
        observed_at,
    )
    .await
    .expect("reconciled join commands");

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command_id, command_id);
    assert_eq!(commands[0].status, "completed");
    assert_eq!(commands[0].reconciliation_status, "observed");
    assert_eq!(
        commands[0].provider_state["observed_via"],
        "tdlib.getSupergroupMembers"
    );
    assert_eq!(commands[0].provider_state["membership_state"], "present");
    assert_eq!(
        commands[0].provider_state["provider_member_id"],
        "user:12345"
    );
    assert!(commands[0].provider_observed_at.is_some());
    assert!(commands[0].completed_at.is_some());

    let row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, provider_state, result_payload, completed_at
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_one(&pool)
    .await
    .expect("reconciled command row");
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
    assert_eq!(provider_state["membership_state"], "present");
    assert_eq!(result_payload["source"], "tdlib.getSupergroupMembers");
    assert!(
        row.try_get::<Option<chrono::DateTime<Utc>>, _>("completed_at")
            .expect("completed at")
            .is_some()
    );
}

#[tokio::test]
async fn telegram_roster_sync_reconciles_leave_when_self_member_is_inactive() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-leave-reconcile-{suffix}");
    let provider_chat_id = format!("leave-reconcile-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    seed_tdlib_qr_account(
        &pool,
        &account_id,
        "Telegram Leave Reconcile",
        "telegram:12345",
    )
    .await;
    post_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": format!("leave-reconcile-message-{suffix}"),
            "chat_kind": "group",
            "chat_title": "Leave Reconcile Room",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Sender",
            "text": "seed chat",
            "import_batch_id": format!("telegram-leave-reconcile-seed-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let telegram_chat_id = first_chat_id(app.clone(), &account_id).await;
    let leave_body = command_response(
        app.clone(),
        &format!(
            "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/leave"
        ),
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id
        }),
    )
    .await;
    let command_id = leave_body["command_id"].as_str().expect("command id");

    let observed_at = sqlx::query_scalar("SELECT now()")
        .fetch_one(&pool)
        .await
        .expect("observed at");
    let commands = reconcile_leave_commands_from_provider_roster(
        &pool,
        &account_id,
        &provider_chat_id,
        "user:12345",
        "left",
        Some("left"),
        Some("member"),
        observed_at,
    )
    .await
    .expect("reconciled leave commands");

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command_id, command_id);
    assert_eq!(commands[0].status, "completed");
    assert_eq!(commands[0].reconciliation_status, "observed");
    assert_eq!(
        commands[0].provider_state["observed_via"],
        "tdlib.getSupergroupMembers"
    );
    assert_eq!(commands[0].provider_state["membership_state"], "left");
    assert_eq!(commands[0].provider_state["status"], "left");
    assert_eq!(commands[0].provider_state["role"], "member");
    assert!(commands[0].provider_observed_at.is_some());
    assert!(commands[0].completed_at.is_some());

    let row = sqlx::query(
        r#"
        SELECT status, reconciliation_status, provider_state, result_payload, completed_at
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
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
    assert_eq!(provider_state["membership_state"], "left");
    assert_eq!(provider_state["status"], "left");
    assert_eq!(result_payload["source"], "tdlib.getSupergroupMembers");
    assert!(
        row.try_get::<Option<chrono::DateTime<Utc>>, _>("completed_at")
            .expect("completed at")
            .is_some()
    );
}

async fn insert_provider_participant(
    pool: &PgPool,
    telegram_chat_id: &str,
    account_id: &str,
    provider_chat_id: &str,
    suffix: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO telegram_chat_participants (
            participant_id, telegram_chat_id, account_id, provider_chat_id, provider_member_id,
            display_name, username, role, status, is_admin, is_owner, permissions, raw_payload,
            source
        )
        VALUES ($1, $2, $3, $4, 'user:42', 'Owner User', 'owner_user', 'owner', 'creator',
                true, true, $5, $6, 'tdlib')
        "#,
    )
    .bind(format!("telegram_participant_test_{suffix}"))
    .bind(telegram_chat_id)
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(json!({"can_invite_users": true}))
    .bind(json!({"@type": "chatMember"}))
    .execute(pool)
    .await
    .expect("insert provider participant");
}

async fn seed_tdlib_qr_account(
    pool: &PgPool,
    account_id: &str,
    display_name: &str,
    external_account_id: &str,
) {
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(
            &NewProviderAccount::new(
                account_id,
                CommunicationProviderKind::TelegramUser,
                display_name.to_owned(),
                external_account_id.to_owned(),
            )
            .config(json!({"runtime": "tdlib_qr_authorized"})),
        )
        .await
        .expect("tdlib qr authorized provider account");
}

async fn first_chat_id<S>(app: S, account_id: &str) -> String
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(get(&format!(
            "/api/v1/communications/conversations?account_id={account_id}&limit=10"
        )))
        .await
        .expect("chat list response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    body["items"][0]["telegram_chat_id"]
        .as_str()
        .expect("telegram chat id")
        .to_owned()
}

async fn command_response<S>(app: S, path: &str, body: Value) -> Value
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(json_post(path, body))
        .await
        .expect("command response");
    assert_eq!(response.status(), StatusCode::OK);
    json_body(response).await
}

async fn assert_command_row(
    pool: &PgPool,
    command_id: &str,
    expected_kind: &str,
    expected_provider_chat_id: &str,
    expected_telegram_chat_id: Option<&str>,
) {
    let row = sqlx::query(
        r#"
        SELECT command_kind, status, provider_chat_id, target_ref, action_class,
               confirmation_decision, reconciliation_status
        FROM telegram_provider_write_commands
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .fetch_one(pool)
    .await
    .expect("provider command row");
    let target_ref: Value = row.try_get("target_ref").expect("target ref");

    assert_eq!(
        row.try_get::<String, _>("command_kind")
            .expect("command kind"),
        expected_kind
    );
    assert_eq!(
        row.try_get::<String, _>("status").expect("status"),
        "queued"
    );
    assert_eq!(
        row.try_get::<String, _>("provider_chat_id")
            .expect("provider chat id"),
        expected_provider_chat_id
    );
    assert_eq!(
        row.try_get::<String, _>("action_class")
            .expect("action class"),
        "provider_write"
    );
    assert_eq!(
        row.try_get::<String, _>("confirmation_decision")
            .expect("confirmation decision"),
        "confirmed"
    );
    assert_eq!(
        row.try_get::<String, _>("reconciliation_status")
            .expect("reconciliation status"),
        "not_observed"
    );
    assert_eq!(
        target_ref["provider_chat_id"],
        json!(expected_provider_chat_id)
    );
    if let Some(expected_telegram_chat_id) = expected_telegram_chat_id {
        assert_eq!(
            target_ref["telegram_chat_id"],
            json!(expected_telegram_chat_id)
        );
    } else {
        assert_eq!(target_ref.get("telegram_chat_id"), None);
    }
}

async fn post_ok<S>(app: S, path: &str, body: Value)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(json_post(path, body))
        .await
        .expect("post response");
    assert_eq!(response.status(), StatusCode::OK);
}

fn get(path: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .body(Body::empty())
        .expect("request")
}

fn json_post(path: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), 1_000_000)
        .await
        .expect("body bytes");
    serde_json::from_slice(&bytes).expect("json body")
}

fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
        .to_string()
}
