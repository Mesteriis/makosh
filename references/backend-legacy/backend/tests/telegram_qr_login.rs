mod telegram_support;

use std::env;

use axum::http::StatusCode;
use makosh_backend_testkit::context::TestContext;
use serde_json::json;
use tower::ServiceExt;

use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::storage::database::Database;
use telegram_support::{
    LOCAL_API_TOKEN, delete_request_with_token, get_request_with_token, json_body,
    json_post_request_with_actor,
};

#[tokio::test]
async fn telegram_qr_login_start_reports_tdlib_runtime_unavailable() {
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN)
            .with_test_tdjson_path("/tmp/makosh-test-missing-libtdjson.dylib"),
        Database::disabled(),
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/login/qr/start",
            json!({
                "account_id": "telegram-qr",
                "display_name": "Telegram QR",
                "external_account_id": "qr-login:telegram-qr",
                "api_id": 12345,
                "api_hash": "telegram-api-hash",
                "session_encryption_key": "telegram-session-key",
                "tdlib_data_path": "docker/data/telegram/telegram-qr",
                "transcription_enabled": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("QR login response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("telegram_tdlib_runtime_unavailable"));
}

#[tokio::test]
async fn telegram_qr_login_start_uses_configured_app_credentials_when_payload_omits_them() {
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN)
            .with_test_tdjson_path("/tmp/makosh-test-missing-libtdjson.dylib")
            .with_test_telegram_app_credentials(12345, "telegram-api-hash"),
        Database::disabled(),
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/login/qr/start",
            json!({
                "account_id": "telegram-qr-configured",
                "display_name": "Telegram QR Configured",
                "external_account_id": "qr-login:telegram-qr-configured",
                "session_encryption_key": "telegram-session-key",
                "tdlib_data_path": "docker/data/telegram/telegram-qr-configured",
                "transcription_enabled": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("QR login response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("telegram_tdlib_runtime_unavailable"));
}

#[tokio::test]
async fn telegram_live_smoke_syncs_configured_account_when_explicitly_enabled() {
    if env::var("MAKOSH_TELEGRAM_LIVE_SMOKE").ok().as_deref() != Some("1") {
        eprintln!("skipping live Telegram TDLib smoke test: MAKOSH_TELEGRAM_LIVE_SMOKE is not 1");
        return;
    }

    let account_id = env::var("MAKOSH_TELEGRAM_LIVE_ACCOUNT_ID")
        .expect("MAKOSH_TELEGRAM_LIVE_ACCOUNT_ID must be set");
    let provider_chat_id =
        env::var("MAKOSH_TELEGRAM_LIVE_CHAT_ID").expect("MAKOSH_TELEGRAM_LIVE_CHAT_ID must be set");
    let tdjson_path = env::var("MAKOSH_TDJSON_PATH").expect("MAKOSH_TDJSON_PATH must be set");
    let telegram_api_id = env::var("MAKOSH_TELEGRAM_API_ID")
        .expect("MAKOSH_TELEGRAM_API_ID must be set")
        .parse::<i64>()
        .expect("MAKOSH_TELEGRAM_API_ID must be a positive integer");
    let telegram_api_hash =
        env::var("MAKOSH_TELEGRAM_API_HASH").expect("MAKOSH_TELEGRAM_API_HASH must be set");
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = test_context.database();
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url,
        )
        .with_test_tdjson_path(tdjson_path)
        .with_test_telegram_app_credentials(telegram_api_id, telegram_api_hash),
        database,
    );

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
    assert_eq!(start_body["runtime_kind"], json!("tdlib_qr_authorized"));
    assert_eq!(start_body["status"], json!("running"));

    let history_response = app
        .clone()
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/provider-sync/history",
            json!({
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
                "limit": 25
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("history sync response");
    assert_eq!(history_response.status(), StatusCode::OK);
    let history_body = json_body(history_response).await;
    assert_eq!(history_body["account_id"], json!(account_id));
    assert_eq!(history_body["provider_chat_id"], json!(provider_chat_id));
    assert_eq!(history_body["runtime_kind"], json!("tdlib_qr_authorized"));
    assert_eq!(history_body["status"], json!("synced"));
}

#[tokio::test]
async fn telegram_qr_login_status_unknown_setup_returns_json_not_found() {
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN),
        Database::disabled(),
    );

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/integrations/telegram/login/qr/missing-setup",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("QR status response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("telegram_qr_login_not_found"));
}

#[tokio::test]
async fn telegram_qr_login_password_unknown_setup_returns_json_not_found() {
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN),
        Database::disabled(),
    );

    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/login/qr/missing-setup/password",
            json!({ "password": "test-password" }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("QR password response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("telegram_qr_login_not_found"));
}

#[tokio::test]
async fn telegram_qr_login_cancel_unknown_setup_returns_json_not_found() {
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN),
        Database::disabled(),
    );

    let response = app
        .oneshot(delete_request_with_token(
            "/api/v1/integrations/telegram/login/qr/missing-setup",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("QR cancel response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("telegram_qr_login_not_found"));
}
