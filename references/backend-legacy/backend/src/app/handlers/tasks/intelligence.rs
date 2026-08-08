use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::domains::tasks::api::TaskStore;
use crate::domains::tasks::brain::TaskBrainService;
use crate::domains::tasks::command_service::TaskCommandService;
use crate::domains::tasks::sync::{export_task_json, export_task_md};

use super::support::database_pool;

pub(crate) async fn post_task_analyze(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let analysis = TaskCommandService::new(pool)
        .analyze_runtime(&task_id)
        .await?;
    Ok(Json(json!({
        "priority": analysis.priority,
        "risk": analysis.risk,
        "readiness": analysis.readiness,
        "missing_context": analysis.missing_context,
        "next_action": analysis.next_action
    })))
}

#[derive(Deserialize)]
pub(crate) struct TaskExportQuery {
    format: Option<String>,
}

pub(crate) async fn get_task_export(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Query(q): Query<TaskExportQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let task = crate::app::api_support::stores::domain_stores::app_store::<TaskStore>(pool)
        .get(&task_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    match q.format.as_deref().unwrap_or("json") {
        "md" => Ok(Json(
            json!({"format":"markdown","content": export_task_md(&task.title, task.description.as_deref(), &task.makosh_status, task.why.as_deref(), task.outcome.as_deref())}),
        )),
        _ => Ok(Json(export_task_json(
            &task.title,
            task.description.as_deref(),
            &task.makosh_status,
            task.priority_score,
            task.due_at.map(|d| d.to_rfc3339()).as_deref(),
        ))),
    }
}

#[derive(Deserialize)]
pub(crate) struct TaskBrainQueryParams {
    q: String,
}

pub(crate) async fn post_task_brain(
    State(state): State<AppState>,
    Json(req): Json<TaskBrainQueryParams>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let answer = TaskBrainService::explain_task(&pool, &req.q)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(answer))
}

#[derive(Deserialize)]
pub(crate) struct TaskSearchQueryParams {
    q: String,
}

pub(crate) async fn get_task_search(
    State(state): State<AppState>,
    Query(q): Query<TaskSearchQueryParams>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let results = TaskBrainService::search_tasks(&pool, &q.q).await?;
    Ok(Json(results))
}

pub(crate) async fn get_task_daily_brief(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let pool = database_pool(&state)?;
    let brief = TaskBrainService::daily_brief(&pool).await?;
    Ok(Json(brief))
}
