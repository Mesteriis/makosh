use super::*;
use crate::app::api_support::stores::domain_stores::communication_blob_store;
use crate::domains::communications::messages::models::ProjectedMessageSummary;
use crate::platform::formatting::text_preview;

#[derive(Serialize)]
pub(crate) struct CommunicationMessagesResponse {
    pub(crate) items: Vec<CommunicationMessageSummaryResponse>,
    pub(crate) next_cursor: Option<String>,
    pub(crate) has_more: bool,
}

#[derive(Serialize)]
pub(crate) struct CommunicationMessageSummaryResponse {
    pub(crate) message_id: String,
    pub(crate) raw_record_id: String,
    pub(crate) observation_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_record_id: String,
    pub(crate) subject: String,
    pub(crate) sender: String,
    pub(crate) recipients: Vec<String>,
    pub(crate) body_text_preview: String,
    pub(crate) occurred_at: Option<DateTime<Utc>>,
    pub(crate) projected_at: DateTime<Utc>,
    pub(crate) channel_kind: String,
    pub(crate) conversation_id: Option<String>,
    pub(crate) sender_display_name: Option<String>,
    pub(crate) delivery_state: String,
    pub(crate) workflow_state: String,
    pub(crate) message_metadata: Value,
    pub(crate) attachment_count: i64,
    pub(crate) local_state: String,
    pub(crate) local_state_changed_at: Option<DateTime<Utc>>,
    pub(crate) is_read: bool,
    pub(crate) read_changed_at: Option<DateTime<Utc>>,
    pub(crate) read_origin: String,
    pub(crate) read_sync_status: String,
}

impl From<ProjectedMessageSummary> for CommunicationMessageSummaryResponse {
    fn from(summary: ProjectedMessageSummary) -> Self {
        Self::from_with_read_sync_status(summary, "synced")
    }
}

impl CommunicationMessageSummaryResponse {
    pub(crate) fn from_with_read_sync_status(
        summary: ProjectedMessageSummary,
        read_sync_status: &str,
    ) -> Self {
        Self {
            message_id: summary.message.message_id,
            raw_record_id: summary.message.raw_record_id,
            observation_id: summary.message.observation_id,
            account_id: summary.message.account_id,
            provider_record_id: summary.message.provider_record_id,
            subject: summary.message.subject,
            sender: summary.message.sender,
            recipients: summary.message.recipients,
            body_text_preview: text_preview(&summary.message.body_text, 240),
            occurred_at: summary.message.occurred_at,
            projected_at: summary.message.projected_at,
            channel_kind: summary.message.channel_kind,
            conversation_id: summary.message.conversation_id,
            sender_display_name: summary.message.sender_display_name,
            delivery_state: summary.message.delivery_state,
            workflow_state: summary.message.workflow_state.as_str().to_owned(),
            message_metadata: summary.message.message_metadata,
            attachment_count: summary.attachment_count,
            local_state: summary.message.local_state.as_str().to_owned(),
            local_state_changed_at: summary.message.local_state_changed_at,
            is_read: summary.message.is_read,
            read_changed_at: summary.message.read_changed_at,
            read_origin: summary.message.read_origin,
            read_sync_status: read_sync_status.to_owned(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct CommunicationMessageDetailResponse {
    pub(crate) message: CommunicationMessageDetailItem,
    pub(crate) attachments: Vec<CommunicationAttachmentResponse>,
}

#[derive(Serialize)]
pub(crate) struct CommunicationMessageDetailItem {
    pub(crate) message_id: String,
    pub(crate) raw_record_id: String,
    pub(crate) observation_id: String,
    pub(crate) account_id: String,
    pub(crate) provider_record_id: String,
    pub(crate) subject: String,
    pub(crate) sender: String,
    pub(crate) recipients: Vec<String>,
    pub(crate) body_text: String,
    pub(crate) body_html: Option<String>,
    pub(crate) occurred_at: Option<DateTime<Utc>>,
    pub(crate) projected_at: DateTime<Utc>,
    pub(crate) channel_kind: String,
    pub(crate) conversation_id: Option<String>,
    pub(crate) sender_display_name: Option<String>,
    pub(crate) delivery_state: String,
    pub(crate) workflow_state: String,
    pub(crate) message_metadata: Value,
    pub(crate) local_state: String,
    pub(crate) local_state_changed_at: Option<DateTime<Utc>>,
    pub(crate) local_state_reason: Option<String>,
    pub(crate) is_read: bool,
    pub(crate) read_changed_at: Option<DateTime<Utc>>,
    pub(crate) read_origin: String,
    pub(crate) read_sync_status: String,
}

impl CommunicationMessageDetailItem {
    pub(crate) fn from_message(message: ProjectedMessage, body_html: Option<String>) -> Self {
        let message_metadata = message.message_metadata.clone();
        Self::from_message_with_metadata_and_read_sync_status(
            message,
            body_html,
            message_metadata,
            "synced",
        )
    }

    pub(crate) fn from_message_with_metadata_and_read_sync_status(
        message: ProjectedMessage,
        body_html: Option<String>,
        message_metadata: Value,
        read_sync_status: &str,
    ) -> Self {
        Self {
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
            occurred_at: message.occurred_at,
            projected_at: message.projected_at,
            channel_kind: message.channel_kind,
            conversation_id: message.conversation_id,
            sender_display_name: message.sender_display_name,
            delivery_state: message.delivery_state,
            workflow_state: message.workflow_state.as_str().to_owned(),
            message_metadata,
            local_state: message.local_state.as_str().to_owned(),
            local_state_changed_at: message.local_state_changed_at,
            local_state_reason: message.local_state_reason,
            is_read: message.is_read,
            read_changed_at: message.read_changed_at,
            read_origin: message.read_origin,
            read_sync_status: read_sync_status.to_owned(),
        }
    }
}

impl From<ProjectedMessage> for CommunicationMessageDetailItem {
    fn from(message: ProjectedMessage) -> Self {
        Self::from_message(message, None)
    }
}

pub(crate) async fn rich_body_html_for_message(
    pool: sqlx::postgres::PgPool,
    message: &ProjectedMessage,
) -> Result<Option<String>, ApiError> {
    let Some(raw) = makosh_communications_postgres::store::CommunicationIngestionStore::new(pool)
        .raw_record(&message.raw_record_id)
        .await?
    else {
        return Ok(None);
    };
    if raw.record_kind != "email_message" {
        return Ok(None);
    }
    if raw
        .payload
        .get("raw_blob_storage_kind")
        .and_then(Value::as_str)
        != Some("local_fs")
    {
        return Ok(None);
    }
    if raw
        .payload
        .get("raw_blob_storage_path")
        .and_then(Value::as_str)
        .is_none()
    {
        return Ok(None);
    }

    let blob_store = communication_blob_store();
    match parse_raw_email_message_from_blob(&blob_store, &raw).await {
        Ok(parsed) => Ok(parsed.body_html.filter(|value| !value.trim().is_empty())),
        Err(error) => {
            tracing::warn!(
                error = %error,
                message_id = %message.message_id,
                raw_record_id = %message.raw_record_id,
                "mail detail rich html extraction failed; falling back to projected body_text"
            );
            Ok(None)
        }
    }
}

#[derive(Serialize)]
pub(crate) struct CommunicationAttachmentResponse {
    pub(crate) attachment_id: String,
    pub(crate) message_id: String,
    pub(crate) raw_record_id: String,
    pub(crate) blob_id: String,
    pub(crate) provider_attachment_id: String,
    pub(crate) filename: Option<String>,
    pub(crate) content_type: String,
    pub(crate) size_bytes: i64,
    pub(crate) sha256: String,
    pub(crate) disposition: &'static str,
    pub(crate) scan_status: &'static str,
    pub(crate) scan_engine: Option<String>,
    pub(crate) scan_checked_at: Option<DateTime<Utc>>,
    pub(crate) scan_summary: Option<String>,
    pub(crate) scan_metadata: Value,
    pub(crate) storage_kind: String,
    pub(crate) storage_path: String,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
}

impl From<StoredCommunicationAttachmentWithBlob> for CommunicationAttachmentResponse {
    fn from(record: StoredCommunicationAttachmentWithBlob) -> Self {
        let attachment = record.attachment;
        Self {
            attachment_id: attachment.attachment_id,
            message_id: attachment.message_id,
            raw_record_id: attachment.raw_record_id,
            blob_id: attachment.blob_id,
            provider_attachment_id: attachment.provider_attachment_id,
            filename: attachment.filename,
            content_type: attachment.content_type,
            size_bytes: attachment.size_bytes,
            sha256: attachment.sha256,
            disposition: attachment.disposition.as_str(),
            scan_status: attachment.scan_status.as_str(),
            scan_engine: attachment.scan_engine,
            scan_checked_at: attachment.scan_checked_at,
            scan_summary: attachment.scan_summary,
            scan_metadata: attachment.scan_metadata,
            storage_kind: record.storage_kind,
            storage_path: record.storage_path,
            created_at: attachment.created_at,
            updated_at: attachment.updated_at,
        }
    }
}
