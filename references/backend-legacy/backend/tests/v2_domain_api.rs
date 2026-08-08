use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use makosh_hub_backend::app::router::{build_router, build_router_with_database};
use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::domains::tasks::api::{NewTask, TaskStore};
use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::storage::database::Database;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

const LOCAL_API_TOKEN: &str = "v2-domain-api-test-token";

#[tokio::test]
async fn domain_routes_build_and_require_local_api_secret() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v1/tasks"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        json_body(response).await,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );

    let secret_only_response = build_router(config_with_api_token())
        .oneshot(get_request_with_token("/api/v1/tasks", LOCAL_API_TOKEN))
        .await
        .expect("secret-only response");

    assert_eq!(
        secret_only_response.status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    let secret_only_body = json_body(secret_only_response).await;
    assert_eq!(secret_only_body["error"], json!("database_not_configured"));
    assert!(secret_only_body["message"].is_string());
}

#[tokio::test]
async fn tasks_endpoint_returns_first_class_task_payload_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let task = TaskStore::new(pool)
        .create(&NewTask {
            title: format!("V1 first-class task {suffix}"),
            description: Some("contract test task".to_owned()),
            source_type: Some("manual".to_owned()),
            makosh_status: Some("ready".to_owned()),
            priority_score: Some(0.7),
            tags: Some(json!(["api-test"])),
            ..Default::default()
        })
        .await
        .expect("seed task");

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let response = app
        .oneshot(get_request_with_token_and_actor(
            "/api/v1/tasks?limit=100",
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let item = body["items"]
        .as_array()
        .expect("items")
        .iter()
        .find(|item| item["task_id"] == task.task_id)
        .expect("seeded task item");

    assert_eq!(item["title"], json!(task.title));
    assert_eq!(item["source_type"], json!("observation"));
    assert_eq!(item["makosh_status"], json!("ready"));
    assert_eq!(item["confidentiality"], json!("private_local"));
    assert_eq!(item["task_metadata"], json!({}));
}

#[tokio::test]
async fn persona_health_endpoint_returns_single_persona_health_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let person = PersonaProjectionStore::new(pool.clone())
        .upsert_email_persona(&format!("health-{suffix}@example.com"))
        .await
        .expect("seed persona");
    seed_person_health(&pool, &person.persona_id).await;

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let response = app
        .oneshot(get_request_with_token_and_actor(
            &format!("/api/v1/personas/{}/health", person.persona_id),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["persona_id"], json!(person.persona_id));
    assert!(body.get("person_id").is_none());
    assert_eq!(body["health_status"], json!("at_risk"));
    assert_eq!(body["communication_gap_days"], json!(42));
    assert!(body.get("items").is_none());
}

fn config_with_api_token() -> AppConfig {
    makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN)
}

fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

fn get_request_with_token_and_actor(uri: &str, token: &str, _actor_id: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

async fn seed_person_health(pool: &PgPool, persona_id: &str) {
    sqlx::query(
        "UPDATE personas SET health_status = 'at_risk', communication_gap_days = 42, watchlist = true WHERE persona_id = $1",
    )
    .bind(persona_id)
    .execute(pool)
    .await
    .expect("update person health");
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
}
