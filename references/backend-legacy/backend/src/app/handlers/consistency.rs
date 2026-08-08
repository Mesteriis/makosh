use crate::platform::audit::store::ApiAuditLog;
use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};

use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::engines::consistency::{
    models::{ContradictionObservation, ContradictionReviewState},
    store::ContradictionObservationStore,
};
use crate::platform::audit::models::NewApiAuditRecord;
use crate::workflows::consistency_review::ContradictionReviewService;

const CONTRADICTION_API_ACTOR_ID: &str = "makosh-frontend";
const DEFAULT_CONTRADICTION_LIMIT: i64 = 50;
const MIN_CONTRADICTION_LIMIT: i64 = 1;
const MAX_CONTRADICTION_LIMIT: i64 = 100;

#[derive(Debug, Deserialize)]
pub(crate) struct ContradictionListQuery {
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ContradictionReviewApiRequest {
    review_state: String,
    resolution: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ContradictionListResponse {
    items: Vec<ContradictionObservation>,
}

pub(crate) async fn get_v1_contradictions(
    State(state): State<AppState>,
    Query(query): Query<ContradictionListQuery>,
) -> Result<Json<ContradictionListResponse>, ApiError> {
    let limit = validate_limit(query.limit)?;
    let items = contradiction_store(&state)?.list_open(limit).await?;

    Ok(Json(ContradictionListResponse { items }))
}

pub(crate) async fn put_v1_contradiction_review(
    State(state): State<AppState>,
    Path(observation_id): Path<String>,
    Json(request): Json<ContradictionReviewApiRequest>,
) -> Result<Json<ContradictionObservation>, ApiError> {
    let observation_id = validate_required_value(&observation_id)?;
    let review_state = ContradictionReviewState::parse(&request.review_state)?;
    let resolution = request
        .resolution
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::contradiction_review_set(
            CONTRADICTION_API_ACTOR_ID,
            &observation_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let observation = ContradictionReviewService::new(pool)
        .review_manual(&observation_id, review_state, resolution)
        .await?;

    Ok(Json(observation))
}

fn contradiction_store(state: &AppState) -> Result<ContradictionObservationStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(crate::app::api_support::stores::domain_stores::app_store::<
        ContradictionObservationStore,
    >(pool.clone()))
}

fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApiAuditLog::new(pool.clone()))
}

fn validate_required_value(value: &str) -> Result<String, ApiError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ApiError::InvalidContradictionReview(
            "missing required contradiction observation id",
        ));
    }

    Ok(value.to_owned())
}

fn validate_limit(limit: Option<i64>) -> Result<i64, ApiError> {
    let limit = limit.unwrap_or(DEFAULT_CONTRADICTION_LIMIT);
    if !(MIN_CONTRADICTION_LIMIT..=MAX_CONTRADICTION_LIMIT).contains(&limit) {
        return Err(ApiError::InvalidContradictionQuery(
            "limit must be between 1 and 100",
        ));
    }

    Ok(limit)
}
