use makosh_communications_api::attachments::{
    CanonicalMediaReadError, CanonicalMediaReadPort, CanonicalMediaRecord,
    CanonicalMessageAttachmentReadError, CanonicalMessageAttachmentReadPort,
    CanonicalMessageAttachmentRecord,
};
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct CanonicalMessageAttachmentReadStore {
    pool: PgPool,
}

impl CanonicalMessageAttachmentReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CanonicalMessageAttachmentReadPort for CanonicalMessageAttachmentReadStore {
    async fn list_for_messages(
        &self,
        message_ids: &[String],
    ) -> Result<Vec<CanonicalMessageAttachmentRecord>, CanonicalMessageAttachmentReadError> {
        if message_ids.is_empty() {
            return Ok(Vec::new());
        }
        let rows = sqlx::query(
            r#"SELECT m.message_id, m.provider_record_id AS provider_message_id,
            COALESCE(m.conversation_id, '') AS provider_chat_id, a.attachment_id,
            a.provider_attachment_id, a.filename, a.content_type, a.size_bytes,
            b.storage_path, m.occurred_at
            FROM communication_attachments a
            JOIN communication_messages m ON m.message_id = a.message_id
            JOIN communication_mail_blobs b ON b.blob_id = a.blob_id
            WHERE m.message_id = ANY($1)
            ORDER BY COALESCE(m.occurred_at, m.projected_at) DESC, a.attachment_id ASC"#,
        )
        .bind(message_ids)
        .fetch_all(&self.pool)
        .await
        .map_err(error)?;
        rows.into_iter()
            .map(|row| {
                Ok(CanonicalMessageAttachmentRecord {
                    message_id: row.try_get("message_id").map_err(error)?,
                    provider_message_id: row.try_get("provider_message_id").map_err(error)?,
                    provider_chat_id: row.try_get("provider_chat_id").map_err(error)?,
                    attachment_id: row.try_get("attachment_id").map_err(error)?,
                    provider_attachment_id: row.try_get("provider_attachment_id").map_err(error)?,
                    filename: row.try_get("filename").map_err(error)?,
                    content_type: row.try_get("content_type").map_err(error)?,
                    size_bytes: row.try_get("size_bytes").map_err(error)?,
                    storage_path: row.try_get("storage_path").map_err(error)?,
                    occurred_at: row.try_get("occurred_at").map_err(error)?,
                })
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl CanonicalMediaReadPort for CanonicalMessageAttachmentReadStore {
    async fn list_whatsapp_media(
        &self,
        account_id: &str,
        provider_chat_id: Option<&str>,
        content_type: Option<&str>,
        limit: i64,
    ) -> Result<Vec<CanonicalMediaRecord>, CanonicalMediaReadError> {
        let rows = sqlx::query(r#"SELECT a.attachment_id, a.message_id, a.raw_record_id, m.account_id, m.channel_kind, COALESCE(c.provider_conversation_id, m.conversation_id) AS provider_conversation_id, m.provider_record_id, a.provider_attachment_id, a.filename, a.content_type, a.size_bytes, a.sha256, a.scan_status, b.storage_kind, b.storage_path, m.subject, m.sender, m.sender_display_name, m.occurred_at, a.created_at FROM communication_attachments a JOIN communication_messages m ON m.message_id = a.message_id JOIN communication_mail_blobs b ON b.blob_id = a.blob_id LEFT JOIN communication_conversations c ON c.conversation_id = m.conversation_id OR c.provider_conversation_id = m.conversation_id WHERE m.account_id = $1 AND m.local_state = 'active' AND m.channel_kind = 'whatsapp_web' AND ($2::text IS NULL OR COALESCE(c.provider_conversation_id, m.conversation_id) = $2) AND ($3::text IS NULL OR a.content_type ILIKE $3 || '%') ORDER BY COALESCE(m.occurred_at, m.projected_at) DESC, a.created_at DESC, a.attachment_id ASC LIMIT $4"#)
            .bind(account_id.trim()).bind(provider_chat_id.map(str::trim).filter(|v| !v.is_empty())).bind(content_type).bind(limit.clamp(1, 200)).fetch_all(&self.pool).await.map_err(media_error)?;
        rows.into_iter()
            .map(|row| {
                Ok(CanonicalMediaRecord {
                    attachment_id: row.try_get("attachment_id").map_err(media_error)?,
                    message_id: row.try_get("message_id").map_err(media_error)?,
                    raw_record_id: row.try_get("raw_record_id").map_err(media_error)?,
                    account_id: row.try_get("account_id").map_err(media_error)?,
                    channel_kind: row.try_get("channel_kind").map_err(media_error)?,
                    provider_chat_id: row
                        .try_get("provider_conversation_id")
                        .map_err(media_error)?,
                    provider_message_id: row.try_get("provider_record_id").map_err(media_error)?,
                    provider_attachment_id: row
                        .try_get("provider_attachment_id")
                        .map_err(media_error)?,
                    filename: row.try_get("filename").map_err(media_error)?,
                    content_type: row.try_get("content_type").map_err(media_error)?,
                    size_bytes: row.try_get("size_bytes").map_err(media_error)?,
                    sha256: row.try_get("sha256").map_err(media_error)?,
                    scan_status: row.try_get("scan_status").map_err(media_error)?,
                    storage_kind: row.try_get("storage_kind").map_err(media_error)?,
                    storage_path: row.try_get("storage_path").map_err(media_error)?,
                    message_subject: row.try_get("subject").map_err(media_error)?,
                    sender: row.try_get("sender").map_err(media_error)?,
                    sender_display_name: row.try_get("sender_display_name").map_err(media_error)?,
                    occurred_at: row.try_get("occurred_at").map_err(media_error)?,
                    created_at: row.try_get("created_at").map_err(media_error)?,
                })
            })
            .collect()
    }
}

fn error(error: sqlx::Error) -> CanonicalMessageAttachmentReadError {
    CanonicalMessageAttachmentReadError(error.to_string())
}
fn media_error(error: sqlx::Error) -> CanonicalMediaReadError {
    CanonicalMediaReadError(error.to_string())
}
