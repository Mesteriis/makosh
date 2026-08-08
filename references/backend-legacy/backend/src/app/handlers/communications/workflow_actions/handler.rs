use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde_json::json;

use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::domains::communications::messages::store::MessageProjectionStore;
use makosh_events_postgres::store::EventStore;

use super::actions::archive::archive_response;
use super::actions::calendar::create_event_response;
use super::actions::documents::{create_document_response, link_document_response};
use super::actions::personas::create_persona_response;
use super::actions::reply::reply_response;
use super::actions::tasks::create_task_response;
use super::constants::WORKFLOW_EVENT_TYPE;
use super::models::{WorkflowActionKind, WorkflowActionRequest, WorkflowActionResponse};
use super::response::response_from_event;
use super::source::load_source_message;
use super::validation::{actor_id_from_headers, normalize_non_empty};

pub(crate) async fn post_v1_workflow_action(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<WorkflowActionRequest>,
) -> Result<Json<WorkflowActionResponse>, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let actor_id = actor_id_from_headers(&headers);
    let response = execute_workflow_action(&pool, &actor_id, request).await?;
    Ok(Json(response))
}

pub(crate) async fn execute_workflow_action(
    pool: &sqlx::postgres::PgPool,
    actor_id: &str,
    request: WorkflowActionRequest,
) -> Result<WorkflowActionResponse, ApiError> {
    let command_id = normalize_non_empty("command_id", &request.command_id)?;
    let event_id = format!("workflow_action:{command_id}");
    let event_store =
        crate::app::api_support::stores::domain_stores::app_store::<EventStore>(pool.clone());
    if let Some(existing) = event_store.get_by_id(&event_id).await? {
        return response_from_event(existing);
    }

    let message_store = crate::app::api_support::stores::domain_stores::app_store::<
        MessageProjectionStore,
    >(pool.clone());
    let source_message = load_source_message(&message_store, request.source.as_ref()).await?;
    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| ApiError::Store(error.into()))?;
    let response = match request.action.clone() {
        WorkflowActionKind::Reply => {
            reply_response(&command_id, &event_id, &request, source_message.as_ref())?
        }
        WorkflowActionKind::CreateTask => {
            create_task_response(
                pool,
                &mut transaction,
                &command_id,
                &event_id,
                actor_id,
                &request,
                source_message.as_ref(),
            )
            .await?
        }
        WorkflowActionKind::CreateNote => {
            create_document_response(
                &mut transaction,
                &command_id,
                &event_id,
                &request,
                source_message.as_ref(),
                true,
            )
            .await?
        }
        WorkflowActionKind::CreateDocument => {
            create_document_response(
                &mut transaction,
                &command_id,
                &event_id,
                &request,
                source_message.as_ref(),
                false,
            )
            .await?
        }
        WorkflowActionKind::CreateEvent => {
            create_event_response(
                &mut transaction,
                &command_id,
                &event_id,
                &request,
                source_message.as_ref(),
            )
            .await?
        }
        WorkflowActionKind::LinkDocument => {
            link_document_response(
                &mut transaction,
                &command_id,
                &event_id,
                &request,
                source_message.as_ref(),
            )
            .await?
        }
        WorkflowActionKind::CreatePersona => {
            create_persona_response(
                &mut transaction,
                &command_id,
                &event_id,
                &request,
                source_message.as_ref(),
            )
            .await?
        }
        WorkflowActionKind::Archive => {
            archive_response(
                &mut transaction,
                &command_id,
                &event_id,
                actor_id,
                &request,
                source_message.as_ref(),
            )
            .await?
        }
    };

    let event = NewEventEnvelope::builder(
        event_id.clone(),
        WORKFLOW_EVENT_TYPE,
        Utc::now(),
        json!({
            "kind": "workflow_action",
            "source_id": command_id,
        }),
        json!({
            "kind": "workflow_action",
            "id": command_id,
        }),
    )
    .actor(json!({ "actor_id": actor_id }))
    .payload(serde_json::to_value(&response).map_err(|_| {
        ApiError::InvalidCommunicationQuery("invalid workflow action response payload")
    })?)
    .provenance(json!({
        "source_kind": response.provenance.source_kind.as_deref(),
        "source_id": response.provenance.source_id.as_deref(),
        "confidence": response.provenance.confidence,
        "evidence": response.provenance.evidence.clone(),
    }))
    .correlation_id(command_id.clone())
    .build()
    .map_err(ApiError::InvalidEnvelope)?;

    match EventStore::append_in_transaction(&mut transaction, &event).await {
        Ok(_) => {
            transaction
                .commit()
                .await
                .map_err(|error| ApiError::Store(error.into()))?;
            Ok(response)
        }
        Err(error) if error.is_unique_violation() => {
            let _ = transaction.rollback().await;
            let Some(existing) = event_store.get_by_id(&event_id).await? else {
                return Err(ApiError::Store(error));
            };
            Ok(response_from_event(existing)?)
        }
        Err(error) => Err(ApiError::Store(error)),
    }
}
