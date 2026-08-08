use makosh_backend_testkit::{app, context::TestContext};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use makosh_hub_backend::app::router::{build_router, build_router_with_database};
use makosh_hub_backend::platform::storage::database::Database;

#[tokio::test]
async fn healthz_returns_ok_status_and_service_name() {
    let app = build_router(app::config());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), 1024)
        .await
        .expect("body bytes");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("json body");

    assert_eq!(
        value,
        json!({
            "status": "ok",
            "service": "makosh-backend"
        })
    );
}

#[tokio::test]
async fn readyz_returns_service_unavailable_when_database_is_not_configured() {
    let app = build_router(app::config());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = to_bytes(response.into_body(), 2048)
        .await
        .expect("body bytes");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("json body");

    assert_eq!(value["status"], json!("degraded"));
    assert_eq!(value["service"], json!("makosh-backend"));
    assert_eq!(
        value["checks"]["database"]["status"],
        json!("not_configured")
    );
    assert!(value["checks"]["database"]["message"].is_string());
    assert_eq!(
        value["checks"]["migrations"]["status"],
        json!("not_configured")
    );
    assert!(value["checks"]["migrations"]["message"].is_string());
}

#[tokio::test]
async fn readyz_reports_database_and_migrations_ok_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(app::config_with_database_url(database_url), database);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), 4096)
        .await
        .expect("body bytes");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("json body");

    assert_eq!(value["status"], "ok");
    assert_eq!(value["checks"]["database"]["status"], "ok");
    assert_eq!(
        value["checks"]["database"]["message"],
        "database is reachable"
    );
    assert_eq!(value["checks"]["migrations"]["status"], "ok");
    assert_eq!(
        value["checks"]["migrations"]["message"],
        "required database migrations are applied"
    );
}
