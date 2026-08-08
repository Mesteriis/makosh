use crate::platform::audit::store::ApiAuditLog;
use axum::Json;
use axum::extract::{Path, Query, State};

use super::models::{DecisionListResponse, DecisionReviewApiRequest};
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::application::review_transitions::DecisionReviewApplicationService;
use crate::domains::decisions::models::decision::Decision;
use crate::domains::decisions::models::states::DecisionReviewState;
use crate::platform::audit::models::NewApiAuditRecord;
use makosh_decisions_api::{DecisionListQuery, DecisionReadPort};
use makosh_decisions_postgres::DecisionPostgresReadQuery;

const DECISION_API_ACTOR_ID: &str = "makosh-frontend";
const DEFAULT_DECISION_LIMIT: i64 = 50;
const MIN_DECISION_LIMIT: i64 = 1;
const MAX_DECISION_LIMIT: i64 = 100;

pub(crate) async fn get_v1_decisions(
    State(state): State<AppState>,
    Query(query): Query<DecisionListQuery>,
) -> Result<Json<DecisionListResponse>, ApiError> {
    let query = makosh_decisions_api::DecisionListQuery {
        limit: Some(validate_limit(query.limit)?),
        entity_kind: query.entity_kind,
        entity_id: query.entity_id,
        review_state: query.review_state,
    };
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = DecisionReadPort::list(&DecisionPostgresReadQuery::new(pool), query)
        .await
        .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?;

    Ok(Json(DecisionListResponse { items }))
}

pub(crate) async fn put_v1_decision_review(
    State(state): State<AppState>,
    Path(decision_id): Path<String>,
    Json(request): Json<DecisionReviewApiRequest>,
) -> Result<Json<Decision>, ApiError> {
    let decision_id = validate_required_query_value(Some(&decision_id))?;
    let review_state = parse_review_state(&request.review_state)?;
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::decision_review_set(
            DECISION_API_ACTOR_ID,
            &decision_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let decision = DecisionReviewApplicationService::new(pool)
        .review_manual(&decision_id, review_state)
        .await?;

    Ok(Json(decision))
}

fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApiAuditLog::new(pool.clone()))
}

fn parse_review_state(value: &str) -> Result<DecisionReviewState, ApiError> {
    DecisionReviewState::parse(value).map_err(ApiError::from)
}

fn validate_required_query_value(value: Option<&str>) -> Result<String, ApiError> {
    let value = value.unwrap_or_default().trim();
    if value.is_empty() {
        return Err(ApiError::InvalidDecisionQuery(
            "missing required decision query field",
        ));
    }

    Ok(value.to_owned())
}

fn validate_limit(limit: Option<i64>) -> Result<i64, ApiError> {
    let limit = limit.unwrap_or(DEFAULT_DECISION_LIMIT);
    if !(MIN_DECISION_LIMIT..=MAX_DECISION_LIMIT).contains(&limit) {
        return Err(ApiError::InvalidDecisionQuery(
            "limit must be between 1 and 100",
        ));
    }

    Ok(limit)
}
