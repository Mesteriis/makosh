use crate::support::*;
use makosh_backend_testkit::context::TestContext;

#[tokio::test]
async fn document_processing_retry_failed_job_requeues_job_against_postgres() {
    let test_context = TestContext::new().await;
    let _database_url = test_context.connection_string();
    let Some((pool, document_store, processing_store)) =
        live_context("retry failed processing job").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_retry_{suffix}");

    document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            "retry.md",
            "# Retry\n\nProcessing retry body.",
        ))
        .await
        .expect("import markdown document");
    let jobs = processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue processing jobs");
    let extract_job = jobs
        .iter()
        .find(|job| step_name(&job.step) == "extract_text")
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

    let command_id = format!("document-processing-retry-{suffix}");
    let result = processing_store
        .retry_failed_job(&DocumentProcessingRetryCommand {
            command_id: command_id.clone(),
            job_id: extract_job.job_id.clone(),
            actor_id: "document-processing-test-actor".to_owned(),
        })
        .await
        .expect("retry failed job");

    assert_eq!(result.job_id, extract_job.job_id);
    assert_eq!(result.status, DocumentProcessingStatus::Queued);
    assert_eq!(
        result.event_id,
        format!("document_processing_retry:{command_id}")
    );

    let persisted = sqlx::query_as::<_, (String, i32, Option<String>)>(
        r#"
        SELECT status, attempts, last_error_summary
        FROM document_processing_jobs
        WHERE job_id = $1
        "#,
    )
    .bind(&extract_job.job_id)
    .fetch_one(&pool)
    .await
    .expect("persisted retried job");

    assert_eq!(persisted.0, "queued");
    assert_eq!(persisted.1, 0);
    assert_eq!(persisted.2, None);
    let requeue_observations: i64 = query_scalar::<_, i64>(
        r#"
        SELECT count(*)::bigint
        FROM observation_links link
        JOIN observations observation
          ON observation.observation_id = link.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE link.domain = 'documents'
          AND link.entity_kind = 'document_processing_job'
          AND link.entity_id = $1
          AND kind.code = 'DOCUMENT_PROCESSING_JOB_STATUS'
          AND link.relationship_kind = 'requeued'
        "#,
    )
    .bind(&extract_job.job_id)
    .fetch_one(&pool)
    .await
    .expect("requeue observations");
    assert!(requeue_observations >= 1);
    quiesce_processing_jobs_for_document(&pool, &document_id).await;
}

#[tokio::test]
async fn run_queued_jobs_requires_retry_command_for_failed_jobs() {
    let test_context = TestContext::new().await;
    let _database_url = test_context.connection_string();
    let Some((pool, document_store, processing_store)) =
        live_context("failed job requires retry command").await
    else {
        return;
    };
    quiesce_retryable_test_processing_jobs(&pool).await;
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_retry_runner_{suffix}");
    let job_id =
        create_failed_extract_text_job(&pool, &document_store, &processing_store, &document_id)
            .await;
    quiesce_document_processing_jobs_except(&pool, &document_id, &job_id).await;

    let skipped_report = processing_store
        .run_queued_jobs(10)
        .await
        .expect("run queued jobs without retry command");
    let failed_state = job_retry_state(&pool, &job_id).await;
    let artifact_count_before_retry = extracted_text_artifact_count(&pool, &document_id).await;

    assert_eq!(skipped_report.jobs_seen, 0);
    assert_eq!(skipped_report.jobs_queued, 0);
    assert_eq!(failed_state.0, "failed");
    assert_eq!(failed_state.1, 2);
    assert!(failed_state.2.is_some());
    assert_eq!(artifact_count_before_retry, 0);

    processing_store
        .retry_failed_job(&DocumentProcessingRetryCommand {
            command_id: format!("document-processing-retry-runner-{suffix}"),
            job_id: job_id.clone(),
            actor_id: "document-processing-test-actor".to_owned(),
        })
        .await
        .expect("retry failed job");
    let retried_report = processing_store
        .run_queued_jobs(10)
        .await
        .expect("run retried job");
    let retried_state = job_retry_state(&pool, &job_id).await;
    let artifact_count_after_retry = extracted_text_artifact_count(&pool, &document_id).await;

    assert_eq!(retried_report.jobs_seen, 1);
    assert_eq!(retried_report.jobs_queued, 1);
    assert_eq!(retried_report.jobs_succeeded, 1);
    assert_eq!(retried_state.0, "succeeded");
    assert_eq!(artifact_count_after_retry, 1);
    quiesce_processing_jobs_for_document(&pool, &document_id).await;
}

#[tokio::test]
async fn document_processing_retry_duplicate_same_command_is_idempotent() {
    let test_context = TestContext::new().await;
    let _database_url = test_context.connection_string();
    let Some((pool, document_store, processing_store)) =
        live_context("duplicate retry command").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_retry_idempotent_{suffix}");
    let job_id =
        create_failed_extract_text_job(&pool, &document_store, &processing_store, &document_id)
            .await;
    let command = DocumentProcessingRetryCommand {
        command_id: format!("document-processing-retry-idempotent-{suffix}"),
        job_id: job_id.clone(),
        actor_id: "document-processing-test-actor".to_owned(),
    };

    let first = processing_store
        .retry_failed_job(&command)
        .await
        .expect("first retry succeeds");
    let second = processing_store
        .retry_failed_job(&command)
        .await
        .expect("duplicate retry is idempotent");

    assert_eq!(first, second);
    assert_eq!(second.job_id, job_id);
    assert_eq!(second.status, DocumentProcessingStatus::Queued);
    assert_eq!(
        second.event_id,
        format!("document_processing_retry:{}", command.command_id)
    );
    quiesce_processing_jobs_for_document(&pool, &document_id).await;
}

#[tokio::test]
async fn document_processing_retry_duplicate_command_for_different_job_is_rejected() {
    let test_context = TestContext::new().await;
    let _database_url = test_context.connection_string();
    let Some((pool, document_store, processing_store)) =
        live_context("duplicate retry command collision").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let existing_document_id = format!("doc_processing_retry_collision_existing_{suffix}");
    let target_document_id = format!("doc_processing_retry_collision_target_{suffix}");
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
    let command_id = format!("document-processing-retry-collision-{suffix}");
    append_retry_event_for_job(&pool, &command_id, &existing_job_id).await;

    let error = processing_store
        .retry_failed_job(&DocumentProcessingRetryCommand {
            command_id,
            job_id: target_job_id.clone(),
            actor_id: "document-processing-test-actor".to_owned(),
        })
        .await
        .expect_err("command collision must be rejected");

    assert!(matches!(
        error,
        DocumentProcessingError::RetryCommandConflict
    ));
    let persisted = job_retry_state(&pool, &target_job_id).await;
    assert_eq!(persisted.0, "failed");
    assert_eq!(persisted.1, 2);
    assert!(persisted.2.is_some());

    quiesce_processing_jobs_for_document(&pool, &existing_document_id).await;
    quiesce_processing_jobs_for_document(&pool, &target_document_id).await;
}

#[tokio::test]
async fn document_processing_retry_non_failed_job_requires_failed_status() {
    let test_context = TestContext::new().await;
    let _database_url = test_context.connection_string();
    let Some((pool, document_store, processing_store)) =
        live_context("non-failed retry command").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let document_id = format!("doc_processing_retry_non_failed_{suffix}");

    document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            "retry-non-failed.md",
            "# Retry\n\nQueued retry body.",
        ))
        .await
        .expect("import markdown document");
    let jobs = processing_store
        .enqueue_for_document(&document_id)
        .await
        .expect("enqueue processing jobs");
    let extract_job = jobs
        .iter()
        .find(|job| step_name(&job.step) == "extract_text")
        .expect("extract text job");

    let error = processing_store
        .retry_failed_job(&DocumentProcessingRetryCommand {
            command_id: format!("document-processing-retry-non-failed-{suffix}"),
            job_id: extract_job.job_id.clone(),
            actor_id: "document-processing-test-actor".to_owned(),
        })
        .await
        .expect_err("queued job retry must be rejected");

    assert!(matches!(
        error,
        DocumentProcessingError::RetryRequiresFailedJob
    ));
    quiesce_processing_jobs_for_document(&pool, &document_id).await;
}

#[tokio::test]
async fn document_processing_retry_missing_job_returns_job_not_found() {
    let test_context = TestContext::new().await;
    let _database_url = test_context.connection_string();
    let Some((_pool, _document_store, processing_store)) =
        live_context("missing retry command").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let error = processing_store
        .retry_failed_job(&DocumentProcessingRetryCommand {
            command_id: format!("document-processing-retry-missing-{suffix}"),
            job_id: format!("document_processing_job:v1:missing-{suffix}:extract_text"),
            actor_id: "document-processing-test-actor".to_owned(),
        })
        .await
        .expect_err("missing job retry must be rejected");

    assert!(matches!(error, DocumentProcessingError::JobNotFound));
}
