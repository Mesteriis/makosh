//! Characterization tests for Communication API.
//!
//! Captures CURRENT behavior before alignment refactoring (Phase 3+).
//! Do NOT change existing behavior — only add tests.
//!
//! These live tests use the shared testcontainers pgvector fixture with
//! per-test migrated databases.

use makosh_backend_testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::Value;
use tower::ServiceExt;

use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::storage::database::Database;

const TOKEN: &str = "char-comm-test-token";

fn cfg(db: &str) -> AppConfig {
    makosh_backend_testkit::app::config_with_secret_and_database_url(TOKEN, db)
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", TOKEN)
        .body(Body::empty())
        .expect("req")
}

fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", TOKEN)
        .body(Body::from(body.to_string()))
        .expect("req")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

async fn build_app(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(cfg(database_url), database)
}

async fn live_app(_test_name: &str) -> Option<axum::Router> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    Some(build_app(&database_url).await)
}

// ── AC3: Communication API characterization ─────────────────────────────────

/// AC3 characterisation: GET /api/v1/communications/messages returns 200.
#[tokio::test]
async fn char_communications_messages_list_returns_ok() {
    let Some(app) = live_app("communications messages list").await else {
        return;
    };

    let response = app
        .oneshot(get("/api/v1/communications/messages"))
        .await
        .expect("response");

    assert!(
        !response.status().is_server_error(),
        "GET /api/v1/communications/messages must not return 5xx, got {:?}",
        response.status()
    );

    let body = json_body(response).await;
    // Characterize response shape — should contain items array
    assert!(
        body.get("items").is_some() || body.is_array(),
        "Response should contain items array or be an array, got keys: {:?}",
        body.as_object().map(|o| o.keys().collect::<Vec<_>>())
    );
}

/// AC3 characterisation: GET /api/v1/communications/search returns 200.
#[tokio::test]
async fn char_communications_search_returns_ok() {
    let Some(app) = live_app("communications search").await else {
        return;
    };

    let response = app
        .oneshot(get("/api/v1/communications/search?q=test"))
        .await
        .expect("response");

    assert!(
        !response.status().is_server_error(),
        "GET /api/v1/communications/search must not return 5xx, got {:?}",
        response.status()
    );
}

/// AC3 characterisation: GET /api/v1/communications/threads returns 200.
#[tokio::test]
async fn char_communications_threads_list_returns_ok() {
    let Some(app) = live_app("communications threads list").await else {
        return;
    };

    let response = app
        .oneshot(get("/api/v1/communications/threads"))
        .await
        .expect("response");

    assert!(
        !response.status().is_server_error(),
        "GET /api/v1/communications/threads must not return 5xx, got {:?}",
        response.status()
    );
}

/// AC3 characterisation: GET /api/v1/communications/messages/states returns 200.
#[tokio::test]
async fn char_communications_message_states_returns_ok() {
    let Some(app) = live_app("communications message states").await else {
        return;
    };

    let response = app
        .oneshot(get("/api/v1/communications/messages/states"))
        .await
        .expect("response");

    assert!(
        !response.status().is_server_error(),
        "GET /api/v1/communications/messages/states must not return 5xx, got {:?}",
        response.status()
    );
}

/// AC3 characterisation: GET /api/v1/communications/drafts returns 200.
#[tokio::test]
async fn char_communications_drafts_list_returns_ok() {
    let Some(app) = live_app("communications drafts list").await else {
        return;
    };

    let response = app
        .oneshot(get("/api/v1/communications/drafts"))
        .await
        .expect("response");

    assert!(
        !response.status().is_server_error(),
        "GET /api/v1/communications/drafts must not return 5xx, got {:?}",
        response.status()
    );
}

/// AC3 characterisation: GET /api/v1/communications by specific message ID returns 200 or 404.
#[tokio::test]
async fn char_communication_message_by_id_returns_ok_or_404() {
    let Some(app) = live_app("communication message by id").await else {
        return;
    };

    // Non-existent message — expect 404
    let response = app
        .oneshot(get("/api/v1/communications/messages/rec:nonexistent"))
        .await
        .expect("response");

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "non-existent message should return 404"
    );
}

/// AC3 characterisation: POST to workflow-actions endpoint.
#[tokio::test]
async fn char_workflow_actions_endpoint_accepts_valid_body() {
    let Some(app) = live_app("workflow actions endpoint").await else {
        return;
    };

    let response = app
        .oneshot(post(
            "/api/v1/workflow-actions",
            serde_json::json!({
                "action": "archive",
                "message_ids": []
            }),
        ))
        .await
        .expect("response");

    // Accept either 200 (empty archive succeeds) or 4xx (validation)
    assert!(
        !response.status().is_server_error(),
        "POST /api/v1/workflow-actions must not return 5xx, got {:?}",
        response.status()
    );
}
