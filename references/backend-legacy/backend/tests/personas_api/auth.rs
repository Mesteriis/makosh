use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

use super::support::{build_personas_app_without_database, get_request, json_body};

#[tokio::test]
async fn personas_rejects_missing_local_api_secret() {
    let app = build_personas_app_without_database();
    let response = app
        .oneshot(get_request("/api/v1/personas"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({"error": "invalid_api_secret", "message": "missing or invalid x-makosh-secret header"})
    );
}
