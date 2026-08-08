use axum::http::HeaderMap;

use crate::app::error::types::ApiError;
use crate::domains::communications::messages::models::ProjectedMessage;

use super::models::WorkflowActionRequest;

pub(super) fn require_source_message<'a>(
    request: &WorkflowActionRequest,
    message: Option<&'a ProjectedMessage>,
) -> Result<&'a ProjectedMessage, ApiError> {
    if request.source.is_none() {
        return Err(ApiError::InvalidCommunicationQuery(
            "workflow action requires source message",
        ));
    }
    message.ok_or(ApiError::CommunicationMessageNotFound)
}

pub(super) fn input_title(
    request: &WorkflowActionRequest,
    message: Option<&ProjectedMessage>,
    fallback: &str,
) -> Result<String, ApiError> {
    if let Some(title) = request
        .input
        .as_ref()
        .and_then(|value| value.title.as_ref())
    {
        return normalize_non_empty("title", title);
    }
    if let Some(message) = message {
        return normalize_non_empty("title", &message.subject);
    }
    Ok(fallback.to_owned())
}

pub(super) fn normalize_non_empty(field: &'static str, value: &str) -> Result<String, ApiError> {
    let normalized = value.trim().to_owned();
    if normalized.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(match field {
            "command_id" => "command_id must not be empty",
            "source.id" => "source id must not be empty",
            "document_id" => "document_id must not be empty",
            "title" => "title must not be empty",
            _ => "workflow action field must not be empty",
        }));
    }
    Ok(normalized)
}

pub(super) fn actor_id_from_headers(headers: &HeaderMap) -> String {
    headers
        .get("x-makosh-actor-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("makosh-frontend")
        .to_owned()
}
