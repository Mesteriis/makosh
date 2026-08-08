use crate::platform::audit::store::ApiAuditLog;
use axum::Json;
use axum::extract::{Path, Query, State};

use super::models::RelationshipListResponse;
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::application::relationship_query::RelationshipReviewAdapter;
use crate::platform::audit::models::NewApiAuditRecord;
use makosh_relationships_api::{
    RelationshipListQuery, RelationshipReviewError, RelationshipReviewPort,
    RelationshipReviewRequest,
};
use makosh_relationships_postgres::RelationshipPostgresQuery;

const RELATIONSHIP_API_ACTOR_ID: &str = "makosh-frontend";
const DEFAULT_RELATIONSHIP_LIMIT: i64 = 50;
const MIN_RELATIONSHIP_LIMIT: i64 = 1;
const MAX_RELATIONSHIP_LIMIT: i64 = 100;

pub(crate) async fn get_v1_relationships(
    State(state): State<AppState>,
    Query(query): Query<RelationshipListQuery>,
) -> Result<Json<RelationshipListResponse>, ApiError> {
    let query = RelationshipListQuery {
        limit: Some(validate_limit(query.limit)?),
        ..query
    };
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let adapter = RelationshipPostgresQuery::new(pool);
    let items = makosh_relationships_api::RelationshipQueryPort::list(&adapter, query)
        .await
        .map_err(|error| match error {
            makosh_relationships_api::RelationshipQueryError::InvalidQuery(message) => {
                ApiError::InvalidRelationshipQuery(message)
            }
            makosh_relationships_api::RelationshipQueryError::Failed(message) => {
                ApiError::FailedPrecondition(message)
            }
        })?;

    Ok(Json(RelationshipListResponse { items }))
}

pub(crate) async fn put_v1_relationship_review(
    State(state): State<AppState>,
    Path(relationship_id): Path<String>,
    Json(request): Json<RelationshipReviewRequest>,
) -> Result<Json<makosh_relationships_api::RelationshipRead>, ApiError> {
    let relationship_id = validate_required_query_value(Some(&relationship_id))?;
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::relationship_review_set(
            RELATIONSHIP_API_ACTOR_ID,
            &relationship_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let adapter = RelationshipReviewAdapter::from_pool(pool);
    let relationship = RelationshipReviewPort::review(&adapter, &relationship_id, request)
        .await
        .map_err(map_review_error)?;

    Ok(Json(relationship))
}

fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApiAuditLog::new(pool.clone()))
}

fn map_review_error(error: RelationshipReviewError) -> ApiError {
    match error {
        RelationshipReviewError::InvalidReview(message) => {
            ApiError::InvalidRelationshipReview(message)
        }
        RelationshipReviewError::Failed(message) => ApiError::FailedPrecondition(message),
    }
}

fn validate_required_query_value(value: Option<&str>) -> Result<String, ApiError> {
    let value = value.unwrap_or_default().trim();
    if value.is_empty() {
        return Err(ApiError::InvalidRelationshipQuery(
            "missing required relationship query field",
        ));
    }

    Ok(value.to_owned())
}

fn validate_limit(limit: Option<i64>) -> Result<i64, ApiError> {
    let limit = limit.unwrap_or(DEFAULT_RELATIONSHIP_LIMIT);
    if !(MIN_RELATIONSHIP_LIMIT..=MAX_RELATIONSHIP_LIMIT).contains(&limit) {
        return Err(ApiError::InvalidRelationshipQuery(
            "limit must be between 1 and 100",
        ));
    }

    Ok(limit)
}
