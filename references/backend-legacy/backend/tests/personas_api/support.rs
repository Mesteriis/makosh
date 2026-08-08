use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, header};
use serde_json::Value;

use makosh_events_postgres::consumers::EventConsumerConfig;
use makosh_events_postgres::consumers::EventConsumerRunner;
use makosh_hub_backend::app::router::{build_router, build_router_with_database};
use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::workflows::persona_derived_evidence::{
    PERSONA_DERIVED_EVIDENCE_CONSUMER, project_persona_derived_evidence_event,
};
use sqlx::postgres::PgPool;

pub const LOCAL_API_TOKEN: &str = "persons-api-test-token";

pub fn config_with_api_token() -> AppConfig {
    app_config_with_pairs(Vec::new())
}

pub fn app_config_with_pairs(mut extra_pairs: Vec<(&'static str, String)>) -> AppConfig {
    let suffix = unique_suffix();
    let vault_home = format!("/tmp/makosh-persons-api-vault-{suffix}");
    let dev_key_path = format!("{vault_home}/dev.key");
    makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN)
        .with_test_dev_vault_paths(vault_home, dev_key_path)
        .with_test_pairs(extra_pairs.drain(..))
        .expect("valid local API config")
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

pub async fn build_personas_app(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_personas_app_with_database(database_url, database)
}

pub fn build_personas_app_with_database(database_url: &str, database: Database) -> axum::Router {
    build_router_with_database(
        app_config_with_pairs(Vec::new()).with_test_database_url(database_url),
        database,
    )
}

pub fn build_personas_app_without_database() -> axum::Router {
    build_router(config_with_api_token())
}

pub async fn live_database_url(test_name: &str) -> Option<String> {
    let _ = test_name;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    // Keep the context alive for tests that return only a URL. Without this,
    // an owned PostgreSQL testcontainer can be dropped before the router opens
    // its pool.
    Box::leak(Box::new(test_context));
    Some(database_url)
}

pub async fn run_persona_derived_evidence_consumer(pool: PgPool) {
    let runner = EventConsumerRunner::new(
        pool.clone(),
        EventConsumerConfig::new(PERSONA_DERIVED_EVIDENCE_CONSUMER),
    );
    runner
        .process_next_batch(|event| project_persona_derived_evidence_event(pool.clone(), event))
        .await
        .expect("persona derived evidence consumer");
}
