use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde_json::json;

use crate::ai::core::agents::{AiAgentListResponse, v3_agents};
use crate::ai::core::constants::AI_EMBEDDING_DIMENSION;
use crate::ai::core::constants::AI_PROMPT_TEMPLATE_VERSION;
use crate::ai::core::helpers::{event_id_from_command, run_id_from_command, text_preview};
use crate::ai::core::runs::{AiAgentRun, AiRunStore, NewAiRun};
use crate::ai::core::types::{
    AiAnswerRequest, AiHubRequestAcceptedResponse, AiMeetingPrepRequest, AiStatusResponse,
    AiTaskCandidateRefreshRequest,
};
use crate::app::api_support::{review_lists::*, stores::ai_runtime::*};
use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::application::ai_signal_dispatch::dispatch_ai_runtime_signal;
use makosh_events_postgres::store::EventStore;
pub(crate) async fn get_ai_status(
    State(state): State<AppState>,
) -> Result<Json<AiStatusResponse>, ApiError> {
    let runtime_settings = ai_runtime_settings(&state).await?;
    let runtime = ai_runtime_client(&state, &runtime_settings)?;
    let version = runtime.version().await;
    let models = runtime.models().await;
    let chat_model = runtime_settings.chat_model;
    let embedding_model = runtime_settings.embedding_model;
    let chat_model_available = models
        .as_ref()
        .map(|models| models.iter().any(|model| model == &chat_model))
        .unwrap_or(false);
    let embedding_model_available = models
        .as_ref()
        .map(|models| models.iter().any(|model| model == &embedding_model))
        .unwrap_or(false);

    Ok(Json(AiStatusResponse {
        runtime: runtime.runtime_name().to_owned(),
        status: if version.is_ok()
            && models.is_ok()
            && chat_model_available
            && embedding_model_available
        {
            "ok"
        } else {
            "unavailable"
        }
        .to_owned(),
        version: version.ok().flatten(),
        chat_model,
        embedding_model,
        embedding_dimension: AI_EMBEDDING_DIMENSION,
        chat_model_available,
        embedding_model_available,
    }))
}

pub(crate) async fn get_ai_agents(
    State(state): State<AppState>,
) -> Result<Json<AiAgentListResponse>, ApiError> {
    let runtime_settings = ai_runtime_settings(&state).await?;
    let mut items = v3_agents(&runtime_settings.chat_model);

    if let Some(persona_attribution) = ai_persona_attribution_port_optional(&state) {
        for item in &mut items {
            let persona = persona_attribution
                .upsert_ai_agent_persona(item.agent_id, item.display_name)
                .await
                .map_err(crate::ai::core::errors::AiError::from)?;
            item.persona_id = Some(persona.persona_id);
            item.persona_type = Some(persona.persona_type);
            item.persona_email = Some(persona.persona_email);
        }
    }

    Ok(Json(AiAgentListResponse { items }))
}

pub(crate) async fn get_ai_runs(
    State(state): State<AppState>,
    Query(query): Query<AiRunsQuery>,
) -> Result<Json<AiRunListResponse>, ApiError> {
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let runs = ai_run_store(&state)?.list_runs(limit).await?;

    Ok(Json(AiRunListResponse { items: runs }))
}

pub(crate) async fn get_ai_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<AiAgentRun>, ApiError> {
    let Some(run) = ai_run_store(&state)?.get_run(&run_id).await? else {
        return Err(ApiError::AiRunNotFound);
    };

    Ok(Json(run))
}

pub(crate) async fn post_ai_answer(
    State(state): State<AppState>,
    Json(request): Json<AiAnswerRequest>,
) -> Result<(StatusCode, Json<AiHubRequestAcceptedResponse>), ApiError> {
    ensure_ai_requests_allowed(&state).await?;
    let actor_id = "makosh-frontend".to_string();
    let accepted = accepted_ai_request(
        "answer",
        &request.command_id,
        request.correlation_id.as_deref(),
    );
    append_ai_hub_event(
        &state,
        &accepted,
        AiHubEventInput {
            event_type: "ai.hub.requested",
            actor_id: &actor_id,
            agent_id: "MNEMOSYNE",
            query: &request.query,
            details: json!({
                "route_slot": "default_chat",
                "workflow": "answer",
            }),
            provenance_extra: json!({
                "source": "ai_api",
                "workflow": "answer",
            }),
        },
    )
    .await?;

    let task_state = state.clone();
    let task_actor = actor_id.clone();
    let task_request = request.clone();
    let task_accepted = accepted.clone();
    tokio::spawn(async move {
        if let Err(error) =
            process_answer_request(task_state, task_request, task_actor, task_accepted).await
        {
            tracing::warn!(error = ?error, "AI answer background execution failed");
        }
    });

    Ok((StatusCode::ACCEPTED, Json(accepted.into_response())))
}

pub(crate) async fn post_ai_task_candidates_refresh(
    State(state): State<AppState>,
    Json(request): Json<AiTaskCandidateRefreshRequest>,
) -> Result<(StatusCode, Json<AiHubRequestAcceptedResponse>), ApiError> {
    ensure_ai_requests_allowed(&state).await?;
    let actor_id = "makosh-frontend".to_string();
    let accepted = accepted_ai_request(
        "task-refresh",
        &request.command_id,
        request.correlation_id.as_deref(),
    );
    append_ai_hub_event(
        &state,
        &accepted,
        AiHubEventInput {
            event_type: "ai.hub.requested",
            actor_id: &actor_id,
            agent_id: "MAKOSH",
            query: &request.query,
            details: json!({
                "route_slot": "extraction",
                "workflow": "task_candidates",
            }),
            provenance_extra: json!({
                "source": "ai_api",
                "workflow": "task_candidates",
            }),
        },
    )
    .await?;

    let task_state = state.clone();
    let task_actor = actor_id.clone();
    let task_request = request.clone();
    let task_accepted = accepted.clone();
    tokio::spawn(async move {
        if let Err(error) =
            process_task_candidate_request(task_state, task_request, task_actor, task_accepted)
                .await
        {
            tracing::warn!(error = ?error, "AI task refresh background execution failed");
        }
    });

    Ok((StatusCode::ACCEPTED, Json(accepted.into_response())))
}

pub(crate) async fn post_ai_meeting_prep(
    State(state): State<AppState>,
    Json(request): Json<AiMeetingPrepRequest>,
) -> Result<(StatusCode, Json<AiHubRequestAcceptedResponse>), ApiError> {
    ensure_ai_requests_allowed(&state).await?;
    let actor_id = "makosh-frontend".to_string();
    let accepted = accepted_ai_request(
        "meeting-prep",
        &request.command_id,
        request.correlation_id.as_deref(),
    );
    let query = request.topic.clone();
    append_ai_hub_event(
        &state,
        &accepted,
        AiHubEventInput {
            event_type: "ai.hub.requested",
            actor_id: &actor_id,
            agent_id: "HESTIA",
            query: &query,
            details: json!({
                "route_slot": "meeting_prep",
                "workflow": "meeting_prep",
                "project_id": request.project_id.clone(),
                "persona_id": request.persona_id.clone(),
            }),
            provenance_extra: json!({
                "source": "ai_api",
                "workflow": "meeting_prep",
            }),
        },
    )
    .await?;

    let task_state = state.clone();
    let task_actor = actor_id.clone();
    let task_request = request.clone();
    let task_accepted = accepted.clone();
    tokio::spawn(async move {
        if let Err(error) =
            process_meeting_prep_request(task_state, task_request, task_actor, task_accepted).await
        {
            tracing::warn!(error = ?error, "AI meeting prep background execution failed");
        }
    });

    Ok((StatusCode::ACCEPTED, Json(accepted.into_response())))
}

async fn ensure_ai_requests_allowed(state: &AppState) -> Result<(), ApiError> {
    if crate::app::api_support::stores::ai_runtime::ai_requests_allowed(state).await? {
        return Ok(());
    }

    Err(ApiError::FailedPrecondition(
        "AI runtime is disabled by Signal Hub policy or runtime state".to_owned(),
    ))
}

#[derive(Clone)]
struct AcceptedAiRequest {
    request_id: String,
    run_id: String,
    status: String,
    event_id: String,
    correlation_id: String,
}

impl AcceptedAiRequest {
    fn into_response(self) -> AiHubRequestAcceptedResponse {
        AiHubRequestAcceptedResponse {
            request_id: self.request_id,
            run_id: self.run_id,
            status: self.status,
            event_id: self.event_id,
            correlation_id: self.correlation_id,
        }
    }
}

struct AiHubEventInput<'a> {
    event_type: &'a str,
    actor_id: &'a str,
    agent_id: &'a str,
    query: &'a str,
    details: serde_json::Value,
    provenance_extra: serde_json::Value,
}

struct InitializationFailureInput<'a> {
    actor_id: &'a str,
    agent_id: &'a str,
    query: &'a str,
    route_slot: &'a str,
    causation_id: Option<&'a str>,
    correlation_id: Option<&'a str>,
    error_summary: String,
}

fn accepted_ai_request(
    workflow: &str,
    command_id: &str,
    correlation_id: Option<&str>,
) -> AcceptedAiRequest {
    let run_id = run_id_from_command(workflow, command_id);
    AcceptedAiRequest {
        request_id: command_id.trim().to_owned(),
        run_id: run_id.clone(),
        status: "accepted".to_owned(),
        event_id: event_id_from_command("ai.hub.requested", command_id),
        correlation_id: correlation_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .unwrap_or(run_id),
    }
}

async fn process_answer_request(
    state: AppState,
    request: AiAnswerRequest,
    actor_id: String,
    accepted: AcceptedAiRequest,
) -> Result<(), ApiError> {
    let service = match ai_service(&state).await {
        Ok(service) => service,
        Err(error) => {
            record_ai_request_initialization_failure(
                &state,
                &accepted,
                InitializationFailureInput {
                    actor_id: &actor_id,
                    agent_id: "MNEMOSYNE",
                    query: &request.query,
                    route_slot: "default_chat",
                    causation_id: request.causation_id.as_deref(),
                    correlation_id: request.correlation_id.as_deref(),
                    error_summary: format!("{error:?}"),
                },
            )
            .await?;
            return Ok(());
        }
    };

    match service.answer(request.clone(), &actor_id).await {
        Ok(_) => {
            append_ai_hub_event(
                &state,
                &accepted,
                AiHubEventInput {
                    event_type: "ai.hub.completed",
                    actor_id: &actor_id,
                    agent_id: request.agent_id.as_deref().unwrap_or("MNEMOSYNE"),
                    query: &request.query,
                    details: json!({
                        "route_slot": "default_chat",
                        "workflow": "answer",
                    }),
                    provenance_extra: json!({
                        "source": "ai_api",
                        "workflow": "answer",
                        "status": "completed",
                    }),
                },
            )
            .await?;
        }
        Err(error) => {
            append_ai_hub_event(
                &state,
                &accepted,
                AiHubEventInput {
                    event_type: "ai.hub.failed",
                    actor_id: &actor_id,
                    agent_id: request.agent_id.as_deref().unwrap_or("MNEMOSYNE"),
                    query: &request.query,
                    details: json!({
                        "route_slot": "default_chat",
                        "workflow": "answer",
                        "reason": error.to_string(),
                    }),
                    provenance_extra: json!({
                        "source": "ai_api",
                        "workflow": "answer",
                        "status": "failed",
                    }),
                },
            )
            .await?;
        }
    }

    Ok(())
}

async fn process_task_candidate_request(
    state: AppState,
    request: AiTaskCandidateRefreshRequest,
    actor_id: String,
    accepted: AcceptedAiRequest,
) -> Result<(), ApiError> {
    let service = match ai_service(&state).await {
        Ok(service) => service,
        Err(error) => {
            record_ai_request_initialization_failure(
                &state,
                &accepted,
                InitializationFailureInput {
                    actor_id: &actor_id,
                    agent_id: "MAKOSH",
                    query: &request.query,
                    route_slot: "extraction",
                    causation_id: request.causation_id.as_deref(),
                    correlation_id: request.correlation_id.as_deref(),
                    error_summary: format!("{error:?}"),
                },
            )
            .await?;
            return Ok(());
        }
    };

    match service
        .refresh_task_candidates(request.clone(), &actor_id)
        .await
    {
        Ok(_) => {
            append_ai_hub_event(
                &state,
                &accepted,
                AiHubEventInput {
                    event_type: "ai.hub.completed",
                    actor_id: &actor_id,
                    agent_id: "MAKOSH",
                    query: &request.query,
                    details: json!({
                        "route_slot": "extraction",
                        "workflow": "task_candidates",
                    }),
                    provenance_extra: json!({
                        "source": "ai_api",
                        "workflow": "task_candidates",
                        "status": "completed",
                    }),
                },
            )
            .await?;
        }
        Err(error) => {
            append_ai_hub_event(
                &state,
                &accepted,
                AiHubEventInput {
                    event_type: "ai.hub.failed",
                    actor_id: &actor_id,
                    agent_id: "MAKOSH",
                    query: &request.query,
                    details: json!({
                        "route_slot": "extraction",
                        "workflow": "task_candidates",
                        "reason": error.to_string(),
                    }),
                    provenance_extra: json!({
                        "source": "ai_api",
                        "workflow": "task_candidates",
                        "status": "failed",
                    }),
                },
            )
            .await?;
        }
    }

    Ok(())
}

async fn process_meeting_prep_request(
    state: AppState,
    request: AiMeetingPrepRequest,
    actor_id: String,
    accepted: AcceptedAiRequest,
) -> Result<(), ApiError> {
    let service = match ai_service(&state).await {
        Ok(service) => service,
        Err(error) => {
            record_ai_request_initialization_failure(
                &state,
                &accepted,
                InitializationFailureInput {
                    actor_id: &actor_id,
                    agent_id: "HESTIA",
                    query: &request.topic,
                    route_slot: "meeting_prep",
                    causation_id: request.causation_id.as_deref(),
                    correlation_id: request.correlation_id.as_deref(),
                    error_summary: format!("{error:?}"),
                },
            )
            .await?;
            return Ok(());
        }
    };

    match service.meeting_prep(request.clone(), &actor_id).await {
        Ok(_) => {
            append_ai_hub_event(
                &state,
                &accepted,
                AiHubEventInput {
                    event_type: "ai.hub.completed",
                    actor_id: &actor_id,
                    agent_id: "HESTIA",
                    query: &request.topic,
                    details: json!({
                        "route_slot": "meeting_prep",
                        "workflow": "meeting_prep",
                        "project_id": request.project_id.clone(),
                        "persona_id": request.persona_id.clone(),
                    }),
                    provenance_extra: json!({
                        "source": "ai_api",
                        "workflow": "meeting_prep",
                        "status": "completed",
                    }),
                },
            )
            .await?;
        }
        Err(error) => {
            append_ai_hub_event(
                &state,
                &accepted,
                AiHubEventInput {
                    event_type: "ai.hub.failed",
                    actor_id: &actor_id,
                    agent_id: "HESTIA",
                    query: &request.topic,
                    details: json!({
                        "route_slot": "meeting_prep",
                        "workflow": "meeting_prep",
                        "project_id": request.project_id.clone(),
                        "persona_id": request.persona_id.clone(),
                        "reason": error.to_string(),
                    }),
                    provenance_extra: json!({
                        "source": "ai_api",
                        "workflow": "meeting_prep",
                        "status": "failed",
                    }),
                },
            )
            .await?;
        }
    }

    Ok(())
}

async fn record_ai_request_initialization_failure(
    state: &AppState,
    accepted: &AcceptedAiRequest,
    failure: InitializationFailureInput<'_>,
) -> Result<(), ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let settings = ai_runtime_settings(state).await?;
    let run_store = AiRunStore::new(pool.clone());
    let runtime_name = match settings.provider {
        crate::platform::config::ai::AiRuntimeProvider::Ollama => "ollama",
        crate::platform::config::ai::AiRuntimeProvider::OmniRoute => "omniroute",
    };

    run_store
        .start_run(&NewAiRun {
            run_id: accepted.run_id.clone(),
            agent_id: failure.agent_id.to_owned(),
            chat_model: settings.chat_model.clone(),
            embedding_model: settings.embedding_model.clone(),
            prompt_template_version: AI_PROMPT_TEMPLATE_VERSION.to_owned(),
            model_config: json!({
                "route_slot": failure.route_slot,
                "route_resolution": "failed_before_execution",
            }),
            query: failure.query.to_owned(),
            actor_id: failure.actor_id.to_owned(),
            agent_persona_id: None,
            owner_persona_id: None,
            causation_id: failure.causation_id.map(str::to_owned),
            correlation_id: failure.correlation_id.map(str::to_owned),
            requested_event_id: accepted.event_id.clone(),
        })
        .await?;
    run_store
        .fail_run(
            &accepted.run_id,
            &failure.error_summary,
            0,
            &event_id_from_command("ai.run.failed", &accepted.request_id),
        )
        .await?;

    append_ai_hub_event(
        state,
        accepted,
        AiHubEventInput {
            event_type: "ai.hub.failed",
            actor_id: failure.actor_id,
            agent_id: failure.agent_id,
            query: failure.query,
            details: json!({
                "route_slot": failure.route_slot,
                "reason": failure.error_summary,
            }),
            provenance_extra: json!({
                "source": "ai_api",
                "status": "failed",
                "runtime": runtime_name,
            }),
        },
    )
    .await?;

    Ok(())
}

async fn append_ai_hub_event(
    state: &AppState,
    accepted: &AcceptedAiRequest,
    input: AiHubEventInput<'_>,
) -> Result<(), ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = NewEventEnvelope::builder(
        if input.event_type == "ai.hub.requested" {
            accepted.event_id.clone()
        } else {
            event_id_from_command(input.event_type, &accepted.request_id)
        },
        input.event_type,
        Utc::now(),
        json!({
            "kind": "ai_run",
            "source_id": accepted.run_id.as_str(),
        }),
        json!({
            "kind": "ai_run",
            "run_id": accepted.run_id.as_str(),
            "agent_id": input.agent_id,
        }),
    )
    .actor(json!({ "actor_id": input.actor_id }))
    .payload(json!({
        "agent_id": input.agent_id,
        "query_preview": text_preview(input.query, 160),
        "details": input.details.clone(),
    }))
    .provenance(input.provenance_extra)
    .correlation_id(&accepted.correlation_id)
    .build()?;
    EventStore::new(pool.clone()).append(&event).await?;

    let signal_kind = match input.event_type {
        "ai.hub.requested" => "hub_requested",
        "ai.hub.completed" => "hub_completed",
        "ai.hub.failed" => "hub_failed",
        _ => return Ok(()),
    };
    let _ = dispatch_ai_runtime_signal(
        pool,
        signal_kind,
        &accepted.run_id,
        json!({
            "kind": "ai_run",
            "source_code": "ai",
            "run_id": accepted.run_id.as_str(),
            "agent_id": input.agent_id,
            "event_type": input.event_type,
        }),
        json!({
            "agent_id": input.agent_id,
            "details": input.details,
        }),
        json!({
            "source": "ai_hub_event",
            "event_type": input.event_type,
        }),
        Some(&accepted.correlation_id),
    )
    .await?;

    Ok(())
}
