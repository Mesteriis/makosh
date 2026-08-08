use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::domains::tasks::api::{NewTask, Task, TaskUpdate};
use crate::domains::tasks::command_service::TaskCommandService;

use super::support::database_pool;
use makosh_tasks_api::{TaskListQuery, TaskReadPort};
use makosh_tasks_postgres::TaskPostgresReadQuery;

#[derive(Serialize)]
pub(crate) struct TaskRecordsResponse {
    items: Vec<makosh_tasks_api::TaskRead>,
}

#[derive(Deserialize)]
pub(crate) struct TaskListQueryParams {
    status: Option<String>,
    project_id: Option<String>,
    source_type: Option<String>,
    limit: Option<i64>,
}

pub(crate) async fn get_tasks(
    State(state): State<AppState>,
    Query(q): Query<TaskListQueryParams>,
) -> Result<Json<TaskRecordsResponse>, ApiError> {
    let pool = database_pool(&state)?;
    let items = TaskReadPort::list(
        &TaskPostgresReadQuery::new(pool),
        TaskListQuery {
            status: q.status,
            project_id: q.project_id,
            source_type: q.source_type,
            limit: q.limit,
        },
    )
    .await
    .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?;
    Ok(Json(TaskRecordsResponse { items }))
}

pub(crate) async fn post_task(
    State(state): State<AppState>,
    Json(req): Json<NewTask>,
) -> Result<Json<Task>, ApiError> {
    let pool = database_pool(&state)?;
    let task = TaskCommandService::new(pool)
        .create_task_manual(&req)
        .await?;
    Ok(Json(task))
}

pub(crate) async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<makosh_tasks_api::TaskRead>, ApiError> {
    let pool = database_pool(&state)?;
    TaskReadPort::get(&TaskPostgresReadQuery::new(pool), &task_id)
        .await
        .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn put_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(update): Json<TaskUpdate>,
) -> Result<Json<Task>, ApiError> {
    let pool = database_pool(&state)?;
    let task = TaskCommandService::new(pool)
        .update_task_manual(&task_id, &update)
        .await?;
    Ok(Json(task))
}

#[derive(Deserialize)]
pub(crate) struct TaskStatusRequest {
    status: String,
}

pub(crate) async fn post_task_status(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(req): Json<TaskStatusRequest>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    TaskCommandService::new(pool)
        .set_status_manual(&task_id, &req.status)
        .await?;
    Ok(Json(json!({"status": req.status})))
}

pub(crate) async fn post_task_archive(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    TaskCommandService::new(pool)
        .archive_manual(&task_id)
        .await?;
    Ok(Json(json!({"archived": true})))
}
