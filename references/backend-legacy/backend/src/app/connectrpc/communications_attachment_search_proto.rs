use crate::domains::communications::attachment_search::{
    AttachmentSearchPage, AttachmentSearchResult,
};
use chrono::{DateTime, Utc};
use makosh_connectrpc_contracts::makosh::communications::v1::{
    AttachmentSearchItem as ProtoAttachmentSearchItem, AttachmentSearchResponse,
};
fn timestamp_string(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}
pub(super) fn page(page: AttachmentSearchPage) -> AttachmentSearchResponse {
    AttachmentSearchResponse {
        items: page.items.into_iter().map(item).collect(),
        next_cursor: page.next_cursor,
        has_more: page.has_more,
        ..Default::default()
    }
}
fn item(item: AttachmentSearchResult) -> ProtoAttachmentSearchItem {
    ProtoAttachmentSearchItem {
        attachment_id: item.attachment_id,
        message_id: item.message_id,
        raw_record_id: item.raw_record_id,
        account_id: item.account_id,
        message_subject: item.message_subject,
        sender: item.sender,
        occurred_at: item.occurred_at.map(timestamp_string),
        blob_id: item.blob_id,
        provider_attachment_id: item.provider_attachment_id,
        filename: item.filename,
        content_type: item.content_type,
        size_bytes: item.size_bytes,
        sha256: item.sha256,
        disposition: serde_json::to_string(&item.disposition)
            .unwrap_or_else(|_| "\"unknown\"".to_owned())
            .trim_matches('"')
            .to_owned(),
        scan_status: serde_json::to_string(&item.scan_status)
            .unwrap_or_else(|_| "\"failed\"".to_owned())
            .trim_matches('"')
            .to_owned(),
        scan_engine: item.scan_engine,
        scan_checked_at: item.scan_checked_at.map(timestamp_string),
        scan_summary: item.scan_summary,
        storage_kind: item.storage_kind,
        storage_path: item.storage_path,
        extracted_text_match: item.extracted_text_match,
        created_at: timestamp_string(item.created_at),
        updated_at: timestamp_string(item.updated_at),
        ..Default::default()
    }
}
