use axum::Json;
use axum::extract::{Path, RawQuery, State};

use crate::platform::audit::models::NewApiAuditRecord;

use crate::domains::documents::processing::models::DocumentProcessingRecord;
use crate::domains::documents::processing::service::DocumentProcessingCommandService;

use crate::app::api_support::{
    query_parsing::documents::*, review_commands::*, stores::domain_stores::*,
};
use crate::app::error::types::ApiError;
use crate::app::state::AppState;

pub(crate) async fn get_document_processing(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
) -> Result<Json<DocumentProcessingRecord>, ApiError> {
    let _ = validate_non_empty_document_id(document_id.as_str())?;

    Ok(Json(
        document_processing_store(&state)?
            .document_processing(&document_id)
            .await?,
    ))
}

pub(crate) async fn get_document_processing_jobs(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<DocumentProcessingJobsResponse>, ApiError> {
    let query = parse_document_processing_jobs_query(raw_query.as_deref())?;
    let items = document_processing_store(&state)?
        .list_jobs(query.limit)
        .await?;

    Ok(Json(DocumentProcessingJobsResponse { items }))
}

pub(crate) async fn post_document_processing_job_retry(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Json(request): Json<DocumentProcessingRetryApiRequest>,
) -> Result<Json<DocumentProcessingRetryApiResponse>, ApiError> {
    let actor_id = "makosh-frontend".to_string();
    let command = request.into_command(job_id, actor_id)?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::document_processing_job_retry(
            &command.actor_id,
            &command.job_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let result = DocumentProcessingCommandService::new(pool)
        .retry_failed_job_manual(&command)
        .await?;
    Ok(Json(result.into()))
}
