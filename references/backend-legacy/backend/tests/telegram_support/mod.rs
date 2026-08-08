#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

pub const LOCAL_API_TOKEN: &str = "telegram-api-test-secret";

pub fn assert_capability_status(body: &Value, capability: &str, status: &str, closure_gate: bool) {
    let capabilities = body["capabilities"].as_array().expect("capabilities");
    let operation = match capability {
        "telegram_fixture_runtime" => "runtime.fixture",
        "automation_dry_run" => "automation.dry_run",
        "tdlib_live_runtime" => "runtime.tdlib_live",
        "automation_live_send" => "automation.live_send",
        "whisper_rs_speech_to_text" => "calls.transcription_live",
        other => other,
    };
    assert!(
        capabilities.iter().any(|item| {
            (item["capability"] == capability || item["operation"] == operation)
                && item["status"] == status
                && item["closure_gate"] == closure_gate
        }),
        "expected capability {capability}/{operation} to have status {status} and closure_gate {closure_gate}"
    );
}

pub async fn ingest_fixture_telegram_message<S>(
    app: S,
    account_id: &str,
    chat_id: &str,
    provider_message_id: &str,
    text: &str,
    occurred_at: &str,
) -> String
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(json_post_request_with_actor(
            "/api/v1/integrations/telegram/fixtures/messages",
            json!({
                "account_id": account_id,
                "provider_chat_id": chat_id,
                "provider_message_id": provider_message_id,
                "chat_kind": "private",
                "chat_title": "Pinned Message Chat",
                "sender_id": "sender-pinned",
                "sender_display_name": "Pinned Sender",
                "text": text,
                "import_batch_id": format!("telegram-pinned-seed-{provider_message_id}"),
                "occurred_at": occurred_at,
                "delivery_state": "received"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("fixture message response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    body["message_id"].as_str().expect("message id").to_owned()
}

pub async fn assert_ok<S>(app: S, path: &str, body: Value)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let response = app
        .oneshot(json_post_request_with_actor(path, body, LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

pub fn account_item<'a>(items: &'a [Value], account_id: &str) -> &'a Value {
    items
        .iter()
        .find(|item| item["account_id"] == json!(account_id))
        .unwrap_or_else(|| panic!("expected account `{account_id}` in account list"))
}

pub fn json_post_request_with_actor(path: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("x-makosh-secret", token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub fn json_post_request_with_explicit_actor_header(
    path: &str,
    body: Value,
    token: &str,
    actor_id: &str,
) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("x-makosh-secret", token)
        .header("x-makosh-actor-id", actor_id)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub fn json_put_request_with_actor(path: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(path)
        .header("x-makosh-secret", token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub fn get_request_with_token(path: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub fn delete_request_with_token(path: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(path)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub fn vault_entropy_events(count: usize) -> Vec<Value> {
    (0..count)
        .map(|index| {
            json!({
                "x": index % 997,
                "y": index % 577,
                "dx": (index % 11) as i64 - 5,
                "dy": (index % 13) as i64 - 6,
                "timestamp_ms": index * 5,
                "velocity": (index % 19) as f64 / 10.0,
                "acceleration": (index % 23) as f64 / 100.0,
                "interval_ms": 5
            })
        })
        .collect()
}

pub async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&bytes).expect("json body")
}

pub fn unique_suffix() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    format!("{now}")
}
