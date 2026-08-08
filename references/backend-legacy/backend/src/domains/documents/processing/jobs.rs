use serde_json::json;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use super::constants::DEFAULT_MAX_ATTEMPTS;
use super::errors::DocumentProcessingError;
use super::evidence::link_document_processing_entity_in_transaction;
use super::ids::job_id;
use super::models::{DocumentProcessingJob, DocumentProcessingStatus, DocumentProcessingStep};
use super::rows::try_row_to_job;
use super::store::DocumentProcessingStore;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;

async fn capture_job_observation(
    tx: &mut Transaction<'_, Postgres>,
    job: &DocumentProcessingJob,
    kind_code: &str,
    relationship_kind: &str,
    actor: &str,
) -> Result<(), DocumentProcessingError> {
    let observed_at = job.updated_at;
    let observation = ObservationStore::capture_in_transaction(
        tx,
        &NewObservation::new(
            kind_code,
            ObservationOriginKind::LocalRuntime,
            observed_at,
            json!({
                "job_id": job.job_id,
                "document_id": job.document_id,
                "step": job.step,
                "status": job.status,
                "attempts": job.attempts,
                "max_attempts": job.max_attempts,
                "last_error_summary": job.last_error_summary,
                "queued_at": job.queued_at,
                "started_at": job.started_at,
                "finished_at": job.finished_at,
                "operation": relationship_kind,
            }),
            format!(
                "document-processing-job://{}/{}",
                job.job_id, relationship_kind
            ),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
        })),
    )
    .await?;
    link_document_processing_entity_in_transaction(
        tx,
        &observation.observation_id,
        "document_processing_job",
        job.job_id.clone(),
        relationship_kind,
        json!({
            "document_id": job.document_id,
            "step": job.step,
            "status": job.status,
        }),
    )
    .await?;
    Ok(())
}

impl DocumentProcessingStore {
    pub(super) async fn upsert_job(
        &self,
        document_id: &str,
        step: DocumentProcessingStep,
    ) -> Result<DocumentProcessingJob, DocumentProcessingError> {
        let job_id = job_id(document_id, step);
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO document_processing_jobs (
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                updated_at
            )
            VALUES ($1, $2, $3, 'queued', 0, $4, now())
            ON CONFLICT (document_id, step)
            DO UPDATE
                SET status = CASE
                    WHEN document_processing_jobs.status IN ('succeeded', 'skipped') THEN document_processing_jobs.status
                    ELSE 'queued'
                END,
                attempts = CASE
                    WHEN document_processing_jobs.status IN ('succeeded', 'skipped') THEN document_processing_jobs.attempts
                    ELSE 0
                END,
                last_error_summary = CASE
                    WHEN document_processing_jobs.status IN ('succeeded', 'skipped') THEN document_processing_jobs.last_error_summary
                    ELSE NULL
                END,
                started_at = CASE
                    WHEN document_processing_jobs.status IN ('succeeded', 'skipped') THEN document_processing_jobs.started_at
                    ELSE NULL
                END,
                finished_at = CASE
                    WHEN document_processing_jobs.status IN ('succeeded', 'skipped') THEN document_processing_jobs.finished_at
                    ELSE NULL
                END,
                updated_at = now()
            RETURNING
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            "#,
        )
        .bind(&job_id)
        .bind(document_id)
        .bind(step.as_str())
        .bind(DEFAULT_MAX_ATTEMPTS)
        .fetch_one(&mut *tx)
        .await?;
        let job = try_row_to_job(row)?;
        capture_job_observation(
            &mut tx,
            &job,
            "DOCUMENT_PROCESSING_JOB",
            "queued",
            "documents.processing.upsert_job",
        )
        .await?;
        tx.commit().await?;
        Ok(job)
    }

    pub(super) async fn next_jobs(
        &self,
        limit: i64,
    ) -> Result<Vec<QueuedJob>, DocumentProcessingError> {
        let rows = sqlx::query(
            r#"
            SELECT
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            FROM document_processing_jobs
            WHERE status = 'queued'
              AND attempts < max_attempts
            ORDER BY queued_at ASC, job_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(QueuedJob {
                    base: try_row_to_job(row)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub(super) async fn mark_running(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        job: &QueuedJob,
    ) -> Result<DocumentProcessingJob, DocumentProcessingError> {
        let row = sqlx::query(
            r#"
            UPDATE document_processing_jobs
            SET status = 'running',
                attempts = attempts + 1,
                started_at = now(),
                updated_at = now()
            WHERE job_id = $1
              AND status = 'queued'
              AND attempts < max_attempts
            RETURNING
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            "#,
        )
        .bind(&job.base.job_id)
        .fetch_optional(&mut **tx)
        .await?;

        let job = row
            .map(try_row_to_job)
            .ok_or(DocumentProcessingError::JobNotFound)??;
        capture_job_observation(
            tx,
            &job,
            "DOCUMENT_PROCESSING_JOB_STATUS",
            "running",
            "documents.processing.mark_running",
        )
        .await?;
        Ok(job)
    }

    pub(super) async fn job_for_update(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        job_id: &str,
    ) -> Result<DocumentProcessingJob, DocumentProcessingError> {
        let row = sqlx::query(
            r#"
            SELECT
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            FROM document_processing_jobs
            WHERE job_id = $1
            FOR UPDATE
            "#,
        )
        .bind(job_id)
        .fetch_optional(&mut **tx)
        .await?;

        row.map(try_row_to_job)
            .ok_or(DocumentProcessingError::JobNotFound)?
    }

    pub(super) async fn requeue_failed_job(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        job_id: &str,
    ) -> Result<DocumentProcessingJob, DocumentProcessingError> {
        let row = sqlx::query(
            r#"
            UPDATE document_processing_jobs
            SET status = 'queued',
                attempts = 0,
                last_error_summary = NULL,
                started_at = NULL,
                finished_at = NULL,
                updated_at = now()
            WHERE job_id = $1
              AND status = 'failed'
            RETURNING
                job_id,
                document_id,
                step,
                status,
                attempts,
                max_attempts,
                last_error_summary,
                queued_at,
                started_at,
                finished_at,
                created_at,
                updated_at
            "#,
        )
        .bind(job_id)
        .fetch_optional(&mut **tx)
        .await?;

        let job = row
            .map(try_row_to_job)
            .ok_or(DocumentProcessingError::RetryRequiresFailedJob)??;
        capture_job_observation(
            tx,
            &job,
            "DOCUMENT_PROCESSING_JOB_STATUS",
            "requeued",
            "documents.processing.requeue_failed_job",
        )
        .await?;
        Ok(job)
    }

    pub(super) async fn finish_job(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        job: &DocumentProcessingJob,
        status: DocumentProcessingStatus,
        last_error_summary: Option<String>,
    ) -> Result<(), DocumentProcessingError> {
        sqlx::query(
            r#"
            UPDATE document_processing_jobs
            SET status = $2,
                last_error_summary = $3,
                finished_at = now(),
                updated_at = now()
            WHERE job_id = $1
            "#,
        )
        .bind(&job.job_id)
        .bind(status.as_str())
        .bind(last_error_summary)
        .execute(&mut **tx)
        .await?;
        let job = self.job_for_update(tx, &job.job_id).await?;
        capture_job_observation(
            tx,
            &job,
            "DOCUMENT_PROCESSING_JOB_STATUS",
            status.as_str(),
            "documents.processing.finish_job",
        )
        .await?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(super) struct QueuedJob {
    pub(super) base: DocumentProcessingJob,
}
