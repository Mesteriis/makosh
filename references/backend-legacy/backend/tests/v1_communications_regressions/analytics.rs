use axum::http::StatusCode;
use makosh_backend_testkit::context::TestContext;
use tower::ServiceExt;

use super::support::{get, response_json, router, seed_projected_message_from_sender, uid};

#[tokio::test]
async fn v1_subscriptions_list_is_cursor_paginated_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-sub-page-{suffix}");
    for index in 0..3 {
        seed_projected_message_from_sender(
            pool.clone(),
            &account_id,
            &format!("sub-a-{suffix}-{index}"),
            "Weekly digest",
            "newsletter-a@example.com",
            "Newsletter body with unsubscribe link",
        )
        .await;
    }
    for index in 0..2 {
        seed_projected_message_from_sender(
            pool.clone(),
            &account_id,
            &format!("sub-b-{suffix}-{index}"),
            "Product newsletter",
            "newsletter-b@example.com",
            "Newsletter body with manage preferences link",
        )
        .await;
    }
    let router = router(&context.connection_string()).await;
    let response = router
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/subscriptions?account_id={account_id}&limit=1"
        )))
        .await
        .expect("subscriptions first page");
    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["has_more"], true);
    let cursor = body["next_cursor"].as_str().expect("next cursor");

    let response = router
        .oneshot(get(&format!(
            "/api/v1/communications/subscriptions?account_id={account_id}&limit=1&cursor={cursor}"
        )))
        .await
        .expect("subscriptions second page");
    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["has_more"], false);
    assert!(body["next_cursor"].is_null());
}

#[tokio::test]
async fn v1_top_senders_list_is_cursor_paginated_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-senders-page-{suffix}");
    for index in 0..3 {
        seed_projected_message_from_sender(
            pool.clone(),
            &account_id,
            &format!("sender-a-{suffix}-{index}"),
            "Message from A",
            "sender-a@example.com",
            "Regular mail body",
        )
        .await;
    }
    for index in 0..2 {
        seed_projected_message_from_sender(
            pool.clone(),
            &account_id,
            &format!("sender-b-{suffix}-{index}"),
            "Message from B",
            "sender-b@example.com",
            "Regular mail body",
        )
        .await;
    }

    let router = router(&context.connection_string()).await;
    let response = router
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/analytics/senders?account_id={account_id}&limit=1"
        )))
        .await
        .expect("top senders first page");
    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["has_more"], true);
    let cursor = body["next_cursor"].as_str().expect("next cursor");

    let response = router
        .oneshot(get(&format!(
            "/api/v1/communications/analytics/senders?account_id={account_id}&limit=1&cursor={cursor}"
        )))
        .await
        .expect("top senders second page");
    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["has_more"], false);
    assert!(body["next_cursor"].is_null());
}
