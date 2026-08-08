use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) use axum::Router;
pub(crate) use axum::body::{Body, to_bytes};
pub(crate) use axum::http::{Method, Request, StatusCode, header};
pub(crate) use makosh_hub_backend::app::router::{build_router, build_router_with_database};
pub(crate) use makosh_hub_backend::platform::config::app_config::AppConfig;
pub(crate) use makosh_hub_backend::platform::storage::database::Database;
pub(crate) use serde_json::{Value, json};
pub(crate) use tower::ServiceExt;

pub(crate) const LOCAL_API_TOKEN: &str = "tasks-api-test-token";

pub(crate) fn config_with_api_token() -> AppConfig {
    makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN)
}

pub(crate) async fn test_database_url(test_name: &str) -> Option<String> {
    let _ = test_name;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    Some(database_url)
}

pub(crate) fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

pub(crate) fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub(crate) fn post_request_with_token(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub(crate) fn put_request_with_token(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub(crate) fn delete_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub(crate) async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

pub(crate) fn urlencoding_percent_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

pub(crate) fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
}

pub(crate) async fn build_tasks_app(database_url: &str) -> Router {
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

pub(crate) async fn create_task(app: &Router, suffix: u128) -> Option<String> {
    let response = app
        .clone()
        .oneshot(post_request_with_token(
            "/api/v1/tasks",
            json!({
                "title": format!("API Task {suffix}"),
                "description": "Task for API testing",
                "status": "active",
                "priority": "medium",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    if response.status().is_server_error() {
        return None;
    }
    json_body(response).await["task_id"]
        .as_str()
        .map(|value| value.to_owned())
}
