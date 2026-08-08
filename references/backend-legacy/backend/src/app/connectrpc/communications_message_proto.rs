use crate::domains::communications::messages::models::{ProjectedMessage, ProjectedMessageSummary};
use chrono::{DateTime, Utc};
use makosh_connectrpc_contracts::makosh::communications::v1::CommunicationMessage as ProtoCommunicationMessage;
fn timestamp_string(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}
fn json_string(value: &serde_json::Value) -> String {
    value.to_string()
}
pub(super) fn summary(
    summary: ProjectedMessageSummary,
    read_sync_status: &str,
) -> ProtoCommunicationMessage {
    message(summary.message, summary.attachment_count, read_sync_status)
}
pub(super) fn message(
    message: ProjectedMessage,
    attachment_count: i64,
    read_sync_status: &str,
) -> ProtoCommunicationMessage {
    message_with_body_html(message, attachment_count, None, read_sync_status)
}
pub(super) fn message_with_body_html(
    message: ProjectedMessage,
    attachment_count: i64,
    body_html: Option<String>,
    read_sync_status: &str,
) -> ProtoCommunicationMessage {
    ProtoCommunicationMessage {
        message_id: message.message_id,
        raw_record_id: message.raw_record_id,
        observation_id: message.observation_id,
        account_id: message.account_id,
        provider_record_id: message.provider_record_id,
        subject: message.subject,
        sender: message.sender,
        recipients: message.recipients,
        body_text: message.body_text,
        body_html,
        occurred_at: message.occurred_at.map(timestamp_string),
        projected_at: timestamp_string(message.projected_at),
        channel_kind: message.channel_kind,
        conversation_id: message.conversation_id,
        sender_display_name: message.sender_display_name,
        delivery_state: message.delivery_state,
        message_metadata_json: json_string(&message.message_metadata),
        workflow_state: message.workflow_state.as_str().to_owned(),
        importance_score: message.importance_score.map(i32::from),
        ai_category: message.ai_category,
        ai_summary: message.ai_summary,
        ai_summary_generated_at: message.ai_summary_generated_at.map(timestamp_string),
        ai_state: message.ai_state.map(|state| state.as_str().to_owned()),
        local_state: message.local_state.as_str().to_owned(),
        local_state_changed_at: message.local_state_changed_at.map(timestamp_string),
        local_state_reason: message.local_state_reason,
        is_read: message.is_read,
        read_changed_at: message.read_changed_at.map(timestamp_string),
        read_origin: message.read_origin,
        read_sync_status: read_sync_status.to_owned(),
        attachment_count,
        ..Default::default()
    }
}
