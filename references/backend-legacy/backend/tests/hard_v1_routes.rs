use axum::body::{Body, to_bytes};
use axum::http::{HeaderValue, Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::router::build_router;
use makosh_hub_backend::platform::config::app_config::AppConfig;

const LOCAL_API_SECRET: &str = "hard-v1-routes-test-secret";

#[tokio::test]
async fn former_versioned_routes_are_not_public_aliases() {
    let app = build_router(config_with_api_secret());

    for path in [
        "/api/v2/tasks",
        "/api/v3/ai/status",
        "/api/v4/capabilities",
        "/api/v5/capabilities",
    ] {
        let response = app
            .clone()
            .oneshot(get_request_with_secret(path))
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
    }
}

#[tokio::test]
async fn telegram_and_whatsapp_capabilities_are_split_under_v1() {
    let app = build_router(config_with_api_secret());

    let telegram = app
        .clone()
        .oneshot(get_request_with_secret(
            "/api/v1/integrations/telegram/capabilities",
        ))
        .await
        .expect("telegram capabilities response");
    assert_eq!(telegram.status(), StatusCode::OK);
    let telegram_body = json_body(telegram).await;
    assert_eq!(telegram_body["version"], json!("2.1"));
    assert_eq!(telegram_body["runtime_mode"], json!("fixture"));
    assert!(telegram_body["planned_features"].is_array());
    assert_has_capability(&telegram_body, "runtime.fixture");

    let whatsapp = app
        .oneshot(get_request_with_secret(
            "/api/v1/integrations/whatsapp/capabilities",
        ))
        .await
        .expect("whatsapp capabilities response");
    assert_eq!(whatsapp.status(), StatusCode::OK);
    let whatsapp_body = json_body(whatsapp).await;
    assert_eq!(whatsapp_body["version"], json!("2.0"));
    assert_eq!(whatsapp_body["runtime_mode"], json!("fixture"));
    assert!(whatsapp_body["planned_features"].is_array());
    assert!(whatsapp_body["provider_shapes"].is_array());
    assert_has_capability(&whatsapp_body, "runtime.fixture");
}

fn config_with_api_secret() -> AppConfig {
    makosh_backend_testkit::app::config_with_secret(LOCAL_API_SECRET)
}

fn get_request_with_secret(path: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .header(
            "x-makosh-secret",
            HeaderValue::from_static(LOCAL_API_SECRET),
        )
        .body(Body::empty())
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    serde_json::from_slice(&body).expect("json body")
}

fn assert_has_capability(body: &Value, capability: &str) {
    let capabilities = body["capabilities"].as_array().expect("capabilities");
    assert!(
        capabilities
            .iter()
            .any(|item| item["capability"] == json!(capability)
                || item["operation"] == json!(capability)),
        "{capability}"
    );
}
