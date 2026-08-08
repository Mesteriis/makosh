//! Characterization tests for the Persona-native API.
//!
//! Captures the retirement of legacy `/api/v1/persons/*` routes and the
//! current Persona-native `/api/v1/personas/*` route contract.
//!
//! These live tests use the shared testcontainers pgvector fixture with
//! per-test migrated databases.

use makosh_backend_testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::storage::database::Database;

const TOKEN: &str = "char-persona-test-token";

struct PersonaNativeTestApp {
    _test_context: TestContext,
    router: axum::Router,
}

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

fn put(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
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

async fn live_app(_test_name: &str) -> Option<PersonaNativeTestApp> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let router = build_app(&database_url).await;

    Some(PersonaNativeTestApp {
        _test_context: test_context,
        router,
    })
}

// ── AC2: Persona-native API characterization ────────────────────────────────

#[tokio::test]
async fn char_legacy_persons_list_is_retired() {
    let Some(app) = live_app("legacy persons list").await else {
        return;
    };

    let response = app
        .router
        .oneshot(get("/api/v1/persons"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn char_personas_list_returns_ok() {
    let Some(app) = live_app("personas list").await else {
        return;
    };

    let response = app
        .router
        .oneshot(get("/api/v1/personas"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    assert!(
        body.get("items").is_some(),
        "GET /api/v1/personas must return 'items' array"
    );
}

#[tokio::test]
async fn char_personas_exist_without_persons_compatibility_routes() {
    let Some(app) = live_app("personas native routes").await else {
        return;
    };

    let personas_resp = app
        .router
        .clone()
        .oneshot(get("/api/v1/personas"))
        .await
        .expect("personas response");
    assert_eq!(personas_resp.status(), StatusCode::OK);

    let persons_resp = app
        .router
        .clone()
        .oneshot(get("/api/v1/persons"))
        .await
        .expect("persons response");
    assert_eq!(persons_resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn char_owner_persona_returns_ok() {
    let Some(app) = live_app("owner persona").await else {
        return;
    };

    let response = app
        .router
        .oneshot(get("/api/v1/personas/owner"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let owner = body
        .get("owner_persona")
        .expect("Owner response should contain owner_persona envelope");
    assert!(
        owner.is_null()
            || (owner.get("persona_id").is_some()
                && owner.get("person_id").is_none()
                && owner.get("is_self").is_some()),
        "Owner persona should be null or contain persona-native fields: {body:?}",
    );
}

#[tokio::test]
async fn char_persona_search_requires_query() {
    let Some(app) = live_app("persona search query validation").await else {
        return;
    };

    let response = app
        .router
        .oneshot(get("/api/v1/personas/search"))
        .await
        .expect("response");
    // Expect 400 for empty search
    assert!(
        response.status().is_client_error(),
        "search without query should return 4xx, got {:?}",
        response.status()
    );
}

#[tokio::test]
async fn char_legacy_person_by_id_is_retired() {
    let Some(app) = live_app("legacy person by id").await else {
        return;
    };

    let response = app
        .router
        .oneshot(get("/api/v1/persons/person:nonexistent"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn char_persona_update_accepts_valid_body() {
    let Some(app) = live_app("persona update").await else {
        return;
    };

    // Non-existent persona — expect 404 or appropriate error
    let response = app
        .router
        .oneshot(put(
            "/api/v1/personas/persona:nonexistent",
            json!({"name": "Updated Name"}),
        ))
        .await
        .expect("response");
    assert!(
        response.status().is_client_error(),
        "updating non-existent persona should return 4xx, got {:?}",
        response.status()
    );
}
