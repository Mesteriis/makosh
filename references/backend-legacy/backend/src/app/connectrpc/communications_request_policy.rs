pub(super) fn trim_non_empty_recipients(recipients: Vec<String>) -> Vec<String> {
    recipients
        .into_iter()
        .map(|recipient| recipient.trim().to_owned())
        .filter(|recipient| !recipient.is_empty())
        .collect()
}

pub(super) fn normalize_limit(limit: u32, default: i64, max: i64) -> i64 {
    match limit {
        0 => default,
        value => i64::from(value).clamp(1, max),
    }
}
use connectrpc::{ConnectError, ErrorCode};

pub(super) fn invalid_argument_error(message: impl Into<String>) -> ConnectError {
    ConnectError::new(ErrorCode::InvalidArgument, message.into())
}

use crate::domains::communications::bulk_actions::BulkMessageAction;
use crate::domains::communications::drafts::DraftStatus;
use crate::domains::communications::messages::models::MessageSearchMatchMode;
use crate::domains::communications::messages::states::{LocalMessageState, WorkflowState};
use crate::domains::communications::outbox::CommunicationOutboxStatus;
use makosh_connectrpc_contracts::makosh::communications::v1::BulkMessageActionRequest as ProtoBulkMessageActionRequest;
pub(super) fn parse_workflow_state(value: &str) -> Result<WorkflowState, ConnectError> {
    value
        .parse()
        .map_err(|_| invalid_argument_error(format!("invalid workflow_state: {value}")))
}
pub(super) fn parse_local_state(value: &str) -> Result<LocalMessageState, ConnectError> {
    value
        .parse()
        .map_err(|_| invalid_argument_error(format!("invalid local_state: {value}")))
}
pub(super) fn parse_match_mode(
    value: Option<&str>,
) -> Result<MessageSearchMatchMode, ConnectError> {
    match value.unwrap_or("all").trim() {
        "" | "all" => Ok(MessageSearchMatchMode::All),
        "any" => Ok(MessageSearchMatchMode::Any),
        other => Err(invalid_argument_error(format!(
            "invalid match_mode: {other}"
        ))),
    }
}

pub(super) fn parse_draft_status(value: &str) -> Result<DraftStatus, ConnectError> {
    DraftStatus::parse(value)
        .ok_or_else(|| invalid_argument_error(format!("invalid draft status: {value}")))
}
pub(super) fn parse_outbox_status(value: &str) -> Result<CommunicationOutboxStatus, ConnectError> {
    CommunicationOutboxStatus::parse(value)
        .ok_or_else(|| invalid_argument_error(format!("invalid outbox status: {value}")))
}

pub(super) fn parse_bulk_action_request(
    request: &ProtoBulkMessageActionRequest,
) -> Result<BulkMessageAction, ConnectError> {
    match request.action.trim() {
        "mark_read" => Ok(BulkMessageAction::MarkRead),
        "mark_unread" => Ok(BulkMessageAction::MarkUnread),
        "archive" => Ok(BulkMessageAction::Archive),
        "trash" => Ok(BulkMessageAction::Trash),
        "restore" => Ok(BulkMessageAction::Restore),
        "pin" => Ok(BulkMessageAction::Pin),
        "unpin" => Ok(BulkMessageAction::Unpin),
        "important" => Ok(BulkMessageAction::Important),
        "not_important" => Ok(BulkMessageAction::NotImportant),
        "star" => Ok(BulkMessageAction::Star),
        "unstar" => Ok(BulkMessageAction::Unstar),
        "add_label" => request
            .label
            .clone()
            .map(BulkMessageAction::AddLabel)
            .ok_or_else(|| invalid_argument_error("label is required for add_label")),
        "remove_label" => request
            .label
            .clone()
            .map(BulkMessageAction::RemoveLabel)
            .ok_or_else(|| invalid_argument_error("label is required for remove_label")),
        "snooze" => {
            let until = request
                .snooze_until
                .as_deref()
                .ok_or_else(|| invalid_argument_error("snooze_until is required for snooze"))?;
            Ok(BulkMessageAction::Snooze(
                super::communications_timestamp_policy::parse_timestamp(until)?,
            ))
        }
        _ => Err(invalid_argument_error("invalid bulk message action")),
    }
}
