mod telegram_support;

use axum::http::StatusCode;
use chrono::Utc;
use serde_json::json;
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::integrations::telegram::client::references;
use makosh_hub_backend::platform::storage::database::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_ok, get_request_with_token, json_body, json_post_request_with_actor,
    unique_suffix,
};
#[tokio::test]
async fn telegram_reference_routes_return_enriched_message_summaries() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-reference-{suffix}");
    let chat_id = format!("reference-chat-{suffix}");
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
            "display_name": "Telegram References",
            "external_account_id": format!("tg-reference-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    let root_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("reference-root-{suffix}"),
                "chat_kind": "group",
                "chat_title": "Reference Room",
                "sender_id": format!("sender-root-{suffix}"),
                "sender_display_name": "Root Sender",
                "text": "Root message for reply targets",
                "import_batch_id": format!("telegram-reference-root-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("root response");
    assert_eq!(root_response.status(), StatusCode::OK);
    let root_body = json_body(root_response).await;
    let root_message_id = root_body["message_id"]
        .as_str()
        .expect("root message id")
        .to_owned();

    let reply_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("reference-reply-{suffix}"),
                "chat_kind": "group",
                "chat_title": "Reference Room",
                "sender_id": format!("sender-reply-{suffix}"),
                "sender_display_name": "Reply Sender",
                "text": "Reply body should appear in chain",
                "import_batch_id": format!("telegram-reference-reply-{suffix}"),
                "occurred_at": "2026-06-06T12:01:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("reply response");
    assert_eq!(reply_response.status(), StatusCode::OK);
    let reply_body = json_body(reply_response).await;
    let reply_message_id = reply_body["message_id"]
        .as_str()
        .expect("reply message id")
        .to_owned();

    let forward_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("reference-forward-{suffix}"),
                "chat_kind": "group",
                "chat_title": "Reference Room",
                "sender_id": format!("sender-forward-{suffix}"),
                "sender_display_name": "Forward Sender",
                "text": "Forward body should appear in summaries",
                "import_batch_id": format!("telegram-reference-forward-{suffix}"),
                "occurred_at": "2026-06-06T12:02:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("forward response");
    assert_eq!(forward_response.status(), StatusCode::OK);
    let forward_body = json_body(forward_response).await;
    let forward_message_id = forward_body["message_id"]
        .as_str()
        .expect("forward message id")
        .to_owned();
    let leaf_message_id = create_reference_message(
        app.clone(),
        &account_id,
        &chat_id,
        &format!("reference-leaf-{suffix}"),
        "Leaf Sender",
        "Leaf reply should appear through bounded traversal",
        &suffix,
    )
    .await;
    let final_forward_message_id = create_reference_message(
        app.clone(),
        &account_id,
        &chat_id,
        &format!("reference-final-forward-{suffix}"),
        "Final Forward Sender",
        "Final forward should appear through bounded traversal",
        &suffix,
    )
    .await;

    let pool = ctx.pool();
    references::insert_reply_ref(
        pool,
        &reply_message_id,
        &root_message_id,
        &account_id,
        &chat_id,
        &format!("reference-reply-{suffix}"),
        &format!("reference-root-{suffix}"),
        false,
    )
    .await
    .expect("insert reply ref");
    references::insert_reply_ref(
        pool,
        &leaf_message_id,
        &reply_message_id,
        &account_id,
        &chat_id,
        &format!("reference-leaf-{suffix}"),
        &format!("reference-reply-{suffix}"),
        false,
    )
    .await
    .expect("insert leaf reply ref");
    references::insert_reply_ref(
        pool,
        &root_message_id,
        &leaf_message_id,
        &account_id,
        &chat_id,
        &format!("reference-root-{suffix}"),
        &format!("reference-leaf-{suffix}"),
        false,
    )
    .await
    .expect("insert reply cycle ref");
    references::insert_forward_ref(
        pool,
        &forward_message_id,
        &account_id,
        &chat_id,
        &format!("reference-forward-{suffix}"),
        Some(&chat_id),
        Some(&format!("reference-root-{suffix}")),
        Some(&format!("sender-root-{suffix}")),
        Some("Root Sender"),
        Some(
            chrono::DateTime::parse_from_rfc3339("2026-06-05T11:00:00Z")
                .expect("timestamp")
                .with_timezone(&Utc),
        ),
    )
    .await
    .expect("insert forward ref");
    references::insert_forward_ref(
        pool,
        &final_forward_message_id,
        &account_id,
        &chat_id,
        &format!("reference-final-forward-{suffix}"),
        Some(&chat_id),
        Some(&format!("reference-forward-{suffix}")),
        Some(&format!("sender-forward-{suffix}")),
        Some("Forward Sender"),
        None,
    )
    .await
    .expect("insert final forward ref");
    references::insert_forward_ref(
        pool,
        &root_message_id,
        &account_id,
        &chat_id,
        &format!("reference-root-{suffix}"),
        Some(&chat_id),
        Some(&format!("reference-final-forward-{suffix}")),
        Some(&format!("sender-final-forward-{suffix}")),
        Some("Final Forward Sender"),
        None,
    )
    .await
    .expect("insert forward cycle ref");

    let reply_chain_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/messages/{reply_message_id}/reply-chain"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("reply chain response");
    assert_eq!(reply_chain_response.status(), StatusCode::OK);
    let reply_chain_body = json_body(reply_chain_response).await;
    assert_eq!(
        reply_chain_body["reply_to"][0]["target_message_summary"]["text"],
        json!("Root message for reply targets")
    );
    assert_eq!(
        reply_chain_body["reply_to"][0]["target_message_summary"]["sender_display_name"],
        json!("Root Sender")
    );

    let root_chain_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/messages/{root_message_id}/reply-chain"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("root chain response");
    assert_eq!(root_chain_response.status(), StatusCode::OK);
    let root_chain_body = json_body(root_chain_response).await;
    assert_eq!(
        root_chain_body["replies"][0]["source_message_summary"]["text"],
        json!("Reply body should appear in chain")
    );
    assert_eq!(
        root_chain_body["replies"][0]["source_message_summary"]["sender_display_name"],
        json!("Reply Sender")
    );
    assert_eq!(root_chain_body["replies"].as_array().unwrap().len(), 2);
    assert_eq!(
        root_chain_body["replies"][1]["source_message_summary"]["text"],
        json!("Leaf reply should appear through bounded traversal")
    );

    let leaf_chain_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/messages/{leaf_message_id}/reply-chain"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("leaf reply chain response");
    assert_eq!(leaf_chain_response.status(), StatusCode::OK);
    let leaf_chain_body = json_body(leaf_chain_response).await;
    assert_eq!(leaf_chain_body["reply_to"].as_array().unwrap().len(), 2);
    assert_eq!(
        leaf_chain_body["reply_to"][1]["target_message_summary"]["text"],
        json!("Root message for reply targets")
    );

    let forward_chain_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/messages/{forward_message_id}/forward-chain"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("forward chain response");
    assert_eq!(forward_chain_response.status(), StatusCode::OK);
    let forward_chain_body = json_body(forward_chain_response).await;
    assert_eq!(
        forward_chain_body["forwards"][0]["source_message_summary"]["text"],
        json!("Forward body should appear in summaries")
    );
    assert_eq!(
        forward_chain_body["forwards"][0]["forward_origin_sender_name"],
        json!("Root Sender")
    );

    let final_forward_chain_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/messages/{final_forward_message_id}/forward-chain"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("final forward chain response");
    assert_eq!(final_forward_chain_response.status(), StatusCode::OK);
    let final_forward_chain_body = json_body(final_forward_chain_response).await;
    assert_eq!(
        final_forward_chain_body["forwards"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        final_forward_chain_body["forwards"][1]["source_message_summary"]["text"],
        json!("Forward body should appear in summaries")
    );
}

async fn create_reference_message(
    app: axum::Router,
    account_id: &str,
    provider_chat_id: &str,
    provider_message_id: &str,
    sender_display_name: &str,
    text: &str,
    suffix: &str,
) -> String {
    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "chat_kind": "group",
                "chat_title": "Reference Idempotency Room",
                "sender_id": format!("sender-{sender_display_name}-{suffix}"),
                "sender_display_name": sender_display_name,
                "text": text,
                "import_batch_id": format!("telegram-reference-idempotent-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("reference message response");
    assert_eq!(response.status(), StatusCode::OK);
    json_body(response).await["message_id"]
        .as_str()
        .expect("message id")
        .to_owned()
}

#[tokio::test]
async fn telegram_chat_detail_and_members_routes_return_projected_data() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-chat-detail-{suffix}");
    let chat_id = format!("chat-detail-{suffix}");
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
            "display_name": "Telegram Chat Detail",
            "external_account_id": format!("tg-chat-detail-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;
    for (index, (sender_id, sender_display_name, text)) in [
        (format!("sender-a-{suffix}"), "Alice", "First chat message"),
        (format!("sender-b-{suffix}"), "Bob", "Second chat message"),
        (format!("sender-a-{suffix}"), "Alice", "Third chat message"),
    ]
    .into_iter()
    .enumerate()
    {
        assert_ok(
            app.clone(),
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("message-{sender_display_name}-{suffix}-{index}"),
                "chat_kind": "group",
                "chat_title": "Project Room",
                "sender_id": sender_id,
                "sender_display_name": sender_display_name,
                "text": text,
                "import_batch_id": format!("telegram-chat-detail-seed-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
        )
        .await;
    }

    let chats_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/conversations?account_id={account_id}&limit=10"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("chat list response");
    assert_eq!(chats_response.status(), StatusCode::OK);
    let chats_body = json_body(chats_response).await;
    let telegram_chat_id = chats_body["items"][0]["telegram_chat_id"]
        .as_str()
        .expect("telegram chat id")
        .to_owned();

    let detail_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/conversations/{telegram_chat_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("chat detail response");
    assert_eq!(detail_response.status(), StatusCode::OK);
    let detail_body = json_body(detail_response).await;
    assert_eq!(detail_body["item"]["provider_chat_id"], json!(chat_id));
    assert_eq!(detail_body["item"]["title"], json!("Project Room"));

    let members_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/communications/conversations/{telegram_chat_id}/members?limit=10"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("chat members response");
    assert_eq!(members_response.status(), StatusCode::OK);
    let members_body = json_body(members_response).await;
    let items = members_body["items"].as_array().expect("member items");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["sender_display_name"], json!("Alice"));
    assert_eq!(items[0]["message_count"], json!(2));
    assert_eq!(items[1]["sender_display_name"], json!("Bob"));
}

#[tokio::test]
async fn telegram_folders_route_returns_projection_backed_filters() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-folders-{suffix}");
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
            "display_name": "Telegram Folder Source",
            "external_account_id": format!("tg-folders-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/folders-{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    for (
        provider_chat_id,
        title,
        folder_name,
        folder_labels,
        provider_folder_id,
        provider_folder_ids,
    ) in [
        (
            "chat-alpha",
            "Alpha Room",
            "Work",
            vec!["Work"],
            Some(7_i64),
            vec![7_i64],
        ),
        (
            "chat-beta",
            "Beta Room",
            "Work",
            vec!["Work", "Pinned"],
            Some(7_i64),
            vec![7_i64, 11_i64],
        ),
        (
            "chat-gamma",
            "Gamma Room",
            "Archive",
            vec!["Archive"],
            Some(9_i64),
            vec![9_i64],
        ),
    ] {
        assert_ok(
            app.clone(),
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": format!("{provider_chat_id}-{suffix}"),
                "provider_message_id": format!("message-{provider_chat_id}-{suffix}"),
                "chat_kind": "group",
                "chat_title": title,
                "sender_id": format!("sender-{provider_chat_id}-{suffix}"),
                "sender_display_name": title,
                "text": format!("Message for {folder_name}"),
                "import_batch_id": format!("telegram-folders-seed-{suffix}"),
                "occurred_at": "2026-06-06T12:00:00Z",
                "delivery_state": "received"
            }),
        )
        .await;

        sqlx::query(
            r#"
            UPDATE telegram_chats
            SET metadata = COALESCE(metadata, '{}'::jsonb)
                || jsonb_build_object(
                    'folder_name', to_jsonb($3::text),
                    'folder_labels', to_jsonb($4::text[]),
                    'provider_folder_ids', to_jsonb($6::bigint[])
                )
                || CASE
                    WHEN $5::bigint IS NULL THEN '{}'::jsonb
                    ELSE jsonb_build_object('provider_folder_id', to_jsonb($5::bigint))
                   END
            WHERE account_id = $1
              AND provider_chat_id = $2
            "#,
        )
        .bind(&account_id)
        .bind(format!("{provider_chat_id}-{suffix}"))
        .bind(folder_name)
        .bind(folder_labels)
        .bind(provider_folder_id)
        .bind(provider_folder_ids)
        .execute(&pool)
        .await
        .expect("folder metadata update");
    }

    let folders_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/telegram/conversation-folders?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("folders response");
    assert_eq!(folders_response.status(), StatusCode::OK);
    let folders_body = json_body(folders_response).await;
    let items = folders_body["items"].as_array().expect("folder items");
    assert_eq!(items.len(), 4);
    assert_eq!(items[0]["id"], json!("local:all"));
    assert_eq!(items[0]["count"], json!(3));
    assert_eq!(items[1]["id"], json!("folder:Archive"));
    assert_eq!(items[1]["count"], json!(1));
    assert_eq!(items[1]["provider_folder_id"], json!(9));
    assert_eq!(items[2]["id"], json!("folder:Pinned"));
    assert_eq!(items[2]["count"], json!(1));
    assert_eq!(items[2]["provider_folder_id"], json!(11));
    assert_eq!(items[3]["id"], json!("folder:Work"));
    assert_eq!(items[3]["count"], json!(2));
    assert_eq!(items[3]["provider_folder_id"], json!(7));
}
