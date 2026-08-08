use chrono::{DateTime, Utc};
use makosh_events_api::NewEventEnvelope;
use serde_json::{Value, json};

use super::constants::{
    TASK_CANDIDATE_REVIEW_EVENT_TYPE, TASK_CANDIDATE_REVIEW_SOURCE_KIND,
    TASK_CANDIDATE_REVIEW_SOURCE_PROVIDER,
};
use super::errors::TaskCandidateError;
use super::models::TaskCandidateReviewState;
use super::validation::validate_non_empty;

pub(crate) struct ReviewCommandEvent {
    pub(crate) command_id: String,
    pub(crate) task_candidate_id: String,
    pub(crate) review_state: TaskCandidateReviewState,
    pub(crate) actor_id: String,
    pub(crate) event_id: String,
    pub(crate) occurred_at: DateTime<Utc>,
}

impl ReviewCommandEvent {
    pub(crate) fn into_event(self) -> Result<NewEventEnvelope, TaskCandidateError> {
        let event_id = self.event_id.clone();
        Ok(NewEventEnvelope::builder(
            event_id,
            TASK_CANDIDATE_REVIEW_EVENT_TYPE,
            self.occurred_at,
            json!({
                "kind": TASK_CANDIDATE_REVIEW_SOURCE_KIND,
                "provider": TASK_CANDIDATE_REVIEW_SOURCE_PROVIDER,
                "source_id": self.command_id,
            }),
            json!({
                "kind": "task_candidate_review",
            }),
        )
        .actor(json!({ "actor_id": self.actor_id }))
        .payload(self.review_payload())
        .build()?)
    }

    fn review_payload(&self) -> Value {
        json!({
            "task_candidate_id": self.task_candidate_id,
            "review_state": self.review_state.as_str(),
        })
    }
}

#[derive(Debug)]
pub(crate) struct ReviewEventPayload {
    pub(crate) task_candidate_id: String,
    pub(crate) review_state: TaskCandidateReviewState,
}

impl ReviewEventPayload {
    pub(crate) fn from_payload(payload: &Value) -> Result<Self, TaskCandidateError> {
        let payload = as_object(payload)?;
        Ok(Self {
            task_candidate_id: required_payload_string(payload, "task_candidate_id")?,
            review_state: TaskCandidateReviewState::parse(required_payload_string(
                payload,
                "review_state",
            )?)?,
        })
    }
}

fn as_object(value: &Value) -> Result<&serde_json::Map<String, Value>, TaskCandidateError> {
    value
        .as_object()
        .ok_or_else(|| TaskCandidateError::InvalidPayload("payload".to_owned()))
}

fn required_payload_string(
    payload: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<String, TaskCandidateError> {
    let raw = payload
        .get(field)
        .ok_or_else(|| TaskCandidateError::MissingPayloadField(field.to_owned()))?;
    let value = raw
        .as_str()
        .ok_or_else(|| TaskCandidateError::InvalidPayload(field.to_owned()))?;
    validate_non_empty(field, value)
}
