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
use makosh_hub_backend::domains::communications::ai_state::{
    CommunicationAiState, CommunicationAiStateStore, MAIL_AI_MAX_ATTEMPTS,
};
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::storage::database::Database;

const T: &str = "v1comms-ai-state-test-token";

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
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

#[tokio::test]
async fn v1_message_ai_state_transitions_are_durable_and_emit_event_against_postgres() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-ai-state-api-{suffix}");
    let message_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-ai-state-api-{suffix}"),
        "AI state transition",
    )
    .await;

    let r = router(&context.connection_string()).await;
    let response = r
        .clone()
        .oneshot(get(&format!(
            "/api/v1/communications/messages/{message_id}/ai-state"
        )))
        .await
        .expect("initial ai state response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["message_id"], message_id);
    assert_eq!(body["ai_state"], "NEW");

    let response = r
        .clone()
        .oneshot(put(
            &format!("/api/v1/communications/messages/{message_id}/ai-state"),
            json!({"ai_state": "PROCESSING"}),
        ))
        .await
        .expect("transition ai state response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["message_id"], message_id);
    assert_eq!(body["ai_state"], "PROCESSING");
    assert!(body["updated_at"].is_string());
    let observation = sqlx::query(
        r#"
        SELECT kind.code AS kind_code,
               observation.origin_kind,
               observation.payload,
               link.relationship_kind
        FROM observations observation
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        JOIN observation_links link
          ON link.observation_id = observation.observation_id
        WHERE link.domain = 'communications'
          AND link.entity_kind = 'communication_message'
          AND link.entity_id = $1
          AND link.relationship_kind = 'ai_state_transition'
        ORDER BY observation.captured_at DESC
        LIMIT 1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("ai state observation");
    assert_eq!(
        observation.try_get::<String, _>("kind_code").unwrap(),
        "COMMUNICATION_MESSAGE"
    );
    assert_eq!(
        observation.try_get::<String, _>("origin_kind").unwrap(),
        "manual"
    );
    let observation_payload = observation.try_get::<Value, _>("payload").unwrap();
    assert_eq!(observation_payload["message_id"], message_id);
    assert_eq!(observation_payload["previous_ai_state"], "NEW");
    assert_eq!(observation_payload["request"]["ai_state"], "PROCESSING");

    let persisted = sqlx::query(
        r#"
        SELECT ai_state, review_reason, last_error
        FROM communication_ai_states
        WHERE message_id = $1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("persisted ai state");
    assert_eq!(
        persisted.try_get::<String, _>("ai_state").unwrap(),
        "PROCESSING"
    );
    assert!(
        persisted
            .try_get::<Option<String>, _>("review_reason")
            .unwrap()
            .is_none()
    );
    assert!(
        persisted
            .try_get::<Option<String>, _>("last_error")
            .unwrap()
            .is_none()
    );

    let event = sqlx::query(
        r#"
        SELECT subject, payload
        FROM event_log
        WHERE event_type = 'mail.ai_state.changed'
          AND subject->>'kind' = 'mail_ai_state'
          AND subject->>'id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("ai state event");
    let subject = event.try_get::<Value, _>("subject").unwrap();
    let payload = event.try_get::<Value, _>("payload").unwrap();
    assert_eq!(subject["message_id"], message_id);
    assert_eq!(payload["ai_state"], "PROCESSING");
    assert_eq!(payload["previous_ai_state"], "NEW");
    assert!(payload.get("body_text").is_none());

    let failed_response = r
        .clone()
        .oneshot(put(
            &format!("/api/v1/communications/messages/{message_id}/ai-state"),
            json!({"ai_state": "FAILED", "last_error": "AI runtime unavailable"}),
        ))
        .await
        .expect("failed ai state response");
    assert_eq!(failed_response.status(), StatusCode::OK);

    let retry_response = r
        .clone()
        .oneshot(put(
            &format!("/api/v1/communications/messages/{message_id}/ai-state"),
            json!({"ai_state": "NEW"}),
        ))
        .await
        .expect("retry ai state response");
    assert_eq!(retry_response.status(), StatusCode::OK);
    let retry_body = response_json(retry_response).await;
    assert_eq!(retry_body["ai_state"], "NEW");
    assert_eq!(retry_body["last_error"], Value::Null);

    let retry_event = sqlx::query(
        r#"
        SELECT payload
        FROM event_log
        WHERE event_type = 'mail.ai_state.changed'
          AND subject->>'id' = $1
        ORDER BY position DESC
        LIMIT 1
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("retry ai state event");
    let retry_payload = retry_event.try_get::<Value, _>("payload").unwrap();
    assert_eq!(retry_payload["ai_state"], "NEW");
    assert_eq!(retry_payload["previous_ai_state"], "FAILED");

    let response = r
        .oneshot(get(&format!(
            "/api/v1/communications/messages/{message_id}/ai-state"
        )))
        .await
        .expect("current ai state response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["ai_state"], "NEW");
}

#[tokio::test]
async fn mail_ai_worker_claims_due_retries_and_recovers_expired_leases() {
    let context = TestContext::new().await;
    let pool = context.pool().clone();
    let suffix = uid();
    let account_id = format!("acct-ai-state-lifecycle-{suffix}");
    let message_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-ai-state-lifecycle-{suffix}"),
        "AI lifecycle",
    )
    .await;
    let store = CommunicationAiStateStore::new(pool.clone());
    let now = chrono::Utc::now();

    assert_eq!(
        store
            .claim_due_mail_messages(10, now)
            .await
            .expect("claim new message"),
        vec![message_id.clone()]
    );
    let claimed = store
        .current(&message_id)
        .await
        .expect("read claimed state")
        .expect("claimed state exists");
    assert_eq!(claimed.ai_state, CommunicationAiState::Processing);
    assert!(claimed.processing_lease_expires_at.is_some());

    let first_failure = store
        .record_mail_processing_failure(&message_id, "temporary runtime failure", true, now)
        .await
        .expect("record first retryable failure")
        .expect("failure state exists");
    assert_eq!(first_failure.ai_state, CommunicationAiState::Failed);
    assert_eq!(first_failure.retry_count, 1);
    let retry_at = first_failure
        .next_attempt_at
        .expect("retryable failure is scheduled");
    assert!(
        store
            .claim_due_mail_messages(10, now)
            .await
            .expect("claim before due")
            .is_empty()
    );
    assert_eq!(
        store
            .claim_due_mail_messages(10, retry_at)
            .await
            .expect("claim scheduled retry"),
        vec![message_id.clone()]
    );

    for attempt in 2..=MAIL_AI_MAX_ATTEMPTS {
        let failure = store
            .record_mail_processing_failure(
                &message_id,
                "temporary runtime failure",
                true,
                retry_at + chrono::Duration::seconds(i64::from(attempt)),
            )
            .await
            .expect("record bounded failure")
            .expect("failure state exists");
        assert_eq!(failure.retry_count, attempt);
        if attempt == MAIL_AI_MAX_ATTEMPTS {
            assert!(failure.next_attempt_at.is_none());
        } else {
            let due = failure
                .next_attempt_at
                .expect("intermediate retry is scheduled");
            assert_eq!(
                store
                    .claim_due_mail_messages(10, due)
                    .await
                    .expect("claim intermediate retry"),
                vec![message_id.clone()]
            );
        }
    }

    let lease_message_id = seed_projected_message(
        pool.clone(),
        &account_id,
        &format!("provider-ai-state-lease-{suffix}"),
        "AI lease recovery",
    )
    .await;
    assert_eq!(
        store
            .claim_due_mail_messages(10, now)
            .await
            .expect("claim lease message"),
        vec![lease_message_id.clone()]
    );
    sqlx::query(
        "UPDATE communication_ai_states SET processing_lease_expires_at = $2 WHERE message_id = $1",
    )
    .bind(&lease_message_id)
    .bind(now - chrono::Duration::seconds(1))
    .execute(&pool)
    .await
    .expect("expire processing lease");

    assert_eq!(
        store
            .recover_expired_mail_processing(now)
            .await
            .expect("recover expired lease"),
        1
    );
    let recovered = store
        .current(&lease_message_id)
        .await
        .expect("read recovered state")
        .expect("recovered state exists");
    assert_eq!(recovered.ai_state, CommunicationAiState::Failed);
    assert_eq!(recovered.retry_count, 1);
    assert!(recovered.next_attempt_at.is_some());
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
                "body_text": "Private body that must not be emitted in AI state events"
            }),
        ))
        .await
        .expect("record raw source");
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}
