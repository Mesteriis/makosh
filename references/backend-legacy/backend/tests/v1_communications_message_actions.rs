use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::storage::database::Database;

const T: &str = "v1comms-action-test-token";

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
        .expect("request")
}

fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn put(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn delete(uri: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
        .expect("request")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

async fn router(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(T, database_url),
        database,
    )
}

macro_rules! v1_msg_post_test {
    ($name:ident, $path_suffix:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let test_context = TestContext::new().await;
            let db = test_context.connection_string();
            let r = router(&db).await;
            let response = r
                .oneshot(post(
                    &format!("/api/v1/communications/messages/msg:fake/{}", $path_suffix),
                    $body,
                ))
                .await
                .expect("response");
            assert!(
                !response.status().is_server_error(),
                "{} status={}",
                stringify!($name),
                response.status()
            );
        }
    };
}

v1_msg_post_test!(
    v1_send,
    "send",
    json!({"to": "test@example.com", "subject": "Test", "body": "Hello"})
);
v1_msg_post_test!(v1_reply, "reply", json!({"body": "Reply text"}));
v1_msg_post_test!(v1_reply_all, "reply-all", json!({"body": "Reply all text"}));
v1_msg_post_test!(v1_forward, "forward", json!({"to": "fwd@example.com"}));
v1_msg_post_test!(
    v1_forward_eml,
    "forward-eml",
    json!({"to": "fwd@example.com"})
);
v1_msg_post_test!(
    v1_redirect_missing_message,
    "redirect",
    json!({"to": ["redirect@example.com"], "confirmed_provider_write": true})
);
v1_msg_post_test!(v1_imap_mark_read, "imap-mark-read", json!({}));
v1_msg_post_test!(v1_imap_delete, "imap-delete", json!({}));
v1_msg_post_test!(v1_translate, "translate", json!({"target_language": "es"}));
v1_msg_post_test!(v1_ai_reply, "ai-reply", json!({"prompt": "Reply to this"}));
v1_msg_post_test!(
    v1_ai_reply_variants,
    "ai-reply-variants",
    json!({"prompt": "Reply variants"})
);
v1_msg_post_test!(v1_extract_tasks, "extract-tasks", json!({}));
v1_msg_post_test!(v1_extract_notes, "extract-notes", json!({}));
v1_msg_post_test!(
    v1_message_analyze,
    "analyze",
    json!({"analysis_type": "sentiment"})
);

#[tokio::test]
async fn v1_message_analyze_returns_structured_ai_summary_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-analyze-summary-{suffix}");
    let message_id = seed_projected_message_with_body(
        pool.clone(),
        &account_id,
        &format!("provider-analyze-summary-{suffix}"),
        "Action Required: Contract review deadline",
        "From: Ada Lovelace <ada@acme.example>\nPlease review the attached MSA and NDA by Friday. The payment risk remains open. Meeting on Monday at 10:00 with Acme Corp. Confirm approval before EOD.",
    )
    .await;
    let r = router(&context.connection_string()).await;

    let response = r
        .oneshot(post(
            &format!("/api/v1/communications/messages/{message_id}/analyze"),
            json!({}),
        ))
        .await
        .expect("analyze response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["message_id"], message_id);
    assert!(
        body["summary_contract"]["key_points"]
            .as_array()
            .expect("key points")
            .iter()
            .any(|item| item.as_str() == Some("Action Required: Contract review deadline"))
    );
    assert!(
        body["summary_contract"]["action_items"]
            .as_array()
            .expect("action items")
            .iter()
            .any(|item| {
                let item = item.as_str().unwrap_or("");
                item.contains("Please review") && item.contains("NDA")
            })
    );
    assert!(
        body["summary_contract"]["risks"]
            .as_array()
            .expect("risks")
            .iter()
            .any(|item| item.as_str().unwrap_or("").contains("payment risk"))
    );
    assert!(
        body["summary_contract"]["deadlines"]
            .as_array()
            .expect("deadlines")
            .iter()
            .any(|item| item.as_str().unwrap_or("").contains("Friday"))
    );
    assert!(
        body["summary_contract"]["event_candidates"]
            .as_array()
            .expect("event candidates")
            .iter()
            .any(|item| item["title"]
                .as_str()
                .unwrap_or("")
                .contains("Meeting on Monday"))
    );
    assert!(
        body["summary_contract"]["persona_candidates"]
            .as_array()
            .expect("persona candidates")
            .iter()
            .any(|item| item["title"]
                .as_str()
                .unwrap_or("")
                .contains("Ada Lovelace"))
    );
    assert!(
        body["summary_contract"]["organization_candidates"]
            .as_array()
            .expect("organization candidates")
            .iter()
            .any(|item| item["title"]
                .as_str()
                .unwrap_or("")
                .contains("acme.example"))
    );
    assert!(
        body["summary_contract"]["document_candidates"]
            .as_array()
            .expect("document candidates")
            .iter()
            .any(|item| item["title"].as_str().unwrap_or("").contains("MSA"))
    );
    assert!(
        body["summary_contract"]["agreement_candidates"]
            .as_array()
            .expect("agreement candidates")
            .iter()
            .any(|item| item["title"].as_str().unwrap_or("").contains("NDA"))
    );

    let metadata: Value = sqlx::query_scalar(
        "SELECT message_metadata FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message metadata");
    assert_eq!(
        metadata["ai_summary_contract"], body["summary_contract"],
        "analyze response must persist the structured summary contract"
    );

    let observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message observation id");
    let knowledge_review_items = sqlx::query(
        r#"
        SELECT metadata
        FROM review_items
        WHERE item_kind = 'knowledge_candidate'
          AND review_item_id IN (
              SELECT review_item_id
              FROM review_item_evidence
              WHERE observation_id = $1
          )
        ORDER BY metadata->>'candidate_group', title
        "#,
    )
    .bind(&observation_id)
    .fetch_all(&pool)
    .await
    .expect("knowledge review items");
    let mut candidate_groups = knowledge_review_items
        .into_iter()
        .map(|row| {
            let metadata: Value = row.try_get("metadata").expect("metadata");
            metadata["candidate_group"]
                .as_str()
                .expect("candidate group")
                .to_owned()
        })
        .collect::<Vec<_>>();
    candidate_groups.sort();
    candidate_groups.dedup();
    assert_eq!(
        candidate_groups,
        vec!["agreement".to_owned(), "document".to_owned()]
    );
}

#[tokio::test]
async fn v1_bulk_actions_mark_read_and_trash_messages_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-bulk-actions-{suffix}");
    let first_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-bulk-actions-{suffix}-1"),
        "Bulk actions first",
    )
    .await;
    let second_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-bulk-actions-{suffix}-2"),
        "Bulk actions second",
    )
    .await;

    let r = router(&context.connection_string()).await;
    let response = r
        .clone()
        .oneshot(post(
            "/api/v1/communications/messages/bulk-actions",
            json!({
                "action": "mark_read",
                "message_ids": [first_id, second_id]
            }),
        ))
        .await
        .expect("bulk mark-read response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["action"], "mark_read");
    assert_eq!(body["matched_count"], 2);
    assert_eq!(body["updated_count"], 2);
    assert_eq!(body["not_found"].as_array().expect("not_found").len(), 0);

    let (first_workflow_state, first_is_read): (String, bool) = sqlx::query_as(
        "SELECT workflow_state, is_read FROM communication_messages WHERE message_id = $1",
    )
    .bind(&first_id)
    .fetch_one(&pool)
    .await
    .expect("first workflow state");
    let (second_workflow_state, second_is_read): (String, bool) = sqlx::query_as(
        "SELECT workflow_state, is_read FROM communication_messages WHERE message_id = $1",
    )
    .bind(&second_id)
    .fetch_one(&pool)
    .await
    .expect("second workflow state");
    assert_eq!(first_workflow_state, "new");
    assert_eq!(second_workflow_state, "new");
    assert!(first_is_read);
    assert!(second_is_read);
    let read_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'communication.message.read_state_changed.v1'
          AND payload->>'action' = 'mark_read'
          AND payload->>'updated_count' = '2'
          AND payload->'message_ids' ? $1
          AND payload->'message_ids' ? $2
        "#,
    )
    .bind(&first_id)
    .bind(&second_id)
    .fetch_one(&pool)
    .await
    .expect("read event count");
    assert_eq!(read_event_count, 1);
    let read_state_links = sqlx::query(
        "SELECT observation_id, entity_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'communication_message'
           AND relationship_kind = 'read_state_transition'
           AND entity_id = ANY($1)
         ORDER BY entity_id ASC, created_at ASC",
    )
    .bind(vec![first_id.clone(), second_id.clone()])
    .fetch_all(&pool)
    .await
    .expect("read-state links");
    assert_eq!(read_state_links.len(), 2);
    for row in &read_state_links {
        let metadata: Value = row.try_get("metadata").expect("read-state metadata");
        assert_eq!(metadata["is_read"], true);
        let observation_id: String = row
            .try_get("observation_id")
            .expect("read-state observation id");
        let observation = sqlx::query(
            "SELECT origin_kind, payload
             FROM observations
             WHERE observation_id = $1",
        )
        .bind(&observation_id)
        .fetch_one(&pool)
        .await
        .expect("read-state observation");
        let origin_kind: String = observation
            .try_get("origin_kind")
            .expect("read-state origin kind");
        let payload: Value = observation.try_get("payload").expect("read-state payload");
        assert_eq!(origin_kind, "manual");
        assert_eq!(payload["action"], "mark_read");
    }

    let missing_id = format!("msg:missing-{suffix}");
    let response = r
        .oneshot(post(
            "/api/v1/communications/messages/bulk-actions",
            json!({
                "action": "trash",
                "message_ids": [first_id, missing_id]
            }),
        ))
        .await
        .expect("bulk trash response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["action"], "trash");
    assert_eq!(body["matched_count"], 1);
    assert_eq!(body["updated_count"], 1);
    assert_eq!(body["not_found"], json!([missing_id]));

    let first_local_state: String =
        sqlx::query_scalar("SELECT local_state FROM communication_messages WHERE message_id = $1")
            .bind(&first_id)
            .fetch_one(&pool)
            .await
            .expect("first local state");
    assert_eq!(first_local_state, "trash");

    let deleted_event_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM event_log
        WHERE event_type = 'mail.message.deleted'
          AND payload->>'action' = 'trash'
          AND payload->>'updated_count' = '1'
          AND payload->'message_ids' ? $1
          AND payload->'not_found' ? $2
        "#,
    )
    .bind(&first_id)
    .bind(&missing_id)
    .fetch_one(&pool)
    .await
    .expect("deleted event count");
    assert_eq!(deleted_event_count, 1);
    let trash_link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'communication_message'
           AND entity_id = $1
           AND relationship_kind = 'local_state_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&first_id)
    .fetch_one(&pool)
    .await
    .expect("trash link");
    let trash_observation_id: String = trash_link
        .try_get("observation_id")
        .expect("trash observation id");
    let trash_metadata: Value = trash_link.try_get("metadata").expect("trash metadata");
    assert_eq!(trash_metadata["local_state"], "trash");
    let trash_observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&trash_observation_id)
    .fetch_one(&pool)
    .await
    .expect("trash observation");
    let trash_origin_kind: String = trash_observation
        .try_get("origin_kind")
        .expect("trash origin kind");
    let trash_payload: Value = trash_observation.try_get("payload").expect("trash payload");
    assert_eq!(trash_origin_kind, "manual");
    assert_eq!(trash_payload["action"], "trash");
    assert_eq!(trash_payload["not_found"], json!([missing_id]));

    let provider_command_kinds = sqlx::query_scalar::<_, String>(
        r#"
        SELECT command_kind
        FROM communication_provider_commands
        WHERE account_id = $1
          AND channel_kind = 'mail'
        ORDER BY created_at ASC, command_id ASC
        "#,
    )
    .bind(&account_id)
    .fetch_all(&pool)
    .await
    .expect("bulk provider command kinds");
    assert_eq!(
        provider_command_kinds,
        vec!["mark_read", "mark_read", "trash"]
    );
}

#[tokio::test]
async fn v1_bulk_actions_star_and_unstar_enqueue_distinct_provider_commands() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-bulk-star-{suffix}");
    let message_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-bulk-star-{suffix}"),
        "Starred message",
    )
    .await;
    let router = router(&context.connection_string()).await;

    for (action, expected_starred) in [("star", true), ("unstar", false)] {
        let response = router
            .clone()
            .oneshot(post(
                "/api/v1/communications/messages/bulk-actions",
                json!({ "action": action, "message_ids": [message_id] }),
            ))
            .await
            .expect("star bulk response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["action"], action);
        assert_eq!(body["updated_count"], 1);

        let starred: bool = sqlx::query_scalar(
            "SELECT COALESCE((message_metadata ->> 'starred')::boolean, false) FROM communication_messages WHERE message_id = $1",
        )
        .bind(&message_id)
        .fetch_one(&pool)
        .await
        .expect("stored starred state");
        assert_eq!(starred, expected_starred);
    }

    let command_kinds = sqlx::query_scalar::<_, String>(
        "SELECT command_kind FROM communication_provider_commands WHERE account_id = $1 AND channel_kind = 'mail' ORDER BY created_at ASC, command_id ASC",
    )
    .bind(&account_id)
    .fetch_all(&pool)
    .await
    .expect("star provider commands");
    assert_eq!(command_kinds, vec!["star", "unstar"]);

    let event_types = sqlx::query_scalar::<_, String>(
        "SELECT event_type FROM event_log WHERE payload ->> 'action' IN ('star', 'unstar') ORDER BY position ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("star events");
    assert_eq!(
        event_types,
        vec!["mail.message.starred", "mail.message.unstarred"]
    );
}

#[tokio::test]
async fn v1_local_state_endpoints_capture_observation_trail_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-local-state-{suffix}");
    let message_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-local-state-{suffix}"),
        "Local state trail",
    )
    .await;
    let r = router(&context.connection_string()).await;

    let mark_read = r
        .clone()
        .oneshot(post(
            &format!("/api/v1/communications/messages/{message_id}/imap-mark-read"),
            json!({}),
        ))
        .await
        .expect("imap mark read response");
    assert_eq!(mark_read.status(), StatusCode::OK);

    let trash = r
        .clone()
        .oneshot(post(
            &format!("/api/v1/communications/messages/{message_id}/trash"),
            json!({}),
        ))
        .await
        .expect("trash response");
    assert_eq!(trash.status(), StatusCode::OK);

    let restore = r
        .oneshot(post(
            &format!("/api/v1/communications/messages/{message_id}/restore"),
            json!({}),
        ))
        .await
        .expect("restore response");
    assert_eq!(restore.status(), StatusCode::OK);

    let read_rows = sqlx::query(
        "SELECT metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'communication_message'
           AND entity_id = $1
           AND relationship_kind = 'read_state_transition'
         ORDER BY created_at ASC",
    )
    .bind(&message_id)
    .fetch_all(&pool)
    .await
    .expect("read-state observation links");
    assert!(!read_rows.is_empty());
    let read_metadata: Value = read_rows
        .last()
        .expect("read-state row")
        .try_get("metadata")
        .expect("read-state metadata");
    assert_eq!(read_metadata["is_read"], true);

    let local_rows = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'communication_message'
           AND entity_id = $1
           AND relationship_kind = 'local_state_transition'
         ORDER BY created_at ASC",
    )
    .bind(&message_id)
    .fetch_all(&pool)
    .await
    .expect("local state observation links");
    assert_eq!(local_rows.len(), 2);
    let local_states: Vec<String> = local_rows
        .iter()
        .map(|row| {
            row.try_get::<Value, _>("metadata").expect("local metadata")["local_state"]
                .as_str()
                .expect("local state")
                .to_owned()
        })
        .collect();
    assert_eq!(local_states, vec!["trash", "active"]);

    let restore_observation_id: String = local_rows[1]
        .try_get("observation_id")
        .expect("restore observation id");
    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&restore_observation_id)
            .fetch_one(&pool)
            .await
            .expect("restore observation origin");
    assert_eq!(origin_kind, "manual");

    let provider_commands = sqlx::query_scalar::<_, String>(
        r#"
        SELECT command_kind
        FROM communication_provider_commands
        WHERE account_id = $1
          AND channel_kind = 'mail'
        ORDER BY created_at ASC, command_id ASC
        "#,
    )
    .bind(&account_id)
    .fetch_all(&pool)
    .await
    .expect("local state provider commands");
    assert_eq!(provider_commands, vec!["mark_read", "trash"]);
}

#[tokio::test]
async fn v1_spam_workflow_state_enqueues_provider_sync_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-spam-state-{suffix}");
    let message_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-spam-state-{suffix}"),
        "Spam state synchronization",
    )
    .await;
    let r = router(&context.connection_string()).await;

    let response = r
        .oneshot(put(
            &format!("/api/v1/communications/messages/{message_id}/workflow-state"),
            json!({ "workflow_state": "spam" }),
        ))
        .await
        .expect("spam state response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["workflow_state"], "spam");
    assert_eq!(body["previous_state"], "new");

    let command: (String, String, Value) = sqlx::query_as(
        r#"
        SELECT command_kind, actor_id, target_ref
        FROM communication_provider_commands
        WHERE account_id = $1
          AND channel_kind = 'mail'
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("spam provider command");
    assert_eq!(command.0, "mark_spam");
    assert_eq!(command.1, "makosh-frontend");
    assert_eq!(command.2["message_id"], message_id);
}

#[tokio::test]
async fn v1_not_spam_workflow_state_enqueues_provider_reconciliation_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-not-spam-state-{suffix}");
    let message_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-not-spam-state-{suffix}"),
        "Not spam reconciliation",
    )
    .await;
    let r = router(&context.connection_string()).await;

    let spam_response = r
        .clone()
        .oneshot(put(
            &format!("/api/v1/communications/messages/{message_id}/workflow-state"),
            json!({ "workflow_state": "spam" }),
        ))
        .await
        .expect("spam state response");
    assert_eq!(spam_response.status(), StatusCode::OK);

    let response = r
        .oneshot(put(
            &format!("/api/v1/communications/messages/{message_id}/workflow-state"),
            json!({ "workflow_state": "new" }),
        ))
        .await
        .expect("not-spam state response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["workflow_state"], "new");
    assert_eq!(body["previous_state"], "spam");

    let command_kinds = sqlx::query_scalar::<_, String>(
        r#"
        SELECT command_kind
        FROM communication_provider_commands
        WHERE account_id = $1
          AND channel_kind = 'mail'
        ORDER BY created_at ASC, command_id ASC
        "#,
    )
    .bind(&account_id)
    .fetch_all(&pool)
    .await
    .expect("not-spam provider commands");
    assert_eq!(command_kinds, vec!["mark_spam", "mark_not_spam"]);

    let (score, spam_count, non_spam_count, last_reason): (i16, i32, i32, String) = sqlx::query_as(
        r#"
            SELECT score, spam_count, non_spam_count, last_reason
            FROM communication_sender_reputation
            WHERE sender_key = 'sender@example.com'
            "#,
    )
    .fetch_one(&pool)
    .await
    .expect("manual sender feedback");
    assert_eq!(score, 60);
    assert_eq!(spam_count, 1);
    assert_eq!(non_spam_count, 1);
    assert_eq!(last_reason, "manual_workflow_state_transition");
}

#[tokio::test]
async fn v1_read_state_keeps_each_local_intent_durable_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-read-intent-{suffix}");
    let message_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-read-intent-{suffix}"),
        "Versioned read intent",
    )
    .await;
    let r = router(&context.connection_string()).await;

    for is_read in [true, false, true] {
        let response = r
            .clone()
            .oneshot(put(
                &format!("/api/v1/communications/messages/{message_id}/read-state"),
                json!({ "is_read": is_read }),
            ))
            .await
            .expect("read-state response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["message_id"], message_id);
        assert_eq!(body["is_read"], is_read);
        assert_eq!(body["read_sync_status"], "queued");
    }

    let command_rows = sqlx::query(
        r#"
        SELECT command_kind, idempotency_key, payload
        FROM communication_provider_commands
        WHERE account_id = $1
          AND channel_kind = 'mail'
        ORDER BY created_at ASC
        "#,
    )
    .bind(&account_id)
    .fetch_all(&pool)
    .await
    .expect("read-state provider commands");
    assert_eq!(command_rows.len(), 3);
    let command_kinds = command_rows
        .iter()
        .map(|row| {
            row.try_get::<String, _>("command_kind")
                .expect("command kind")
        })
        .collect::<Vec<_>>();
    assert_eq!(command_kinds, vec!["mark_read", "mark_unread", "mark_read"]);
    let idempotency_keys = command_rows
        .iter()
        .map(|row| {
            row.try_get::<String, _>("idempotency_key")
                .expect("idempotency key")
        })
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(idempotency_keys.len(), 3);
    let desired_states = command_rows
        .iter()
        .map(|row| {
            row.try_get::<Value, _>("payload").expect("command payload")["desired_is_read"]
                .as_bool()
                .expect("desired read state")
        })
        .collect::<Vec<_>>();
    assert_eq!(desired_states, vec![true, false, true]);

    let stored_is_read: bool =
        sqlx::query_scalar("SELECT is_read FROM communication_messages WHERE message_id = $1")
            .bind(&message_id)
            .fetch_one(&pool)
            .await
            .expect("stored read state");
    assert!(stored_is_read);

    let read_list = r
        .oneshot(get(&format!(
            "/api/v1/communications/messages?account_id={account_id}&is_read=true"
        )))
        .await
        .expect("read-only message list");
    assert_eq!(read_list.status(), StatusCode::OK);
    let read_list = response_json(read_list).await;
    assert_eq!(read_list["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(read_list["items"][0]["message_id"], message_id);
    assert_eq!(read_list["items"][0]["is_read"], true);
    assert_eq!(read_list["items"][0]["read_sync_status"], "queued");
}

#[tokio::test]
async fn v1_delete_draft() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let draft_id = format!("draft:fake-{}", uid());
    let r = router(&db).await;
    let response = r
        .oneshot(delete(&format!("/api/v1/communications/drafts/{draft_id}")))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "delete draft={}",
        response.status()
    );
}

#[tokio::test]
async fn v1_delete_message_label() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let r = router(&db).await;
    let response = r
        .oneshot(delete("/api/v1/communications/messages/msg:fake/labels"))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "delete label={}",
        response.status()
    );
}

#[tokio::test]
async fn v1_imap_delete_alias_moves_message_to_local_trash_against_postgres() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let database = Database::connect(Some(&db)).await.expect("database");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let account_id = format!("acct-local-trash-api-{suffix}");
    let message_id = seed_projected_message(
        pool,
        &account_id,
        &format!("provider-local-trash-api-{suffix}"),
        "Local trash API",
    )
    .await;

    let r = router(&db).await;
    let response = r
        .clone()
        .oneshot(post(
            &format!("/api/v1/communications/messages/{message_id}/imap-delete"),
            json!({}),
        ))
        .await
        .expect("imap delete alias");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["deleted"], true);
    assert_eq!(body["local_state"], "trash");

    let response = r
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/messages?account_id={account_id}&q=Local%20trash%20API"
        )))
        .await
        .expect("active list");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["items"].as_array().expect("items").len(), 0);

    let response = r
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/messages?account_id={account_id}&q=Local%20trash%20API&local_state=trash"
        )))
        .await
        .expect("trash list");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["items"].as_array().expect("items").len(), 1);
    assert_eq!(body["items"][0]["local_state"], "trash");

    let response = r
        .oneshot(post(
            &format!("/api/v1/communications/messages/{message_id}/restore"),
            json!({}),
        ))
        .await
        .expect("restore local trash");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["local_state"], "active");
}

#[tokio::test]
async fn v1_redirect_message_enqueues_outbox_redirect_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-redirect-api-{suffix}");
    let message_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-redirect-api-{suffix}"),
        "Redirect original subject",
    )
    .await;

    let r = router(&context.connection_string()).await;
    let response = r
        .oneshot(post(
            &format!("/api/v1/communications/messages/{message_id}/redirect"),
            json!({
                "to": ["redirect@example.com"],
                "cc": ["copy@example.com"],
                "confirmed_provider_write": true
            }),
        ))
        .await
        .expect("redirect response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["transport"], "outbox");
    assert_eq!(body["status"], "queued");
    assert_eq!(body["accepted_recipients"], json!(["redirect@example.com"]));
    let outbox_id = body["outbox_id"].as_str().expect("outbox id");

    let row = sqlx::query(
        r#"
        SELECT
            account_id,
            to_participants AS to_recipients,
            cc_participants AS cc_recipients,
            subject,
            body_text,
            metadata
        FROM communication_outbox
        WHERE outbox_id = $1
        "#,
    )
    .bind(outbox_id)
    .fetch_one(&pool)
    .await
    .expect("outbox row");
    assert_eq!(row.try_get::<String, _>("account_id").unwrap(), account_id);
    assert_eq!(
        row.try_get::<String, _>("subject").unwrap(),
        "Redirect original subject"
    );
    assert_eq!(
        row.try_get::<String, _>("body_text").unwrap(),
        "Body for local trash API"
    );
    assert_eq!(
        row.try_get::<Value, _>("to_recipients").unwrap(),
        json!(["redirect@example.com"])
    );
    assert_eq!(
        row.try_get::<Value, _>("cc_recipients").unwrap(),
        json!(["copy@example.com"])
    );
    let metadata = row.try_get::<Value, _>("metadata").unwrap();
    assert_eq!(metadata["redirect_of"], message_id);
    assert_eq!(metadata["redirect_mode"], "resent");
    assert_eq!(metadata["original_sender"], "sender@example.com");
    let link = sqlx::query(
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
    .expect("redirect outbox observation link");
    let observation_id: String = link.try_get("observation_id").expect("observation id");
    let link_metadata: Value = link.try_get("metadata").expect("link metadata");
    assert_eq!(link_metadata["operation"], "outbox_redirect_enqueue");
    assert_eq!(link_metadata["status"], "queued");
    assert_eq!(link_metadata["redirect_of"], message_id);
    let observation = sqlx::query(
        "SELECT origin_kind, payload
         FROM observations
         WHERE observation_id = $1",
    )
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("redirect outbox observation");
    let origin_kind: String = observation.try_get("origin_kind").expect("origin kind");
    let payload: Value = observation.try_get("payload").expect("payload");
    assert_eq!(origin_kind, "manual");
    assert_eq!(payload["operation"], "outbox_redirect_enqueue");
}

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

async fn seed_projected_message(
    pool: sqlx::PgPool,
    account_id: &str,
    provider_record_id: &str,
    subject: &str,
) -> String {
    seed_projected_message_with_body(
        pool,
        account_id,
        provider_record_id,
        subject,
        "Body for local trash API",
    )
    .await
}

async fn seed_projected_message_with_body(
    pool: sqlx::PgPool,
    account_id: &str,
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
) -> String {
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let message_store = MessageProjectionStore::new(pool);
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            account_id,
            CommunicationProviderKind::Gmail,
            "Seed Gmail",
            format!("{account_id}@example.com"),
        ))
        .await
        .expect("store provider account");
    let raw = communication_store
        .record_raw_source(&NewRawCommunicationRecord::new(
            format!("raw-{provider_record_id}"),
            account_id,
            "email_message",
            provider_record_id,
            format!("sha256:{provider_record_id}"),
            format!("batch-{provider_record_id}"),
            json!({
                "subject": subject,
                "from": "sender@example.com",
                "to": ["recipient@example.com"],
                "body_text": body_text
            }),
        ))
        .await
        .expect("record raw source");
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}
