use axum::body::to_bytes;
use axum::http::StatusCode;
use makosh_backend_testkit::app::{TEST_API_SECRET, get, post_json};
use makosh_backend_testkit::context::TestContext;
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::router::build_router_with_database;

#[tokio::test]
async fn maintenance_overview_reports_inventory_backups_and_guarded_actions() {
    let context = TestContext::new().await;
    let app = build_router_with_database(context.app_config(TEST_API_SECRET), context.database());

    let response = app
        .oneshot(get("/api/v1/maintenance/overview"))
        .await
        .expect("maintenance overview response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;

    let inventory_ids = body["inventory"]
        .as_array()
        .expect("inventory")
        .iter()
        .map(|item| item["id"].as_str().expect("inventory id"))
        .collect::<Vec<_>>();
    assert!(inventory_ids.contains(&"database"));
    assert!(inventory_ids.contains(&"mail_blobs"));
    assert!(inventory_ids.contains(&"dev_logs"));
    assert!(inventory_ids.contains(&"backups"));

    let actions = body["actions"].as_array().expect("actions");
    let restore = actions
        .iter()
        .find(|action| action["id"] == "restore_database")
        .expect("restore action");
    assert_eq!(restore["enabled"], json!(false));
    assert_eq!(restore["destructive"], json!(true));
    assert_eq!(restore["confirmation_phrase"], json!("RESTORE"));
}

#[tokio::test]
async fn maintenance_action_rejects_restore_from_live_backend() {
    let context = TestContext::new().await;
    let app = build_router_with_database(context.app_config(TEST_API_SECRET), context.database());

    let response = app
        .oneshot(post_json(
            "/api/v1/maintenance/actions/restore_database",
            json!({ "confirmation": "RESTORE" }),
        ))
        .await
        .expect("maintenance action response");

    assert_eq!(response.status(), StatusCode::PRECONDITION_FAILED);
    let body = json_body(response).await;
    assert_eq!(body["error"], json!("failed_precondition"));
    assert!(
        body["message"]
            .as_str()
            .expect("message")
            .contains("make vault-restore")
    );
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    serde_json::from_slice(&body).expect("json body")
}
