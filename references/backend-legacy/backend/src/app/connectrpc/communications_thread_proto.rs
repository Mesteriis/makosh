use crate::domains::communications::threads::CommunicationThread;
use crate::domains::communications::threads::ThreadMessage;
use chrono::{DateTime, Utc};
use makosh_connectrpc_contracts::makosh::communications::v1::ThreadMessage as ProtoThreadMessage;
fn timestamp_string(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}
pub(super) fn message(item: ThreadMessage) -> ProtoThreadMessage {
    ProtoThreadMessage {
        message_id: item.message_id,
        provider_record_id: item.provider_record_id,
        account_id: item.account_id,
        subject: item.subject,
        sender: item.sender,
        sender_display_name: item.sender_display_name,
        body_text: item.body_text,
        occurred_at: item.occurred_at.map(timestamp_string),
        projected_at: timestamp_string(item.projected_at),
        workflow_state: item.workflow_state,
        importance_score: item.importance_score.map(i32::from),
        ai_category: item.ai_category,
        ai_summary: item.ai_summary,
        delivery_state: item.delivery_state,
        attachment_count: item.attachment_count,
        attachments: item
            .attachments
            .into_iter()
            .map(super::communications_attachment_proto::from_thread)
            .collect(),
        ..Default::default()
    }
}

pub(super) fn thread(
    item: CommunicationThread,
) -> makosh_connectrpc_contracts::makosh::communications::v1::CommunicationThread {
    makosh_connectrpc_contracts::makosh::communications::v1::CommunicationThread {
        thread_id: item.thread_id,
        account_id: item.account_id,
        subject: item.subject,
        message_count: item.message_count,
        participant_count: item.participant_count,
        first_message_at: item.first_message_at.map(timestamp_string),
        last_message_at: item.last_message_at.map(timestamp_string),
        last_activity_at: timestamp_string(item.last_activity_at),
        has_open_action: item.has_open_action,
        has_attachments: item.has_attachments,
        dominant_workflow_state: item.dominant_workflow_state,
        ..Default::default()
    }
}
