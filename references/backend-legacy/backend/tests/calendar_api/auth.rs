use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

use makosh_hub_backend::app::router::build_router;

use super::support::{config_with_api_token, get_request, json_body};

#[tokio::test]
async fn calendar_accounts_rejects_missing_local_api_secret() {
    let app = build_router(config_with_api_token());
    let response = app
        .oneshot(get_request("/api/v1/calendar/accounts"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({"error": "invalid_api_secret", "message": "missing or invalid x-makosh-secret header"})
    );
}

#[tokio::test]
async fn calendar_events_rejects_missing_local_api_secret() {
    let app = build_router(config_with_api_token());
    let response = app
        .oneshot(get_request("/api/v1/calendar/events"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
