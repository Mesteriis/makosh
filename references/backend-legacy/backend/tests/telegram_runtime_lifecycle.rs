mod telegram_support;

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::storage::database::Database;
use telegram_support::{
    LOCAL_API_TOKEN, account_item, delete_request_with_token, get_request_with_token, json_body,
    json_post_request_with_actor, unique_suffix,
};
#[tokio::test]
async fn telegram_fixture_runtime_reconnects_enabled_account_and_accepts_idempotent_start() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-runtime-{suffix}");
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
                "display_name": "Telegram Runtime",
                "external_account_id": format!("tg-runtime-{suffix}"),
                "tdlib_data_path": format!("docker/data/telegram/{suffix}"),
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let initial_status = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/telegram/runtime/status?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("initial runtime status");
    assert_eq!(initial_status.status(), StatusCode::OK);
    let initial_body = json_body(initial_status).await;
    assert_eq!(initial_body["account_id"], json!(account_id));
    assert_eq!(initial_body["provider_kind"], json!("telegram_user"));
    assert_eq!(initial_body["runtime_kind"], json!("fixture"));
    assert_eq!(initial_body["status"], json!("running"));
    assert_eq!(initial_body["live_send_available"], json!(false));
    assert_eq!(initial_body["fixture_runtime"], json!(true));
    assert_eq!(initial_body["telegram_api_id_configured"], json!(false));
    assert_eq!(initial_body["telegram_api_hash_configured"], json!(false));
    assert_eq!(
        initial_body["telegram_app_credentials_configured"],
        json!(false)
    );
    assert_eq!(initial_body["runtime_blockers"], json!([]));

    let start_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/runtime/start",
            json!({ "account_id": account_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime start response");
    assert_eq!(start_response.status(), StatusCode::OK);
    let start_body = json_body(start_response).await;
    assert_eq!(start_body["account_id"], json!(account_id));
    assert_eq!(start_body["status"], json!("running"));
    assert_eq!(start_body["runtime_kind"], json!("fixture"));

    let running_status = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/telegram/runtime/status?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("running runtime status");
    assert_eq!(running_status.status(), StatusCode::OK);
    let running_body = json_body(running_status).await;
    assert_eq!(running_body["status"], json!("running"));
    assert_eq!(running_body["last_error"], Value::Null);
    assert_eq!(running_body["runtime_blockers"], json!([]));

    let restart_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/runtime/restart",
            json!({ "account_id": account_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime restart response");
    assert_eq!(restart_response.status(), StatusCode::OK);
    let restart_body = json_body(restart_response).await;
    assert_eq!(restart_body["account_id"], json!(account_id));
    assert_eq!(restart_body["runtime_kind"], json!("fixture"));
    assert_eq!(restart_body["status"], json!("running"));

    let stop_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/runtime/stop",
            json!({ "account_id": account_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime stop response");
    assert_eq!(stop_response.status(), StatusCode::OK);
    let stop_body = json_body(stop_response).await;
    assert_eq!(stop_body["account_id"], json!(account_id));
    assert_eq!(stop_body["runtime_kind"], json!("fixture"));
    assert_eq!(stop_body["status"], json!("stopped"));
    assert_eq!(stop_body["live_send_available"], json!(false));

    let stopped_status = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/telegram/runtime/status?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("stopped runtime status");
    assert_eq!(stopped_status.status(), StatusCode::OK);
    let stopped_body = json_body(stopped_status).await;
    assert_eq!(stopped_body["status"], json!("stopped"));

    let audit_metadata: Value = sqlx::query_scalar(
        r#"
        SELECT metadata
        FROM api_audit_log
        WHERE operation = 'telegram.runtime.stop'
          AND target_id = $1
        ORDER BY audit_id DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(ctx.pool())
    .await
    .expect("runtime stop audit metadata");
    assert_eq!(audit_metadata["capability"], json!("telegram.runtime.stop"));
    assert_eq!(audit_metadata["action_class"], json!("local_write"));
    assert_eq!(audit_metadata["account_id"], json!(account_id));
    assert_eq!(audit_metadata["runtime_kind"], json!("fixture"));
    assert_eq!(audit_metadata["status"], json!("stopped"));

    let restart_audit_metadata: Value = sqlx::query_scalar(
        r#"
        SELECT metadata
        FROM api_audit_log
        WHERE operation = 'telegram.runtime.restart'
          AND target_id = $1
        ORDER BY audit_id DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(ctx.pool())
    .await
    .expect("runtime restart audit metadata");
    assert_eq!(
        restart_audit_metadata["capability"],
        json!("telegram.runtime.restart")
    );
    assert_eq!(restart_audit_metadata["action_class"], json!("local_write"));
    assert_eq!(restart_audit_metadata["account_id"], json!(account_id));
    assert_eq!(restart_audit_metadata["runtime_kind"], json!("fixture"));
    assert_eq!(restart_audit_metadata["status"], json!("running"));
}

#[tokio::test]
async fn telegram_runtime_status_reports_tdlib_diagnostics_for_qr_authorized_user_accounts() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let account_id = format!("telegram-runtime-health-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_pairs([
            ("MAKOSH_DEV_MODE", "true"),
            (
                "MAKOSH_TDJSON_PATH",
                "/tmp/makosh-test-missing-libtdjson-runtime-health.dylib",
            ),
            ("MAKOSH_TELEGRAM_API_ID", "12345"),
            ("MAKOSH_TELEGRAM_API_HASH", "telegram-api-hash"),
        ])
        .expect("config"),
        database,
    );

    let account_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "Telegram Runtime Health",
                "external_account_id": format!("telegram:{suffix}"),
                "tdlib_data_path": format!("docker/data/telegram/runtime-health-{suffix}"),
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");
    assert_eq!(account_response.status(), StatusCode::OK);

    let runtime_status = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/telegram/runtime/status?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime status response");
    assert_eq!(runtime_status.status(), StatusCode::OK);
    let body = json_body(runtime_status).await;
    assert_eq!(body["runtime_kind"], json!("tdlib_qr_authorized"));
    assert_eq!(
        body["tdjson_path"],
        json!("/tmp/makosh-test-missing-libtdjson-runtime-health.dylib")
    );
    assert_eq!(body["tdjson_runtime_available"], json!(false));
    assert_eq!(body["telegram_api_id_configured"], json!(true));
    assert_eq!(body["telegram_api_hash_configured"], json!(true));
    assert_eq!(body["telegram_app_credentials_configured"], json!(true));
    assert_eq!(body["live_send_available"], json!(false));
    assert!(
        body["tdjson_probe_error"]
            .as_str()
            .expect("tdjson probe error")
            .contains("unable to load libtdjson")
    );
    assert!(
        body["runtime_blockers"]
            .as_array()
            .expect("runtime blockers")
            .iter()
            .any(|value| value == "tdjson_runtime_unavailable")
    );
}

#[tokio::test]
async fn telegram_account_lifecycle_lists_logs_out_and_removes_without_deleting_evidence() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-lifecycle-{suffix}");
    let second_account_id = format!("telegram-lifecycle-second-{suffix}");
    let chat_id = format!("lifecycle-chat-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_dev_mode(),
        database,
    );

    for (id, display_name) in [
        (&account_id, "Telegram Lifecycle"),
        (&second_account_id, "Telegram Lifecycle Second"),
    ] {
        let response = app
            .clone()
            .oneshot(json_post_request_with_actor(
                "/api/v1/integrations/telegram/fixtures/accounts",
                json!({
                    "account_id": id,
                    "provider_kind": "telegram_user",
                    "display_name": display_name,
                    "external_account_id": format!("tg-{id}"),
                    "tdlib_data_path": format!("docker/data/telegram/{id}"),
                    "transcription_enabled": false
                }),
                LOCAL_API_TOKEN,
            ))
            .await
            .expect("account response");
        assert_eq!(response.status(), StatusCode::OK);
        let signal_connection = sqlx::query(
            r#"
            SELECT source_code, status, settings
            FROM signal_connections
            WHERE source_code = 'telegram'
              AND settings->>'account_id' = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("telegram signal connection");
        let signal_settings: Value = signal_connection
            .try_get("settings")
            .expect("telegram signal settings");
        assert_eq!(
            signal_connection
                .try_get::<String, _>("status")
                .expect("signal status"),
            "connected"
        );
        assert_eq!(signal_settings["account_id"], json!(id));
        assert_eq!(signal_settings["provider_kind"], json!("telegram_user"));
    }

    let start_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/runtime/start",
            json!({ "account_id": account_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("runtime start response");
    assert_eq!(start_response.status(), StatusCode::OK);

    let message_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": format!("lifecycle-message-{suffix}"),
                "chat_kind": "private",
                "chat_title": "Telegram Lifecycle",
                "sender_id": format!("telegram-lifecycle-sender-{suffix}"),
                "sender_display_name": "Telegram Lifecycle Sender",
                "text": "Keep this local evidence after account removal.",
                "import_batch_id": format!("telegram-lifecycle-fixture-{suffix}"),
                "occurred_at": "2026-06-10T12:30:00Z",
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("message response");
    assert_eq!(message_response.status(), StatusCode::OK);

    let list_response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/integrations/telegram/accounts",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account list response");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = json_body(list_response).await;
    let items = list_body["items"].as_array().expect("account list items");
    let account = account_item(items, &account_id);
    assert_eq!(account["provider_kind"], json!("telegram_user"));
    assert_eq!(account["display_name"], json!("Telegram Lifecycle"));
    assert_eq!(account["runtime"], json!("fixture"));
    assert_eq!(account["lifecycle_state"], json!("active"));
    assert_eq!(account["transcription_enabled"], json!(false));
    assert_eq!(
        account["tdlib_data_path"],
        json!(format!("docker/data/telegram/{account_id}"))
    );
    assert!(account.get("config").is_none());
    assert!(account.get("api_hash").is_none());
    assert!(account.get("bot_token").is_none());
    account_item(items, &second_account_id);

    let logout_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            &format!("/api/v1/integrations/telegram/accounts/{account_id}/logout"),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("logout response");
    assert_eq!(logout_response.status(), StatusCode::OK);
    let logout_body = json_body(logout_response).await;
    assert_eq!(logout_body["account"]["account_id"], json!(account_id));
    assert_eq!(
        logout_body["account"]["lifecycle_state"],
        json!("logged_out")
    );
    assert_eq!(logout_body["stopped_runtime_actor"], json!(true));

    let logged_out_status = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/telegram/runtime/status?account_id={account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("logged out runtime status");
    assert_eq!(logged_out_status.status(), StatusCode::OK);
    let logged_out_status_body = json_body(logged_out_status).await;
    assert_eq!(logged_out_status_body["status"], json!("stopped"));
    let logged_out_signal_status: String = sqlx::query_scalar(
        r#"
        SELECT status
        FROM signal_connections
        WHERE source_code = 'telegram'
          AND settings->>'account_id' = $1
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("logged out signal status");
    assert_eq!(logged_out_signal_status, "disconnected");

    let logged_out_list_response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/integrations/telegram/accounts",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("logged out account list response");
    assert_eq!(logged_out_list_response.status(), StatusCode::OK);
    let logged_out_list_body = json_body(logged_out_list_response).await;
    let logged_out_items = logged_out_list_body["items"]
        .as_array()
        .expect("logged out account list items");
    assert_eq!(
        account_item(logged_out_items, &account_id)["lifecycle_state"],
        json!("logged_out")
    );

    let remove_response = app
        .clone()
        .oneshot(delete_request_with_token(
            &format!("/api/v1/integrations/telegram/accounts/{account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("remove response");
    assert_eq!(remove_response.status(), StatusCode::OK);
    let remove_body = json_body(remove_response).await;
    assert_eq!(remove_body["account"]["account_id"], json!(account_id));
    assert_eq!(remove_body["account"]["lifecycle_state"], json!("removed"));
    let removed_signal_status: String = sqlx::query_scalar(
        r#"
        SELECT status
        FROM signal_connections
        WHERE source_code = 'telegram'
          AND settings->>'account_id' = $1
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("removed signal status");
    assert_eq!(removed_signal_status, "removed");

    let default_list_response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/integrations/telegram/accounts",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("default account list response");
    assert_eq!(default_list_response.status(), StatusCode::OK);
    let default_list_body = json_body(default_list_response).await;
    let default_items = default_list_body["items"]
        .as_array()
        .expect("default account list items");
    assert!(
        !default_items
            .iter()
            .any(|item| item["account_id"] == json!(account_id))
    );
    account_item(default_items, &second_account_id);

    let removed_list_response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/integrations/telegram/accounts?include_removed=true",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("removed account list response");
    assert_eq!(removed_list_response.status(), StatusCode::OK);
    let removed_list_body = json_body(removed_list_response).await;
    let removed_items = removed_list_body["items"]
        .as_array()
        .expect("removed account list items");
    assert_eq!(
        account_item(removed_items, &account_id)["lifecycle_state"],
        json!("removed")
    );

    let raw_record_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM communication_raw_records WHERE account_id = $1")
            .bind(&account_id)
            .fetch_one(&pool)
            .await
            .expect("raw record count");
    assert_eq!(raw_record_count, 1);
    let message_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM communication_messages WHERE account_id = $1")
            .bind(&account_id)
            .fetch_one(&pool)
            .await
            .expect("message count");
    assert_eq!(message_count, 1);

    let account_config: Value = sqlx::query_scalar(
        "SELECT config FROM communication_provider_accounts WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("removed account config");
    assert_eq!(account_config["runtime"], json!("fixture"));
    assert_eq!(account_config["lifecycle_state"], json!("removed"));
    assert!(account_config.get("removed_at").is_some());
    assert!(account_config.get("api_hash").is_none());
    assert!(account_config.get("bot_token").is_none());

    let audit_rows = sqlx::query(
        r#"
        SELECT operation, metadata
        FROM api_audit_log
        WHERE target_kind = 'communication_provider_account'
          AND target_id = $1
        ORDER BY audit_id ASC
        "#,
    )
    .bind(&account_id)
    .fetch_all(&pool)
    .await
    .expect("audit rows");
    let audit_operations = audit_rows
        .iter()
        .map(|row| row.try_get::<String, _>("operation").expect("operation"))
        .collect::<Vec<_>>();
    assert_eq!(
        audit_operations,
        vec!["telegram.account.logout", "telegram.account.remove"]
    );
    for row in audit_rows {
        let metadata: Value = row.try_get("metadata").expect("metadata");
        assert_eq!(metadata["action_class"], json!("local_write"));
        assert_eq!(metadata["decision"], json!("allowed"));
        assert_eq!(metadata["account_id"], json!(account_id));
        assert!(metadata.get("api_hash").is_none());
        assert!(metadata.get("bot_token").is_none());
        assert!(metadata.get("session_encryption_key").is_none());
    }
}
