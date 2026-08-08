use crate::domains::communications::outbox::CommunicationOutboxItem;
use chrono::{DateTime, Utc};
use makosh_connectrpc_contracts::makosh::communications::v1::CommunicationOutboxItem as ProtoCommunicationOutboxItem;
fn timestamp_string(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}
pub(super) fn item(item: CommunicationOutboxItem) -> ProtoCommunicationOutboxItem {
    ProtoCommunicationOutboxItem {
        outbox_id: item.outbox_id,
        account_id: item.account_id,
        draft_id: item.draft_id,
        to_recipients: item.to_recipients,
        cc_recipients: item.cc_recipients,
        bcc_recipients: item.bcc_recipients,
        subject: item.subject,
        body_text: item.body_text,
        body_html: item.body_html,
        status: item.status.as_str().to_owned(),
        scheduled_send_at: item.scheduled_send_at.map(timestamp_string),
        undo_deadline_at: item.undo_deadline_at.map(timestamp_string),
        send_attempts: item.send_attempts,
        claimed_at: item.claimed_at.map(timestamp_string),
        sent_at: item.sent_at.map(timestamp_string),
        provider_message_id: item.provider_message_id,
        last_error: item.last_error,
        metadata_json: item.metadata.to_string(),
        created_at: timestamp_string(item.created_at),
        updated_at: timestamp_string(item.updated_at),
        ..Default::default()
    }
}
