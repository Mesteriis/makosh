use crate::app::handlers::communications::workflow_actions::models::{
    WorkflowActionKind, WorkflowActionResponse, WorkflowActionStatus, WorkflowActionTargetKind,
};
use makosh_connectrpc_contracts::makosh::communications::v1::{
    WorkflowActionProvenance as ProtoWorkflowActionProvenance,
    WorkflowActionResponse as ProtoWorkflowActionResponse,
    WorkflowActionTarget as ProtoWorkflowActionTarget,
};

fn kind(value: &WorkflowActionKind) -> &'static str {
    match value {
        WorkflowActionKind::Reply => "reply",
        WorkflowActionKind::CreateTask => "create_task",
        WorkflowActionKind::CreateNote => "create_note",
        WorkflowActionKind::CreateDocument => "create_document",
        WorkflowActionKind::CreateEvent => "create_event",
        WorkflowActionKind::LinkDocument => "link_document",
        WorkflowActionKind::CreatePersona => "create_persona",
        WorkflowActionKind::Archive => "archive",
    }
}
fn status(value: &WorkflowActionStatus) -> &'static str {
    match value {
        WorkflowActionStatus::Created => "created",
        WorkflowActionStatus::Updated => "updated",
        WorkflowActionStatus::Linked => "linked",
        WorkflowActionStatus::Opened => "opened",
        WorkflowActionStatus::Archived => "archived",
        WorkflowActionStatus::Noop => "noop",
    }
}
fn target_kind(value: &WorkflowActionTargetKind) -> &'static str {
    match value {
        WorkflowActionTargetKind::Message => "message",
        WorkflowActionTargetKind::Task => "task",
        WorkflowActionTargetKind::Compose => "compose",
        WorkflowActionTargetKind::Document => "document",
        WorkflowActionTargetKind::CalendarEvent => "calendar_event",
        WorkflowActionTargetKind::Persona => "persona",
    }
}
pub(super) fn response(item: WorkflowActionResponse) -> ProtoWorkflowActionResponse {
    ProtoWorkflowActionResponse {
        command_id: item.command_id,
        event_id: item.event_id,
        action: kind(&item.action).to_owned(),
        status: status(&item.status).to_owned(),
        target: Some(ProtoWorkflowActionTarget {
            kind: target_kind(&item.target.kind).to_owned(),
            id: item.target.id,
            ..Default::default()
        })
        .into(),
        provenance: Some(ProtoWorkflowActionProvenance {
            source_kind: item.provenance.source_kind,
            source_id: item.provenance.source_id,
            confidence: item.provenance.confidence,
            evidence: item.provenance.evidence,
            ..Default::default()
        })
        .into(),
        ..Default::default()
    }
}
