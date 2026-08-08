mod telegram_support;

use axum::http::StatusCode;
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::storage::database::Database;
use telegram_support::{
    LOCAL_API_TOKEN, delete_request_with_token, get_request_with_token,
    ingest_fixture_telegram_message, json_body, json_post_request_with_actor, unique_suffix,
};

#[tokio::test]
async fn removed_account_blocks_message_lifecycle_and_reaction_routes_before_side_effects() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-message-gates-{suffix}");
    let provider_chat_id = format!("message-gates-chat-{suffix}");
    let provider_message_id = format!("{provider_chat_id}:42");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "Telegram Message Capability Gates",
                "external_account_id": format!("tg-message-gates-{suffix}"),
                "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let message_id = ingest_fixture_telegram_message(
        app.clone(),
        &account_id,
        &provider_chat_id,
        &provider_message_id,
        "Message lifecycle actions should obey capability gates.",
        "2026-06-06T12:00:00Z",
    )
    .await;

    let account_delete_response = app
        .clone()
        .oneshot(delete_request_with_token(
            &format!("/api/v1/integrations/telegram/accounts/{account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("remove account response");
    assert_eq!(account_delete_response.status(), StatusCode::OK);

    let detail_before = message_detail(app.clone(), &message_id).await;
    let initial_metadata = detail_before["item"]["metadata"].clone();

    let edit_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/integrations/telegram/provider-commands/messages/{message_id}/edit"),
            json!({
                "command_id": format!("edit-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "new_text": "edited after account removal"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("edit response");
    assert_eq!(edit_response.status(), StatusCode::BAD_REQUEST);

    let delete_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!(
                "/api/v1/integrations/telegram/provider-commands/messages/{message_id}/delete"
            ),
            json!({
                "command_id": format!("delete-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "reason_class": "deleted_by_owner",
                "actor_class": "owner",
                "is_provider_delete": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("delete response");
    assert_eq!(delete_response.status(), StatusCode::BAD_REQUEST);

    let restore_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/integrations/telegram/provider-commands/messages/{message_id}/restore-visibility"),
            json!({
                "command_id": format!("restore-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "reason": "undo"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("restore response");
    assert_eq!(restore_response.status(), StatusCode::BAD_REQUEST);

    let pin_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/integrations/telegram/provider-commands/messages/{message_id}/pin"),
            json!({
                "command_id": format!("pin-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "is_pinned": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("pin response");
    assert_eq!(pin_response.status(), StatusCode::BAD_REQUEST);

    let add_reaction_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/communications/messages/{message_id}/reactions"),
            json!({
                "command_id": format!("react-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "reaction_emoji": "👍",
                "sender_id": "owner",
                "sender_display_name": "Owner"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("add reaction response");
    assert_eq!(add_reaction_response.status(), StatusCode::BAD_REQUEST);

    let remove_reaction_response = app
        .clone()
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/communications/messages/{message_id}/reactions?account_id={account_id}&provider_chat_id={provider_chat_id}&provider_message_id={provider_message_id}&reaction_emoji=%F0%9F%91%8D&sender_id=owner&sender_display_name=Owner&command_id=unreact-{suffix}"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("remove reaction response");
    assert_eq!(remove_reaction_response.status(), StatusCode::BAD_REQUEST);

    let command_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM telegram_provider_write_commands WHERE account_id = $1 AND command_kind = ANY($2)",
    )
    .bind(&account_id)
    .bind(vec![
        "edit",
        "delete",
        "restore_visibility",
        "pin",
        "react",
        "unreact",
    ])
    .fetch_one(&pool)
    .await
    .expect("message lifecycle command count");
    assert_eq!(command_count, 0);

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM api_audit_log WHERE target_id = $1 AND operation = ANY($2)",
    )
    .bind(&message_id)
    .bind(vec![
        "telegram.message.edit",
        "telegram.message.delete",
        "telegram.message.restore_visibility",
        "telegram.message.pin",
        "telegram.message.reaction",
    ])
    .fetch_one(&pool)
    .await
    .expect("message lifecycle audit count");
    assert_eq!(audit_count, 0);

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM event_log WHERE (subject->>'id' = $1 OR payload->>'message_id' = $1) AND event_type = ANY($2)",
    )
    .bind(&message_id)
    .bind(vec![
        "telegram.message.updated",
        "telegram.message.deleted",
        "telegram.message.visibility_restored",
        "telegram.reaction.changed",
        "telegram.command.status_changed",
    ])
    .fetch_one(&pool)
    .await
    .expect("message lifecycle event count");
    assert_eq!(event_count, 0);

    let version_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM telegram_message_versions WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("version count");
    assert_eq!(version_count, 0);

    let tombstone_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM telegram_message_tombstones WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("tombstone count");
    assert_eq!(tombstone_count, 0);

    let reaction_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::BIGINT FROM telegram_message_reactions WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("reaction count");
    assert_eq!(reaction_count, 0);

    let detail_after = message_detail(app, &message_id).await;
    let final_metadata = &detail_after["item"]["metadata"];
    assert_eq!(final_metadata["pinned"], initial_metadata["pinned"]);
    assert_eq!(final_metadata["is_pinned"], initial_metadata["is_pinned"]);
}

#[tokio::test]
async fn message_lifecycle_status_events_include_command_identity_for_realtime_command_inserts() {
    let ctx = TestContext::new().await;
    let pool = ctx.pool().clone();
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-message-status-{suffix}");
    let provider_chat_id = format!("message-status-chat-{suffix}");
    let provider_message_id = format!("{provider_chat_id}:42");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "Telegram Message Status Events",
                "external_account_id": format!("tg-message-status-{suffix}"),
                "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let message_id = ingest_fixture_telegram_message(
        app.clone(),
        &account_id,
        &provider_chat_id,
        &provider_message_id,
        "Message lifecycle realtime command payload should stay self-describing.",
        "2026-06-06T12:00:00Z",
    )
    .await;

    let edit_command_id = format!("edit-status-{suffix}");
    let edit_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/integrations/telegram/provider-commands/messages/{message_id}/edit"),
            json!({
                "command_id": edit_command_id,
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "new_text": "edited from command status event test"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("edit response");
    assert_eq!(edit_response.status(), StatusCode::OK);

    let edit_diff: Value = sqlx::query_scalar(
        r#"
        SELECT raw_diff_payload
        FROM telegram_message_versions
        WHERE message_id = $1
        ORDER BY version_number DESC
        LIMIT 1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("edit version diff");
    assert_eq!(
        edit_diff["previous_preview"],
        json!("Message lifecycle realtime command payload should stay self-describing.")
    );
    assert_eq!(
        edit_diff["new_preview"],
        json!("edited from command status event test")
    );
    assert!(edit_diff["previous_sha256"].is_string());
    assert!(edit_diff["new_sha256"].is_string());
    assert_eq!(edit_diff["changed"], json!(true));

    let delete_command_id = format!("delete-status-{suffix}");
    let delete_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!(
                "/api/v1/integrations/telegram/provider-commands/messages/{message_id}/delete"
            ),
            json!({
                "command_id": delete_command_id,
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "reason_class": "deleted_by_owner",
                "actor_class": "owner",
                "is_provider_delete": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("delete response");
    assert_eq!(delete_response.status(), StatusCode::OK);

    let edit_event: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type = 'telegram.command.status_changed'
        ORDER BY position ASC
        LIMIT 1
        "#,
    )
    .bind(&edit_command_id)
    .fetch_one(&pool)
    .await
    .expect("edit status payload");
    assert_eq!(edit_event["account_id"], json!(account_id));
    assert_eq!(edit_event["command_kind"], json!("edit"));
    assert_eq!(edit_event["action"], json!("edit"));
    assert_eq!(edit_event["provider_chat_id"], json!(provider_chat_id));
    assert_eq!(edit_event["message_id"], json!(message_id));
    assert_eq!(
        edit_event["provider_message_id"],
        json!(provider_message_id)
    );
    assert_eq!(
        edit_event["payload"]["telegram_message_id"],
        json!(message_id)
    );
    assert_eq!(
        edit_event["payload"]["new_text"],
        json!("edited from command status event test")
    );

    let delete_event: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM event_log
        WHERE subject->>'id' = $1
          AND event_type = 'telegram.command.status_changed'
        ORDER BY position ASC
        LIMIT 1
        "#,
    )
    .bind(&delete_command_id)
    .fetch_one(&pool)
    .await
    .expect("delete status payload");
    assert_eq!(delete_event["account_id"], json!(account_id));
    assert_eq!(delete_event["command_kind"], json!("delete"));
    assert_eq!(delete_event["action"], json!("delete"));
    assert_eq!(delete_event["provider_chat_id"], json!(provider_chat_id));
    assert_eq!(delete_event["message_id"], json!(message_id));
    assert_eq!(
        delete_event["provider_message_id"],
        json!(provider_message_id)
    );
    assert_eq!(
        delete_event["payload"]["telegram_message_id"],
        json!(message_id)
    );
    assert_eq!(
        delete_event["payload"]["reason_class"],
        json!("deleted_by_owner")
    );
    assert_eq!(delete_event["payload"]["is_provider_delete"], json!(true));
    assert!(delete_event["payload"]["tombstone_id"].is_string());
}

async fn message_detail<S>(app: S, message_id: &str) -> Value
where
    S: tower::Service<axum::http::Request<axum::body::Body>, Response = axum::response::Response>
        + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/communications/messages?limit=20",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message list response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    body["items"]
        .as_array()
        .expect("message items")
        .iter()
        .find(|item| item["message_id"] == json!(message_id))
        .cloned()
        .expect("message detail")
}
