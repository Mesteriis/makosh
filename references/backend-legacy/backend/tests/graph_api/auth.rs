use crate::support::*;

#[tokio::test]
async fn graph_summary_rejects_missing_local_api_secret() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v1/graph/summary"))
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
async fn graph_summary_accepts_secret_without_actor_header() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request_with_token_without_actor(
            "/api/v1/graph/summary",
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
async fn graph_search_rejects_missing_local_api_secret_before_missing_query_validation() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v1/graph/search"))
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
async fn graph_nodes_rejects_missing_local_api_secret() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v1/graph/nodes"))
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
async fn graph_neighborhood_rejects_missing_local_api_secret_before_malformed_query_validation() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request(
            "/api/v1/graph/neighborhood?node_id=graph:node:v1:persona:alex&depth=not-a-number",
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
