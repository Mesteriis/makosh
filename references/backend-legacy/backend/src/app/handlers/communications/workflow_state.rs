use super::*;
use crate::domains::communications::ai_state::{
    CommunicationAiState, CommunicationAiStateStore, CommunicationAiStateTransitionRequest,
};
use crate::domains::communications::command_service::CommunicationCommandService;
use crate::workflows::review_inbox::refresh_message_knowledge_candidates_into_review;

#[derive(Deserialize)]
pub(crate) struct WorkflowStateTransitionApiRequest {
    pub(super) workflow_state: String,
}

#[derive(Serialize)]
pub(crate) struct WorkflowStateTransitionApiResponse {
    pub(super) message_id: String,
    pub(super) workflow_state: String,
    pub(super) previous_state: String,
}

#[derive(Serialize)]
pub(crate) struct WorkflowStateCountsApiResponse {
    pub(super) counts: Vec<WorkflowStateCountApiItem>,
}

#[derive(Serialize)]
pub(crate) struct WorkflowStateCountApiItem {
    pub(super) state: String,
    pub(super) count: i64,
}

#[derive(Deserialize)]
pub(crate) struct WorkflowStateCountsQuery {
    pub(super) account_id: Option<String>,
    pub(super) local_state: Option<String>,
}
pub(crate) async fn put_v1_message_workflow_state(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<WorkflowStateTransitionApiRequest>,
) -> Result<Json<WorkflowStateTransitionApiResponse>, ApiError> {
    let actor_id = "makosh-frontend".to_string();

    let new_state = request
        .workflow_state
        .parse::<WorkflowState>()
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid workflow state value"))?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::message_workflow_state_set(
            &actor_id,
            &message_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let result = CommunicationCommandService::new(pool)
        .transition_message_workflow_state(&message_id, new_state, &actor_id)
        .await?;

    Ok(Json(WorkflowStateTransitionApiResponse {
        message_id: result.updated.message_id,
        workflow_state: result.updated.workflow_state.as_str().to_owned(),
        previous_state: result.previous_state,
    }))
}

pub(crate) async fn get_v1_message_workflow_state_counts(
    State(state): State<AppState>,
    Query(query): Query<WorkflowStateCountsQuery>,
) -> Result<Json<WorkflowStateCountsApiResponse>, ApiError> {
    let local_state = query
        .local_state
        .as_deref()
        .unwrap_or("active")
        .parse::<LocalMessageState>()
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid local_state value"))?;
    let counts = message_store(&state)?
        .count_messages_by_state_with_local_state(query.account_id.as_deref(), local_state)
        .await?
        .into_iter()
        .map(|c| WorkflowStateCountApiItem {
            state: c.state.as_str().to_owned(),
            count: c.count,
        })
        .collect();

    Ok(Json(WorkflowStateCountsApiResponse { counts }))
}

#[derive(Serialize)]
pub(crate) struct MessageAnalyzeResponse {
    pub(super) message_id: String,
    pub(super) analyzed: bool,
    pub(super) category: Option<String>,
    pub(super) summary: Option<String>,
    pub(super) summary_contract: EmailSummaryContract,
    pub(super) importance_score: Option<i16>,
    pub(super) workflow_state: String,
    pub(super) source: String,
    pub(super) confidence: Option<f64>,
    pub(super) evidence: Vec<String>,
}

pub(crate) async fn post_v1_message_analyze(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<MessageAnalyzeResponse>, ApiError> {
    let store = message_store(&state)?;
    let ai_state_store =
        crate::app::api_support::stores::domain_stores::app_store::<CommunicationAiStateStore>(
            state
                .database
                .pool()
                .ok_or(ApiError::DatabaseNotConfigured)?
                .clone(),
        );

    let message = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;

    // Mark analysis as processing to reflect runtime activity for UI/state consumers.
    let _ = ai_state_store
        .transition(
            &message_id,
            CommunicationAiStateTransitionRequest {
                ai_state: CommunicationAiState::Processing,
                review_reason: None,
                last_error: None,
            },
        )
        .await?;

    // Always run heuristics (fast, no external dependency)
    let heuristic_score = EmailIntelligenceService::heuristic_score(&message);
    let heuristic_category = EmailIntelligenceService::heuristic_category(&message);
    let summary_contract = EmailIntelligenceService::heuristic_structured_summary(&message);

    store
        .set_ai_analysis(
            &message_id,
            heuristic_category.as_deref(),
            None,
            Some(heuristic_score),
        )
        .await?;
    let mut metadata = message.message_metadata.clone();
    metadata["ai_summary_contract"] = serde_json::to_value(&summary_contract).map_err(|_| {
        ApiError::InvalidCommunicationQuery("summary contract serialization failed")
    })?;
    store.set_message_metadata(&message_id, &metadata).await?;

    // If score is high, auto-transition to needs_action
    if heuristic_score >= 75 && message.workflow_state.as_str() == "new" {
        let _ = store
            .transition_workflow_state(&message_id, WorkflowState::NeedsAction)
            .await;
    }

    let _ = ai_state_store
        .transition(
            &message_id,
            CommunicationAiStateTransitionRequest {
                ai_state: CommunicationAiState::Processed,
                review_reason: None,
                last_error: None,
            },
        )
        .await?;

    let updated = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let _ = refresh_message_knowledge_candidates_into_review(&pool, std::slice::from_ref(&updated))
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "message knowledge candidate review sync failed");
            ApiError::InvalidCommunicationQuery("message knowledge candidate review sync failed")
        })?;
    let evidence = crate::domains::communications::explain::explain_importance(&updated).reasons;

    Ok(Json(MessageAnalyzeResponse {
        message_id: updated.message_id,
        analyzed: true,
        category: updated.ai_category,
        summary: updated.ai_summary,
        summary_contract,
        importance_score: updated.importance_score,
        workflow_state: updated.workflow_state.as_str().to_owned(),
        source: "local_heuristic".to_owned(),
        confidence: None,
        evidence,
    }))
}

#[derive(Deserialize)]
pub(crate) struct ThreadListQuery {
    pub(super) account_id: Option<String>,
    pub(super) cursor: Option<String>,
    pub(super) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct ThreadListResponse {
    pub(super) items: Vec<crate::domains::communications::threads::CommunicationThread>,
    pub(super) next_cursor: Option<String>,
    pub(super) has_more: bool,
}

#[derive(Deserialize)]
pub(crate) struct ThreadMessagesQuery {
    pub(super) account_id: Option<String>,
    pub(super) subject: Option<String>,
    pub(super) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct ThreadMessagesResponse {
    pub(super) items: Vec<crate::domains::communications::threads::ThreadMessage>,
}
