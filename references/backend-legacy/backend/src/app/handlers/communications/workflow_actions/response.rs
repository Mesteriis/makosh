use crate::app::error::types::ApiError;
use crate::domains::communications::messages::models::ProjectedMessage;
use makosh_events_api::EventEnvelope;

use super::models::{
    WorkflowActionKind, WorkflowActionProvenance, WorkflowActionResponse, WorkflowActionStatus,
    WorkflowActionTarget,
};

pub(super) fn base_response(
    command_id: &str,
    event_id: &str,
    action: WorkflowActionKind,
    status: WorkflowActionStatus,
    target: WorkflowActionTarget,
    message: Option<&ProjectedMessage>,
    evidence: Vec<String>,
) -> WorkflowActionResponse {
    WorkflowActionResponse {
        command_id: command_id.to_owned(),
        event_id: event_id.to_owned(),
        action,
        status,
        target,
        provenance: WorkflowActionProvenance {
            source_kind: message.map(|_| "communication_message".to_owned()),
            source_id: message.map(|value| value.message_id.clone()),
            confidence: None,
            evidence,
        },
    }
}

pub(super) fn response_from_event(
    event: EventEnvelope,
) -> Result<WorkflowActionResponse, ApiError> {
    let event_id = event.event_id.clone();
    serde_json::from_value::<WorkflowActionResponse>(event.payload).map_err(|error| {
        tracing::error!(error = %error, event_id = %event_id, "stored workflow action payload is invalid");
        ApiError::InvalidCommunicationQuery("stored workflow action payload is invalid")
    })
}
