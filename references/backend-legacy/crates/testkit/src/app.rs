use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, header};
use makosh_hub_backend::platform::config::app_config::AppConfig;
use serde_json::Value;

use crate::context::TestContext;
use crate::vault;

pub const TEST_API_SECRET: &str = "makosh-test-api-secret";

pub struct TestApp {
    router: Router,
    context: TestContext,
}

impl TestApp {
    pub fn new(context: TestContext, router: Router) -> Self {
        Self { context, router }
    }

    pub fn context(&self) -> &TestContext {
        &self.context
    }

    pub fn router(&self) -> &Router {
        &self.router
    }

    pub fn into_router(self) -> Router {
        self.router
    }

    pub fn clone_router(&self) -> Router {
        self.router.clone()
    }
}

pub fn config() -> AppConfig {
    vault::retain_test_vault_and_apply(AppConfig::test_with_api_secret(TEST_API_SECRET))
}

pub fn config_with_database_url(database_url: impl Into<String>) -> AppConfig {
    vault::retain_test_vault_and_apply(AppConfig::test_with_api_secret_and_database_url(
        TEST_API_SECRET,
        database_url,
    ))
}

pub fn config_with_secret(api_secret: impl Into<String>) -> AppConfig {
    vault::retain_test_vault_and_apply(AppConfig::test_with_api_secret(api_secret))
}

pub fn config_with_secret_and_database_url(
    api_secret: impl Into<String>,
    database_url: impl Into<String>,
) -> AppConfig {
    vault::retain_test_vault_and_apply(AppConfig::test_with_api_secret_and_database_url(
        api_secret,
        database_url,
    ))
}

pub fn empty_request(method: Method, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("x-makosh-secret", TEST_API_SECRET)
        .body(Body::empty())
        .expect("request")
}

pub fn json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("x-makosh-secret", TEST_API_SECRET)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub fn get(uri: &str) -> Request<Body> {
    empty_request(Method::GET, uri)
}

pub fn post_json(uri: &str, body: Value) -> Request<Body> {
    json_request(Method::POST, uri, body)
}

pub fn put_json(uri: &str, body: Value) -> Request<Body> {
    json_request(Method::PUT, uri, body)
}

pub fn patch_json(uri: &str, body: Value) -> Request<Body> {
    json_request(Method::PATCH, uri, body)
}

pub fn delete(uri: &str) -> Request<Body> {
    empty_request(Method::DELETE, uri)
}
