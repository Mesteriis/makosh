use axum::Json;
use axum::extract::{Path, RawQuery, State};

use crate::platform::formatting::text_preview;

use crate::platform::audit::models::NewApiAuditRecord;

use crate::domains::projects::core::errors::ProjectStoreError;
use crate::domains::projects::link_reviews::models::{
    ProjectLinkReviewState, ProjectLinkTargetKind,
};
use crate::domains::projects::link_reviews::service::ProjectLinkReviewService;
use crate::platform::graph::GraphNodeKind;
use crate::platform::graph::node_id;
use makosh_projects_api::{
    ProjectCandidateReadPort, ProjectDetail, ProjectListResponse, ProjectReadPort,
};
use makosh_projects_postgres::ProjectPostgresReadQuery;

use crate::app::api_support::{
    query_parsing::projects::*, review_commands::*, review_lists::*, stores::domain_stores::*,
};
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::domains::projects::read_port::project_read_port;

pub(crate) async fn get_projects(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<ProjectListResponse>, ApiError> {
    let query = parse_projects_query(raw_query.as_deref())?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = project_read_port(pool)
        .list(query.limit)
        .await
        .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?;
    Ok(Json(items))
}

pub(crate) async fn get_project_detail(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<ProjectDetail>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let Some(project) = project_read_port(pool)
        .detail(&project_id)
        .await
        .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?
    else {
        return Err(ApiError::ProjectNotFound);
    };

    Ok(Json(project))
}

pub(crate) async fn get_project_link_candidates(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<ProjectLinkCandidateListResponse>, ApiError> {
    let query = parse_project_link_candidates_query(raw_query.as_deref())?;
    let project_id = validate_non_empty_project_link_field("project_id", &project_id)?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();

    let review_store = project_link_review_store(&state)?;
    let mut candidates = Vec::new();

    let read_query = ProjectPostgresReadQuery::new(pool.clone());
    for candidate in read_query
        .candidates(&project_id)
        .await
        .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?
    {
        let target_kind = match candidate.target_kind.as_str() {
            "message" => ProjectLinkTargetKind::Message,
            "document" => ProjectLinkTargetKind::Document,
            _ => continue,
        };
        let graph_kind = match target_kind {
            ProjectLinkTargetKind::Message => GraphNodeKind::Message,
            ProjectLinkTargetKind::Document => GraphNodeKind::Document,
        };
        let graph_node_id = node_id(graph_kind, &candidate.target_id);
        let title = text_preview(&candidate.title, 140);
        let subtitle = text_preview(&candidate.subtitle, 140);
        crate::application::project_link_review_mirror::ensure_project_link_candidate_review_item(
            &pool,
            &project_id,
            target_kind,
            &candidate.target_id,
            &title,
            &subtitle,
            1.0,
            &candidate.observation_id,
            Some(&graph_node_id),
        )
        .await
        .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?;
        let review_state = review_store
            .explicit_review(&project_id, target_kind, &candidate.target_id)
            .await?
            .map(|review| review.review_state)
            .unwrap_or(ProjectLinkReviewState::Suggested);
        candidates.push(ProjectLinkCandidate {
            project_id: project_id.clone(),
            target_kind: target_kind.as_str().to_owned(),
            target_id: candidate.target_id,
            graph_node_id,
            title,
            subtitle: candidate.subtitle,
            source_label: candidate
                .account_id
                .or(candidate.source_fingerprint)
                .unwrap_or_default(),
            occurred_at: candidate.occurred_at,
            review_state: review_state.as_str().to_owned(),
            evidence_excerpt: Some(subtitle),
        });
    }

    candidates.sort_by(|left, right| right.occurred_at.cmp(&left.occurred_at));
    candidates.truncate(query.limit.unwrap_or(25));

    Ok(Json(ProjectLinkCandidateListResponse { items: candidates }))
}

pub(crate) async fn put_project_link_review(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(request): Json<ProjectLinkReviewApiRequest>,
) -> Result<Json<ProjectLinkReviewApiResponse>, ApiError> {
    let actor_id = "makosh-frontend".to_string();
    let command = request.into_command(project_id, actor_id)?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::project_link_review_set(
            &command.actor_id,
            &command.project_id,
            command.target_kind.as_str(),
            &command.target_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();

    let result = ProjectLinkReviewService::new(pool.clone())
        .review_manual(&command)
        .await?;
    let stored_event = makosh_events_postgres::store::EventStore::new(pool.clone())
        .get_by_id(&result.event_id)
        .await?
        .ok_or_else(|| {
            ApiError::Store(
                makosh_events_postgres::errors::EventStoreError::ConsumerHandlerFailed(
                    "project link review event was not stored".to_owned(),
                ),
            )
        })?;
    crate::workflows::project_link_review_effects::project_link_review_effect(&pool, &stored_event)
        .await
        .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?;
    let observation_id = sqlx::query_scalar::<_, String>(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE domain = 'projects'
          AND entity_kind = 'project_link_review'
          AND entity_id = $1
          AND relationship_kind = 'review_transition'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&result.event_id)
    .fetch_optional(&pool)
    .await
    .map_err(ProjectStoreError::from)
    .map_err(ApiError::Projects)?
    .ok_or_else(|| {
        ApiError::Store(
            makosh_events_postgres::errors::EventStoreError::ConsumerHandlerFailed(
                "project link review observation link was not stored".to_owned(),
            ),
        )
    })?;
    let project_query = ProjectPostgresReadQuery::new(pool.clone());
    let candidates = project_query
        .candidates(&command.project_id)
        .await
        .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?;
    let mut transaction = pool
        .begin()
        .await
        .map_err(ProjectStoreError::from)
        .map_err(ApiError::Projects)?;
    match command.target_kind {
        ProjectLinkTargetKind::Message => {
            let message = candidates
                .iter()
                .find(|item| item.target_kind == "message" && item.target_id == command.target_id)
                .ok_or(ApiError::ProjectLinkTargetNotFound)?;
            let title = text_preview(&message.title, 120);
            let summary = text_preview(&message.subtitle, 140);
            crate::application::project_link_review_mirror::sync_project_link_review_state_in_transaction(
                &mut transaction,
                &command.project_id,
                command.target_kind,
                &command.target_id,
                command.review_state,
                &title,
                &summary,
                1.0,
                &observation_id,
            )
            .await
            .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?;
        }
        ProjectLinkTargetKind::Document => {
            let document = candidates
                .iter()
                .find(|item| item.target_kind == "document" && item.target_id == command.target_id)
                .ok_or(ApiError::ProjectLinkTargetNotFound)?;
            let title = text_preview(&document.title, 140);
            crate::application::project_link_review_mirror::sync_project_link_review_state_in_transaction(
                &mut transaction,
                &command.project_id,
                command.target_kind,
                &command.target_id,
                command.review_state,
                &title,
                &title,
                1.0,
                &observation_id,
            )
            .await
            .map_err(|error| ApiError::FailedPrecondition(error.to_string()))?;
        }
    }
    transaction
        .commit()
        .await
        .map_err(ProjectStoreError::from)
        .map_err(ApiError::Projects)?;

    Ok(Json(result.into()))
}
