use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use futures::StreamExt;
use serde_json::json;
use tokio::time::timeout;
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::storage::database::Database;

const LOCAL_API_TOKEN: &str = "events-stream-test-token";

#[tokio::test]
async fn event_stream_replays_event_log_positions_as_sse_against_postgres() {
    let context = TestContext::new().await;
    let app = app_with_database(&context.connection_string()).await;
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let event_id = format!("evt_api_stream_{suffix}");

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
                "subject": {"kind": "system", "entity_id": "backend"}
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
            &format!("/api/realtime/v2/events?after_position={}", position - 1),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("stream response");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.starts_with("text/event-stream")),
        Some(true)
    );

    let mut stream = response.into_body().into_data_stream();
    let chunk = timeout(Duration::from_secs(2), stream.next())
        .await
        .expect("first SSE chunk timed out")
        .expect("first SSE chunk")
        .expect("first SSE chunk bytes");
    let text = std::str::from_utf8(&chunk).expect("SSE chunk is UTF-8");

    assert!(text.contains(&format!("id: {position}")), "{text}");
    assert!(text.contains("event: event"), "{text}");
    assert!(text.contains("system_api_test_event"), "{text}");
    assert!(text.contains("correlation_id"), "{text}");
    assert!(text.contains(&event_id), "{text}");
}

#[tokio::test]
async fn event_stream_without_cursor_starts_at_current_tail_against_postgres() {
    let context = TestContext::new().await;
    let app = app_with_database(&context.connection_string()).await;
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let event_id = format!("evt_api_stream_tail_{suffix}");

    let create_response = app
        .clone()
        .oneshot(json_request_with_token(
            "/api/v1/events",
            json!({
                "event_id": event_id,
                "event_type": "system_api_tail_test_event",
                "occurred_at": Utc::now(),
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
    let create_body = json_body(create_response).await;
    let position = create_body["position"].as_i64().expect("position");

    let response = app
        .oneshot(get_request_with_token(
            "/api/realtime/v2/events?heartbeat_seconds=1",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("stream response");

    assert_eq!(response.status(), StatusCode::OK);

    let mut stream = response.into_body().into_data_stream();
    let chunk = timeout(Duration::from_secs(2), stream.next())
        .await
        .expect("first SSE chunk timed out")
        .expect("first SSE chunk")
        .expect("first SSE chunk bytes");
    let text = std::str::from_utf8(&chunk).expect("SSE chunk is UTF-8");

    assert!(text.contains("event: heartbeat"), "{text}");
    assert!(
        text.contains(&format!("\"after_position\":{position}")),
        "{text}"
    );
    assert!(!text.contains(&event_id), "{text}");
}

#[tokio::test]
async fn event_trace_api_returns_causal_edges_against_postgres() {
    let context = TestContext::new().await;
    let app = app_with_database(&context.connection_string()).await;
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let trace_id = format!("trace_api_{suffix}");
    let root_id = format!("evt_api_trace_root_{suffix}");
    let child_id = format!("evt_api_trace_child_{suffix}");

    let root_response = app
        .clone()
        .oneshot(json_request_with_token(
            "/api/v1/events",
            json!({
                "event_id": root_id,
                "event_type": "system_api_trace_test_event",
                "occurred_at": Utc::now(),
                "source": {
                    "kind": "test",
                    "provider": "integration",
                    "source_id": root_id
                },
                "subject": {"kind": "system", "entity_id": "backend"},
                "correlation_id": trace_id
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("root create response");
    assert_eq!(root_response.status(), StatusCode::CREATED);

    let child_response = app
        .clone()
        .oneshot(json_request_with_token(
            "/api/v1/events",
            json!({
                "event_id": child_id,
                "event_type": "system_api_trace_test_event",
                "occurred_at": Utc::now(),
                "source": {
                    "kind": "test",
                    "provider": "integration",
                    "source_id": child_id
                },
                "subject": {"kind": "system", "entity_id": "backend"},
                "causation_id": root_id,
                "correlation_id": trace_id
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("child create response");
    assert_eq!(child_response.status(), StatusCode::CREATED);

    let trace_response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!("/api/v1/events/{child_id}/trace"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("trace response");
    assert_eq!(trace_response.status(), StatusCode::OK);
    let trace_body = json_body(trace_response).await;

    assert_eq!(trace_body["correlation_id"], json!(trace_id));
    assert_eq!(trace_body["root_event_ids"], json!([root_id]));
    assert_eq!(trace_body["events"].as_array().expect("events").len(), 2);
    assert_eq!(
        trace_body["edges"],
        json!([{
            "parent_event_id": root_id,
            "child_event_id": child_id
        }])
    );
    assert_eq!(trace_body["missing_parent_ids"], json!([]));
    assert_eq!(trace_body["orphan_event_ids"], json!([]));

    let children_response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/events/{root_id}/children"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("children response");
    assert_eq!(children_response.status(), StatusCode::OK);
    let children_body = json_body(children_response).await;
    assert_eq!(children_body.as_array().expect("children").len(), 1);
    assert_eq!(children_body[0]["event"]["event_id"], json!(child_id));
}

async fn app_with_database(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(config_with_api_token(), database)
}

fn config_with_api_token() -> AppConfig {
    makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN)
}

fn json_request_with_token(uri: &str, value: serde_json::Value, token: &str) -> Request<Body> {
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

async fn json_body(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 4096)
        .await
        .expect("body bytes");

    serde_json::from_slice(&body).expect("json body")
}
