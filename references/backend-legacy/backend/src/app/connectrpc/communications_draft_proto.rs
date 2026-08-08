use crate::domains::communications::drafts::CommunicationDraft;
use chrono::{DateTime, Utc};
use makosh_connectrpc_contracts::makosh::communications::v1::{
    CommunicationDraft as ProtoCommunicationDraft,
    CommunicationDraftAttachmentRef as ProtoCommunicationDraftAttachmentRef,
};
fn timestamp_string(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}
pub(super) fn draft(item: CommunicationDraft) -> ProtoCommunicationDraft {
    ProtoCommunicationDraft {
        draft_id: item.draft_id,
        account_id: item.account_id,
        persona_id: item.persona_id,
        to_recipients: item.to_recipients,
        cc_recipients: item.cc_recipients,
        bcc_recipients: item.bcc_recipients,
        subject: item.subject,
        body_text: item.body_text,
        body_html: item.body_html,
        in_reply_to: item.in_reply_to,
        references: item.references,
        attachment_ids: item.attachment_ids,
        attachments: item
            .attachments
            .into_iter()
            .map(|attachment| ProtoCommunicationDraftAttachmentRef {
                attachment_id: attachment.attachment_id,
                filename: attachment.filename,
                content_type: attachment.content_type,
                size_bytes: attachment.size_bytes,
                scan_status: attachment.scan_status,
                scan_engine: attachment.scan_engine,
                scan_checked_at: attachment.scan_checked_at.map(timestamp_string),
                scan_summary: attachment.scan_summary,
                ..Default::default()
            })
            .collect(),
        status: item.status.as_str().to_owned(),
        scheduled_send_at: item.scheduled_send_at.map(timestamp_string),
        send_attempts: item.send_attempts,
        last_error: item.last_error,
        metadata_json: item.metadata.to_string(),
        created_at: timestamp_string(item.created_at),
        updated_at: timestamp_string(item.updated_at),
        ..Default::default()
    }
}
