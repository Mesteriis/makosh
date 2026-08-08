use axum::http::StatusCode;
use chrono::{Duration, Utc};
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_postgres::store::CommunicationIngestionStore;

use makosh_backend_testkit::context::TestContext;
use serde_json::json;
use sqlx::Row;
use tower::ServiceExt;

use super::support::{delete, get, post, post_with_actor, response_json, router, uid};

#[tokio::test]
async fn v1_post_draft_allows_empty_subject_for_autosave_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-draft-autosave-{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Imap,
            "Draft Autosave IMAP",
            format!("draft-autosave-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let router = router(&context.connection_string()).await;
    let draft_id = format!("draft-autosave-{suffix}");
    let response = router
        .clone()
        .oneshot(post(
            "/api/v1/communications/drafts",
            json!({
                "draft_id": draft_id,
                "account_id": account_id,
                "to_recipients": [],
                "cc_recipients": [],
                "bcc_recipients": [],
                "subject": "",
                "body_text": "Body typed before subject",
                "body_html": null,
                "metadata": {"compose_mode": "compose"}
            }),
        ))
        .await
        .expect("draft autosave response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["draft_id"], draft_id);
    assert_eq!(body["subject"], "");
    assert_eq!(body["body_text"], "Body typed before subject");

    let created_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'mail.draft.created'
          AND subject->>'id' = $1
          AND payload->>'draft_id' = $1
          AND payload->>'account_id' = $2
          AND payload->>'status' = 'draft'
          AND NOT payload ? 'body_text'
          AND NOT payload ? 'subject'
        "#,
    )
    .bind(&draft_id)
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("draft created event count");
    assert_eq!(created_event_count, 1);
    let created_link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'draft'
           AND entity_id = $1
           AND relationship_kind = 'draft_upsert'
         ORDER BY created_at ASC
         LIMIT 1",
    )
    .bind(&draft_id)
    .fetch_one(&pool)
    .await
    .expect("draft create observation link");
    let created_observation_id: String = created_link
        .try_get("observation_id")
        .expect("draft create observation id");
    let created_metadata: serde_json::Value = created_link
        .try_get("metadata")
        .expect("draft create metadata");
    assert_eq!(created_metadata["operation"], "draft_create");
    let created_observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&created_observation_id)
    .fetch_one(&pool)
    .await
    .expect("draft create observation");
    let created_origin_kind: String = created_observation
        .try_get("origin_kind")
        .expect("draft create origin kind");
    let created_payload: serde_json::Value = created_observation
        .try_get("payload")
        .expect("draft create payload");
    assert_eq!(created_origin_kind, "manual");
    assert_eq!(created_payload["operation"], "draft_create");

    let response = router
        .clone()
        .oneshot(post(
            "/api/v1/communications/drafts",
            json!({
                "draft_id": draft_id,
                "account_id": account_id,
                "to_recipients": ["recipient@example.com"],
                "subject": "",
                "body_text": "Updated autosave body",
                "body_html": "<p>Updated autosave body</p>",
                "metadata": {"compose_mode": "compose"}
            }),
        ))
        .await
        .expect("draft autosave update response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["draft_id"], draft_id);
    assert_eq!(body["body_text"], "Updated autosave body");

    let updated_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'mail.draft.updated'
          AND subject->>'id' = $1
          AND payload->>'draft_id' = $1
          AND payload->>'account_id' = $2
          AND payload->>'status' = 'draft'
          AND payload->>'has_body_html' = 'true'
          AND payload->>'to_recipient_count' = '1'
          AND NOT payload ? 'body_text'
          AND NOT payload ? 'subject'
        "#,
    )
    .bind(&draft_id)
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("draft updated event count");
    assert_eq!(updated_event_count, 1);
    let upsert_links_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*)
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'draft'
           AND entity_id = $1
           AND relationship_kind = 'draft_upsert'",
    )
    .bind(&draft_id)
    .fetch_one(&pool)
    .await
    .expect("draft upsert observation count");
    assert_eq!(upsert_links_count, 2);

    let response = router
        .oneshot(delete(&format!("/api/v1/communications/drafts/{draft_id}")))
        .await
        .expect("draft delete response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["deleted"], true);

    let deleted_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'mail.draft.deleted'
          AND subject->>'id' = $1
          AND payload->>'draft_id' = $1
          AND payload->>'account_id' = $2
          AND payload->>'status' = 'draft'
          AND NOT payload ? 'body_text'
          AND NOT payload ? 'subject'
        "#,
    )
    .bind(&draft_id)
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("draft deleted event count");
    assert_eq!(deleted_event_count, 1);
    let deleted_link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'draft'
           AND entity_id = $1
           AND relationship_kind = 'draft_delete'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&draft_id)
    .fetch_one(&pool)
    .await
    .expect("draft delete observation link");
    let deleted_observation_id: String = deleted_link
        .try_get("observation_id")
        .expect("draft delete observation id");
    let deleted_metadata: serde_json::Value = deleted_link
        .try_get("metadata")
        .expect("draft delete metadata");
    assert_eq!(deleted_metadata["operation"], "draft_delete");
    let deleted_observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&deleted_observation_id)
    .fetch_one(&pool)
    .await
    .expect("draft delete observation");
    let deleted_origin_kind: String = deleted_observation
        .try_get("origin_kind")
        .expect("draft delete origin kind");
    let deleted_payload: serde_json::Value = deleted_observation
        .try_get("payload")
        .expect("draft delete payload");
    assert_eq!(deleted_origin_kind, "manual");
    assert_eq!(deleted_payload["operation"], "draft_delete");
}

#[tokio::test]
async fn v1_drafts_list_is_cursor_paginated_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-draft-page-{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Imap,
            "Draft Pagination IMAP",
            format!("draft-page-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let router = router(&context.connection_string()).await;
    for index in 0..2 {
        let response = router
            .clone()
            .oneshot(post(
                "/api/v1/communications/drafts",
                json!({
                    "draft_id": format!("draft-page-{suffix}-{index}"),
                    "account_id": account_id,
                    "to_recipients": ["recipient@example.com"],
                    "subject": format!("Paged draft {index}"),
                    "body_text": format!("Draft body {index}"),
                    "metadata": {"compose_mode": "compose"}
                }),
            ))
            .await
            .expect("draft create response");
        assert_eq!(response.status(), StatusCode::OK);
    }

    let response = router
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/drafts?account_id={account_id}&limit=1"
        )))
        .await
        .expect("draft list first page");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["has_more"], true);
    let cursor = body["next_cursor"].as_str().expect("next cursor");

    let response = router
        .oneshot(get(&format!(
            "/api/v1/communications/drafts?account_id={account_id}&limit=1&cursor={cursor}"
        )))
        .await
        .expect("draft list second page");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["has_more"], false);
    assert!(body["next_cursor"].is_null());
}

#[tokio::test]
async fn v1_send_schedules_outbox_message_and_allows_undo_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-outbox-api-{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Imap,
            "Outbox API IMAP",
            format!("outbox-api-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let router = router(&context.connection_string()).await;
    let scheduled_send_at = Utc::now() + Duration::hours(1);
    let send = router
        .clone()
        .oneshot(post_with_actor(
            "/api/v1/communications/send",
            json!({
                "account_id": account_id,
                "to": ["recipient@example.com"],
                "subject": "Scheduled outbox",
                "body_text": "This should be queued, not sent immediately.",
                "scheduled_send_at": scheduled_send_at,
                "undo_send_seconds": 30,
                "confirmed_provider_write": true
            }),
        ))
        .await
        .expect("scheduled send response");

    assert_eq!(send.status(), StatusCode::OK);
    let send_body = response_json(send).await;
    assert_eq!(send_body["transport"], "outbox");
    assert_eq!(send_body["status"], "scheduled");
    let outbox_id = send_body["outbox_id"].as_str().expect("outbox id");
    assert!(!outbox_id.trim().is_empty());

    let list = router
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/outbox?account_id={account_id}&status=scheduled"
        )))
        .await
        .expect("outbox list response");
    assert_eq!(list.status(), StatusCode::OK);
    let list_body = response_json(list).await;
    let items = list_body["items"].as_array().expect("outbox items");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["outbox_id"], outbox_id);
    assert_eq!(items[0]["subject"], "Scheduled outbox");
    let enqueue_link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'outbox_item'
           AND entity_id = $1
           AND relationship_kind = 'outbox_status_transition'
         ORDER BY created_at ASC
         LIMIT 1",
    )
    .bind(outbox_id)
    .fetch_one(&pool)
    .await
    .expect("enqueue outbox observation link");
    let enqueue_observation_id: String = enqueue_link
        .try_get("observation_id")
        .expect("enqueue observation id");
    let enqueue_metadata: serde_json::Value =
        enqueue_link.try_get("metadata").expect("enqueue metadata");
    assert_eq!(enqueue_metadata["status"], "scheduled");
    let enqueue_observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&enqueue_observation_id)
    .fetch_one(&pool)
    .await
    .expect("enqueue observation");
    let enqueue_origin_kind: String = enqueue_observation
        .try_get("origin_kind")
        .expect("enqueue origin kind");
    let enqueue_payload: serde_json::Value = enqueue_observation
        .try_get("payload")
        .expect("enqueue payload");
    assert_eq!(enqueue_origin_kind, "manual");
    assert_eq!(enqueue_payload["operation"], "outbox_schedule");

    let undo = router
        .clone()
        .oneshot(post(
            &format!("/api/v1/communications/outbox/{outbox_id}/undo"),
            json!({}),
        ))
        .await
        .expect("outbox undo response");
    assert_eq!(undo.status(), StatusCode::OK);
    let undo_body = response_json(undo).await;
    assert_eq!(undo_body["outbox_id"], outbox_id);
    assert_eq!(undo_body["status"], "canceled");
    let transition_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*)
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'outbox_item'
           AND entity_id = $1
           AND relationship_kind = 'outbox_status_transition'",
    )
    .bind(outbox_id)
    .fetch_one(&pool)
    .await
    .expect("outbox transition count");
    assert_eq!(transition_count, 2);

    let canceled = router
        .oneshot(get(&format!(
            "/api/v1/communications/outbox?account_id={account_id}&status=canceled"
        )))
        .await
        .expect("canceled outbox list response");
    assert_eq!(canceled.status(), StatusCode::OK);
    let canceled_body = response_json(canceled).await;
    assert_eq!(
        canceled_body["items"]
            .as_array()
            .expect("canceled outbox items")
            .len(),
        1
    );
    let undo_link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'outbox_item'
           AND entity_id = $1
           AND relationship_kind = 'outbox_status_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(outbox_id)
    .fetch_one(&pool)
    .await
    .expect("undo outbox observation link");
    let undo_observation_id: String = undo_link
        .try_get("observation_id")
        .expect("undo observation id");
    let undo_metadata: serde_json::Value = undo_link.try_get("metadata").expect("undo metadata");
    assert_eq!(undo_metadata["operation"], "outbox_undo");
    assert_eq!(undo_metadata["status"], "canceled");
    let undo_observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&undo_observation_id)
    .fetch_one(&pool)
    .await
    .expect("undo observation");
    let undo_origin_kind: String = undo_observation
        .try_get("origin_kind")
        .expect("undo origin kind");
    let undo_payload: serde_json::Value =
        undo_observation.try_get("payload").expect("undo payload");
    assert_eq!(undo_origin_kind, "manual");
    assert_eq!(undo_payload["operation"], "outbox_undo");
}
