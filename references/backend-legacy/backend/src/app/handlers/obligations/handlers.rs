use crate::platform::audit::store::ApiAuditLog;
use axum::Json;
use axum::extract::{Path, Query, State};

use super::models::{ObligationListResponse, ObligationReviewApiRequest};
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::application::review_transitions::ObligationReviewApplicationService;
use crate::domains::obligations::models::read_model::Obligation;
use crate::domains::obligations::models::states::ObligationReviewState;
use crate::platform::audit::models::NewApiAuditRecord;
use makosh_obligations_api::{ObligationListQuery, ObligationReadPort};
use makosh_obligations_postgres::ObligationPostgresReadQuery;

const OBLIGATION_API_ACTOR_ID: &str = "makosh-frontend";
const DEFAULT_OBLIGATION_LIMIT: i64 = 50;
const MIN_OBLIGATION_LIMIT: i64 = 1;
const MAX_OBLIGATION_LIMIT: i64 = 100;

pub(crate) async fn get_v1_obligations(
    State(state): State<AppState>,
    Query(query): Query<ObligationListQuery>,
) -> Result<Json<ObligationListResponse>, ApiError> {
    let query = ObligationListQuery {
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
    let items = ObligationReadPort::list(&ObligationPostgresReadQuery::new(pool), query)
        .await
        .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?;

    Ok(Json(ObligationListResponse { items }))
}

pub(crate) async fn put_v1_obligation_review(
    State(state): State<AppState>,
    Path(obligation_id): Path<String>,
    Json(request): Json<ObligationReviewApiRequest>,
) -> Result<Json<Obligation>, ApiError> {
    let obligation_id = validate_required_query_value(Some(&obligation_id))?;
    let review_state = parse_review_state(&request.review_state)?;
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::obligation_review_set(
            OBLIGATION_API_ACTOR_ID,
            &obligation_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let obligation = ObligationReviewApplicationService::new(pool)
        .review_manual(&obligation_id, review_state)
        .await?;

    Ok(Json(obligation))
}

fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApiAuditLog::new(pool.clone()))
}

fn parse_review_state(value: &str) -> Result<ObligationReviewState, ApiError> {
    ObligationReviewState::parse(value).map_err(ApiError::from)
}

fn validate_required_query_value(value: Option<&str>) -> Result<String, ApiError> {
    let value = value.unwrap_or_default().trim();
    if value.is_empty() {
        return Err(ApiError::InvalidObligationQuery(
            "missing required obligation query field",
        ));
    }

    Ok(value.to_owned())
}

fn validate_limit(limit: Option<i64>) -> Result<i64, ApiError> {
    let limit = limit.unwrap_or(DEFAULT_OBLIGATION_LIMIT);
    if !(MIN_OBLIGATION_LIMIT..=MAX_OBLIGATION_LIMIT).contains(&limit) {
        return Err(ApiError::InvalidObligationQuery(
            "limit must be between 1 and 100",
        ));
    }

    Ok(limit)
}
