mod telegram_support;

use axum::http::StatusCode;
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::storage::database::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, get_request_with_token, json_body, json_post_request_with_actor,
    unique_suffix,
};

#[tokio::test]
async fn fixture_account_blocks_dialog_actions_before_side_effects() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-dialog-gates-{suffix}");
    let provider_chat_id = format!("dialog-gates-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram Dialog Capability Gates",
            "external_account_id": format!("tg-dialog-gates-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": format!("{provider_chat_id}:1"),
            "chat_kind": "private",
            "chat_title": "Dialog Capability Gates Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "Dialog actions must respect capability gates before side effects.",
            "import_batch_id": format!("telegram-dialog-gates-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let chats_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/conversations?account_id={account_id}&limit=10"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("chats response");
    assert_eq!(chats_response.status(), StatusCode::OK);
    let chats_body = json_body(chats_response).await;
    let telegram_chat_id = chats_body["items"][0]["telegram_chat_id"]
        .as_str()
        .expect("telegram chat id")
        .to_owned();

    let detail_before = chat_detail(app.clone(), &telegram_chat_id).await;
    let initial_metadata = detail_before["item"]["metadata"].clone();
    let read_target_message_id = format!("{provider_chat_id}:777");

    for (action, body) in [
        (
            "pin",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "unpin",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "archive",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "unarchive",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "mute",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "unmute",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "read",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "last_read_inbox_provider_message_id": read_target_message_id
            }),
        ),
        (
            "unread",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "folders/7",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "folders/7/remove",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id
            }),
        ),
        (
            "folders/reassign",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "target_provider_folder_ids": [7]
            }),
        ),
    ] {
        let response = app
            .clone()
            .oneshot(json_post_request_with_actor(
                &format!(
                    "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/{action}"
                ),
                body,
                LOCAL_API_TOKEN,
            ))
            .await
            .expect("dialog action response");
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "expected {action} to be capability-blocked"
        );
    }

    let command_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM telegram_provider_write_commands WHERE account_id = $1 AND command_kind = ANY($2)",
    )
    .bind(&account_id)
    .bind(vec![
        "pin",
        "unpin",
        "archive",
        "unarchive",
        "mute",
        "unmute",
        "mark_read",
        "mark_unread",
        "folder_add",
        "folder_remove",
    ])
    .fetch_one(&pool)
    .await
    .expect("dialog command count");
    assert_eq!(command_count, 0);

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM api_audit_log WHERE target_id = $1 AND operation = ANY($2)",
    )
    .bind(&telegram_chat_id)
    .bind(vec![
        "telegram.chat.pin",
        "telegram.chat.unpin",
        "telegram.chat.archive",
        "telegram.chat.unarchive",
        "telegram.chat.mute",
        "telegram.chat.unmute",
        "telegram.chat.mark_read",
        "telegram.chat.mark_unread",
        "telegram.chat.folder_add",
        "telegram.chat.folder_remove",
        "telegram.chat.folder_reassign",
    ])
    .fetch_one(&pool)
    .await
    .expect("dialog audit count");
    assert_eq!(audit_count, 0);

    let command_event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = 'telegram.command.status_changed' AND payload->>'telegram_chat_id' = $1",
    )
    .bind(&telegram_chat_id)
    .fetch_one(&pool)
    .await
    .expect("dialog command event count");
    assert_eq!(command_event_count, 0);

    let chat_event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE event_type = ANY($1) AND payload->>'telegram_chat_id' = $2",
    )
    .bind(vec![
        "telegram.chat.updated",
        "telegram.chat.pinned",
        "telegram.chat.archived",
        "telegram.chat.muted",
    ])
    .bind(&telegram_chat_id)
    .fetch_one(&pool)
    .await
    .expect("dialog chat event count");
    assert_eq!(chat_event_count, 0);

    let detail_after = chat_detail(app, &telegram_chat_id).await;
    let final_metadata = &detail_after["item"]["metadata"];
    assert_eq!(final_metadata["is_pinned"], initial_metadata["is_pinned"]);
    assert_eq!(
        final_metadata["is_archived"],
        initial_metadata["is_archived"]
    );
    assert_eq!(final_metadata["is_muted"], initial_metadata["is_muted"]);
    assert_eq!(
        final_metadata["last_read_inbox_provider_message_id"],
        initial_metadata["last_read_inbox_provider_message_id"]
    );
    assert_eq!(
        final_metadata["unread_count"],
        initial_metadata["unread_count"]
    );
    assert_eq!(
        final_metadata["mention_count"],
        initial_metadata["mention_count"]
    );
}

async fn chat_detail<S>(app: S, telegram_chat_id: &str) -> Value
where
    S: tower::Service<axum::http::Request<axum::body::Body>, Response = axum::response::Response>
        + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/conversations/{telegram_chat_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("chat detail response");
    assert_eq!(response.status(), StatusCode::OK);
    json_body(response).await
}
