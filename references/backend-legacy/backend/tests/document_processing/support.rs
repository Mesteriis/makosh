use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) use chrono::Utc;
pub(crate) use makosh_events_api::NewEventEnvelope;
use makosh_events_postgres::store::EventStore;
pub(crate) use makosh_hub_backend::domains::documents::core::{
    models::NewDocumentImport, store::DocumentImportStore,
};
pub(crate) use makosh_hub_backend::domains::documents::processing::{
    errors::DocumentProcessingError,
    models::{DocumentProcessingRetryCommand, DocumentProcessingStatus, DocumentProcessingStep},
    store::DocumentProcessingStore,
};
pub(crate) use makosh_hub_backend::platform::storage::database::Database;
pub(crate) use serde_json::json;
pub(crate) use sqlx::postgres::PgPool;
pub(crate) use sqlx::query_scalar;

pub(crate) async fn live_context(
    _test_name: &str,
) -> Option<(PgPool, DocumentImportStore, DocumentProcessingStore)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    Some((
        pool.clone(),
        DocumentImportStore::new(pool.clone()),
        DocumentProcessingStore::new(pool),
    ))
}

pub(crate) fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

pub(crate) fn step_name(step: &DocumentProcessingStep) -> &'static str {
    match step {
        DocumentProcessingStep::ExtractText => "extract_text",
        DocumentProcessingStep::Ocr => "ocr",
    }
}

pub(crate) async fn terminal_state_for_document(
    pool: &sqlx::postgres::PgPool,
    document_id: &str,
) -> Vec<(String, String, i32)> {
    sqlx::query_as::<_, (String, String, i32)>(
        "SELECT step, status, attempts FROM document_processing_jobs WHERE document_id = $1 ORDER BY step",
    )
    .bind(document_id)
    .fetch_all(pool)
    .await
    .expect("terminal state")
}

pub(crate) async fn create_failed_extract_text_job(
    pool: &sqlx::postgres::PgPool,
    document_store: &DocumentImportStore,
    processing_store: &DocumentProcessingStore,
    document_id: &str,
) -> String {
    document_store
        .import_document(&NewDocumentImport::markdown(
            document_id,
            "retry-collision.md",
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

pub(crate) async fn fail_processing_job(pool: &sqlx::postgres::PgPool, job_id: &str) {
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

pub(crate) async fn append_retry_event_for_job(
    pool: &sqlx::postgres::PgPool,
    command_id: &str,
    job_id: &str,
) {
    let event = NewEventEnvelope::builder(
        format!("document_processing_retry:{command_id}"),
        "document_processing.retry_requested",
        Utc::now(),
        json!({
            "kind": "document_processing_retry",
            "provider": "local_api",
            "source_id": command_id,
        }),
        json!({
            "kind": "document_processing_job",
            "job_id": job_id,
        }),
    )
    .actor(json!({ "actor_id": "document-processing-test-actor" }))
    .payload(json!({ "job_id": job_id }))
    .build()
    .expect("retry event envelope");

    EventStore::new(pool.clone())
        .append(&event)
        .await
        .expect("append retry collision event");
}

pub(crate) async fn job_retry_state(
    pool: &sqlx::postgres::PgPool,
    job_id: &str,
) -> (String, i32, Option<String>) {
    sqlx::query_as::<_, (String, i32, Option<String>)>(
        r#"
        SELECT status, attempts, last_error_summary
        FROM document_processing_jobs
        WHERE job_id = $1
        "#,
    )
    .bind(job_id)
    .fetch_one(pool)
    .await
    .expect("job retry state")
}

pub(crate) async fn extracted_text_artifact_count(
    pool: &sqlx::postgres::PgPool,
    document_id: &str,
) -> i64 {
    query_scalar::<_, i64>(
        "SELECT count(*) FROM document_artifacts WHERE document_id = $1 AND artifact_kind = 'extracted_text'",
    )
    .bind(document_id)
    .fetch_one(pool)
    .await
    .expect("extracted text artifact count")
}

pub(crate) async fn quiesce_document_processing_jobs_except(
    pool: &sqlx::postgres::PgPool,
    document_id: &str,
    active_job_id: &str,
) {
    sqlx::query(
        r#"
        UPDATE document_processing_jobs
        SET status = 'skipped',
            last_error_summary = COALESCE(last_error_summary, 'test cleanup'),
            started_at = NULL,
            finished_at = COALESCE(finished_at, now()),
            updated_at = now()
        WHERE document_id = $1
          AND job_id <> $2
          AND status IN ('queued', 'failed', 'running')
        "#,
    )
    .bind(document_id)
    .bind(active_job_id)
    .execute(pool)
    .await
    .expect("quiesce non-target document processing jobs");
}

pub(crate) async fn quiesce_retryable_test_processing_jobs(pool: &sqlx::postgres::PgPool) {
    sqlx::query(
        r#"
        UPDATE document_processing_jobs
        SET status = 'skipped',
            last_error_summary = COALESCE(last_error_summary, 'test cleanup'),
            started_at = NULL,
            finished_at = COALESCE(finished_at, now()),
            updated_at = now()
        WHERE document_id LIKE 'doc_processing_%'
          AND status IN ('queued', 'failed', 'running')
        "#,
    )
    .execute(pool)
    .await
    .expect("quiesce retryable test processing jobs");
}

pub(crate) async fn quiesce_processing_jobs_for_document(
    pool: &sqlx::postgres::PgPool,
    document_id: &str,
) {
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
    .expect("quiesce document processing jobs for test document");
}
