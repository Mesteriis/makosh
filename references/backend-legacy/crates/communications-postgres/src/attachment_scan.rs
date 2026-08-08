use makosh_communications_api::attachments::{
    CommunicationMessageAttachmentRow, CommunicationMessageAttachmentScanError,
    CommunicationMessageAttachmentScanFuture, CommunicationMessageAttachmentScanPort,
};
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct PostgresCommunicationMessageAttachmentScan {
    pool: PgPool,
}

impl PostgresCommunicationMessageAttachmentScan {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl CommunicationMessageAttachmentScanPort for PostgresCommunicationMessageAttachmentScan {
    fn scan_message_attachments<'a>(
        &'a self,
        account_id: &'a str,
        channel_kind: &'a str,
        limit: i64,
    ) -> CommunicationMessageAttachmentScanFuture<'a> {
        Box::pin(async move {
            if limit <= 0 {
                return Ok(Vec::new());
            }
            let rows = sqlx::query(r#"SELECT account_id, provider_record_id, message_metadata FROM communication_messages WHERE account_id = $1 AND channel_kind = $2 AND jsonb_typeof(message_metadata -> 'attachments') = 'array' ORDER BY COALESCE(occurred_at, projected_at) ASC, message_id ASC LIMIT $3"#)
                .bind(account_id.trim()).bind(channel_kind.trim()).bind(limit).fetch_all(&self.pool).await
                .map_err(|error| CommunicationMessageAttachmentScanError(error.to_string()))?;
            rows.into_iter()
                .map(|row| {
                    Ok(CommunicationMessageAttachmentRow {
                        account_id: row
                            .try_get("account_id")
                            .map_err(|e| CommunicationMessageAttachmentScanError(e.to_string()))?,
                        provider_record_id: row
                            .try_get("provider_record_id")
                            .map_err(|e| CommunicationMessageAttachmentScanError(e.to_string()))?,
                        message_metadata: row
                            .try_get("message_metadata")
                            .map_err(|e| CommunicationMessageAttachmentScanError(e.to_string()))?,
                    })
                })
                .collect()
        })
    }
}
