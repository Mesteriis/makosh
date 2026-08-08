use crate::domains::communications::folders::{FolderMessage, FolderMessageActionResponse};
use chrono::{DateTime, Utc};
use makosh_connectrpc_contracts::makosh::communications::v1::{
    FolderMessage as ProtoFolderMessage,
    FolderMessageActionResult as ProtoFolderMessageActionResult,
};
fn timestamp_string(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}
pub(super) fn message(item: FolderMessage) -> ProtoFolderMessage {
    ProtoFolderMessage {
        folder_id: item.folder_id,
        message_id: item.message_id,
        account_id: item.account_id,
        subject: item.subject,
        sender: item.sender,
        occurred_at: item.occurred_at.map(timestamp_string),
        projected_at: timestamp_string(item.projected_at),
        workflow_state: item.workflow_state.as_str().to_owned(),
        local_state: item.local_state.as_str().to_owned(),
        added_at: timestamp_string(item.added_at),
        attachment_count: item.attachment_count,
        ..Default::default()
    }
}
pub(super) fn message_action(item: FolderMessageActionResponse) -> ProtoFolderMessageActionResult {
    ProtoFolderMessageActionResult {
        operation: item.operation.as_str().to_owned(),
        folder_id: item.folder_id,
        message_id: item.message_id,
        message: Some(message(item.message)).into(),
        ..Default::default()
    }
}
