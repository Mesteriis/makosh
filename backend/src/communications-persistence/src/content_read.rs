//! Owner-local authorization read for canonical Communications body content.

use makosh_communications_api::CommunicationMessageIdV1;
use sqlx::Row;

use crate::{CommunicationsDurablePersistence, CommunicationsPersistenceError};

const REQUIRED_BACKUP_CLASS_V1: u32 = 1;
const MAX_MESSAGE_BODY_BYTES_V1: u64 = 256 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsBodyContentReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub plaintext_sha256: [u8; 32],
    pub backup_class: u32,
    pub media_type: String,
}

impl CommunicationsDurablePersistence {
    pub async fn current_message_body_content_receipt(
        &self,
        message_id: CommunicationMessageIdV1,
    ) -> Result<Option<CommunicationsBodyContentReceiptV1>, CommunicationsPersistenceError> {
        let row = sqlx::query(
            "SELECT evidence.body_blob_reference_id, evidence.body_blob_declared_bytes, \
             evidence.body_blob_sha256, evidence.body_media_type \
             FROM makosh_data.communications_messages AS message \
             JOIN makosh_data.communications_evidence_summaries AS evidence \
               ON evidence.observation_id = message.last_evidence_id \
             WHERE message.message_id = $1 \
               AND message.lifecycle_state = 1 \
               AND message.canonical_body_state = 4 \
               AND evidence.body_state = 4 \
               AND evidence.body_blob_ref IS NOT NULL",
        )
        .bind(message_id.bytes().as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| CommunicationsPersistenceError::StorageUnavailable)?;

        row.map(|row| {
            content_receipt_from_parts(
                row.try_get::<Vec<u8>, _>("body_blob_reference_id")
                    .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
                row.try_get::<i64, _>("body_blob_declared_bytes")
                    .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
                row.try_get::<Vec<u8>, _>("body_blob_sha256")
                    .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
                row.try_get::<String, _>("body_media_type")
                    .map_err(|_| CommunicationsPersistenceError::InvalidRow)?,
            )
        })
        .transpose()
    }
}

fn content_receipt_from_parts(
    reference_id: Vec<u8>,
    declared_bytes: i64,
    plaintext_sha256: Vec<u8>,
    media_type: String,
) -> Result<CommunicationsBodyContentReceiptV1, CommunicationsPersistenceError> {
    let reference_id: [u8; 16] = reference_id
        .try_into()
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let plaintext_sha256: [u8; 32] = plaintext_sha256
        .try_into()
        .map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    let declared_bytes =
        u64::try_from(declared_bytes).map_err(|_| CommunicationsPersistenceError::InvalidRow)?;
    if reference_id.iter().all(|byte| *byte == 0)
        || plaintext_sha256.iter().all(|byte| *byte == 0)
        || !(1..=MAX_MESSAGE_BODY_BYTES_V1).contains(&declared_bytes)
        || !matches!(media_type.as_str(), "text/plain" | "text/html")
    {
        return Err(CommunicationsPersistenceError::InvalidRow);
    }
    Ok(CommunicationsBodyContentReceiptV1 {
        reference_id,
        declared_bytes,
        plaintext_sha256,
        backup_class: REQUIRED_BACKUP_CLASS_V1,
        media_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_receipt_requires_exact_non_zero_bounded_blob_metadata() {
        assert!(
            content_receipt_from_parts(vec![1; 16], 1, vec![2; 32], "text/html".to_owned()).is_ok()
        );
        for result in [
            content_receipt_from_parts(vec![0; 16], 1, vec![2; 32], "text/plain".to_owned()),
            content_receipt_from_parts(vec![1; 16], 0, vec![2; 32], "text/plain".to_owned()),
            content_receipt_from_parts(
                vec![1; 16],
                256 * 1024 + 1,
                vec![2; 32],
                "text/plain".to_owned(),
            ),
            content_receipt_from_parts(vec![1; 16], 1, vec![0; 32], "text/plain".to_owned()),
            content_receipt_from_parts(
                vec![1; 16],
                1,
                vec![2; 32],
                "application/octet-stream".to_owned(),
            ),
        ] {
            assert_eq!(result, Err(CommunicationsPersistenceError::InvalidRow));
        }
    }
}
