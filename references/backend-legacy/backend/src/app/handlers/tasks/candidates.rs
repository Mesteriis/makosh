use axum::Json;
use axum::extract::{Path, RawQuery, State};

use crate::app::api_support::{
    query_parsing::tasks::*, review_commands::*, review_lists::*, stores::domain_stores::*,
};
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::application::review_transitions::TaskCandidateReviewApplicationService;
use crate::platform::audit::models::NewApiAuditRecord;
pub(crate) async fn get_task_candidates(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<TaskCandidateListResponse>, ApiError> {
    let query = parse_task_candidates_query(raw_query.as_deref())?;
    let items = task_candidate_store(&state)?
        .list_candidates(query.limit)
        .await?;

    Ok(Json(TaskCandidateListResponse { items }))
}

pub(crate) async fn put_task_candidate_review(
    State(state): State<AppState>,
    Path(task_candidate_id): Path<String>,
    Json(request): Json<TaskCandidateReviewApiRequest>,
) -> Result<Json<TaskCandidateReviewApiResponse>, ApiError> {
    let actor_id = "makosh-frontend".to_string();
    let command = request.into_command(task_candidate_id, actor_id)?;
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::task_candidate_review_set(
            &command.actor_id,
            &command.task_candidate_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let result = TaskCandidateReviewApplicationService::new(pool)
        .review_manual(&command)
        .await?;

    Ok(Json(result.into()))
}
