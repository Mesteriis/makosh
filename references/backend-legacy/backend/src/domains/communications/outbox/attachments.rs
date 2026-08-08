use makosh_communications_api::email::OutgoingEmailAttachment;
use sha2::{Digest, Sha256};
use sqlx::Row;
use sqlx::postgres::PgPool;

use crate::domains::communications::storage::blob_store::LocalCommunicationBlobStore;
use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;

use super::delivery::OutboxDeliveryError;

const MAX_OUTBOX_ATTACHMENT_BYTES: usize = 50 * 1024 * 1024;

pub(super) async fn load_sendable_attachments(
    pool: &PgPool,
    outbox_id: &str,
) -> Result<Vec<OutgoingEmailAttachment>, OutboxDeliveryError> {
    let rows = sqlx::query(
        r#"
        SELECT
            imported.filename,
            imported.content_type,
            imported.size_bytes,
            imported.sha256,
            imported.scan_status,
            blob.storage_path,
            link.disposition,
            link.content_id
        FROM communication_outbox_attachments link
        JOIN communication_attachment_imports imported
          ON imported.attachment_id = link.attachment_id
        JOIN communication_mail_blobs blob
          ON blob.blob_id = imported.blob_id
        WHERE link.outbox_id = $1
        ORDER BY link.sort_order
        "#,
    )
    .bind(outbox_id)
    .fetch_all(pool)
    .await
    .map_err(|error| loader_error("mail attachment metadata lookup failed", error))?;

    let blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
    let mut total_bytes = 0usize;
    let mut attachments = Vec::with_capacity(rows.len());
    for row in rows {
        let scan_status: String = row
            .try_get("scan_status")
            .map_err(|error| loader_error("mail attachment metadata is invalid", error))?;
        if scan_status != "clean" {
            return Err(OutboxDeliveryError::Permanent(
                "mail attachment is not cleared for sending".to_owned(),
            ));
        }
        let storage_path: String = row
            .try_get("storage_path")
            .map_err(|error| loader_error("mail attachment metadata is invalid", error))?;
        let bytes: Vec<u8> = blob_store
            .read_blob(&storage_path)
            .await
            .map_err(|error| loader_error("mail attachment blob could not be read", error))?;
        let expected_size: i64 = row
            .try_get("size_bytes")
            .map_err(|error| loader_error("mail attachment metadata is invalid", error))?;
        let expected_sha256: String = row
            .try_get("sha256")
            .map_err(|error| loader_error("mail attachment metadata is invalid", error))?;
        if i64::try_from(bytes.len()).ok() != Some(expected_size)
            || attachment_sha256(&bytes) != expected_sha256
        {
            return Err(OutboxDeliveryError::Permanent(
                "mail attachment blob integrity check failed".to_owned(),
            ));
        }
        total_bytes = total_bytes.saturating_add(bytes.len());
        if total_bytes > MAX_OUTBOX_ATTACHMENT_BYTES {
            return Err(OutboxDeliveryError::Permanent(
                "mail attachments exceed the 50 MiB send limit".to_owned(),
            ));
        }
        let filename = row
            .try_get::<Option<String>, _>("filename")
            .map_err(|error| loader_error("mail attachment metadata is invalid", error))?
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "attachment.bin".to_owned());
        let content_type: String = row
            .try_get("content_type")
            .map_err(|error| loader_error("mail attachment metadata is invalid", error))?;
        if !safe_mime_header_value(&filename) || !safe_mime_header_value(&content_type) {
            return Err(OutboxDeliveryError::Permanent(
                "mail attachment contains unsafe MIME metadata".to_owned(),
            ));
        }
        attachments.push(OutgoingEmailAttachment {
            filename,
            content_type,
            disposition: row
                .try_get("disposition")
                .map_err(|error| loader_error("mail attachment metadata is invalid", error))?,
            content_id: row
                .try_get("content_id")
                .map_err(|error| loader_error("mail attachment metadata is invalid", error))?,
            bytes,
        });
    }
    Ok(attachments)
}

fn safe_mime_header_value(value: &str) -> bool {
    !value.is_empty() && !value.contains(['\r', '\n', '\0'])
}

fn attachment_sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn loader_error(
    public_message: &'static str,
    error: impl std::fmt::Display,
) -> OutboxDeliveryError {
    tracing::warn!(error = %error, "outbox attachment loading failed");
    OutboxDeliveryError::Transport(public_message.to_owned())
}

#[cfg(test)]
mod tests {
    use makosh_backend_testkit::context::TestContext;
    use serde_json::json;

    use super::load_sendable_attachments;
    use crate::domains::communications::outbox::delivery::OutboxDeliveryError;
    use crate::domains::communications::storage::blob_store::LocalCommunicationBlobStore;
    use crate::domains::communications::storage::models::{
        NewCommunicationAttachmentImport, NewCommunicationBlob,
    };
    use crate::domains::communications::storage::store::CommunicationStorageStore;
    use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
    use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
    use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;

    #[tokio::test]
    async fn loader_blocks_unscanned_attachments_and_verifies_clean_blob_bytes() {
        let context = TestContext::new().await;
        let pool = context.pool().clone();
        let account_id = "outbox-attachment-loader";
        CommunicationProviderAccountStore::new(pool.clone())
            .upsert(&NewProviderAccount::new(
                account_id,
                CommunicationProviderKind::Gmail,
                "Attachment loader",
                "loader@example.test",
            ))
            .await
            .expect("provider account");
        let blob_store = LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT);
        let local_blob = blob_store
            .put_blob(b"clean attachment bytes")
            .await
            .expect("local blob");
        let storage = CommunicationStorageStore::new(pool.clone());
        let blob = storage
            .upsert_blob(
                &NewCommunicationBlob::from_local_blob(&local_blob).content_type("text/plain"),
            )
            .await
            .expect("blob metadata");
        storage
            .upsert_imported_attachment(
                &NewCommunicationAttachmentImport::new(
                    "loader-attachment",
                    &blob.blob_id,
                    "text/plain",
                    local_blob.size_bytes,
                    &local_blob.sha256,
                    "attachment-loader-test",
                )
                .account_id(account_id)
                .channel_kind("mail")
                .filename("loader.txt"),
            )
            .await
            .expect("attachment import");
        sqlx::query(
            r#"
            INSERT INTO communication_outbox (
                outbox_id, account_id, to_participants, status, metadata
            ) VALUES ($1, $2, $3, 'queued', '{}'::jsonb)
            "#,
        )
        .bind("loader-outbox")
        .bind(account_id)
        .bind(json!(["recipient@example.test"]))
        .execute(&pool)
        .await
        .expect("outbox item");
        sqlx::query(
            r#"
            INSERT INTO communication_outbox_attachments (
                outbox_id, attachment_id, disposition, sort_order
            ) VALUES ('loader-outbox', 'loader-attachment', 'attachment', 0)
            "#,
        )
        .execute(&pool)
        .await
        .expect("outbox attachment link");

        let blocked = load_sendable_attachments(&pool, "loader-outbox")
            .await
            .expect_err("not-scanned attachment must be blocked");
        assert!(matches!(blocked, OutboxDeliveryError::Permanent(_)));

        sqlx::query(
            r#"
            UPDATE communication_attachment_imports
            SET scan_status = 'clean',
                scan_engine = 'test-scanner',
                scan_checked_at = now()
            WHERE attachment_id = 'loader-attachment'
            "#,
        )
        .execute(&pool)
        .await
        .expect("clean scan verdict");
        let attachments = load_sendable_attachments(&pool, "loader-outbox")
            .await
            .expect("clean attachment bytes");
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].filename, "loader.txt");
        assert_eq!(attachments[0].bytes, b"clean attachment bytes");
    }
}
