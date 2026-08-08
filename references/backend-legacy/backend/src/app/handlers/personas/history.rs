use axum::Json;
use axum::extract::{Path, Query, RawQuery, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, header};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::app::api_support::{
    query_parsing::personas::*, review_commands::*, stores::domain_stores::*,
};
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::domains::personas::analytics::PersonaAnalyticsService;
use crate::domains::personas::export::{ExportFormat, PersonaExportService};
use crate::platform::audit::models::NewApiAuditRecord;
// ── Persona Analytics ───────────────────────────────────────────────────────

pub(crate) async fn get_persona_analytics(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let analytics = PersonaAnalyticsService::new(pool)
        .compute(&persona_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&analytics).unwrap_or_default()))
}

// ── Persona Export ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct PersonaDownloadQuery {
    format: Option<String>,
}

pub(crate) async fn get_persona_export_handler(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Query(query): Query<PersonaDownloadQuery>,
) -> Result<(HeaderMap, String), ApiError> {
    let format = query
        .format
        .as_deref()
        .and_then(ExportFormat::parse)
        .unwrap_or(ExportFormat::Json);
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let content = PersonaExportService::new(pool)
        .export(&persona_id, format.clone())
        .await
        .map_err(ApiError::from)?;
    let mut headers_map = HeaderMap::new();
    headers_map.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(format.content_type())
            .unwrap_or(HeaderValue::from_static("application/json")),
    );
    headers_map.insert(
        HeaderName::from_static("content-disposition"),
        HeaderValue::from_str(&format!(
            "attachment; filename=persona_{}.{}",
            persona_id,
            format.extension()
        ))
        .unwrap(),
    );
    Ok((headers_map, content))
}

// ── Persona Snapshots & History Diff ────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonaSnapshotsResponse {
    items: Vec<crate::domains::personas::memory::snapshots::PersonaSnapshot>,
}

pub(crate) async fn get_persona_snapshots(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<PersonaSnapshotsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::stores::domain_stores::app_store::<
        crate::domains::personas::memory::snapshots::PersonaSnapshotStore,
    >(pool)
    .list(&persona_id)
    .await
    .map_err(ApiError::from)?;
    Ok(Json(PersonaSnapshotsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct HistoryDiffQuery {
    from: String,
    to: String,
}

pub(crate) async fn get_persona_history_diff(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
    Query(query): Query<HistoryDiffQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let from_date = DateTime::parse_from_rfc3339(&query.from)
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid from date"))?
        .with_timezone(&Utc);
    let to_date = DateTime::parse_from_rfc3339(&query.to)
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid to date"))?
        .with_timezone(&Utc);
    let diff = crate::app::api_support::stores::domain_stores::app_store::<
        crate::domains::personas::memory::snapshots::PersonaSnapshotStore,
    >(pool)
    .history_diff(&persona_id, from_date, to_date)
    .await
    .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&diff).unwrap_or_default()))
}

pub(crate) async fn get_identity_candidates(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<PersonaIdentityCandidateListResponse>, ApiError> {
    let query = parse_persona_identity_candidates_query(raw_query.as_deref())?;
    let items = persona_identity_review_store(&state)?
        .list_candidates(query.limit)
        .await?;

    Ok(Json(PersonaIdentityCandidateListResponse::from(items)))
}

pub(crate) async fn put_identity_candidate_review(
    State(state): State<AppState>,
    Path(identity_candidate_id): Path<String>,
    Json(request): Json<PersonaIdentityReviewApiRequest>,
) -> Result<Json<PersonaIdentityReviewApiResponse>, ApiError> {
    let actor_id = "makosh-frontend".to_string();
    let command = request.into_command(identity_candidate_id, actor_id)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::persona_identity_review_set(
            &command.actor_id,
            &command.identity_candidate_id,
        ))
        .await?;

    let result = crate::domains::personas::command_service::PersonaCommandService::new(pool)
        .review_identity_candidate_manual(&command)
        .await?;

    Ok(Json(result.into()))
}

pub(crate) async fn get_persona_identity(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<PersonaIdentityDetailResponse>, ApiError> {
    let _ = validate_non_empty_persona_identity_field("persona_id", &persona_id)?;

    let detail = persona_identity_review_store(&state)?
        .persona_identity(&persona_id)
        .await?;
    Ok(Json(detail.into()))
}
