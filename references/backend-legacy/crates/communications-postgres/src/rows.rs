use sqlx::Row;
use sqlx::postgres::PgRow;

use makosh_communications_api::evidence::{IngestionCheckpoint, StoredRawCommunicationRecord};

use crate::errors::CommunicationIngestionError;

pub(super) fn row_to_raw_record(
    row: PgRow,
) -> Result<StoredRawCommunicationRecord, CommunicationIngestionError> {
    Ok(StoredRawCommunicationRecord {
        raw_record_id: row.try_get("raw_record_id")?,
        observation_id: row.try_get("observation_id")?,
        account_id: row.try_get("account_id")?,
        record_kind: row.try_get("record_kind")?,
        provider_record_id: row.try_get("provider_record_id")?,
        source_fingerprint: row.try_get("source_fingerprint")?,
        import_batch_id: row.try_get("import_batch_id")?,
        occurred_at: row.try_get("occurred_at")?,
        captured_at: row.try_get("captured_at")?,
        payload: row.try_get("payload")?,
        provenance: row.try_get("provenance")?,
    })
}

pub(super) fn row_to_checkpoint(
    row: PgRow,
) -> Result<IngestionCheckpoint, CommunicationIngestionError> {
    Ok(IngestionCheckpoint {
        account_id: row.try_get("account_id")?,
        stream_id: row.try_get("stream_id")?,
        checkpoint: row.try_get("checkpoint")?,
        updated_at: row.try_get("updated_at")?,
    })
}
