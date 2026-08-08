mod telegram_support;

use axum::http::StatusCode;
use chrono::Utc;
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::communications::messages::provider_observation_projection::consume_accepted_signal_event;
use makosh_hub_backend::domains::signal_hub::telegram::dispatch_telegram_raw_signal;
use makosh_hub_backend::platform::storage::database::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, get_request_with_token, json_body, json_post_request_with_actor,
    json_post_request_with_explicit_actor_header, json_put_request_with_actor, unique_suffix,
};
#[tokio::test]
async fn telegram_manual_send_records_sent_message_and_redacted_provider_write_audit() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-send-{suffix}");
    let chat_id = format!("send-chat-{suffix}");
    let command_id = format!("manual-send-{suffix}");
    let message_text = "Manual Telegram reply from Макошь.";
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
            "display_name": "Telegram Send",
            "external_account_id": format!("tg-send-{suffix}"),
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
            "provider_chat_id": chat_id,
            "provider_message_id": format!("incoming-{suffix}"),
            "chat_kind": "private",
            "chat_title": "Manual Send Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "Can Макошь reply here?",
            "import_batch_id": format!("telegram-fixture-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let send_response = app
        .clone()
        .oneshot(json_post_request_with_explicit_actor_header(
            "/api/v1/integrations/telegram/provider-commands/messages/send",
            json!({
                "command_id": command_id,
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "text": message_text
            }),
            LOCAL_API_TOKEN,
            "legacy-telegram-test-actor",
        ))
        .await
        .expect("send response");
    assert_eq!(send_response.status(), StatusCode::OK);
    let send_body = json_body(send_response).await;
    assert_eq!(send_body["account_id"], json!(account_id));
    assert_eq!(send_body["provider_chat_id"], json!(chat_id));
    assert_eq!(send_body["delivery_state"], json!("sent"));
    assert_eq!(send_body["status"], json!("sent"));
    assert_eq!(send_body["runtime_kind"], json!("fixture"));
    assert!(
        send_body["message_id"]
            .as_str()
            .expect("message id")
            .starts_with("message:v4:telegram:")
    );
    assert!(
        send_body["rendered_preview_hash"]
            .as_str()
            .expect("preview hash")
            .starts_with("sha256:")
    );

    let messages_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/messages?account_id={account_id}&conversation_id={chat_id}"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("messages response");
    assert_eq!(messages_response.status(), StatusCode::OK);
    let messages_body = json_body(messages_response).await;
    let sent_message = messages_body["items"]
        .as_array()
        .expect("messages")
        .iter()
        .find(|message| message["delivery_state"] == "sent")
        .expect("sent message");
    assert_eq!(sent_message["body_text_preview"], json!(message_text));
    assert_eq!(sent_message["sender_display_name"], json!("Макошь"));

    let audit_metadata: Value = sqlx::query_scalar(
        r#"
        SELECT metadata
        FROM api_audit_log
        WHERE operation = 'telegram.message.send'
          AND actor_id = $1
          AND target_id = $2
        ORDER BY audit_id DESC
        LIMIT 1
        "#,
    )
    .bind("makosh-frontend")
    .bind(send_body["message_id"].as_str().expect("message id"))
    .fetch_one(&pool)
    .await
    .expect("manual send audit metadata");
    assert_eq!(audit_metadata["action_class"], json!("provider_write"));
    assert_eq!(audit_metadata["capability"], json!("telegram.message.send"));
    assert_eq!(audit_metadata["decision"], json!("allowed"));
    assert_eq!(
        audit_metadata["reason"],
        json!("explicit_user_confirmation")
    );
    assert_eq!(audit_metadata["confirmation_required"], json!(false));
    assert_eq!(audit_metadata["account_id"], json!(account_id));
    assert_eq!(audit_metadata["provider_chat_id"], json!(chat_id));
    assert_eq!(
        audit_metadata["rendered_preview_hash"],
        send_body["rendered_preview_hash"]
    );
    assert!(audit_metadata.get("text").is_none());
    assert!(audit_metadata.get("message_text").is_none());
    assert!(audit_metadata.get("rendered_text").is_none());

    let neutral_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/communications/conversations/{chat_id}/messages"),
            json!({
                "account_id": account_id,
                "text": "Provider-neutral Telegram send from Макошь."
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("provider-neutral send response");
    assert_eq!(neutral_response.status(), StatusCode::OK);
    let neutral_body = json_body(neutral_response).await;
    assert!(
        neutral_body["message_id"]
            .as_str()
            .expect("neutral message id")
            .starts_with("message:v4:telegram:")
    );
    assert!(
        !neutral_body["raw_record_id"]
            .as_str()
            .expect("neutral raw record id")
            .is_empty()
    );
    assert_eq!(neutral_body["conversation_id"], json!(chat_id));
    assert_eq!(neutral_body["channel_kind"], json!("telegram"));
    assert_eq!(neutral_body["provider"], json!("telegram"));
    assert_eq!(neutral_body["status"], json!("sent"));
}

#[tokio::test]
async fn telegram_raw_message_endpoint_returns_sanitized_source_evidence() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-raw-{suffix}");
    let chat_id = format!("raw-chat-{suffix}");
    let provider_message_id = format!("raw-message-{suffix}");
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
            "display_name": "Telegram Raw Evidence",
            "external_account_id": format!("tg-raw-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    let raw_record_id = format!("telegram-raw-record:{suffix}");
    let raw_record = CommunicationIngestionStore::new(pool.clone())
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "telegram_message",
                &provider_message_id,
                format!("sha256:raw-{suffix}"),
                format!("telegram-raw-fixture-{suffix}"),
                json!({
                    "provider_chat_id": chat_id,
                    "chat_title": "Raw Evidence Chat",
                    "sender_id": format!("sender-{suffix}"),
                    "sender_display_name": "Raw Sender",
                    "text": "Raw evidence should stay visible.",
                    "delivery_state": "received",
                    "tdlib_raw": {
                        "@type": "message",
                        "id": 42,
                        "nested": {
                            "api_hash": "telegram-api-hash",
                            "token": "telegram-token"
                        }
                    }
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({
                "provider": "telegram",
                "provider_kind": "telegram_user",
                "runtime": "tdlib",
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "password": "provider-password"
            })),
        )
        .await
        .expect("raw source");
    let accepted_event = dispatch_telegram_raw_signal(pool.clone(), &raw_record)
        .await
        .expect("dispatch raw telegram signal")
        .expect("accepted telegram signal");
    let projected = consume_accepted_signal_event(pool.clone(), &accepted_event)
        .await
        .expect("project accepted telegram signal")
        .expect("projected telegram message");
    let message_id = projected.message_id;

    let raw_response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/messages/{message_id}/raw-evidence"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("raw evidence response");
    assert_eq!(raw_response.status(), StatusCode::OK);
    let raw_body = json_body(raw_response).await;
    assert_eq!(
        raw_body["raw_record"]["raw_record_id"],
        json!(raw_record_id)
    );
    assert_eq!(
        raw_body["raw_record"]["provider_record_id"],
        json!(provider_message_id)
    );
    assert_eq!(
        raw_body["raw_record"]["payload"]["text"],
        json!("Raw evidence should stay visible.")
    );
    assert_eq!(
        raw_body["raw_record"]["payload"]["tdlib_raw"]["@type"],
        json!("message")
    );
    assert_eq!(
        raw_body["raw_record"]["payload"]["tdlib_raw"]["nested"]["api_hash"],
        json!("[redacted]")
    );
    assert_eq!(
        raw_body["raw_record"]["payload"]["tdlib_raw"]["nested"]["token"],
        json!("[redacted]")
    );
    assert_eq!(
        raw_body["raw_record"]["provenance"]["password"],
        json!("[redacted]")
    );
}

#[tokio::test]
async fn telegram_fixture_sync_chats_returns_account_chat_metadata() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-sync-chats-{suffix}");
    let chat_id = format!("sync-chat-{suffix}");
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
            "display_name": "Telegram Sync",
            "external_account_id": format!("tg-sync-{suffix}"),
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
            "provider_chat_id": chat_id,
            "provider_message_id": format!("incoming-{suffix}"),
            "chat_kind": "private",
            "chat_title": "Selected Sync Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "Fixture chat metadata should sync.",
            "import_batch_id": format!("telegram-fixture-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let sync_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/provider-sync/chats",
            json!({
                "account_id": account_id,
                "limit": 25
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("chat sync response");

    assert_eq!(sync_response.status(), StatusCode::OK);
    let sync_body = json_body(sync_response).await;
    assert_eq!(sync_body["account_id"], json!(account_id));
    assert_eq!(sync_body["runtime_kind"], json!("fixture"));
    assert_eq!(sync_body["status"], json!("synced"));
    assert_eq!(sync_body["synced_count"], json!(1));
    assert_eq!(sync_body["items"][0]["provider_chat_id"], json!(chat_id));
    assert_eq!(sync_body["items"][0]["title"], json!("Selected Sync Chat"));
}

#[tokio::test]
async fn telegram_fixture_sync_selected_history_returns_projected_messages() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-sync-history-{suffix}");
    let selected_chat_id = format!("selected-chat-{suffix}");
    let other_chat_id = format!("other-chat-{suffix}");
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
            "display_name": "Telegram History Sync",
            "external_account_id": format!("tg-history-{suffix}"),
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
            "provider_chat_id": selected_chat_id,
            "provider_message_id": format!("selected-incoming-{suffix}"),
            "chat_kind": "private",
            "chat_title": "Selected History Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "Selected chat history message.",
            "import_batch_id": format!("telegram-fixture-{suffix}"),
            "occurred_at": "2026-06-06T12:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/messages",
        json!({
            "account_id": account_id,
            "provider_chat_id": other_chat_id,
            "provider_message_id": format!("other-incoming-{suffix}"),
            "chat_kind": "private",
            "chat_title": "Other History Chat",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "This other chat must not be returned.",
            "import_batch_id": format!("telegram-fixture-{suffix}"),
            "occurred_at": "2026-06-06T12:01:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let sync_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/provider-sync/history",
            json!({
                "account_id": account_id,
                "provider_chat_id": selected_chat_id,
                "limit": 50
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("history sync response");

    assert_eq!(sync_response.status(), StatusCode::OK);
    let sync_body = json_body(sync_response).await;
    assert_eq!(sync_body["account_id"], json!(account_id));
    assert_eq!(sync_body["provider_chat_id"], json!(selected_chat_id));
    assert_eq!(sync_body["runtime_kind"], json!("fixture"));
    assert_eq!(sync_body["status"], json!("synced"));
    assert_eq!(sync_body["synced_count"], json!(1));
    assert_eq!(
        sync_body["items"][0]["provider_chat_id"],
        json!(selected_chat_id)
    );
    assert_eq!(
        sync_body["items"][0]["text"],
        json!("Selected chat history message.")
    );
}

#[tokio::test]
async fn telegram_group_history_policy_is_local_and_persisted() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-history-policy-{suffix}");
    let provider_chat_id = format!("group-history-policy-{suffix}");
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
            "display_name": "Telegram History Policy",
            "external_account_id": format!("tg-history-policy-{suffix}"),
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
            "provider_message_id": format!("group-history-message-{suffix}"),
            "chat_kind": "group",
            "chat_title": "Group History Policy",
            "sender_id": format!("sender-{suffix}"),
            "sender_display_name": "Maria Petrova",
            "text": "Configure full history locally.",
            "import_batch_id": format!("telegram-history-policy-{suffix}"),
            "occurred_at": "2026-07-12T20:00:00Z",
            "delivery_state": "received"
        }),
    )
    .await;

    let chats_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/conversations?account_id={account_id}&channel_kind=telegram"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("chat list response");
    assert_eq!(chats_response.status(), StatusCode::OK);
    let chats_body = json_body(chats_response).await;
    let telegram_chat_id = chats_body["items"][0]["telegram_chat_id"]
        .as_str()
        .expect("telegram chat id");

    let policy_response = app
        .clone()
        .oneshot(json_put_request_with_actor(
            &format!("/api/v1/communications/conversations/{telegram_chat_id}/history-policy"),
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "full_history_sync_enabled": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("history policy response");
    assert_eq!(policy_response.status(), StatusCode::OK);
    let policy_body = json_body(policy_response).await;
    assert_eq!(policy_body["action"], json!("history_policy_updated"));
    assert_eq!(
        policy_body["metadata"]["full_history_sync_enabled"],
        json!(true)
    );

    let persisted_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/communications/conversations?account_id={account_id}&channel_kind=telegram"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("persisted chat list response");
    assert_eq!(persisted_response.status(), StatusCode::OK);
    let persisted_body = json_body(persisted_response).await;
    assert_eq!(
        persisted_body["items"][0]["metadata"]["full_history_sync_enabled"],
        json!(true)
    );

    let receipt_policy_response = app
        .clone()
        .oneshot(json_put_request_with_actor(
            &format!("/api/v1/communications/conversations/{telegram_chat_id}/read-receipt-policy"),
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "read_receipt_reports_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("read receipt policy response");
    assert_eq!(receipt_policy_response.status(), StatusCode::OK);
    let receipt_policy_body = json_body(receipt_policy_response).await;
    assert_eq!(
        receipt_policy_body["action"],
        json!("read_receipt_policy_updated")
    );
    assert_eq!(
        receipt_policy_body["metadata"]["read_receipt_reports_enabled"],
        json!(false)
    );

    let read_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!(
                "/api/v1/integrations/telegram/provider-commands/conversations/{telegram_chat_id}/read"
            ),
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "last_read_inbox_provider_message_id": format!("group-history-message-{suffix}")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("local-only read response");
    assert_eq!(read_response.status(), StatusCode::OK);
    let read_body = json_body(read_response).await;
    assert_eq!(read_body["action"], json!("mark_read_local_only"));
    assert_eq!(read_body["status"], json!("read_local_only"));

    let counter_policy_response = app
        .oneshot(json_put_request_with_actor(
            &format!(
                "/api/v1/communications/conversations/{telegram_chat_id}/unread-counter-policy"
            ),
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "hide_unread_counter": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("unread counter policy response");
    assert_eq!(counter_policy_response.status(), StatusCode::OK);
    let counter_policy_body = json_body(counter_policy_response).await;
    assert_eq!(
        counter_policy_body["action"],
        json!("unread_counter_policy_updated")
    );
    assert_eq!(
        counter_policy_body["metadata"]["hide_unread_counter"],
        json!(true)
    );
}
