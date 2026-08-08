use chrono::{DateTime, Utc};
use makosh_connectrpc_contracts::makosh::communications::v1::CommunicationMessageAttachment;
use serde_json::Value;

use crate::domains::communications::storage::models::StoredCommunicationAttachmentWithBlob;
use crate::domains::communications::threads::ThreadMessageAttachment;

fn timestamp_string(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}
fn json_string(value: &Value) -> String {
    value.to_string()
}

pub(super) fn from_storage(
    attachment: StoredCommunicationAttachmentWithBlob,
) -> CommunicationMessageAttachment {
    let record = attachment.attachment;
    CommunicationMessageAttachment {
        attachment_id: record.attachment_id,
        message_id: record.message_id,
        raw_record_id: record.raw_record_id,
        blob_id: record.blob_id,
        provider_attachment_id: record.provider_attachment_id,
        filename: record.filename,
        content_type: record.content_type,
        size_bytes: record.size_bytes,
        sha256: record.sha256,
        disposition: record.disposition.as_str().to_owned(),
        scan_status: record.scan_status.as_str().to_owned(),
        scan_engine: record.scan_engine,
        scan_checked_at: record.scan_checked_at.map(timestamp_string),
        scan_summary: record.scan_summary,
        scan_metadata_json: json_string(&record.scan_metadata),
        storage_kind: attachment.storage_kind,
        storage_path: attachment.storage_path,
        created_at: timestamp_string(record.created_at),
        updated_at: timestamp_string(record.updated_at),
        ..Default::default()
    }
}

pub(super) fn from_thread(attachment: ThreadMessageAttachment) -> CommunicationMessageAttachment {
    CommunicationMessageAttachment {
        attachment_id: attachment.attachment_id,
        message_id: attachment.message_id,
        raw_record_id: attachment.raw_record_id,
        blob_id: attachment.blob_id,
        provider_attachment_id: attachment.provider_attachment_id,
        filename: attachment.filename,
        content_type: attachment.content_type,
        size_bytes: attachment.size_bytes,
        sha256: attachment.sha256,
        disposition: attachment.disposition,
        scan_status: attachment.scan_status,
        scan_engine: attachment.scan_engine,
        scan_checked_at: attachment.scan_checked_at.map(timestamp_string),
        scan_summary: attachment.scan_summary,
        scan_metadata_json: json_string(&attachment.scan_metadata),
        storage_kind: attachment.storage_kind,
        storage_path: attachment.storage_path,
        created_at: timestamp_string(attachment.created_at),
        updated_at: timestamp_string(attachment.updated_at),
        ..Default::default()
    }
}
