use sqlx::{Row, postgres::PgRow};

use crate::{
    ObligationsBlobCleanupV1, ObligationsBlobReceiptV1, ObligationsOutboxRecordV1,
    ObligationsPersistenceErrorV1, PersistedReviewedCandidateCommandV1,
};

pub(crate) fn decode_command(
    row: &PgRow,
) -> Result<PersistedReviewedCandidateCommandV1, ObligationsPersistenceErrorV1> {
    let materialized_reference: Option<Vec<u8>> = get(row, "materialized_blob_reference_id")?;
    let candidate_content = ObligationsBlobReceiptV1 {
        reference_id: fixed(get(row, "candidate_blob_reference_id")?)?,
        declared_bytes: positive_u64(get(row, "candidate_blob_declared_bytes")?)?,
        sha256: fixed(get(row, "candidate_blob_sha256")?)?,
        custody_transfer_source_proof: get(row, "candidate_blob_custody_proof")?,
    };
    let materialization = materialized_reference
        .map(|reference_id| {
            Ok(ObligationsBlobCleanupV1 {
                reference_id: fixed(reference_id)?,
                declared_bytes: candidate_content.declared_bytes,
                sha256: candidate_content.sha256,
                custody_proof: candidate_content.custody_transfer_source_proof.clone(),
            })
        })
        .transpose()?;
    Ok(PersistedReviewedCandidateCommandV1 {
        logical_owner_id: get(row, "logical_owner_id")?,
        command_message_id: fixed(get(row, "command_message_id")?)?,
        command_envelope_sha256: fixed(get(row, "command_envelope_sha256")?)?,
        command_id: fixed(get(row, "command_id")?)?,
        command_fingerprint: fixed(get(row, "command_fingerprint")?)?,
        approved_candidate_id: fixed(get(row, "approved_candidate_id")?)?,
        candidate_digest: fixed(get(row, "candidate_digest")?)?,
        source_evidence_id: fixed(get(row, "source_evidence_id")?)?,
        source_evidence_revision: positive_u64(get(row, "source_evidence_revision")?)?,
        review_id: fixed(get(row, "review_id")?)?,
        decision_revision: positive_u64(get(row, "decision_revision")?)?,
        decided_by_owner_device_id: fixed(get(row, "decided_by_owner_device_id")?)?,
        candidate_content,
        materialization,
        cleanup_completed_at_unix_millis: get(row, "cleanup_completed_at_unix_millis")?,
        completed: get(row, "completed")?,
        rejected: get(row, "rejected")?,
        obligation_id: optional_fixed(get(row, "obligation_id")?)?,
        received_at_unix_millis: get(row, "received_at_unix_millis")?,
    })
}

pub(crate) fn decode_outbox(
    row: &PgRow,
) -> Result<ObligationsOutboxRecordV1, ObligationsPersistenceErrorV1> {
    Ok(ObligationsOutboxRecordV1 {
        message_id: fixed(get(row, "message_id")?)?,
        envelope_sha256: fixed(get(row, "envelope_sha256")?)?,
        envelope_bytes: get(row, "envelope_bytes")?,
    })
}

fn get<T>(row: &PgRow, column: &str) -> Result<T, ObligationsPersistenceErrorV1>
where
    for<'r> T: sqlx::Decode<'r, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
{
    row.try_get(column)
        .map_err(|_| ObligationsPersistenceErrorV1::InvalidRow)
}

fn fixed<const N: usize>(value: Vec<u8>) -> Result<[u8; N], ObligationsPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| ObligationsPersistenceErrorV1::InvalidRow)
}

fn optional_fixed<const N: usize>(
    value: Option<Vec<u8>>,
) -> Result<Option<[u8; N]>, ObligationsPersistenceErrorV1> {
    value.map(fixed).transpose()
}

fn positive_u64(value: i64) -> Result<u64, ObligationsPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(ObligationsPersistenceErrorV1::InvalidRow)
}
