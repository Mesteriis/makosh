mod telegram_support;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::Row;
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::storage::database::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, delete_request_with_token, get_request_with_token, json_body,
    json_post_request_with_actor, unique_suffix,
};

#[tokio::test]
async fn telegram_restore_and_reaction_actions_record_durable_command_rows() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-lifecycle-actions-{suffix}");
    let chat_id = format!("lifecycle-chat-{suffix}");
    let provider_message_id = format!("lifecycle-message-{suffix}");
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
            "display_name": "Telegram Lifecycle Actions",
            "external_account_id": format!("tg-lifecycle-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;
    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "chat_kind": "private",
                "chat_title": "Lifecycle Action Chat",
                "sender_id": format!("sender-{suffix}"),
                "sender_display_name": "Pavel Sidorov",
                "text": "Lifecycle actions should create durable command rows.",
                "import_batch_id": format!("telegram-lifecycle-fixture-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);
    let message_body = json_body(message_response).await;
    let message_id = message_body["message_id"]
        .as_str()
        .expect("message id")
        .to_owned();

    let restore_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/integrations/telegram/provider-commands/messages/{message_id}/restore-visibility"),
            json!({
                "command_id": format!("restore-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "reason": "manual_restore"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("restore response");
    assert_eq!(restore_response.status(), StatusCode::OK);

    let add_reaction_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/communications/messages/{message_id}/reactions"),
            json!({
                "command_id": format!("react-{suffix}"),
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "reaction_emoji": "👍",
                "sender_id": "owner",
                "sender_display_name": "Owner"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("add reaction response");
    assert_eq!(add_reaction_response.status(), StatusCode::OK);

    let remove_reaction_response = app
        .clone()
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/communications/messages/{message_id}/reactions?account_id={account_id}&provider_chat_id={chat_id}&provider_message_id={provider_message_id}&reaction_emoji=%F0%9F%91%8D&sender_id=owner&sender_display_name=Owner&command_id=unreact-{suffix}"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("remove reaction response");
    assert_eq!(remove_reaction_response.status(), StatusCode::OK);

    let commands_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/telegram/commands?account_id={account_id}&limit=20"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("commands response");
    assert_eq!(commands_response.status(), StatusCode::OK);
    let commands_body = json_body(commands_response).await;
    let items = commands_body["items"].as_array().expect("command items");
    let kinds: Vec<&str> = items
        .iter()
        .filter_map(|item| item["command_kind"].as_str())
        .collect();

    for expected_kind in ["restore_visibility", "react", "unreact"] {
        assert!(
            kinds.iter().any(|kind| kind == &expected_kind),
            "expected command row for {expected_kind}, got {kinds:?}"
        );
    }

    let reaction_rows = sqlx::query(
        r#"
        SELECT reaction_id, is_active
        FROM telegram_message_reactions
        WHERE message_id = $1
          AND sender_id = 'owner'
          AND reaction_emoji = '👍'
        "#,
    )
    .bind(&message_id)
    .fetch_all(&pool)
    .await
    .expect("reaction rows");
    assert_eq!(reaction_rows.len(), 1);
    let reaction_id = reaction_rows[0].get::<String, _>("reaction_id");
    assert!(!reaction_rows[0].get::<bool, _>("is_active"));

    let reaction_observation_rows = sqlx::query(
        r#"
        SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'telegram'
          AND link.entity_kind = 'message_reaction'
          AND link.entity_id = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&reaction_id)
    .fetch_all(&pool)
    .await
    .expect("reaction observations");
    assert!(
        reaction_observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "TELEGRAM_MESSAGE_REACTION"
                && row.get::<String, _>("relationship_kind") == "local_add"
                && row.get::<serde_json::Value, _>("payload")["is_active"] == json!(true)
        }),
        "local_add reaction observation must exist"
    );
    assert!(
        reaction_observation_rows.iter().any(|row| {
            row.get::<String, _>("kind_code") == "TELEGRAM_MESSAGE_REACTION"
                && row.get::<String, _>("relationship_kind") == "local_remove"
                && row.get::<serde_json::Value, _>("payload")["is_active"] == json!(false)
        }),
        "local_remove reaction observation must exist"
    );
}
