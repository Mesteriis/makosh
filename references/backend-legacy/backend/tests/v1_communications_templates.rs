use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::platform::storage::database::Database;

const T: &str = "v1comms-template-test-token";

async fn router(db: &str) -> axum::Router {
    let database = Database::connect(Some(db)).await.expect("db");
    build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(T, db),
        database,
    )
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
        .expect("req")
}

fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .body(Body::from(body.to_string()))
        .expect("req")
}

fn delete_req(uri: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
        .expect("req")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("t")
        .as_nanos()
}

#[tokio::test]
async fn rich_template_save_list_render_and_delete_uses_durable_template_store() {
    let context = TestContext::new().await;
    let db = context.connection_string();
    let suffix = uid();
    let template_id = format!("mail-template-{suffix}");
    let r = router(&db).await;

    let save_resp = r
        .clone()
        .oneshot(post(
            "/api/v1/communications/templates/rich",
            json!({
                "template_id": template_id,
                "name": "Project update",
                "subject_template": "Hello {{name}}",
                "body_template": "Project {{project}} is {{status}}.",
                "variables": ["name", "project", "status"],
                "language": "en"
            }),
        ))
        .await
        .expect("save template");
    assert_eq!(save_resp.status(), StatusCode::OK);
    let saved_body: Value =
        serde_json::from_slice(&to_bytes(save_resp.into_body(), 1024 * 1024).await.unwrap())
            .unwrap();
    assert_eq!(saved_body["template"]["template_id"], template_id);
    assert_eq!(
        saved_body["template"]["placeholder_variables"],
        json!(["name", "project", "status"])
    );
    assert_eq!(saved_body["template"]["undeclared_variables"], json!([]));
    assert_eq!(saved_body["template"]["unused_variables"], json!([]));
    assert_eq!(saved_body["template"]["malformed_placeholders"], json!([]));

    let list_resp = r
        .clone()
        .oneshot(get("/api/v1/communications/templates/rich"))
        .await
        .expect("list templates");
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body: Value =
        serde_json::from_slice(&to_bytes(list_resp.into_body(), 1024 * 1024).await.unwrap())
            .unwrap();
    assert!(
        list_body["templates"]
            .as_array()
            .unwrap()
            .iter()
            .any(|template| template["template_id"] == template_id)
    );
    let listed_template = list_body["templates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|template| template["template_id"] == template_id)
        .unwrap();
    assert_eq!(
        listed_template["placeholder_variables"],
        json!(["name", "project", "status"])
    );

    let render_resp = r
        .clone()
        .oneshot(post(
            "/api/v1/communications/templates/rich/render",
            json!({
                "template_id": template_id,
                "variables": {
                    "name": "Alex",
                    "project": "Макошь",
                    "status": "green"
                }
            }),
        ))
        .await
        .expect("render template");
    assert_eq!(render_resp.status(), StatusCode::OK);
    let render_body: Value = serde_json::from_slice(
        &to_bytes(render_resp.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(render_body["rendered"]["subject"], "Hello Alex");
    assert_eq!(render_body["rendered"]["body"], "Project Макошь is green.");
    assert_eq!(render_body["rendered"]["missing_variables"], json!([]));
    assert_eq!(render_body["rendered"]["unresolved_variables"], json!([]));
    assert_eq!(render_body["rendered"]["malformed_placeholders"], json!([]));

    let preview_resp = r
        .clone()
        .oneshot(post(
            "/api/v1/communications/templates/rich/mail-merge-preview",
            json!({
                "template_id": template_id,
                "rows": [
                    {
                        "row_id": "r1",
                        "variables": {
                            "name": "Alex",
                            "project": "Макошь",
                            "status": "green"
                        }
                    },
                    {
                        "row_id": "r2",
                        "variables": {
                            "name": "Sam",
                            "project": "Макошь"
                        }
                    }
                ]
            }),
        ))
        .await
        .expect("mail merge preview");
    assert_eq!(preview_resp.status(), StatusCode::OK);
    let preview_body: Value = serde_json::from_slice(
        &to_bytes(preview_resp.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(preview_body["template_id"], template_id);
    assert_eq!(preview_body["row_count"], 2);
    assert_eq!(preview_body["ready_count"], 1);
    assert_eq!(preview_body["blocked_count"], 1);
    assert_eq!(preview_body["items"][0]["row_id"], "r1");
    assert_eq!(preview_body["items"][0]["ready"], true);
    assert_eq!(
        preview_body["items"][0]["rendered"]["subject"],
        "Hello Alex"
    );
    assert_eq!(
        preview_body["items"][0]["rendered"]["body"],
        "Project Макошь is green."
    );
    assert_eq!(preview_body["items"][1]["row_id"], "r2");
    assert_eq!(preview_body["items"][1]["ready"], false);
    assert_eq!(
        preview_body["items"][1]["rendered"]["missing_variables"],
        json!(["status"])
    );

    let delete_path = format!("/api/v1/communications/templates/rich/{template_id}");
    let delete_resp = r
        .clone()
        .oneshot(delete_req(&delete_path))
        .await
        .expect("delete template");
    assert_eq!(delete_resp.status(), StatusCode::OK);
    let delete_body: Value = serde_json::from_slice(
        &to_bytes(delete_resp.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(delete_body["template_id"], template_id);
    assert_eq!(delete_body["deleted"], true);

    let list_resp = r
        .oneshot(get("/api/v1/communications/templates/rich"))
        .await
        .expect("list templates after delete");
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body: Value =
        serde_json::from_slice(&to_bytes(list_resp.into_body(), 1024 * 1024).await.unwrap())
            .unwrap();
    assert!(
        !list_body["templates"]
            .as_array()
            .unwrap()
            .iter()
            .any(|template| template["template_id"] == template_id)
    );
}
