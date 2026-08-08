use axum::http::StatusCode;
use makosh_backend_testkit::context::TestContext;
use serde_json::json;
use tower::ServiceExt;

use super::support::{
    get, post, response_json, router, seed_projected_message, seed_projected_message_with_body, uid,
};

#[tokio::test]
async fn v1_messages_list_uses_cursor_pagination_without_duplicates_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-cursor-api-{suffix}");
    let mut seeded_message_ids = Vec::new();

    for index in 0..3 {
        let message_id = seed_projected_message(
            pool.clone(),
            &account_id,
            &format!("provider-cursor-api-{suffix}-{index}"),
            &format!("Cursor page subject {suffix} {index}"),
        )
        .await;
        sqlx::query(
            r#"
            UPDATE communication_messages
            SET occurred_at = now() - ($2::int * interval '1 minute'),
                projected_at = now() - ($2::int * interval '1 minute')
            WHERE message_id = $1
            "#,
        )
        .bind(&message_id)
        .bind(index)
        .execute(&pool)
        .await
        .expect("set deterministic message ordering");
        seeded_message_ids.push(message_id);
    }

    let router = router(&context.connection_string()).await;
    let first = router
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/messages?account_id={account_id}&limit=2"
        )))
        .await
        .expect("first cursor page");
    assert_eq!(first.status(), StatusCode::OK);
    let first_body = response_json(first).await;
    let first_items = first_body["items"].as_array().expect("first items");
    assert_eq!(first_items.len(), 2);
    assert_eq!(first_body["has_more"], true);
    let cursor = first_body["next_cursor"]
        .as_str()
        .expect("next cursor")
        .to_owned();
    assert!(!cursor.trim().is_empty());

    let second = router
        .oneshot(get(&format!(
            "/api/v1/communications/messages?account_id={account_id}&limit=2&cursor={cursor}"
        )))
        .await
        .expect("second cursor page");
    assert_eq!(second.status(), StatusCode::OK);
    let second_body = response_json(second).await;
    let second_items = second_body["items"].as_array().expect("second items");
    assert_eq!(second_items.len(), 1);
    assert_eq!(second_body["has_more"], false);
    assert!(second_body["next_cursor"].is_null());

    let returned_ids = first_items
        .iter()
        .chain(second_items.iter())
        .map(|item| item["message_id"].as_str().expect("message id").to_owned())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(returned_ids.len(), 3);
    for message_id in seeded_message_ids {
        assert!(returned_ids.contains(&message_id), "missing {message_id}");
    }
}

#[tokio::test]
async fn v1_threads_list_uses_cursor_pagination_without_duplicates_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-thread-cursor-api-{suffix}");

    for index in 0..3 {
        let message_id = seed_projected_message(
            pool.clone(),
            &account_id,
            &format!("provider-thread-cursor-api-{suffix}-{index}"),
            &format!("Thread Cursor Subject {suffix} {index}"),
        )
        .await;
        sqlx::query(
            r#"
            UPDATE communication_messages
            SET occurred_at = now() - ($2::int * interval '1 minute'),
                projected_at = now() - ($2::int * interval '1 minute')
            WHERE message_id = $1
            "#,
        )
        .bind(&message_id)
        .bind(index)
        .execute(&pool)
        .await
        .expect("set deterministic thread ordering");
    }

    let router = router(&context.connection_string()).await;
    let first = router
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/threads?account_id={account_id}&limit=2"
        )))
        .await
        .expect("first thread cursor page");
    assert_eq!(first.status(), StatusCode::OK);
    let first_body = response_json(first).await;
    let first_items = first_body["items"].as_array().expect("first thread items");
    assert_eq!(first_items.len(), 2);
    assert_eq!(first_body["has_more"], true);
    let cursor = first_body["next_cursor"]
        .as_str()
        .expect("next thread cursor")
        .to_owned();
    assert!(!cursor.trim().is_empty());

    let second = router
        .oneshot(get(&format!(
            "/api/v1/communications/threads?account_id={account_id}&limit=2&cursor={cursor}"
        )))
        .await
        .expect("second thread cursor page");
    assert_eq!(second.status(), StatusCode::OK);
    let second_body = response_json(second).await;
    let second_items = second_body["items"]
        .as_array()
        .expect("second thread items");
    assert_eq!(second_items.len(), 1);
    assert_eq!(second_body["has_more"], false);
    assert!(second_body["next_cursor"].is_null());

    let returned_ids = first_items
        .iter()
        .chain(second_items.iter())
        .map(|item| item["thread_id"].as_str().expect("thread id").to_owned())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(returned_ids.len(), 3);
}

#[tokio::test]
async fn v1_translate_thread_returns_per_message_fallbacks_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-thread-translate-{suffix}");
    let subject = "Thread Translation";
    let first_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("thread-translate-1-{suffix}"),
        subject,
        "Привет, нужна проверка договора.",
    )
    .await;
    let second_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("thread-translate-2-{suffix}"),
        &format!("Re: {subject}"),
        "Hello, the agreement is attached.",
    )
    .await;
    let router = router(&context.connection_string()).await;
    let response = router
        .oneshot(post(
            &format!(
                "/api/v1/communications/threads/translate?account_id={account_id}&subject=Thread%20Translation"
            ),
            json!({ "target_language": "en" }),
        ))
        .await
        .expect("thread translate response");

    let status = response.status();
    let body = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "response body: {body}");
    assert_eq!(body["account_id"], account_id);
    assert_eq!(body["subject"], subject);
    assert_eq!(body["target_language"], "en");
    let items = body["items"].as_array().expect("translation items");
    assert_eq!(items.len(), 2);
    let returned_ids = items
        .iter()
        .map(|item| item["message_id"].as_str().expect("message id").to_owned())
        .collect::<std::collections::HashSet<_>>();
    assert!(returned_ids.contains(&first_id));
    assert!(returned_ids.contains(&second_id));
    assert!(
        items
            .iter()
            .any(|item| item["original_language"] == "ru" && item["translated"] == false)
    );
    assert!(items.iter().all(|item| {
        item["reason"]
            .as_str()
            .map(|reason| !reason.trim().is_empty())
            .unwrap_or(false)
    }));
}
