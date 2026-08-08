mod telegram_support;

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::Row;
use tempfile::tempdir;
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::secrets::models::{SecretKind, SecretStoreKind};
use makosh_hub_backend::platform::secrets::store::SecretReferenceStore;
use makosh_hub_backend::platform::storage::database::Database;
use telegram_support::{
    LOCAL_API_TOKEN, assert_capability_status, assert_ok, get_request_with_token, json_body,
    json_post_request_with_actor, unique_suffix, vault_entropy_events,
};
#[tokio::test]
async fn telegram_live_account_setup_stores_bot_token_in_host_vault() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-bot-{suffix}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_pairs([
            ("MAKOSH_DEV_MODE", "true"),
            (
                "MAKOSH_VAULT_HOME",
                vault_dir.path().join("vault").to_str().expect("vault path"),
            ),
            (
                "MAKOSH_DEV_KEY_PATH",
                vault_dir
                    .path()
                    .join("dev")
                    .join("master.key")
                    .to_str()
                    .expect("dev key path"),
            ),
        ])
        .expect("config"),
        database,
    );

    let entropy_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/vault/collect-entropy",
            json!({ "events": vault_entropy_events(2_000) }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("entropy response");
    assert_eq!(entropy_response.status(), StatusCode::OK);
    let create_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/vault/create",
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("vault create response");
    assert_eq!(create_response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_bot",
                "display_name": "Telegram Bot",
                "external_account_id": format!("@makosh_bot_{suffix}"),
                "bot_token": "123456:telegram-bot-token",
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["account_id"], json!(account_id));
    assert_eq!(body["provider_kind"], json!("telegram_bot"));
    assert_eq!(body["runtime"], json!("live_blocked"));
    assert_eq!(
        body["credential_bindings"][0]["secret_purpose"],
        json!("telegram_bot_token")
    );
    assert_eq!(
        body["credential_bindings"][0]["secret_kind"],
        json!("api_token")
    );
    assert_eq!(
        body["credential_bindings"][0]["store_kind"],
        json!("host_vault")
    );

    let account = sqlx::query(
        "SELECT provider_kind, config FROM communication_provider_accounts WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("provider account");
    let provider_kind: String = account.try_get("provider_kind").expect("provider kind");
    let config: Value = account.try_get("config").expect("config");
    assert_eq!(provider_kind, "telegram_bot");
    assert_eq!(config["runtime"], json!("live_blocked"));
    assert_eq!(config["transcription_enabled"], json!(false));
    assert!(config.get("bot_token").is_none());
    assert!(config.get("api_hash").is_none());

    let secret_ref = body["credential_bindings"][0]["secret_ref"]
        .as_str()
        .expect("secret ref");
    let secret_store = SecretReferenceStore::new(pool.clone());
    let reference = secret_store
        .secret_reference(secret_ref)
        .await
        .expect("secret reference query")
        .expect("secret reference exists");
    assert_eq!(reference.secret_kind, SecretKind::ApiToken);
    assert_eq!(reference.store_kind, SecretStoreKind::HostVault);
    assert_eq!(reference.metadata["provider"], json!("telegram_bot"));
    assert_eq!(reference.metadata["account_id"], json!(account_id));

    let database_payload_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM encrypted_secret_vault_entries WHERE secret_ref = $1",
    )
    .bind(secret_ref)
    .fetch_one(&pool)
    .await
    .expect("database payload count");
    assert_eq!(database_payload_count, 0);
}
#[tokio::test]
async fn telegram_qr_authorized_account_setup_persists_metadata_without_host_vault_secret() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-user-{suffix}");
    let tdlib_data_path = format!("docker/data/telegram/{account_id}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_pairs([
            ("MAKOSH_DEV_MODE", "true"),
            (
                "MAKOSH_VAULT_HOME",
                vault_dir.path().join("vault").to_str().expect("vault path"),
            ),
            (
                "MAKOSH_DEV_KEY_PATH",
                vault_dir
                    .path()
                    .join("dev")
                    .join("master.key")
                    .to_str()
                    .expect("dev key path"),
            ),
            ("MAKOSH_TELEGRAM_API_ID", "12345"),
            ("MAKOSH_TELEGRAM_API_HASH", "telegram-api-hash"),
        ])
        .expect("config"),
        database,
    );

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "@second_account",
                "external_account_id": format!("telegram:{suffix}"),
                "tdlib_data_path": tdlib_data_path,
                "transcription_enabled": false,
                "qr_authorized": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["account_id"], json!(account_id));
    assert_eq!(body["provider_kind"], json!("telegram_user"));
    assert_eq!(body["runtime"], json!("tdlib_qr_authorized"));
    assert_eq!(body["credential_bindings"], json!([]));

    let account = sqlx::query(
        "SELECT provider_kind, display_name, external_account_id, config FROM communication_provider_accounts WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("provider account");
    let provider_kind: String = account.try_get("provider_kind").expect("provider kind");
    let display_name: String = account.try_get("display_name").expect("display name");
    let external_account_id: String = account
        .try_get("external_account_id")
        .expect("external account id");
    let config: Value = account.try_get("config").expect("config");
    assert_eq!(provider_kind, "telegram_user");
    assert_eq!(display_name, "@second_account");
    assert_eq!(external_account_id, format!("telegram:{suffix}"));
    assert_eq!(config["runtime"], json!("tdlib_qr_authorized"));
    assert_eq!(config["tdlib_data_path"], json!(tdlib_data_path));
    assert_eq!(config["transcription_enabled"], json!(false));
    assert!(config.get("api_hash").is_none());
    assert!(config.get("bot_token").is_none());

    let binding_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM communication_provider_account_secret_refs WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("binding count");
    assert_eq!(binding_count, 0);
}
#[tokio::test]
async fn telegram_finalized_qr_account_setup_infers_qr_authorized_runtime() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let account_id = format!("telegram-user-inferred-{suffix}");
    let tdlib_data_path = format!("docker/data/telegram/{account_id}");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        )
        .with_test_pairs([
            ("MAKOSH_DEV_MODE", "true"),
            (
                "MAKOSH_VAULT_HOME",
                vault_dir.path().join("vault").to_str().expect("vault path"),
            ),
            (
                "MAKOSH_DEV_KEY_PATH",
                vault_dir
                    .path()
                    .join("dev")
                    .join("master.key")
                    .to_str()
                    .expect("dev key path"),
            ),
            ("MAKOSH_TELEGRAM_API_ID", "12345"),
            ("MAKOSH_TELEGRAM_API_HASH", "telegram-api-hash"),
        ])
        .expect("config"),
        database,
    );

    let response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/accounts",
            json!({
                "account_id": account_id,
                "provider_kind": "telegram_user",
                "display_name": "@inferred_qr",
                "external_account_id": format!("telegram:{suffix}"),
                "tdlib_data_path": tdlib_data_path,
                "transcription_enabled": false
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["runtime"], json!("tdlib_qr_authorized"));
    assert_eq!(body["credential_bindings"], json!([]));

    let config: Value = sqlx::query_scalar(
        "SELECT config FROM communication_provider_accounts WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("provider account config");
    assert_eq!(config["runtime"], json!("tdlib_qr_authorized"));
    assert_eq!(config["tdlib_data_path"], json!(tdlib_data_path));
    assert!(config.get("api_hash").is_none());

    let binding_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM communication_provider_account_secret_refs WHERE account_id = $1",
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("binding count");
    assert_eq!(binding_count, 0);
}
#[tokio::test]
async fn telegram_live_account_setup_api_requires_configured_database() {
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN),
        Database::disabled(),
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/accounts",
            json!({
                "account_id": "telegram-no-db",
                "provider_kind": "telegram_bot",
                "display_name": "Telegram No DB",
                "external_account_id": "@telegram_no_db",
                "bot_token": "123456:telegram-bot-token"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("account response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("database_not_configured"));
}

#[tokio::test]
async fn telegram_capabilities_report_qr_login_readiness_inputs() {
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN)
            .with_test_pairs([
                ("MAKOSH_DEV_MODE", "true"),
                (
                    "MAKOSH_TDJSON_PATH",
                    "/tmp/makosh-test-missing-libtdjson.dylib",
                ),
                ("MAKOSH_TELEGRAM_API_ID", "12345"),
                ("MAKOSH_TELEGRAM_API_HASH", "telegram-api-hash"),
            ])
            .expect("config"),
        Database::disabled(),
    );

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/integrations/telegram/capabilities",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("capabilities response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["telegram_app_credentials_configured"], json!(true));
    assert_eq!(body["tdjson_runtime_available"], json!(false));
    assert_eq!(body["qr_login_ready"], json!(false));
    assert_capability_status(&body, "tdlib_live_runtime", "blocked", true);
}

#[tokio::test]
async fn telegram_account_capabilities_report_account_scope_and_bot_overrides() {
    let ctx = TestContext::new().await;
    let database_url = ctx.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let suffix = unique_suffix();
    let user_account_id = format!("telegram-cap-user-{suffix}");
    let bot_account_id = format!("telegram-cap-bot-{suffix}");
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
            "account_id": user_account_id,
            "provider_kind": "telegram_user",
            "display_name": "Telegram Capability User",
            "external_account_id": format!("tg-cap-user-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/cap-user-{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;
    assert_ok(
        app.clone(),
        "/api/v1/integrations/telegram/fixtures/accounts",
        json!({
            "account_id": bot_account_id,
            "provider_kind": "telegram_bot",
            "display_name": "Telegram Capability Bot",
            "external_account_id": format!("tg-cap-bot-{suffix}"),
            "tdlib_data_path": format!("docker/data/telegram/cap-bot-{suffix}"),
            "transcription_enabled": false
        }),
    )
    .await;

    let user_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/telegram/accounts/{user_account_id}/capabilities"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("user account capabilities response");
    assert_eq!(user_response.status(), StatusCode::OK);
    let user_body = json_body(user_response).await;
    assert_eq!(
        user_body["account_scope"]["account_id"],
        json!(user_account_id)
    );
    assert_eq!(
        user_body["account_scope"]["provider_kind"],
        json!("telegram_user")
    );
    assert_eq!(user_body["account_scope"]["runtime_kind"], json!("fixture"));
    assert_eq!(
        user_body["account_scope"]["lifecycle_state"],
        json!("active")
    );
    assert_eq!(user_body["runtime_mode"], json!("fixture"));
    assert_capability_status(&user_body, "messages.send_text", "degraded", true);
    assert_capability_status(&user_body, "messages.edit", "degraded", true);
    assert_capability_status(&user_body, "messages.delete", "degraded", true);
    assert_capability_status(
        &user_body,
        "messages.restore_visibility",
        "available",
        false,
    );
    assert_capability_status(&user_body, "messages.mark_read", "blocked", true);
    assert_capability_status(&user_body, "messages.pin", "degraded", true);
    assert_capability_status(&user_body, "reactions.add", "degraded", true);
    assert_capability_status(&user_body, "reactions.remove", "degraded", true);
    assert_capability_status(&user_body, "runtime.stop", "available", false);
    assert_capability_status(&user_body, "runtime.restart", "available", false);
    assert_capability_status(&user_body, "dialogs.mark_read", "unsupported", false);
    assert_capability_status(&user_body, "dialogs.mark_unread", "unsupported", false);
    assert_capability_status(&user_body, "dialogs.folder_reassign", "blocked", false);
    assert_capability_status(&user_body, "topics.list", "degraded", false);
    assert_capability_status(&user_body, "topics.create", "blocked", true);
    assert_capability_status(&user_body, "topics.close", "blocked", true);
    for operation in [
        "dialogs.pin",
        "dialogs.archive",
        "dialogs.mute",
        "dialogs.mark_read",
        "dialogs.mark_unread",
    ] {
        let capability = user_body["capabilities"]
            .as_array()
            .expect("capabilities")
            .iter()
            .find(|item| item["operation"] == operation)
            .expect("dialog provider-write capability");
        assert_eq!(capability["action_class"], json!("provider_write"));
    }
    let dialog_mark_read = user_body["capabilities"]
        .as_array()
        .expect("capabilities")
        .iter()
        .find(|item| item["operation"] == "dialogs.mark_read")
        .expect("dialogs.mark_read capability");
    assert_eq!(dialog_mark_read["action_class"], json!("provider_write"));

    let bot_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/integrations/telegram/accounts/{bot_account_id}/capabilities"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("bot account capabilities response");
    assert_eq!(bot_response.status(), StatusCode::OK);
    let bot_body = json_body(bot_response).await;
    assert_eq!(
        bot_body["account_scope"]["account_id"],
        json!(bot_account_id)
    );
    assert_eq!(
        bot_body["account_scope"]["provider_kind"],
        json!("telegram_bot")
    );
    assert_capability_status(&bot_body, "runtime.tdlib_live", "unsupported", true);
    assert_capability_status(&bot_body, "auth.qr_start", "unsupported", true);
    assert_capability_status(&bot_body, "topics.list", "unsupported", false);
    assert_capability_status(&bot_body, "topics.create", "unsupported", true);
    assert_capability_status(&bot_body, "topics.close", "unsupported", true);
}
