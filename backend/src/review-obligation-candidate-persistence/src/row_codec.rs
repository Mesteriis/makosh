use makosh_review_obligation_candidate_core::{
    ReviewObligationCandidatePromotionStatusV1, ReviewObligationCandidateStateV1,
    ReviewObligationCandidateTimestampV1, ReviewObligationCandidateV1,
    ReviewObligationEvidenceLinkV1, validate_review_obligation_candidate_v1,
};
use sqlx::{PgConnection, Row, postgres::PgRow};

use crate::{
    PersistedReviewObligationCandidateSubmissionV1, ReviewObligationCandidateBlobCleanupV1,
    ReviewObligationCandidateBlobReceiptV1, ReviewObligationCandidatePersistenceErrorV1,
};

pub(crate) const SELECT_REVIEW_BY_ID: &str =
    "SELECT logical_owner_id, review_id, candidate_id, candidate_digest,
            source_evidence_id, source_evidence_revision, statement, due_at_unix_seconds,
            due_at_nanos, obligated_party_id, beneficiary_party_id,
            condition, state, promotion_status, review_revision,
            decided_by_owner_device_id, decided_at_unix_seconds, decided_at_nanos,
            promoted_obligation_id, updated_at_unix_seconds, updated_at_nanos,
            ARRAY(SELECT evidence_link_id FROM makosh_data.review_obligation_candidate_evidence e
                WHERE e.logical_owner_id=$1 AND e.review_id=$2 ORDER BY evidence_link_id) evidence_link_ids,
            ARRAY(SELECT evidence_owner_id FROM makosh_data.review_obligation_candidate_evidence e
                WHERE e.logical_owner_id=$1 AND e.review_id=$2 ORDER BY evidence_link_id) evidence_owner_ids,
            ARRAY(SELECT evidence_record_id FROM makosh_data.review_obligation_candidate_evidence e
                WHERE e.logical_owner_id=$1 AND e.review_id=$2 ORDER BY evidence_link_id) evidence_record_ids,
            ARRAY(SELECT evidence_revision FROM makosh_data.review_obligation_candidate_evidence e
                WHERE e.logical_owner_id=$1 AND e.review_id=$2 ORDER BY evidence_link_id) evidence_revisions,
            ARRAY(SELECT evidence_digest FROM makosh_data.review_obligation_candidate_evidence e
                WHERE e.logical_owner_id=$1 AND e.review_id=$2 ORDER BY evidence_link_id) evidence_digests
     FROM makosh_data.review_obligation_candidate_state
     WHERE logical_owner_id=$1 AND review_id=$2";

pub(crate) const SELECT_REVIEW_FOR_UPDATE: &str =
    "SELECT logical_owner_id, review_id, candidate_id, candidate_digest,
            source_evidence_id, source_evidence_revision, statement, due_at_unix_seconds,
            due_at_nanos, obligated_party_id, beneficiary_party_id,
            condition, state, promotion_status, review_revision,
            decided_by_owner_device_id, decided_at_unix_seconds, decided_at_nanos,
            promoted_obligation_id, updated_at_unix_seconds, updated_at_nanos,
            ARRAY(SELECT evidence_link_id FROM makosh_data.review_obligation_candidate_evidence e
                WHERE e.logical_owner_id=$1 AND e.review_id=$2 ORDER BY evidence_link_id) evidence_link_ids,
            ARRAY(SELECT evidence_owner_id FROM makosh_data.review_obligation_candidate_evidence e
                WHERE e.logical_owner_id=$1 AND e.review_id=$2 ORDER BY evidence_link_id) evidence_owner_ids,
            ARRAY(SELECT evidence_record_id FROM makosh_data.review_obligation_candidate_evidence e
                WHERE e.logical_owner_id=$1 AND e.review_id=$2 ORDER BY evidence_link_id) evidence_record_ids,
            ARRAY(SELECT evidence_revision FROM makosh_data.review_obligation_candidate_evidence e
                WHERE e.logical_owner_id=$1 AND e.review_id=$2 ORDER BY evidence_link_id) evidence_revisions,
            ARRAY(SELECT evidence_digest FROM makosh_data.review_obligation_candidate_evidence e
                WHERE e.logical_owner_id=$1 AND e.review_id=$2 ORDER BY evidence_link_id) evidence_digests
     FROM makosh_data.review_obligation_candidate_state
     WHERE logical_owner_id=$1 AND review_id=$2 FOR UPDATE";

pub(crate) const SELECT_PENDING_PROMOTIONS: &str =
    "SELECT logical_owner_id, review_id, candidate_id, candidate_digest,
            source_evidence_id, source_evidence_revision, statement, due_at_unix_seconds,
            due_at_nanos, obligated_party_id, beneficiary_party_id,
            condition, state, promotion_status, review_revision,
            decided_by_owner_device_id, decided_at_unix_seconds, decided_at_nanos,
            promoted_obligation_id, updated_at_unix_seconds, updated_at_nanos,
            ARRAY(SELECT evidence_link_id FROM makosh_data.review_obligation_candidate_evidence e
                WHERE e.logical_owner_id=$1 AND e.review_id=review_obligation_candidate_state.review_id ORDER BY evidence_link_id) evidence_link_ids,
            ARRAY(SELECT evidence_owner_id FROM makosh_data.review_obligation_candidate_evidence e
                WHERE e.logical_owner_id=$1 AND e.review_id=review_obligation_candidate_state.review_id ORDER BY evidence_link_id) evidence_owner_ids,
            ARRAY(SELECT evidence_record_id FROM makosh_data.review_obligation_candidate_evidence e
                WHERE e.logical_owner_id=$1 AND e.review_id=review_obligation_candidate_state.review_id ORDER BY evidence_link_id) evidence_record_ids,
            ARRAY(SELECT evidence_revision FROM makosh_data.review_obligation_candidate_evidence e
                WHERE e.logical_owner_id=$1 AND e.review_id=review_obligation_candidate_state.review_id ORDER BY evidence_link_id) evidence_revisions,
            ARRAY(SELECT evidence_digest FROM makosh_data.review_obligation_candidate_evidence e
                WHERE e.logical_owner_id=$1 AND e.review_id=review_obligation_candidate_state.review_id ORDER BY evidence_link_id) evidence_digests
     FROM makosh_data.review_obligation_candidate_state
     WHERE logical_owner_id=$1 AND state=2 AND promotion_status=2
     ORDER BY review_revision, review_id LIMIT $2";

pub(crate) const SELECT_SUBMISSION_BY_MESSAGE_ID: &str =
    "SELECT logical_owner_id, submission_message_id, submission_envelope_sha256,
            submission_id, candidate_id, candidate_digest, source_evidence_id,
            source_evidence_revision, candidate_blob_reference_id,
            candidate_blob_declared_bytes, candidate_blob_sha256,
            candidate_blob_custody_proof, materialized_blob_reference_id,
            cleanup_completed_at_unix_millis, completed, review_id, rejected,
            received_at_unix_millis
     FROM makosh_data.review_obligation_candidate_submissions
     WHERE logical_owner_id=$1 AND submission_message_id=$2";

pub(crate) const SELECT_SUBMISSION_FOR_UPDATE: &str =
    "SELECT logical_owner_id, submission_message_id, submission_envelope_sha256,
            submission_id, candidate_id, candidate_digest, source_evidence_id,
            source_evidence_revision, candidate_blob_reference_id,
            candidate_blob_declared_bytes, candidate_blob_sha256,
            candidate_blob_custody_proof, materialized_blob_reference_id,
            cleanup_completed_at_unix_millis, completed, review_id, rejected,
            received_at_unix_millis
     FROM makosh_data.review_obligation_candidate_submissions
     WHERE logical_owner_id=$1 AND submission_message_id=$2 FOR UPDATE";

pub(crate) const SELECT_RECOVERABLE_SUBMISSIONS: &str =
    "SELECT logical_owner_id, submission_message_id, submission_envelope_sha256,
            submission_id, candidate_id, candidate_digest, source_evidence_id,
            source_evidence_revision, candidate_blob_reference_id,
            candidate_blob_declared_bytes, candidate_blob_sha256,
            candidate_blob_custody_proof, materialized_blob_reference_id,
            cleanup_completed_at_unix_millis, completed, review_id, rejected,
            received_at_unix_millis
     FROM makosh_data.review_obligation_candidate_submissions
     WHERE logical_owner_id=$1
       AND (NOT completed OR (materialized_blob_reference_id IS NOT NULL
            AND cleanup_completed_at_unix_millis IS NULL))
     ORDER BY received_at_unix_millis, submission_message_id LIMIT $2";

pub(crate) fn review_from_row(
    row: PgRow,
) -> Result<ReviewObligationCandidateV1, ReviewObligationCandidatePersistenceErrorV1> {
    let decided_seconds: Option<i64> = field(&row, "decided_at_unix_seconds")?;
    let decided_nanos: Option<i32> = field(&row, "decided_at_nanos")?;
    let decided_at = match (decided_seconds, decided_nanos) {
        (Some(unix_seconds), Some(nanos)) => Some(ReviewObligationCandidateTimestampV1 {
            unix_seconds,
            nanos,
        }),
        (None, None) => None,
        _ => return Err(ReviewObligationCandidatePersistenceErrorV1::InvalidRow),
    };
    let due_seconds: Option<i64> = field(&row, "due_at_unix_seconds")?;
    let due_nanos: Option<i32> = field(&row, "due_at_nanos")?;
    let due_at = match (due_seconds, due_nanos) {
        (Some(unix_seconds), Some(nanos)) => Some(ReviewObligationCandidateTimestampV1 {
            unix_seconds,
            nanos,
        }),
        (None, None) => None,
        _ => return Err(ReviewObligationCandidatePersistenceErrorV1::InvalidRow),
    };
    let evidence_link_ids: Vec<Vec<u8>> = field(&row, "evidence_link_ids")?;
    let evidence_owner_ids: Vec<String> = field(&row, "evidence_owner_ids")?;
    let evidence_record_ids: Vec<Vec<u8>> = field(&row, "evidence_record_ids")?;
    let evidence_revisions: Vec<i64> = field(&row, "evidence_revisions")?;
    let evidence_digests: Vec<Vec<u8>> = field(&row, "evidence_digests")?;
    let evidence_len = evidence_link_ids.len();
    if [
        evidence_owner_ids.len(),
        evidence_record_ids.len(),
        evidence_revisions.len(),
        evidence_digests.len(),
    ]
    .into_iter()
    .any(|len| len != evidence_len)
    {
        return Err(ReviewObligationCandidatePersistenceErrorV1::InvalidRow);
    }
    let evidence_links = evidence_link_ids
        .into_iter()
        .zip(evidence_owner_ids)
        .zip(evidence_record_ids)
        .zip(evidence_revisions)
        .zip(evidence_digests)
        .map(
            |(
                (((evidence_link_id, evidence_owner_id), evidence_record_id), evidence_revision),
                evidence_digest,
            )| {
                Ok(ReviewObligationEvidenceLinkV1 {
                    evidence_link_id: array(evidence_link_id)?,
                    evidence_owner_id,
                    evidence_record_id: array(evidence_record_id)?,
                    evidence_revision: unsigned(evidence_revision)?,
                    evidence_digest: array(evidence_digest)?,
                })
            },
        )
        .collect::<Result<Vec<_>, ReviewObligationCandidatePersistenceErrorV1>>()?;
    let review = ReviewObligationCandidateV1 {
        review_id: array(field::<Vec<u8>>(&row, "review_id")?)?,
        logical_owner_id: field(&row, "logical_owner_id")?,
        candidate_id: array(field::<Vec<u8>>(&row, "candidate_id")?)?,
        candidate_digest: array(field::<Vec<u8>>(&row, "candidate_digest")?)?,
        source_evidence_id: array(field::<Vec<u8>>(&row, "source_evidence_id")?)?,
        source_evidence_revision: unsigned(field(&row, "source_evidence_revision")?)?,
        statement: field(&row, "statement")?,
        due_at,
        condition: field(&row, "condition")?,
        obligated_party_id: array(field::<Vec<u8>>(&row, "obligated_party_id")?)?,
        beneficiary_party_id: optional_array(field(&row, "beneficiary_party_id")?)?,
        evidence_links,
        state: state_from_code(field(&row, "state")?)?,
        promotion_status: promotion_from_code(field(&row, "promotion_status")?)?,
        review_revision: unsigned(field(&row, "review_revision")?)?,
        decided_by_owner_device_id: optional_array(field(&row, "decided_by_owner_device_id")?)?,
        decided_at,
        promoted_obligation_id: optional_array(field(&row, "promoted_obligation_id")?)?,
        updated_at: ReviewObligationCandidateTimestampV1 {
            unix_seconds: field(&row, "updated_at_unix_seconds")?,
            nanos: field(&row, "updated_at_nanos")?,
        },
    };
    validate_review_obligation_candidate_v1(&review)
        .map_err(|_| ReviewObligationCandidatePersistenceErrorV1::InvalidRow)?;
    Ok(review)
}

pub(crate) fn submission_from_row(
    row: PgRow,
) -> Result<
    PersistedReviewObligationCandidateSubmissionV1,
    ReviewObligationCandidatePersistenceErrorV1,
> {
    let materialized_reference_id: Option<[u8; 16]> =
        optional_array(field(&row, "materialized_blob_reference_id")?)?;
    let materialization = match materialized_reference_id {
        Some(reference_id) => Some(ReviewObligationCandidateBlobCleanupV1 {
            reference_id,
            declared_bytes: unsigned(field(&row, "candidate_blob_declared_bytes")?)?,
            sha256: array(field::<Vec<u8>>(&row, "candidate_blob_sha256")?)?,
            custody_proof: field(&row, "candidate_blob_custody_proof")?,
        }),
        None => None,
    };
    Ok(PersistedReviewObligationCandidateSubmissionV1 {
        logical_owner_id: field(&row, "logical_owner_id")?,
        submission_message_id: array(field::<Vec<u8>>(&row, "submission_message_id")?)?,
        submission_envelope_sha256: array(field::<Vec<u8>>(&row, "submission_envelope_sha256")?)?,
        submission_id: array(field::<Vec<u8>>(&row, "submission_id")?)?,
        candidate_id: array(field::<Vec<u8>>(&row, "candidate_id")?)?,
        candidate_digest: array(field::<Vec<u8>>(&row, "candidate_digest")?)?,
        source_evidence_id: array(field::<Vec<u8>>(&row, "source_evidence_id")?)?,
        source_evidence_revision: unsigned(field(&row, "source_evidence_revision")?)?,
        candidate_content: ReviewObligationCandidateBlobReceiptV1 {
            reference_id: array(field::<Vec<u8>>(&row, "candidate_blob_reference_id")?)?,
            declared_bytes: unsigned(field(&row, "candidate_blob_declared_bytes")?)?,
            sha256: array(field::<Vec<u8>>(&row, "candidate_blob_sha256")?)?,
            custody_transfer_source_proof: field(&row, "candidate_blob_custody_proof")?,
        },
        materialization,
        cleanup_completed_at_unix_millis: field(&row, "cleanup_completed_at_unix_millis")?,
        completed: field(&row, "completed")?,
        review_id: optional_array(field(&row, "review_id")?)?,
        rejected: field(&row, "rejected")?,
        received_at_unix_millis: field(&row, "received_at_unix_millis")?,
    })
}

pub(crate) async fn insert_review(
    connection: &mut PgConnection,
    review: &ReviewObligationCandidateV1,
) -> Result<(), ReviewObligationCandidatePersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.review_obligation_candidate_state (
           logical_owner_id, review_id, candidate_id, candidate_digest,
           source_evidence_id, source_evidence_revision, statement, due_at_unix_seconds,
           due_at_nanos, condition, obligated_party_id, beneficiary_party_id,
           state, promotion_status, review_revision,
           decided_by_owner_device_id, decided_at_unix_seconds, decided_at_nanos,
           promoted_obligation_id, updated_at_unix_seconds, updated_at_nanos
         ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21)",
    )
    .bind(&review.logical_owner_id)
    .bind(review.review_id.as_slice())
    .bind(review.candidate_id.as_slice())
    .bind(review.candidate_digest.as_slice())
    .bind(review.source_evidence_id.as_slice())
    .bind(signed(review.source_evidence_revision)?)
    .bind(&review.statement)
    .bind(review.due_at.map(|value| value.unix_seconds))
    .bind(review.due_at.map(|value| value.nanos))
    .bind(&review.condition)
    .bind(review.obligated_party_id.as_slice())
    .bind(review.beneficiary_party_id.map(|value| value.to_vec()))
    .bind(state_code(review.state))
    .bind(promotion_code(review.promotion_status))
    .bind(signed(review.review_revision)?)
    .bind(
        review
            .decided_by_owner_device_id
            .as_ref()
            .map(<[u8; 16]>::as_slice),
    )
    .bind(review.decided_at.map(|value| value.unix_seconds))
    .bind(review.decided_at.map(|value| value.nanos))
    .bind(
        review
            .promoted_obligation_id
            .as_ref()
            .map(<[u8; 16]>::as_slice),
    )
    .bind(review.updated_at.unix_seconds)
    .bind(review.updated_at.nanos)
    .execute(&mut *connection)
    .await
    .map_err(storage_error)?;
    for evidence in &review.evidence_links {
        sqlx::query(
            "INSERT INTO makosh_data.review_obligation_candidate_evidence (
               logical_owner_id, review_id, evidence_link_id, evidence_owner_id,
               evidence_record_id, evidence_revision, evidence_digest
             ) VALUES ($1,$2,$3,$4,$5,$6,$7)",
        )
        .bind(&review.logical_owner_id)
        .bind(review.review_id.as_slice())
        .bind(evidence.evidence_link_id.as_slice())
        .bind(&evidence.evidence_owner_id)
        .bind(evidence.evidence_record_id.as_slice())
        .bind(signed(evidence.evidence_revision)?)
        .bind(evidence.evidence_digest.as_slice())
        .execute(&mut *connection)
        .await
        .map_err(storage_error)?;
    }
    Ok(())
}

pub(crate) async fn update_review(
    connection: &mut PgConnection,
    current_revision: u64,
    review: &ReviewObligationCandidateV1,
) -> Result<(), ReviewObligationCandidatePersistenceErrorV1> {
    let affected = sqlx::query(
        "UPDATE makosh_data.review_obligation_candidate_state
         SET state=$1, promotion_status=$2, review_revision=$3,
             decided_by_owner_device_id=$4, decided_at_unix_seconds=$5,
             decided_at_nanos=$6, promoted_obligation_id=$7,
             updated_at_unix_seconds=$8, updated_at_nanos=$9
         WHERE logical_owner_id=$10 AND review_id=$11 AND review_revision=$12",
    )
    .bind(state_code(review.state))
    .bind(promotion_code(review.promotion_status))
    .bind(signed(review.review_revision)?)
    .bind(
        review
            .decided_by_owner_device_id
            .as_ref()
            .map(<[u8; 16]>::as_slice),
    )
    .bind(review.decided_at.map(|value| value.unix_seconds))
    .bind(review.decided_at.map(|value| value.nanos))
    .bind(
        review
            .promoted_obligation_id
            .as_ref()
            .map(<[u8; 16]>::as_slice),
    )
    .bind(review.updated_at.unix_seconds)
    .bind(review.updated_at.nanos)
    .bind(&review.logical_owner_id)
    .bind(review.review_id.as_slice())
    .bind(signed(current_revision)?)
    .execute(connection)
    .await
    .map_err(storage_error)?
    .rows_affected();
    if affected != 1 {
        return Err(ReviewObligationCandidatePersistenceErrorV1::RevisionConflict);
    }
    Ok(())
}

pub(crate) const fn state_code(value: ReviewObligationCandidateStateV1) -> i16 {
    match value {
        ReviewObligationCandidateStateV1::Pending => 1,
        ReviewObligationCandidateStateV1::Approved => 2,
        ReviewObligationCandidateStateV1::Rejected => 3,
    }
}

pub(crate) const fn promotion_code(value: ReviewObligationCandidatePromotionStatusV1) -> i16 {
    match value {
        ReviewObligationCandidatePromotionStatusV1::NotRequested => 1,
        ReviewObligationCandidatePromotionStatusV1::Pending => 2,
        ReviewObligationCandidatePromotionStatusV1::Succeeded => 3,
        ReviewObligationCandidatePromotionStatusV1::Failed => 4,
    }
}

pub(crate) fn timestamp_millis(
    value: ReviewObligationCandidateTimestampV1,
) -> Result<i64, ReviewObligationCandidatePersistenceErrorV1> {
    value
        .unix_seconds
        .checked_mul(1_000)
        .and_then(|millis| millis.checked_add(i64::from(value.nanos / 1_000_000)))
        .filter(|millis| *millis > 0)
        .ok_or(ReviewObligationCandidatePersistenceErrorV1::InvalidInput)
}

pub(crate) fn signed(value: u64) -> Result<i64, ReviewObligationCandidatePersistenceErrorV1> {
    i64::try_from(value).map_err(|_| ReviewObligationCandidatePersistenceErrorV1::InvalidInput)
}

pub(crate) fn unsigned(value: i64) -> Result<u64, ReviewObligationCandidatePersistenceErrorV1> {
    u64::try_from(value).map_err(|_| ReviewObligationCandidatePersistenceErrorV1::InvalidRow)
}

pub(crate) fn storage_error(_: sqlx::Error) -> ReviewObligationCandidatePersistenceErrorV1 {
    ReviewObligationCandidatePersistenceErrorV1::StorageUnavailable
}

fn state_from_code(
    value: i16,
) -> Result<ReviewObligationCandidateStateV1, ReviewObligationCandidatePersistenceErrorV1> {
    match value {
        1 => Ok(ReviewObligationCandidateStateV1::Pending),
        2 => Ok(ReviewObligationCandidateStateV1::Approved),
        3 => Ok(ReviewObligationCandidateStateV1::Rejected),
        _ => Err(ReviewObligationCandidatePersistenceErrorV1::InvalidRow),
    }
}

fn promotion_from_code(
    value: i16,
) -> Result<ReviewObligationCandidatePromotionStatusV1, ReviewObligationCandidatePersistenceErrorV1>
{
    match value {
        1 => Ok(ReviewObligationCandidatePromotionStatusV1::NotRequested),
        2 => Ok(ReviewObligationCandidatePromotionStatusV1::Pending),
        3 => Ok(ReviewObligationCandidatePromotionStatusV1::Succeeded),
        4 => Ok(ReviewObligationCandidatePromotionStatusV1::Failed),
        _ => Err(ReviewObligationCandidatePersistenceErrorV1::InvalidRow),
    }
}

fn field<T>(row: &PgRow, name: &str) -> Result<T, ReviewObligationCandidatePersistenceErrorV1>
where
    for<'r> T: sqlx::Decode<'r, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
{
    row.try_get(name)
        .map_err(|_| ReviewObligationCandidatePersistenceErrorV1::InvalidRow)
}

fn array<const N: usize>(
    value: Vec<u8>,
) -> Result<[u8; N], ReviewObligationCandidatePersistenceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; N]| value.iter().any(|byte| *byte != 0))
        .ok_or(ReviewObligationCandidatePersistenceErrorV1::InvalidRow)
}

fn optional_array<const N: usize>(
    value: Option<Vec<u8>>,
) -> Result<Option<[u8; N]>, ReviewObligationCandidatePersistenceErrorV1> {
    value.map(array).transpose()
}
