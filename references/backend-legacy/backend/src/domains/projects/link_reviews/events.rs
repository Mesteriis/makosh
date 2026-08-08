use chrono::Utc;
use makosh_events_api::NewEventEnvelope;
use serde_json::{Value, json};

use super::constants::{
    PROJECT_LINK_REVIEW_EVENT_TYPE, PROJECT_LINK_REVIEW_SOURCE_KIND,
    PROJECT_LINK_REVIEW_SOURCE_PROVIDER,
};
use super::errors::ProjectLinkReviewError;
use super::models::{ProjectLinkReviewCommand, ProjectLinkReviewState, ProjectLinkTargetKind};
use super::validation::validate_non_empty;

impl ProjectLinkReviewCommand {
    pub(crate) fn to_review_event(
        &self,
        event_id: &str,
    ) -> Result<NewEventEnvelope, ProjectLinkReviewError> {
        Ok(NewEventEnvelope::builder(
            event_id,
            PROJECT_LINK_REVIEW_EVENT_TYPE,
            Utc::now(),
            json!({
                "kind": PROJECT_LINK_REVIEW_SOURCE_KIND,
                "provider": PROJECT_LINK_REVIEW_SOURCE_PROVIDER,
                "source_id": self.command_id,
            }),
            json!({
                "kind": "project_link_review",
                "project_id": self.project_id,
            }),
        )
        .actor(json!({ "actor_id": self.actor_id }))
        .payload(self.review_payload())
        .build()?)
    }

    fn review_payload(&self) -> Value {
        json!({
            "project_id": self.project_id,
            "target_kind": self.target_kind.as_str(),
            "target_id": self.target_id,
            "review_state": self.review_state.as_str(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReviewEvent {
    pub(crate) project_id: String,
    pub(crate) target_kind: ProjectLinkTargetKind,
    pub(crate) target_id: String,
    pub(crate) review_state: ProjectLinkReviewState,
}

impl ReviewEvent {
    pub(crate) fn from_payload(payload: &Value) -> Result<Self, ProjectLinkReviewError> {
        let payload = as_object(payload)?;
        Ok(Self {
            project_id: required_payload_string(payload, "project_id")?,
            target_kind: ProjectLinkTargetKind::parse(required_payload_string(
                payload,
                "target_kind",
            )?)?,
            target_id: required_payload_string(payload, "target_id")?,
            review_state: ProjectLinkReviewState::parse(required_payload_string(
                payload,
                "review_state",
            )?)?,
        })
    }
}

fn as_object(value: &Value) -> Result<&serde_json::Map<String, Value>, ProjectLinkReviewError> {
    value
        .as_object()
        .ok_or_else(|| ProjectLinkReviewError::InvalidPayload("payload".to_owned()))
}

fn required_payload_string(
    payload: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<String, ProjectLinkReviewError> {
    let raw = payload
        .get(field)
        .ok_or_else(|| ProjectLinkReviewError::MissingPayloadField(field.to_owned()))?;
    let value = raw
        .as_str()
        .ok_or_else(|| ProjectLinkReviewError::InvalidPayload(field.to_owned()))?;
    validate_non_empty(field, value)
}
