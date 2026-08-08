use makosh_backend_testkit::context::TestContext;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::{build_router, build_router_with_database};
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;

use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::storage::database::Database;

const T: &str = "v1-workflow-action-test-token";

fn cfg() -> AppConfig {
    makosh_backend_testkit::app::config_with_secret(T)
}

fn post_with_actor(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .header("x-makosh-actor-id", "makosh-frontend")
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

async fn response_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

#[tokio::test]
async fn workflow_action_endpoint_exists_without_database() {
    let app = build_router(cfg());
    let response = app
        .oneshot(post_with_actor(
            "/api/v1/workflow-actions",
            json!({
                "command_id": "workflow-action-no-db",
                "action": "reply",
                "source": { "kind": "communication_message", "id": "msg:no-db" }
            }),
        ))
        .await
        .expect("workflow action no-db response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn v1_put_workflow_state_captures_observation_trail() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let database = Database::connect(Some(&db)).await.expect("database");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let message_id = seed_projected_message(
        pool.clone(),
        &format!("acct-workflow-state-{suffix}"),
        &format!("provider-workflow-state-{suffix}"),
        &format!("Workflow state {suffix}"),
    )
    .await;
    let r = router(&db).await;
    let response = r
        .oneshot(put(
            &format!("/api/v1/communications/messages/{message_id}/workflow-state"),
            json!({"workflow_state": "reviewed"}),
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["message_id"], json!(message_id));
    assert_eq!(body["workflow_state"], "reviewed");
    assert_eq!(body["previous_state"], "new");

    let workflow_state: String = sqlx::query_scalar(
        "SELECT workflow_state FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("workflow state");
    assert_eq!(workflow_state, "reviewed");

    let row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'communication_message'
           AND entity_id = $1
           AND relationship_kind = 'workflow_state_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("workflow state observation link");
    let observation_id: String = row.try_get("observation_id").expect("observation id");
    let metadata: Value = row.try_get("metadata").expect("metadata");
    assert_eq!(metadata["previous_state"], "new");
    assert_eq!(metadata["workflow_state"], "reviewed");

    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("observation origin");
    assert_eq!(origin_kind, "manual");
}

#[tokio::test]
async fn workflow_action_create_task_is_idempotent_and_records_safe_event() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let database = Database::connect(Some(&db)).await.expect("database");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let message_id = seed_projected_message(
        pool.clone(),
        &format!("acct-workflow-action-{suffix}"),
        &format!("provider-workflow-action-{suffix}"),
        &format!("Workflow action task {suffix}"),
    )
    .await;
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message observation id");
    let r = router(&db).await;
    let command_id = format!("workflow-action-task-{suffix}");
    let body = json!({
        "command_id": command_id,
        "action": "create_task",
        "source": { "kind": "communication_message", "id": message_id },
        "input": { "title": "Confirm integration access" }
    });

    let first = r
        .clone()
        .oneshot(post_with_actor("/api/v1/workflow-actions", body.clone()))
        .await
        .expect("first workflow action response");
    assert_eq!(first.status(), StatusCode::OK);
    let first_body = response_json(first).await;
    assert_eq!(
        first_body["event_id"],
        json!(format!("workflow_action:{command_id}"))
    );
    assert_eq!(first_body["target"]["kind"], "task");
    assert_eq!(first_body["provenance"]["source_id"], message_id);

    let second = r
        .oneshot(post_with_actor("/api/v1/workflow-actions", body))
        .await
        .expect("second workflow action response");
    assert_eq!(second.status(), StatusCode::OK);
    let second_body = response_json(second).await;
    assert_eq!(second_body["target"], first_body["target"]);

    let task_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM tasks WHERE source_id = $1 AND source_type = 'observation' AND source_kind = 'observation'",
    )
    .bind(&message_observation_id)
    .fetch_one(&pool)
    .await
    .expect("task count");
    assert_eq!(task_count, 1);

    let task_id = first_body["target"]["id"]
        .as_str()
        .expect("task id")
        .to_owned();

    let task_create_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'tasks'
          AND entity_kind = 'task'
          AND entity_id = $2
          AND relationship_kind = 'task_create'
        "#,
    )
    .bind(&message_observation_id)
    .bind(&task_id)
    .fetch_one(&pool)
    .await
    .expect("task create observation links");
    assert_eq!(task_create_link_count, 1);

    let workflow_projection_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'tasks'
          AND entity_kind = 'task'
          AND entity_id = $2
          AND relationship_kind = 'workflow_action_projection'
        "#,
    )
    .bind(&message_observation_id)
    .bind(&task_id)
    .fetch_one(&pool)
    .await
    .expect("task workflow projection observation links");
    assert_eq!(workflow_projection_link_count, 1);

    let event_payload: Value =
        sqlx::query_scalar("SELECT payload FROM event_log WHERE event_id = $1")
            .bind(format!("workflow_action:{command_id}"))
            .fetch_one(&pool)
            .await
            .expect("workflow event payload");
    assert!(
        !event_payload
            .to_string()
            .contains("Body for local trash API")
    );
}

#[tokio::test]
async fn workflow_action_create_persona_reuses_message_observation_for_persona_projection() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let database = Database::connect(Some(&db)).await.expect("database");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let message_id = seed_projected_message(
        pool.clone(),
        &format!("acct-workflow-persona-{suffix}"),
        &format!("provider-workflow-persona-{suffix}"),
        &format!("Workflow action persona {suffix}"),
    )
    .await;
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message observation id");
    let r = router(&db).await;
    let command_id = format!("workflow-action-persona-{suffix}");

    let response = r
        .oneshot(post_with_actor(
            "/api/v1/workflow-actions",
            json!({
                "command_id": command_id,
                "action": "create_persona",
                "source": { "kind": "communication_message", "id": message_id }
            }),
        ))
        .await
        .expect("workflow persona response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["target"]["kind"], "persona");
    let persona_id = body["target"]["id"]
        .as_str()
        .expect("persona id")
        .to_owned();

    let persona_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'personas'
          AND entity_kind = 'persona'
          AND entity_id = $2
          AND relationship_kind = 'workflow_action_projection'
        "#,
    )
    .bind(&message_observation_id)
    .bind(&persona_id)
    .fetch_one(&pool)
    .await
    .expect("persona workflow action observation links");
    assert_eq!(persona_link_count, 1);

    let identity_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'personas'
          AND entity_kind = 'identity'
          AND relationship_kind = 'workflow_action_projection'
        "#,
    )
    .bind(&message_observation_id)
    .fetch_one(&pool)
    .await
    .expect("identity workflow action observation links");
    assert_eq!(identity_link_count, 1);
}

#[tokio::test]
async fn workflow_action_create_persona_can_create_persona_without_email() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let database = Database::connect(Some(&db)).await.expect("database");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let r = router(&db).await;
    let command_id = format!("workflow-action-persona-no-email-{suffix}");
    let display_name = format!("Phone Only Persona {suffix}");

    let response = r
        .oneshot(post_with_actor(
            "/api/v1/workflow-actions",
            json!({
                "command_id": command_id,
                "action": "create_persona",
                "input": {
                    "display_name": display_name
                }
            }),
        ))
        .await
        .expect("workflow persona without email response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["target"]["kind"], "persona");
    let persona_id = body["target"]["id"]
        .as_str()
        .expect("persona id")
        .to_owned();

    let row: (String, Option<String>, bool) = sqlx::query_as(
        "SELECT display_name, email_address, is_address_book FROM personas WHERE persona_id = $1",
    )
    .bind(&persona_id)
    .fetch_one(&pool)
    .await
    .expect("persona without email");
    assert_eq!(row.0, display_name);
    assert_eq!(row.1, None);
    assert!(!row.2);

    let identity_count: i64 =
        sqlx::query_scalar("SELECT count(*)::BIGINT FROM persona_identities WHERE persona_id = $1")
            .bind(&persona_id)
            .fetch_one(&pool)
            .await
            .expect("persona identities");
    assert_eq!(identity_count, 0);

    let persona_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE domain = 'personas'
          AND entity_kind = 'persona'
          AND entity_id = $1
          AND relationship_kind = 'workflow_action_projection'
        "#,
    )
    .bind(&persona_id)
    .fetch_one(&pool)
    .await
    .expect("persona workflow action observation links");
    assert_eq!(persona_link_count, 1);
}

#[tokio::test]
async fn workflow_action_create_note_creates_markdown_document() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let database = Database::connect(Some(&db)).await.expect("database");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let r = router(&db).await;
    let command_id = format!("workflow-action-note-{suffix}");

    let response = r
        .oneshot(post_with_actor(
            "/api/v1/workflow-actions",
            json!({
                "command_id": command_id,
                "action": "create_note",
                "input": {
                    "title": "Follow-up note",
                    "body": "Remember to verify keys with the integration owner."
                }
            }),
        ))
        .await
        .expect("workflow note response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["target"]["kind"], "document");
    let document_id = body["target"]["id"].as_str().expect("document id");
    let document_kind: String =
        sqlx::query_scalar("SELECT document_kind FROM documents WHERE document_id = $1")
            .bind(document_id)
            .fetch_one(&pool)
            .await
            .expect("document kind");
    assert_eq!(document_kind, "markdown");

    let document_origin_kind: String = sqlx::query_scalar(
        "SELECT observation.origin_kind
         FROM documents
         JOIN observations observation
           ON observation.observation_id = documents.observation_id
         WHERE documents.document_id = $1",
    )
    .bind(document_id)
    .fetch_one(&pool)
    .await
    .expect("document observation origin kind");
    assert_eq!(document_origin_kind, "manual");

    let import_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE domain = 'documents'
          AND entity_kind = 'document'
          AND entity_id = $1
          AND relationship_kind = 'import'
        "#,
    )
    .bind(document_id)
    .fetch_one(&pool)
    .await
    .expect("document import links");
    assert_eq!(import_link_count, 1);
}

#[tokio::test]
async fn workflow_action_link_document_reuses_message_observation_for_document_projection() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let database = Database::connect(Some(&db)).await.expect("database");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let message_id = seed_projected_message(
        pool.clone(),
        &format!("acct-workflow-link-doc-{suffix}"),
        &format!("provider-workflow-link-doc-{suffix}"),
        &format!("Workflow link document {suffix}"),
    )
    .await;
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message observation id");
    let r = router(&db).await;
    let command_id = format!("workflow-action-link-document-{suffix}");

    let response = r
        .oneshot(post_with_actor(
            "/api/v1/workflow-actions",
            json!({
                "command_id": command_id,
                "action": "link_document",
                "source": { "kind": "communication_message", "id": message_id }
            }),
        ))
        .await
        .expect("workflow link document response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["target"]["kind"], "document");
    let document_id = body["target"]["id"]
        .as_str()
        .expect("document id")
        .to_owned();

    let source_projection_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'documents'
          AND entity_kind = 'document'
          AND entity_id = $2
          AND relationship_kind = 'workflow_action_projection'
        "#,
    )
    .bind(&message_observation_id)
    .bind(&document_id)
    .fetch_one(&pool)
    .await
    .expect("workflow document projection links");
    assert_eq!(source_projection_link_count, 1);

    let document_origin_kind: String = sqlx::query_scalar(
        "SELECT observation.origin_kind
         FROM documents
         JOIN observations observation
           ON observation.observation_id = documents.observation_id
         WHERE documents.document_id = $1",
    )
    .bind(&document_id)
    .fetch_one(&pool)
    .await
    .expect("linked document observation origin kind");
    assert_eq!(document_origin_kind, "manual");
}

#[tokio::test]
async fn workflow_action_create_event_requires_start_and_end() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let r = router(&db).await;
    let response = r
        .oneshot(post_with_actor(
            "/api/v1/workflow-actions",
            json!({
                "command_id": format!("workflow-action-event-missing-{}", uid()),
                "action": "create_event",
                "input": { "title": "Missing time event" }
            }),
        ))
        .await
        .expect("workflow event validation response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn workflow_action_create_event_reuses_message_observation_for_calendar_projection() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let database = Database::connect(Some(&db)).await.expect("database");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let message_id = seed_projected_message(
        pool.clone(),
        &format!("acct-workflow-event-{suffix}"),
        &format!("provider-workflow-event-{suffix}"),
        &format!("Workflow action event {suffix}"),
    )
    .await;
    let message_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("message observation id");
    let r = router(&db).await;

    let response = r
        .oneshot(post_with_actor(
            "/api/v1/workflow-actions",
            json!({
                "command_id": format!("workflow-action-event-{suffix}"),
                "action": "create_event",
                "source": { "kind": "communication_message", "id": message_id },
                "input": {
                    "title": "Follow-up meeting",
                    "body": "Discuss imported communication context",
                    "starts_at": "2026-07-01T10:00:00Z",
                    "ends_at": "2026-07-01T11:00:00Z"
                }
            }),
        ))
        .await
        .expect("workflow event response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["target"]["kind"], "calendar_event");
    let event_id = body["target"]["id"].as_str().expect("event id").to_owned();

    let projection_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'calendar'
          AND entity_kind = 'event'
          AND entity_id = $2
          AND relationship_kind = 'workflow_action_projection'
        "#,
    )
    .bind(&message_observation_id)
    .bind(&event_id)
    .fetch_one(&pool)
    .await
    .expect("workflow event projection links");
    assert_eq!(projection_link_count, 1);
}

#[tokio::test]
async fn workflow_action_archive_transitions_message_locally() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let database = Database::connect(Some(&db)).await.expect("database");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = uid();
    let message_id = seed_projected_message(
        pool.clone(),
        &format!("acct-workflow-archive-{suffix}"),
        &format!("provider-workflow-archive-{suffix}"),
        &format!("Workflow archive {suffix}"),
    )
    .await;
    let r = router(&db).await;

    let response = r
        .oneshot(post_with_actor(
            "/api/v1/workflow-actions",
            json!({
                "command_id": format!("workflow-action-archive-{suffix}"),
                "action": "archive",
                "source": { "kind": "communication_message", "id": message_id }
            }),
        ))
        .await
        .expect("workflow archive response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["status"], "archived");
    let workflow_state: String = sqlx::query_scalar(
        "SELECT workflow_state FROM communication_messages WHERE message_id = $1",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("workflow state");
    assert_eq!(workflow_state, "archived");
    let provider_command: (String, String, Value) = sqlx::query_as(
        r#"
        SELECT command_kind, actor_id, target_ref
        FROM communication_provider_commands
        WHERE target_ref->>'message_id' = $1
          AND channel_kind = 'mail'
        "#,
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("archive provider command");
    assert_eq!(provider_command.0, "archive");
    assert_eq!(provider_command.1, "makosh-frontend");
    assert_eq!(provider_command.2["message_id"], message_id);
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
                "body_text": "Body for local trash API"
            }),
        ))
        .await
        .expect("record raw source");
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}
