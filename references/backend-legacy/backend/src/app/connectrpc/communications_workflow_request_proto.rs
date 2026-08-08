use crate::app::handlers::communications::workflow_actions::models::{
    WorkflowActionInput, WorkflowActionKind, WorkflowActionRequest, WorkflowActionSource,
};
use connectrpc::{ConnectError, ErrorCode};
use makosh_connectrpc_contracts::makosh::communications::v1::WorkflowActionRequest as ProtoWorkflowActionRequest;
fn invalid(message: impl Into<String>) -> ConnectError {
    ConnectError::new(ErrorCode::InvalidArgument, message.into())
}
fn action(value: &str) -> Result<WorkflowActionKind, ConnectError> {
    match value.trim() {
        "reply" => Ok(WorkflowActionKind::Reply),
        "create_task" => Ok(WorkflowActionKind::CreateTask),
        "create_note" => Ok(WorkflowActionKind::CreateNote),
        "create_document" => Ok(WorkflowActionKind::CreateDocument),
        "create_event" => Ok(WorkflowActionKind::CreateEvent),
        "link_document" => Ok(WorkflowActionKind::LinkDocument),
        "create_persona" | "create_contact" => Ok(WorkflowActionKind::CreatePersona),
        "archive" => Ok(WorkflowActionKind::Archive),
        _ => Err(invalid(format!("invalid workflow action: {value}"))),
    }
}
pub(super) fn request(
    req: ProtoWorkflowActionRequest,
) -> Result<WorkflowActionRequest, ConnectError> {
    Ok(WorkflowActionRequest {
        command_id: req.command_id,
        action: action(req.action.as_str())?,
        source: req.source.as_option().map(|source| WorkflowActionSource {
            kind: source.kind.clone(),
            id: source.id.clone(),
        }),
        input: req
            .input
            .as_option()
            .map(|input| {
                Ok::<_, ConnectError>(WorkflowActionInput {
                    title: input.title.clone(),
                    body: input.body.clone(),
                    email: input.email.clone(),
                    display_name: input.display_name.clone(),
                    starts_at: input
                        .starts_at
                        .as_deref()
                        .map(super::communications_timestamp_policy::parse_timestamp)
                        .transpose()?,
                    ends_at: input
                        .ends_at
                        .as_deref()
                        .map(super::communications_timestamp_policy::parse_timestamp)
                        .transpose()?,
                    due_at: input
                        .due_at
                        .as_deref()
                        .map(super::communications_timestamp_policy::parse_timestamp)
                        .transpose()?,
                    document_id: input.document_id.clone(),
                })
            })
            .transpose()?,
    })
}
