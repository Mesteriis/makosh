use crate::domains::communications::bulk_actions::BulkMessageActionOutcome;
use crate::domains::communications::messages::models::WorkflowStateCount;
use makosh_connectrpc_contracts::makosh::communications::v1::{
    BulkMessageActionResponse as ProtoBulkMessageActionResponse,
    CommunicationSearchResult as ProtoCommunicationSearchResult,
    WorkflowStateCount as ProtoWorkflowStateCount,
};

pub(super) fn search_result(
    item: crate::engines::search::models::SearchResult,
) -> ProtoCommunicationSearchResult {
    ProtoCommunicationSearchResult {
        object_id: item.object_id,
        object_kind: item.object_kind,
        title: item.title,
        ..Default::default()
    }
}
pub(super) fn bulk_message_action_outcome(
    outcome: BulkMessageActionOutcome,
) -> ProtoBulkMessageActionResponse {
    ProtoBulkMessageActionResponse {
        action: outcome.action,
        requested_count: outcome.requested_count as u32,
        matched_count: outcome.matched_count as u32,
        updated_count: outcome.updated_count as u32,
        not_found: outcome.not_found,
        ..Default::default()
    }
}

pub(super) fn workflow_state_count(item: WorkflowStateCount) -> ProtoWorkflowStateCount {
    ProtoWorkflowStateCount {
        state: item.state.as_str().to_owned(),
        count: item.count,
        ..Default::default()
    }
}
