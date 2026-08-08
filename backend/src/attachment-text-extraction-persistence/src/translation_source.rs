//! Owner-local inbox/outbox for event-only Attachment Translation source delivery.

use sha2::{Digest, Sha256};
use sqlx::Row;

use crate::{
    AttachmentTextExtractionPersistenceErrorV1, AttachmentTextExtractionPersistenceV1,
    PersistTranslationSourceResultOutcomeV1, PersistTranslationSourceResultV1,
    PersistedAttachmentTextArtifactV1, TranslationSourceSnapshotOutcomeV1,
    TranslationSourceSnapshotV1, UnpublishedTranslationSourceResultV1,
    model::{format_from_code, valid_id16, valid_owner, valid_sha256, valid_timestamp_millis},
};

const MAX_OUTBOX_ITEMS_V1: u16 = 64;
const MAX_ENVELOPE_BYTES_V1: usize = 8_192;
const READY_STATE_CODE_V1: i16 = 4;

impl AttachmentTextExtractionPersistenceV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn translation_source_request_already_processed(
        &self,
        logical_owner_id: &str,
        request_message_id: [u8; 16],
        request_envelope_sha256: [u8; 32],
        request_id: [u8; 16],
        translation_run_id: [u8; 16],
        source_extraction_run_id: [u8; 16],
        expected_source_revision: u64,
    ) -> Result<bool, AttachmentTextExtractionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_id16(&request_message_id)
            || !valid_sha256(&request_envelope_sha256)
            || request_message_id != request_id
            || !valid_id16(&translation_run_id)
            || !valid_id16(&source_extraction_run_id)
            || expected_source_revision == 0
        {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT request_envelope_sha256,request_id,translation_run_id,source_extraction_run_id,expected_source_revision FROM makosh_data.attachment_text_extraction_translation_source_inbox WHERE logical_owner_id=$1 AND request_message_id=$2",
        )
        .bind(logical_owner_id)
        .bind(request_message_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_unavailable)?;
        let Some(row) = row else {
            return Ok(false);
        };
        if row
            .try_get::<Vec<u8>, _>("request_envelope_sha256")
            .map_err(invalid_row)?
            != request_envelope_sha256
            || row
                .try_get::<Vec<u8>, _>("request_id")
                .map_err(invalid_row)?
                != request_id
            || row
                .try_get::<Vec<u8>, _>("translation_run_id")
                .map_err(invalid_row)?
                != translation_run_id
            || row
                .try_get::<Vec<u8>, _>("source_extraction_run_id")
                .map_err(invalid_row)?
                != source_extraction_run_id
            || u64::try_from(
                row.try_get::<i64, _>("expected_source_revision")
                    .map_err(invalid_row)?,
            )
            .map_err(invalid_row)?
                != expected_source_revision
        {
            return Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict);
        }
        Ok(true)
    }

    pub async fn translation_source_snapshot(
        &self,
        logical_owner_id: &str,
        source_extraction_run_id: [u8; 16],
        expected_source_revision: u64,
    ) -> Result<TranslationSourceSnapshotOutcomeV1, AttachmentTextExtractionPersistenceErrorV1>
    {
        if !valid_owner(logical_owner_id)
            || !valid_id16(&source_extraction_run_id)
            || expected_source_revision == 0
        {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT r.state,r.state_revision,a.run_id,a.derived_reference_id,a.derived_receipt_sha256,a.source_receipt_sha256,a.parser_identity_sha256,a.format_code,a.extracted_size_bytes,a.extraction_truncated FROM makosh_data.attachment_text_extraction_runs r LEFT JOIN makosh_data.attachment_text_extraction_artifacts a ON a.logical_owner_id=r.logical_owner_id AND a.run_id=r.run_id WHERE r.logical_owner_id=$1 AND r.run_id=$2",
        )
        .bind(logical_owner_id)
        .bind(source_extraction_run_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_unavailable)?;
        let Some(row) = row else {
            return Ok(TranslationSourceSnapshotOutcomeV1::NotReady);
        };
        let state: i16 = row.try_get("state").map_err(invalid_row)?;
        let revision = u64::try_from(
            row.try_get::<i64, _>("state_revision")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?;
        if revision != expected_source_revision {
            return Ok(TranslationSourceSnapshotOutcomeV1::StaleRevision);
        }
        if state != READY_STATE_CODE_V1 {
            return Ok(TranslationSourceSnapshotOutcomeV1::NotReady);
        }
        let artifact = artifact_from_joined_row(&row)?;
        if artifact.run_id != source_extraction_run_id {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow);
        }
        Ok(TranslationSourceSnapshotOutcomeV1::Ready(
            TranslationSourceSnapshotV1 {
                source_revision: revision,
                artifact,
            },
        ))
    }

    pub async fn persist_translation_source_result(
        &self,
        logical_owner_id: &str,
        record: &PersistTranslationSourceResultV1,
        prepared_snapshot: Option<TranslationSourceSnapshotV1>,
    ) -> Result<PersistTranslationSourceResultOutcomeV1, AttachmentTextExtractionPersistenceErrorV1>
    {
        validate_result(logical_owner_id, record, prepared_snapshot)?;
        let mut transaction = self.pool.begin().await.map_err(storage_unavailable)?;
        if let Some(snapshot) = prepared_snapshot {
            let row = sqlx::query(
                "SELECT r.state,r.state_revision,a.derived_reference_id,a.derived_receipt_sha256,a.extracted_size_bytes FROM makosh_data.attachment_text_extraction_runs r JOIN makosh_data.attachment_text_extraction_artifacts a ON a.logical_owner_id=r.logical_owner_id AND a.run_id=r.run_id WHERE r.logical_owner_id=$1 AND r.run_id=$2 FOR UPDATE OF r",
            )
            .bind(logical_owner_id)
            .bind(record.source_extraction_run_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(storage_unavailable)?
            .ok_or(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)?;
            let state: i16 = row.try_get("state").map_err(invalid_row)?;
            let revision = u64::try_from(
                row.try_get::<i64, _>("state_revision")
                    .map_err(invalid_row)?,
            )
            .map_err(invalid_row)?;
            if state != READY_STATE_CODE_V1
                || revision != record.expected_source_revision
                || revision != snapshot.source_revision
                || row
                    .try_get::<Vec<u8>, _>("derived_reference_id")
                    .map_err(invalid_row)?
                    != snapshot.artifact.derived_reference_id
                || row
                    .try_get::<Vec<u8>, _>("derived_receipt_sha256")
                    .map_err(invalid_row)?
                    != snapshot.artifact.derived_receipt_sha256
                || u64::try_from(
                    row.try_get::<i64, _>("extracted_size_bytes")
                        .map_err(invalid_row)?,
                )
                .map_err(invalid_row)?
                    != snapshot.artifact.extracted_size_bytes
            {
                return Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict);
            }
        }
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.attachment_text_extraction_translation_source_inbox (logical_owner_id,request_message_id,request_envelope_sha256,request_id,translation_run_id,source_extraction_run_id,expected_source_revision,result_message_id,result_envelope_sha256,processed_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) ON CONFLICT (logical_owner_id,request_message_id) DO NOTHING",
        )
        .bind(logical_owner_id)
        .bind(record.request_message_id.as_slice())
        .bind(record.request_envelope_sha256.as_slice())
        .bind(record.request_id.as_slice())
        .bind(record.translation_run_id.as_slice())
        .bind(record.source_extraction_run_id.as_slice())
        .bind(i64::try_from(record.expected_source_revision).map_err(invalid_input)?)
        .bind(record.result_message_id.as_slice())
        .bind(record.result_envelope_sha256.as_slice())
        .bind(record.processed_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        if inserted.rows_affected() == 0 {
            let existing = sqlx::query(
                "SELECT request_envelope_sha256,request_id,translation_run_id,source_extraction_run_id,expected_source_revision,result_message_id,result_envelope_sha256 FROM makosh_data.attachment_text_extraction_translation_source_inbox WHERE logical_owner_id=$1 AND request_message_id=$2",
            )
            .bind(logical_owner_id)
            .bind(record.request_message_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(storage_unavailable)?
            .ok_or(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)?;
            let outbox = sqlx::query(
                "SELECT exact_envelope_bytes FROM makosh_data.attachment_text_extraction_translation_source_outbox WHERE logical_owner_id=$1 AND request_message_id=$2",
            )
            .bind(logical_owner_id)
            .bind(record.request_message_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(storage_unavailable)?
            .ok_or(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)?;
            if existing
                .try_get::<Vec<u8>, _>("request_envelope_sha256")
                .map_err(invalid_row)?
                != record.request_envelope_sha256
                || existing
                    .try_get::<Vec<u8>, _>("request_id")
                    .map_err(invalid_row)?
                    != record.request_id
                || existing
                    .try_get::<Vec<u8>, _>("translation_run_id")
                    .map_err(invalid_row)?
                    != record.translation_run_id
                || existing
                    .try_get::<Vec<u8>, _>("source_extraction_run_id")
                    .map_err(invalid_row)?
                    != record.source_extraction_run_id
                || u64::try_from(
                    existing
                        .try_get::<i64, _>("expected_source_revision")
                        .map_err(invalid_row)?,
                )
                .map_err(invalid_row)?
                    != record.expected_source_revision
                || existing
                    .try_get::<Vec<u8>, _>("result_message_id")
                    .map_err(invalid_row)?
                    != record.result_message_id
                || existing
                    .try_get::<Vec<u8>, _>("result_envelope_sha256")
                    .map_err(invalid_row)?
                    != record.result_envelope_sha256
                || outbox
                    .try_get::<Vec<u8>, _>("exact_envelope_bytes")
                    .map_err(invalid_row)?
                    != record.exact_result_envelope_bytes
            {
                return Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict);
            }
            transaction.commit().await.map_err(storage_unavailable)?;
            return Ok(PersistTranslationSourceResultOutcomeV1::Replayed);
        }
        sqlx::query(
            "INSERT INTO makosh_data.attachment_text_extraction_translation_source_outbox (logical_owner_id,result_message_id,request_message_id,envelope_sha256,exact_envelope_bytes,published_at_unix_millis,created_at_unix_millis) VALUES ($1,$2,$3,$4,$5,NULL,$6)",
        )
        .bind(logical_owner_id)
        .bind(record.result_message_id.as_slice())
        .bind(record.request_message_id.as_slice())
        .bind(record.result_envelope_sha256.as_slice())
        .bind(&record.exact_result_envelope_bytes)
        .bind(record.processed_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_unavailable)?;
        transaction.commit().await.map_err(storage_unavailable)?;
        Ok(PersistTranslationSourceResultOutcomeV1::Recorded)
    }

    pub async fn unpublished_translation_source_outbox(
        &self,
        logical_owner_id: &str,
        limit: u16,
    ) -> Result<Vec<UnpublishedTranslationSourceResultV1>, AttachmentTextExtractionPersistenceErrorV1>
    {
        if !valid_owner(logical_owner_id) || !(1..=MAX_OUTBOX_ITEMS_V1).contains(&limit) {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(
            "SELECT result_message_id,envelope_sha256,exact_envelope_bytes FROM makosh_data.attachment_text_extraction_translation_source_outbox WHERE logical_owner_id=$1 AND published_at_unix_millis IS NULL ORDER BY created_at_unix_millis,result_message_id LIMIT $2",
        )
        .bind(logical_owner_id)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(storage_unavailable)?
        .into_iter()
        .map(|row| {
            let item = UnpublishedTranslationSourceResultV1 {
                message_id: id16_row(row.try_get("result_message_id").map_err(invalid_row)?)?,
                envelope_sha256: id32_row(row.try_get("envelope_sha256").map_err(invalid_row)?)?,
                exact_envelope_bytes: row.try_get("exact_envelope_bytes").map_err(invalid_row)?,
            };
            if item.exact_envelope_bytes.is_empty()
                || item.exact_envelope_bytes.len() > MAX_ENVELOPE_BYTES_V1
                || Sha256::digest(&item.exact_envelope_bytes).as_slice() != item.envelope_sha256
            {
                return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow);
            }
            Ok(item)
        })
        .collect()
    }

    pub async fn mark_translation_source_published(
        &self,
        logical_owner_id: &str,
        message_id: [u8; 16],
        envelope_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), AttachmentTextExtractionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_id16(&message_id)
            || !valid_sha256(&envelope_sha256)
            || !valid_timestamp_millis(published_at_unix_millis)
        {
            return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
        }
        let changed = sqlx::query(
            "UPDATE makosh_data.attachment_text_extraction_translation_source_outbox SET published_at_unix_millis=$1 WHERE logical_owner_id=$2 AND result_message_id=$3 AND envelope_sha256=$4 AND published_at_unix_millis IS NULL AND created_at_unix_millis <= $1",
        )
        .bind(published_at_unix_millis)
        .bind(logical_owner_id)
        .bind(message_id.as_slice())
        .bind(envelope_sha256.as_slice())
        .execute(&self.pool)
        .await
        .map_err(storage_unavailable)?
        .rows_affected();
        if changed == 1 {
            Ok(())
        } else {
            Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict)
        }
    }
}

fn validate_result(
    logical_owner_id: &str,
    record: &PersistTranslationSourceResultV1,
    prepared_snapshot: Option<TranslationSourceSnapshotV1>,
) -> Result<(), AttachmentTextExtractionPersistenceErrorV1> {
    let digest: [u8; 32] = Sha256::digest(&record.exact_result_envelope_bytes).into();
    if !valid_owner(logical_owner_id)
        || !valid_id16(&record.request_message_id)
        || record.request_message_id != record.request_id
        || !valid_sha256(&record.request_envelope_sha256)
        || !valid_id16(&record.translation_run_id)
        || !valid_id16(&record.source_extraction_run_id)
        || record.expected_source_revision == 0
        || !valid_id16(&record.result_message_id)
        || !valid_sha256(&record.result_envelope_sha256)
        || record.result_envelope_sha256 != digest
        || record.exact_result_envelope_bytes.is_empty()
        || record.exact_result_envelope_bytes.len() > MAX_ENVELOPE_BYTES_V1
        || !valid_timestamp_millis(record.processed_at_unix_millis)
        || prepared_snapshot.is_some_and(|snapshot| {
            snapshot.source_revision != record.expected_source_revision
                || snapshot.artifact.run_id != record.source_extraction_run_id
        })
    {
        return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn artifact_from_joined_row(
    row: &sqlx::postgres::PgRow,
) -> Result<PersistedAttachmentTextArtifactV1, AttachmentTextExtractionPersistenceErrorV1> {
    let artifact = PersistedAttachmentTextArtifactV1 {
        run_id: id16_row(row.try_get("run_id").map_err(invalid_row)?)?,
        derived_reference_id: id16_row(row.try_get("derived_reference_id").map_err(invalid_row)?)?,
        derived_receipt_sha256: id32_row(
            row.try_get("derived_receipt_sha256").map_err(invalid_row)?,
        )?,
        source_receipt_sha256: id32_row(
            row.try_get("source_receipt_sha256").map_err(invalid_row)?,
        )?,
        parser_identity_sha256: id32_row(
            row.try_get("parser_identity_sha256").map_err(invalid_row)?,
        )?,
        format: format_from_code(row.try_get("format_code").map_err(invalid_row)?)?,
        extracted_size_bytes: u64::try_from(
            row.try_get::<i64, _>("extracted_size_bytes")
                .map_err(invalid_row)?,
        )
        .map_err(invalid_row)?,
        extraction_truncated: row.try_get("extraction_truncated").map_err(invalid_row)?,
    };
    if !valid_id16(&artifact.run_id)
        || !valid_id16(&artifact.derived_reference_id)
        || !valid_sha256(&artifact.derived_receipt_sha256)
        || !valid_sha256(&artifact.source_receipt_sha256)
        || !valid_sha256(&artifact.parser_identity_sha256)
        || !(1..=1_048_576).contains(&artifact.extracted_size_bytes)
    {
        return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow);
    }
    Ok(artifact)
}

fn id16_row(value: Vec<u8>) -> Result<[u8; 16], AttachmentTextExtractionPersistenceErrorV1> {
    value.try_into().map_err(invalid_row)
}

fn id32_row(value: Vec<u8>) -> Result<[u8; 32], AttachmentTextExtractionPersistenceErrorV1> {
    value.try_into().map_err(invalid_row)
}

fn storage_unavailable<T>(_: T) -> AttachmentTextExtractionPersistenceErrorV1 {
    AttachmentTextExtractionPersistenceErrorV1::StorageUnavailable
}

fn invalid_input<T>(_: T) -> AttachmentTextExtractionPersistenceErrorV1 {
    AttachmentTextExtractionPersistenceErrorV1::InvalidInput
}

fn invalid_row<T>(_: T) -> AttachmentTextExtractionPersistenceErrorV1 {
    AttachmentTextExtractionPersistenceErrorV1::InvalidRow
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_validation_rejects_envelope_substitution() {
        let mut record = PersistTranslationSourceResultV1 {
            request_message_id: [1; 16],
            request_envelope_sha256: [2; 32],
            request_id: [1; 16],
            translation_run_id: [3; 16],
            source_extraction_run_id: [4; 16],
            expected_source_revision: 5,
            result_message_id: [6; 16],
            result_envelope_sha256: Sha256::digest(b"result").into(),
            exact_result_envelope_bytes: b"result".to_vec(),
            processed_at_unix_millis: 1,
        };
        assert_eq!(validate_result("owner-1", &record, None), Ok(()));
        record.exact_result_envelope_bytes.push(0);
        assert_eq!(
            validate_result("owner-1", &record, None),
            Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput)
        );
    }
}
