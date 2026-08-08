use makosh_backend_testkit::{self, context::TestContext};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;
use tower::ServiceExt;

use makosh_hub_backend::app::router::{build_router, build_router_with_database};
use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::storage::database::Database;

const LOCAL_API_TOKEN: &str = "events-api-test-token";

#[tokio::test]
async fn post_event_rejects_when_local_api_secret_is_not_configured() {
    let app = build_router(
        makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN)
            .with_test_pairs([("MAKOSH_DEV_MODE", "true")])
            .expect("app config"),
    );

    let response = app
        .oneshot(json_request(
            "/api/v1/events",
            json!({
                "event_id": "evt_api_no_db",
                "event_type": "system_api_test_event",
                "occurred_at": Utc::now(),
                "source": {"kind": "test", "source_id": "evt_api_no_db"},
                "subject": {"kind": "system", "entity_id": "backend"}
            }),
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn post_event_rejects_missing_local_api_secret() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(json_request(
            "/api/v1/events",
            json!({
                "event_id": "evt_api_missing_token",
                "event_type": "system_api_test_event",
                "occurred_at": Utc::now(),
                "source": {"kind": "test", "source_id": "evt_api_missing_token"},
                "subject": {"kind": "system", "entity_id": "backend"}
            }),
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn post_event_rejects_invalid_local_api_secret() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(json_request_with_token(
            "/api/v1/events",
            json!({
                "event_id": "evt_api_invalid_token",
                "event_type": "system_api_test_event",
                "occurred_at": Utc::now(),
                "source": {"kind": "test", "source_id": "evt_api_invalid_token"},
                "subject": {"kind": "system", "entity_id": "backend"}
            }),
            "wrong-token",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn post_event_accepts_secret_without_actor_header_before_database_access() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(json_request_with_token_without_actor(
            "/api/v1/events",
            json!({
                "event_id": "evt_api_missing_actor",
                "event_type": "system_api_test_event",
                "occurred_at": Utc::now(),
                "source": {"kind": "test", "source_id": "evt_api_missing_actor"},
                "subject": {"kind": "system", "entity_id": "backend"}
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = json_body(response).await;
    assert_eq!(body["error"], json!("database_not_configured"));
    assert!(body["message"].is_string());
}

#[tokio::test]
async fn get_event_ignores_actor_header_before_database_access() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token_and_actor(
            "/api/v1/events/evt_api_invalid_actor",
            LOCAL_API_TOKEN,
            "invalid actor",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = json_body(response).await;
    assert_eq!(body["error"], json!("database_not_configured"));
    assert!(body["message"].is_string());
}

#[tokio::test]
async fn get_event_rejects_missing_local_api_secret_before_database_access() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v1/events/evt_api_missing_token"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn get_event_rejects_invalid_local_api_secret_before_database_access() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/events/evt_api_invalid_token",
            "wrong-token",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn get_audit_events_rejects_missing_local_api_secret_before_database_access() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request(
            "/api/v1/audit/events?target_id=evt_api_audit_missing_token",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn post_event_returns_service_unavailable_when_database_is_not_configured() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(json_request_with_token(
            "/api/v1/events",
            json!({
                "event_id": "evt_api_no_db",
                "event_type": "system_api_test_event",
                "occurred_at": Utc::now(),
                "source": {"kind": "test", "source_id": "evt_api_no_db"},
                "subject": {"kind": "system", "entity_id": "backend"}
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = json_body(response).await;
    assert_eq!(body["error"], json!("database_not_configured"));
    assert!(body["message"].is_string());
}

#[tokio::test]
async fn post_event_rejects_invalid_envelope() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let app = app_with_database(&database_url).await;

    let response = app
        .oneshot(json_request_with_token(
            "/api/v1/events",
            json!({
                "event_id": "evt_api_invalid",
                "event_type": " ",
                "occurred_at": Utc::now(),
                "source": {"kind": "test", "source_id": "evt_api_invalid"},
                "subject": {"kind": "system", "entity_id": "backend"}
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_event_envelope",
            "message": "event_type must not be empty"
        })
    );
}

#[tokio::test]
async fn post_then_get_event_round_trips_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let (app, pool) = app_and_pool_with_database(&database_url).await;
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let event_id = format!("evt_api_round_trip_{suffix}");
    let occurred_at = Utc::now();

    let create_response = app
        .clone()
        .oneshot(json_request_with_token(
            "/api/v1/events",
            json!({
                "event_id": event_id,
                "event_type": "system_api_test_event",
                "occurred_at": occurred_at,
                "source": {
                    "kind": "test",
                    "provider": "integration",
                    "source_id": event_id
                },
                "subject": {"kind": "system", "entity_id": "backend"},
                "payload": {"api": true},
                "provenance": {"confidence": 1.0},
                "correlation_id": "corr_events_api_test"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("create response");

    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body = json_body(create_response).await;
    assert_eq!(create_body["event_id"], event_id);
    assert!(create_body["position"].as_i64().expect("position") > 0);

    let get_response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/events/{event_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("get response");

    assert_eq!(get_response.status(), StatusCode::OK);

    let get_body = json_body(get_response).await;
    assert_eq!(get_body["event_id"], event_id);
    assert_eq!(get_body["event_type"], "system_api_test_event");
    assert_eq!(get_body["payload"], json!({"api": true}));
    assert_eq!(get_body["provenance"], json!({"confidence": 1.0}));

    let audit_operations = audit_operations_for_target(&pool, &event_id).await;
    assert_eq!(
        audit_operations,
        vec!["event.append".to_owned(), "event.get".to_owned()]
    );

    let mutation =
        sqlx::query("UPDATE api_audit_log SET metadata = '{}'::jsonb WHERE target_id = $1")
            .bind(&event_id)
            .execute(&pool)
            .await;
    assert!(mutation.is_err(), "api_audit_log must be append-only");
}

#[tokio::test]
async fn get_event_returns_not_found_for_missing_event_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let (app, pool) = app_and_pool_with_database(&database_url).await;
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let event_id = format!("evt_api_missing_{suffix}");

    let response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/events/{event_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let audit_operations = audit_operations_for_target(&pool, &event_id).await;
    assert_eq!(audit_operations, vec!["event.get".to_owned()]);
}

#[tokio::test]
async fn get_audit_events_returns_records_without_self_auditing_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let (app, pool) = app_and_pool_with_database(&database_url).await;
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let event_id = format!("evt_api_audit_query_{suffix}");
    let occurred_at = Utc::now();

    let create_response = app
        .clone()
        .oneshot(json_request_with_token(
            "/api/v1/events",
            json!({
                "event_id": event_id,
                "event_type": "system_api_test_event",
                "occurred_at": occurred_at,
                "source": {
                    "kind": "test",
                    "provider": "integration",
                    "source_id": event_id
                },
                "subject": {"kind": "system", "entity_id": "backend"}
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("create response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let get_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/events/{event_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("get response");
    assert_eq!(get_response.status(), StatusCode::OK);

    let audit_count_before = audit_record_count(&pool).await;

    let audit_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/audit/events?target_id={event_id}&limit=10"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("audit response");
    assert_eq!(audit_response.status(), StatusCode::OK);

    let audit_body = json_body(audit_response).await;
    let items = audit_body["items"].as_array().expect("audit items");
    let operations = items
        .iter()
        .map(|item| item["operation"].as_str().expect("operation").to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        operations,
        vec!["event.append".to_owned(), "event.get".to_owned()]
    );

    for item in items {
        assert!(item["audit_id"].as_i64().expect("audit_id") > 0);
        assert_eq!(item["actor_kind"], "frontend");
        assert_eq!(item["actor_id"], "makosh-frontend");
        assert_eq!(item["target_kind"], "event");
        assert_eq!(item["target_id"], event_id);
        assert!(
            item["recorded_at"]
                .as_str()
                .expect("recorded_at")
                .contains('T')
        );
        assert_eq!(item["metadata"], json!({}));
    }

    let first_page_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/audit/events?target_id={event_id}&limit=1"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("first audit page response");
    assert_eq!(first_page_response.status(), StatusCode::OK);
    let first_page_body = json_body(first_page_response).await;
    let first_page_items = first_page_body["items"]
        .as_array()
        .expect("first page audit items");
    assert_eq!(first_page_items.len(), 1);
    assert_eq!(first_page_items[0]["operation"], "event.append");
    let first_page_audit_id = first_page_items[0]["audit_id"].as_i64().expect("audit_id");

    let second_page_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/audit/events?target_id={event_id}&after_audit_id={first_page_audit_id}&limit=1"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("second audit page response");
    assert_eq!(second_page_response.status(), StatusCode::OK);
    let second_page_body = json_body(second_page_response).await;
    let second_page_items = second_page_body["items"]
        .as_array()
        .expect("second page audit items");
    assert_eq!(second_page_items.len(), 1);
    assert_eq!(second_page_items[0]["operation"], "event.get");
    assert!(second_page_items[0]["audit_id"].as_i64().expect("audit_id") > first_page_audit_id);

    let actor_filtered_response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/audit/events?target_id={event_id}&actor_id=makosh-frontend"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("actor-filtered audit response");
    assert_eq!(actor_filtered_response.status(), StatusCode::OK);
    let actor_filtered_body = json_body(actor_filtered_response).await;
    assert_eq!(
        actor_filtered_body["items"]
            .as_array()
            .expect("actor-filtered audit items")
            .len(),
        2
    );

    assert_eq!(audit_record_count(&pool).await, audit_count_before);
}

async fn app_with_database(database_url: &str) -> axum::Router {
    app_and_pool_with_database(database_url).await.0
}

async fn app_and_pool_with_database(database_url: &str) -> (axum::Router, PgPool) {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    (
        build_router_with_database(config_with_api_token(), database),
        pool,
    )
}

fn config_with_api_token() -> AppConfig {
    makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN)
}

fn json_request(uri: &str, value: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(value.to_string()))
        .expect("request")
}

fn json_request_with_token(uri: &str, value: serde_json::Value, token: &str) -> Request<Body> {
    json_request_with_token_and_actor(uri, value, token, "makosh-frontend")
}

fn json_request_with_token_without_actor(
    uri: &str,
    value: serde_json::Value,
    token: &str,
) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(value.to_string()))
        .expect("request")
}

fn json_request_with_token_and_actor(
    uri: &str,
    value: serde_json::Value,
    token: &str,
    _actor_id: &str,
) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(value.to_string()))
        .expect("request")
}

fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    get_request_with_token_and_actor(uri, token, "makosh-frontend")
}

fn get_request_with_token_and_actor(uri: &str, token: &str, _actor_id: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 4096)
        .await
        .expect("body bytes");

    serde_json::from_slice(&body).expect("json body")
}

async fn audit_operations_for_target(pool: &PgPool, target_id: &str) -> Vec<String> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT operation
        FROM api_audit_log
        WHERE target_kind = 'event'
          AND target_id = $1
        ORDER BY audit_id ASC
        "#,
    )
    .bind(target_id)
    .fetch_all(pool)
    .await
    .expect("audit operations")
}

async fn audit_record_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT count(*) FROM api_audit_log")
        .fetch_one(pool)
        .await
        .expect("audit record count")
}
