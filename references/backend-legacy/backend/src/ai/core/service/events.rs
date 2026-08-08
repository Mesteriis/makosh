use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde_json::{Value, json};

use crate::application::ai_signal_dispatch::dispatch_ai_runtime_signal;
use makosh_events_postgres::store::EventStore;

use super::super::constants::AI_PROMPT_TEMPLATE_VERSION;
use super::super::errors::AiError;
use super::super::helpers::text_preview;
use super::core::AiService;

pub(super) struct AiRunEvent<'a> {
    pub(super) event_id: &'a str,
    pub(super) event_type: &'a str,
    pub(super) run_id: &'a str,
    pub(super) agent_id: &'a str,
    pub(super) actor_id: &'a str,
    pub(super) query: &'a str,
    pub(super) payload: Value,
    pub(super) correlation_id: Option<&'a str>,
}

impl AiService {
    pub(super) async fn append_run_event(&self, event: AiRunEvent<'_>) -> Result<(), AiError> {
        let event_store = EventStore::new(self.pool.clone());
        let builder = NewEventEnvelope::builder(
            event.event_id,
            event.event_type,
            Utc::now(),
            json!({
                "kind": "ai_run",
                "source_id": event.run_id,
            }),
            json!({
                "kind": "ai_run",
                "run_id": event.run_id,
                "agent_id": event.agent_id,
            }),
        )
        .actor(json!({ "actor_id": event.actor_id }))
        .payload(json!({
            "agent_id": event.agent_id,
            "query_preview": text_preview(event.query, 160),
            "details": event.payload,
        }))
        .provenance(json!({
            "runtime": self.hub.runtime_name(),
            "chat_model": self.chat_model,
            "embedding_model": self.embedding_model,
            "prompt_template_version": AI_PROMPT_TEMPLATE_VERSION,
        }));
        let trace_id = event
            .correlation_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(event.run_id);
        let ai_event = builder.correlation_id(trace_id).build()?;
        event_store.append(&ai_event).await?;
        self.append_ai_signal_event(&event, trace_id).await?;
        Ok(())
    }

    async fn append_ai_signal_event(
        &self,
        event: &AiRunEvent<'_>,
        correlation_id: &str,
    ) -> Result<(), AiError> {
        let Some(event_kind) = ai_raw_signal_event_kind(event.event_type) else {
            return Ok(());
        };
        let _ = dispatch_ai_runtime_signal(
            self.pool.clone(),
            event_kind,
            event.run_id,
            json!({
                "kind": "ai_run",
                "source_code": "ai",
                "run_id": event.run_id,
                "agent_id": event.agent_id,
                "event_type": event.event_type,
            }),
            json!({
                "agent_id": event.agent_id,
                "workflow": event.payload.get("workflow").cloned(),
                "details": signal_safe_payload(&event.payload),
            }),
            json!({
                "source": "ai_run_event",
                "source_code": "ai",
                "runtime": self.hub.runtime_name(),
                "chat_model": self.chat_model,
                "embedding_model": self.embedding_model,
                "prompt_template_version": AI_PROMPT_TEMPLATE_VERSION,
                "ai_event_type": event.event_type,
            }),
            Some(correlation_id),
        )
        .await?;
        Ok(())
    }
}

fn ai_raw_signal_event_kind(event_type: &str) -> Option<&'static str> {
    match event_type {
        "ai.run.requested" => Some("run_requested"),
        "ai.run.completed" => Some("run_completed"),
        "ai.run.failed" => Some("run_failed"),
        "ai.task_extraction.completed" => Some("task_extraction"),
        _ => None,
    }
}

fn signal_safe_payload(payload: &Value) -> Value {
    let mut redacted = payload.clone();
    if let Some(object) = redacted.as_object_mut() {
        object.remove("query");
        object.remove("answer");
        object.remove("briefing");
    }
    redacted
}
