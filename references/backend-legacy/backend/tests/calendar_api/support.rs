#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, header};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::storage::database::Database;

pub const LOCAL_API_TOKEN: &str = "cal-api-test-token";

pub fn config_with_api_token() -> AppConfig {
    makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN)
}

pub fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

pub fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub fn post_request_with_token(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub fn put_request_with_token(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub fn delete_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

pub fn urlencoding_percent_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

pub fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
}

pub async fn build_cal_app(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url,
        ),
        database,
    )
}

pub async fn create_cal_event(app: &axum::Router, suffix: u128) -> Option<(String, String)> {
    let response = app
        .clone()
        .oneshot(post_request_with_token(
            "/api/v1/calendar/accounts",
            json!({
                "provider": "google",
                "account_name": format!("Evt Acct {suffix}"),
                "email": format!("evt-{suffix}@example.com")
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    if response.status().is_server_error() {
        return None;
    }
    let account_id = json_body(response).await["account_id"].as_str()?.to_owned();

    let now = Utc::now();
    let start = now + Duration::hours(1);
    let end = now + Duration::hours(2);

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            "/api/v1/calendar/events",
            json!({
                "account_id": &account_id,
                "title": format!("Test Event {suffix}"),
                "start_at": start.to_rfc3339(),
                "end_at": end.to_rfc3339(),
                "status": "confirmed",
                "event_type": "meeting",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    if response.status().is_server_error() {
        return None;
    }
    let body = json_body(response).await;
    let event_id = body["event_id"].as_str()?;
    Some((account_id, event_id.to_owned()))
}
