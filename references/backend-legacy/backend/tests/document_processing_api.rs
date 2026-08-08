use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use makosh_events_postgres::store::EventStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::documents::core::{
    models::NewDocumentImport, store::DocumentImportStore,
};
use makosh_hub_backend::domains::documents::processing::{
    models::DocumentProcessingStep, store::DocumentProcessingStore,
};
use makosh_hub_backend::platform::storage::database::Database;
use serde_json::Value;
use sqlx::query_scalar;
use tower::ServiceExt;

const LOCAL_API_TOKEN: &str = "document-processing-api-test-token";

#[tokio::test]
async fn get_document_processing_jobs_rejects_missing_local_api_secret() {
    let app = makosh_hub_backend::app::router::build_router(
        makosh_backend_testkit::app::config_with_secret(LOCAL_API_TOKEN),
    );

    let response = app
        .oneshot(get_request("/api/v1/document-processing/jobs"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = json_body(response).await;
    assert_eq!(
        body,
        serde_json::json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn get_document_processing_for_missing_document_returns_404() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );
    let missing_document_id = format!("doc_processing_api_missing_{:x}", unique_suffix());

    let response = app
        .oneshot(get_request_with_actor(
            &format!("/api/v1/documents/{missing_document_id}/processing"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json_body(response).await;
    assert_eq!(
        body,
        serde_json::json!({
            "error": "document_processing_store_error",
            "message": "document processing job was not found"
        })
    );
}

#[tokio::test]
async fn document_processing_api_returns_expected_payloads() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let document_id = format!("doc_processing_api_{:x}", unique_suffix());

    let document_store = DocumentImportStore::new(pool.clone());
    document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            "pipeline api doc",
            "# Pipeline API\nMarkdown body for API test.",
        ))
        .await
        .expect("import markdown document");

    let processing_store = DocumentProcessingStore::new(pool.clone());
    processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue jobs");

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );

    let jobs_response = app
        .clone()
        .oneshot(get_request_with_actor(
            "/api/v1/document-processing/jobs?limit=10",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(jobs_response.status(), StatusCode::OK);
    let jobs_body = json_body(jobs_response).await;
    let items = jobs_body["items"]
        .as_array()
        .expect("document processing jobs");
    assert!(!items.is_empty());
    let has_target = items
        .iter()
        .any(|item| item["document_id"] == Value::String(document_id.clone()));
    assert!(has_target, "jobs should include enqueued document");

    let detail_response = app
        .oneshot(get_request_with_actor(
            &format!("/api/v1/documents/{document_id}/processing"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(detail_response.status(), StatusCode::OK);
    let detail_body = json_body(detail_response).await;
    assert_eq!(
        detail_body["document_id"],
        Value::String(document_id.clone())
    );
    assert!(detail_body["jobs"].is_array());
    assert!(detail_body["jobs"].as_array().unwrap().len() >= 2);

    let _ = query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM document_processing_jobs WHERE document_id = $1)",
    )
    .bind(&document_id)
    .fetch_one(&pool)
    .await
    .expect("jobs inserted for processing document");
}

#[tokio::test]
async fn post_document_processing_job_retry_requeues_failed_job() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_api_retry_{suffix:x}");

    let document_store = DocumentImportStore::new(pool.clone());
    document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            "retry api doc",
            "# Retry API\nMarkdown body for retry API test.",
        ))
        .await
        .expect("import markdown document");

    let processing_store = DocumentProcessingStore::new(pool.clone());
    let jobs = processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue jobs");
    let extract_job = jobs
        .iter()
        .find(|job| job.step == DocumentProcessingStep::ExtractText)
        .expect("extract text job");

    sqlx::query(
        r#"
        UPDATE document_processing_jobs
        SET status = 'failed',
            attempts = 2,
            last_error_summary = 'temporary extractor failure',
            started_at = now(),
            finished_at = now(),
            updated_at = now()
        WHERE job_id = $1
        "#,
    )
    .bind(&extract_job.job_id)
    .execute(&pool)
    .await
    .expect("mark extract job failed");

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );
    let command_id = format!("document-processing-retry-{suffix:x}");
    let retry_path = format!(
        "/api/v1/document-processing/jobs/{}/retry",
        extract_job.job_id
    );
    let request_body = serde_json::json!({ "command_id": command_id });

    let retry_response = app
        .oneshot(post_json_request(
            &retry_path,
            LOCAL_API_TOKEN,
            request_body.clone(),
        ))
        .await
        .expect("retry response");
    assert_eq!(retry_response.status(), StatusCode::OK);
    let retry_body = json_body(retry_response).await;
    assert_eq!(
        retry_body,
        serde_json::json!({
            "job_id": extract_job.job_id,
            "status": "queued",
            "event_id": format!("document_processing_retry:{}", request_body["command_id"].as_str().unwrap())
        })
    );
    let audit_record =
        sqlx::query_as::<_, (String, String, String, String, String, Option<String>)>(
            r#"
            SELECT operation, actor_id, method, path_template, target_kind, target_id
            FROM api_audit_log
            WHERE target_kind = 'document_processing_job'
              AND target_id = $1
            ORDER BY audit_id ASC
            "#,
        )
        .bind(&extract_job.job_id)
        .fetch_one(&pool)
        .await
        .expect("document processing retry audit record");
    assert_eq!(audit_record.0, "document_processing.job.retry");
    assert_eq!(audit_record.1, "makosh-frontend");
    assert_eq!(audit_record.2, "POST");
    assert_eq!(
        audit_record.3,
        "/api/v1/document-processing/jobs/{job_id}/retry"
    );
    assert_eq!(audit_record.4, "document_processing_job");
    assert_eq!(audit_record.5.as_deref(), Some(extract_job.job_id.as_str()));

    let retry_observation_link_count: i64 = query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'documents'
           AND entity_kind = 'document_processing_job'
           AND entity_id = $1
           AND relationship_kind = 'retry_command'",
    )
    .bind(&extract_job.job_id)
    .fetch_one(&pool)
    .await
    .expect("document processing retry observation link count");
    assert_eq!(retry_observation_link_count, 1);
    let retry_observation_kind: String = query_scalar(
        r#"
        SELECT kinds.code
        FROM observation_links links
        JOIN observations observation
          ON observation.observation_id = links.observation_id
        JOIN observation_kind_definitions kinds
          ON kinds.kind_definition_id = observation.kind_definition_id
        WHERE links.domain = 'documents'
          AND links.entity_kind = 'document_processing_job'
          AND links.entity_id = $1
          AND links.relationship_kind = 'retry_command'
        ORDER BY links.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&extract_job.job_id)
    .fetch_one(&pool)
    .await
    .expect("document processing retry observation kind");
    assert_eq!(retry_observation_kind, "DOCUMENT_PROCESSING_JOB_STATUS");
    quiesce_processing_jobs_for_document(&pool, &document_id).await;
}

#[tokio::test]
async fn post_document_processing_job_retry_rejects_non_failed_job_with_stable_body() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_api_retry_non_failed_{suffix:x}");

    let document_store = DocumentImportStore::new(pool.clone());
    document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            "retry api non failed doc",
            "# Retry API\nMarkdown body for non-failed retry API test.",
        ))
        .await
        .expect("import markdown document");

    let processing_store = DocumentProcessingStore::new(pool.clone());
    let jobs = processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue jobs");
    let extract_job = jobs
        .iter()
        .find(|job| job.step == DocumentProcessingStep::ExtractText)
        .expect("extract text job");

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );
    let retry_path = format!(
        "/api/v1/document-processing/jobs/{}/retry",
        extract_job.job_id
    );

    let response = app
        .oneshot(post_json_request(
            &retry_path,
            LOCAL_API_TOKEN,
            serde_json::json!({
                "command_id": format!("document-processing-retry-non-failed-{suffix:x}")
            }),
        ))
        .await
        .expect("retry response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(
        body,
        serde_json::json!({
            "error": "document_processing_store_error",
            "message": "document processing retry requires a failed job"
        })
    );
    quiesce_processing_jobs_for_document(&pool, &document_id).await;
}

#[tokio::test]
async fn post_document_processing_job_retry_command_collision_returns_stable_conflict() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let existing_document_id = format!("doc_processing_api_retry_collision_existing_{suffix:x}");
    let target_document_id = format!("doc_processing_api_retry_collision_target_{suffix:x}");
    let document_store = DocumentImportStore::new(pool.clone());
    let processing_store = DocumentProcessingStore::new(pool.clone());
    let existing_job_id = create_failed_extract_text_job(
        &pool,
        &document_store,
        &processing_store,
        &existing_document_id,
    )
    .await;
    let target_job_id = create_failed_extract_text_job(
        &pool,
        &document_store,
        &processing_store,
        &target_document_id,
    )
    .await;
    let command_id = format!("document-processing-retry-api-collision-{suffix:x}");
    append_retry_event_for_job(&pool, &command_id, &existing_job_id).await;

    let app = build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(
            LOCAL_API_TOKEN,
            database_url.as_str(),
        ),
        database,
    );
    let retry_path = format!("/api/v1/document-processing/jobs/{target_job_id}/retry");

    let response = app
        .oneshot(post_json_request(
            &retry_path,
            LOCAL_API_TOKEN,
            serde_json::json!({ "command_id": command_id }),
        ))
        .await
        .expect("retry response");

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body = json_body(response).await;
    assert_eq!(
        body,
        serde_json::json!({
            "error": "document_processing_store_error",
            "message": "document processing retry command conflicts with existing event"
        })
    );
    let target_status: String =
        query_scalar("SELECT status FROM document_processing_jobs WHERE job_id = $1")
            .bind(&target_job_id)
            .fetch_one(&pool)
            .await
            .expect("target status");
    assert_eq!(target_status, "failed");

    quiesce_processing_jobs_for_document(&pool, &existing_document_id).await;
    quiesce_processing_jobs_for_document(&pool, &target_document_id).await;
}

fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

fn get_request_with_actor(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

fn post_json_request(uri: &str, token: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("x-makosh-secret", token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    serde_json::from_slice(&body).expect("json body")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

async fn quiesce_processing_jobs_for_document(pool: &sqlx::postgres::PgPool, document_id: &str) {
    sqlx::query(
        r#"
        UPDATE document_processing_jobs
        SET status = 'skipped',
            last_error_summary = COALESCE(last_error_summary, 'test cleanup'),
            started_at = NULL,
            finished_at = COALESCE(finished_at, now()),
            updated_at = now()
        WHERE document_id = $1
          AND status IN ('queued', 'failed', 'running')
        "#,
    )
    .bind(document_id)
    .execute(pool)
    .await
    .expect("quiesce document processing jobs for API test document");
}

async fn create_failed_extract_text_job(
    pool: &sqlx::postgres::PgPool,
    document_store: &DocumentImportStore,
    processing_store: &DocumentProcessingStore,
    document_id: &str,
) -> String {
    document_store
        .import_document(&NewDocumentImport::markdown(
            document_id,
            "retry-api-collision.md",
            "# Retry\n\nProcessing retry body.",
        ))
        .await
        .expect("import markdown document");
    let jobs = processing_store
        .enqueue_for_document(document_id)
        .await
        .expect("enqueue processing jobs");
    let job_id = jobs
        .iter()
        .find(|job| step_name(&job.step) == "extract_text")
        .expect("extract text job")
        .job_id
        .clone();

    fail_processing_job(pool, &job_id).await;

    job_id
}

fn step_name(step: &DocumentProcessingStep) -> &'static str {
    match step {
        DocumentProcessingStep::ExtractText => "extract_text",
        DocumentProcessingStep::Ocr => "ocr",
    }
}

async fn fail_processing_job(pool: &sqlx::postgres::PgPool, job_id: &str) {
    sqlx::query(
        r#"
        UPDATE document_processing_jobs
        SET status = 'failed',
            attempts = 2,
            last_error_summary = 'temporary extractor failure',
            started_at = now(),
            finished_at = now(),
            updated_at = now()
        WHERE job_id = $1
        "#,
    )
    .bind(job_id)
    .execute(pool)
    .await
    .expect("mark extract job failed");
}

async fn append_retry_event_for_job(pool: &sqlx::postgres::PgPool, command_id: &str, job_id: &str) {
    let event = NewEventEnvelope::builder(
        format!("document_processing_retry:{command_id}"),
        "document_processing.retry_requested",
        Utc::now(),
        serde_json::json!({
            "kind": "document_processing_retry",
            "provider": "local_api",
            "source_id": command_id,
        }),
        serde_json::json!({
            "kind": "document_processing_job",
            "job_id": job_id,
        }),
    )
    .actor(serde_json::json!({ "actor_id": "document-processing-api-test-client" }))
    .payload(serde_json::json!({ "job_id": job_id }))
    .build()
    .expect("retry event envelope");

    EventStore::new(pool.clone())
        .append(&event)
        .await
        .expect("append retry collision event");
}
