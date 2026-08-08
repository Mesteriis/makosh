use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::storage::database::Database;

const LOCAL_API_TOKEN: &str = "events-long-poll-test-token";

#[tokio::test]
async fn get_events_lists_replay_batch_and_audits_access_against_postgres() {
    let context = TestContext::new().await;
    let (app, pool) = app_and_pool_with_database(&context.connection_string()).await;
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let event_id = format!("evt_api_list_{suffix}");

    let create_response = app
        .clone()
        .oneshot(json_request_with_token(
            "/api/v1/events",
            json!({
                "event_id": event_id,
                "event_type": "system_api_test_event",
                "occurred_at": Utc::now(),
                "source": {
                    "kind": "test",
                    "provider": "integration",
                    "source_id": event_id
                },
                "subject": {"kind": "system", "entity_id": "backend"},
                "payload": {"list": true}
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("create response");
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_body = json_body(create_response).await;
    let position = create_body["position"].as_i64().expect("position");

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/events?after_position={}&limit=10&wait_seconds=0",
                position - 1
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("list response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["position"], position);
    assert_eq!(items[0]["event"]["event_id"], event_id);
    assert_eq!(items[0]["event"]["payload"], json!({"list": true}));
    assert_eq!(body["next_after_position"], position);
    assert_eq!(body["has_more"], false);

    let audit = latest_event_list_audit_record(&pool).await;
    assert_eq!(audit["operation"], "event.list");
    assert_eq!(audit["method"], "GET");
    assert_eq!(audit["target_kind"], "event");
    assert!(audit["target_id"].is_null());
    assert_eq!(audit["metadata"]["after_position"], position - 1);
    assert_eq!(audit["metadata"]["limit"], 10);
    assert_eq!(audit["metadata"]["wait_seconds"], 0);
}

async fn app_and_pool_with_database(database_url: &str) -> (axum::Router, sqlx::PgPool) {
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

fn json_request_with_token(uri: &str, value: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(value.to_string()))
        .expect("request")
}

fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), 4096)
        .await
        .expect("body bytes");

    serde_json::from_slice(&body).expect("json body")
}

async fn latest_event_list_audit_record(pool: &sqlx::PgPool) -> Value {
    let row = sqlx::query(
        r#"
        SELECT operation, method, target_kind, target_id, metadata
        FROM api_audit_log
        WHERE operation = 'event.list'
        ORDER BY audit_id DESC
        LIMIT 1
        "#,
    )
    .fetch_one(pool)
    .await
    .expect("event list audit record");

    json!({
        "operation": row.try_get::<String, _>("operation").expect("operation"),
        "method": row.try_get::<String, _>("method").expect("method"),
        "target_kind": row.try_get::<String, _>("target_kind").expect("target_kind"),
        "target_id": row.try_get::<Option<String>, _>("target_id").expect("target_id"),
        "metadata": row.try_get::<Value, _>("metadata").expect("metadata")
    })
}
