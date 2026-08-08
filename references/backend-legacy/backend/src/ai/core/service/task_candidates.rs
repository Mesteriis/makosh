use std::time::Instant;

use serde_json::json;

use super::super::constants::AI_PROMPT_TEMPLATE_VERSION;
use super::super::errors::AiError;
use super::super::helpers::{
    elapsed_ms, event_id_from_command, run_id_from_command, validate_non_empty,
};
use super::super::prompts::{parse_task_candidate_drafts, task_candidate_prompt};
use super::super::runs::{AiRunStore, NewAiRun};
use super::super::types::{AiTaskCandidateRefreshRequest, AiTaskCandidateRefreshResponse};
use super::core::AiService;
use super::events::AiRunEvent;
use crate::ai::hub::AiModelRoute;
use crate::workflows::review_inbox::sync_ai_run_task_candidates_to_review;

impl AiService {
    pub async fn refresh_task_candidates(
        &self,
        request: AiTaskCandidateRefreshRequest,
        actor_id: &str,
    ) -> Result<AiTaskCandidateRefreshResponse, AiError> {
        let command_id = validate_non_empty("command_id", &request.command_id)?;
        let query = validate_non_empty("query", &request.query)?;
        let agent_id = "MAKOSH".to_owned();
        let started_at = Instant::now();
        let run_id = run_id_from_command("task-refresh", &command_id);
        let requested_event_id = event_id_from_command("ai.run.requested", &command_id);
        let completed_event_id = event_id_from_command("ai.run.completed", &command_id);
        let extraction_event_id =
            event_id_from_command("ai.task_extraction.completed", &command_id);
        let run_store = AiRunStore::new(self.pool.clone());
        let chat_model = self.model_routing.extraction.clone();
        let attribution = self.run_attribution(&agent_id).await?;

        run_store
            .start_run(&NewAiRun {
                run_id: run_id.clone(),
                agent_id: agent_id.clone(),
                chat_model: chat_model.clone(),
                embedding_model: self.model_routing.embeddings.clone(),
                prompt_template_version: AI_PROMPT_TEMPLATE_VERSION.to_owned(),
                model_config: self.model_config(),
                query: query.clone(),
                actor_id: actor_id.to_owned(),
                agent_persona_id: Some(attribution.agent_persona_id.clone()),
                owner_persona_id: attribution.owner_persona_id.clone(),
                causation_id: request.causation_id.clone(),
                correlation_id: request.correlation_id.clone(),
                requested_event_id: requested_event_id.clone(),
            })
            .await?;
        self.append_run_event(AiRunEvent {
            event_id: &requested_event_id,
            event_type: "ai.run.requested",
            run_id: &run_id,
            agent_id: &agent_id,
            actor_id,
            query: &query,
            payload: json!({ "workflow": "task_candidates" }),
            correlation_id: request.correlation_id.as_deref(),
        })
        .await?;

        let result: Result<AiTaskCandidateRefreshResponse, AiError> = async {
            let citations = self.retrieve_citations(&query).await?;
            let prompt = task_candidate_prompt(&query, &citations);
            let chat = self.hub.chat(AiModelRoute::Extraction, &prompt).await?;
            let drafts = parse_task_candidate_drafts(&chat.content, &citations)?;
            let created_count = self
                .upsert_ai_task_candidates(&run_id, &drafts, &citations)
                .await?;
            let _ = sync_ai_run_task_candidates_to_review(&self.pool, &run_id).await?;
            let duration_ms = elapsed_ms(started_at);
            let answer = format!("Created {created_count} suggested task candidate(s).");
            let stored = run_store
                .complete_run(
                    &run_id,
                    &answer,
                    &citations,
                    duration_ms,
                    &completed_event_id,
                )
                .await?;
            self.append_run_event(AiRunEvent {
                event_id: &completed_event_id,
                event_type: "ai.run.completed",
                run_id: &run_id,
                agent_id: &agent_id,
                actor_id,
                query: &query,
                payload: json!({
                    "workflow": "task_candidates",
                    "created_count": created_count,
                    "duration_ms": duration_ms,
                }),
                correlation_id: request.correlation_id.as_deref(),
            })
            .await?;
            self.append_run_event(AiRunEvent {
                event_id: &extraction_event_id,
                event_type: "ai.task_extraction.completed",
                run_id: &run_id,
                agent_id: &agent_id,
                actor_id,
                query: &query,
                payload: json!({
                    "created_count": created_count,
                    "candidate_state": "suggested",
                }),
                correlation_id: request.correlation_id.as_deref(),
            })
            .await?;

            Ok(AiTaskCandidateRefreshResponse {
                run_id: run_id.clone(),
                agent_id: agent_id.clone(),
                agent_persona_id: attribution.agent_persona_id,
                owner_persona_id: attribution.owner_persona_id,
                status: stored.status,
                created_count,
                citations,
                model: chat.model,
                embedding_model: self.model_routing.embeddings.clone(),
                created_at: stored.started_at,
                duration_ms,
            })
        }
        .await;

        match result {
            Ok(response) => Ok(response),
            Err(error) => {
                let duration_ms = elapsed_ms(started_at);
                let failed_event_id = event_id_from_command("ai.run.failed", &command_id);
                let error_summary = error.to_string();
                run_store
                    .fail_run(&run_id, &error_summary, duration_ms, &failed_event_id)
                    .await?;
                self.append_run_event(AiRunEvent {
                    event_id: &failed_event_id,
                    event_type: "ai.run.failed",
                    run_id: &run_id,
                    agent_id: &agent_id,
                    actor_id,
                    query: &query,
                    payload: json!({
                        "workflow": "task_candidates",
                        "duration_ms": duration_ms,
                        "reason": error_summary,
                    }),
                    correlation_id: request.correlation_id.as_deref(),
                })
                .await?;
                Err(error)
            }
        }
    }
}
