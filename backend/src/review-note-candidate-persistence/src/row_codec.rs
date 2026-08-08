use makosh_review_note_candidate_core::{
    ReviewNoteCandidatePromotionStatusV1, ReviewNoteCandidateStateV1,
    ReviewNoteCandidateTimestampV1, ReviewNoteCandidateV1, ReviewNoteSourceBasisV1,
    ReviewNoteTopicHintV1, validate_review_note_candidate_v1,
};
use sqlx::{PgConnection, Row, postgres::PgRow};

use crate::{
    PersistedReviewNoteCandidateSubmissionV1, ReviewNoteCandidateBlobCleanupV1,
    ReviewNoteCandidateBlobReceiptV1, ReviewNoteCandidatePersistenceErrorV1,
};

pub(crate) const SELECT_REVIEW_BY_ID: &str =
    "SELECT logical_owner_id, review_id, candidate_id, candidate_digest,
            source_evidence_id, source_evidence_revision, title, excerpt,
            topic_hints, source_basis, confidence_basis_points, state, promotion_status, review_revision,
            decided_by_owner_device_id, decided_at_unix_seconds, decided_at_nanos,
            promoted_note_id, updated_at_unix_seconds, updated_at_nanos
     FROM makosh_data.review_note_candidate_state
     WHERE logical_owner_id=$1 AND review_id=$2";

pub(crate) const SELECT_REVIEW_FOR_UPDATE: &str =
    "SELECT logical_owner_id, review_id, candidate_id, candidate_digest,
            source_evidence_id, source_evidence_revision, title, excerpt,
            topic_hints, source_basis, confidence_basis_points, state, promotion_status, review_revision,
            decided_by_owner_device_id, decided_at_unix_seconds, decided_at_nanos,
            promoted_note_id, updated_at_unix_seconds, updated_at_nanos
     FROM makosh_data.review_note_candidate_state
     WHERE logical_owner_id=$1 AND review_id=$2 FOR UPDATE";

pub(crate) const SELECT_PENDING_PROMOTIONS: &str =
    "SELECT logical_owner_id, review_id, candidate_id, candidate_digest,
            source_evidence_id, source_evidence_revision, title, excerpt,
            topic_hints, source_basis, confidence_basis_points, state, promotion_status, review_revision,
            decided_by_owner_device_id, decided_at_unix_seconds, decided_at_nanos,
            promoted_note_id, updated_at_unix_seconds, updated_at_nanos
     FROM makosh_data.review_note_candidate_state
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
     FROM makosh_data.review_note_candidate_submissions
     WHERE logical_owner_id=$1 AND submission_message_id=$2";

pub(crate) const SELECT_SUBMISSION_FOR_UPDATE: &str =
    "SELECT logical_owner_id, submission_message_id, submission_envelope_sha256,
            submission_id, candidate_id, candidate_digest, source_evidence_id,
            source_evidence_revision, candidate_blob_reference_id,
            candidate_blob_declared_bytes, candidate_blob_sha256,
            candidate_blob_custody_proof, materialized_blob_reference_id,
            cleanup_completed_at_unix_millis, completed, review_id, rejected,
            received_at_unix_millis
     FROM makosh_data.review_note_candidate_submissions
     WHERE logical_owner_id=$1 AND submission_message_id=$2 FOR UPDATE";

pub(crate) const SELECT_RECOVERABLE_SUBMISSIONS: &str =
    "SELECT logical_owner_id, submission_message_id, submission_envelope_sha256,
            submission_id, candidate_id, candidate_digest, source_evidence_id,
            source_evidence_revision, candidate_blob_reference_id,
            candidate_blob_declared_bytes, candidate_blob_sha256,
            candidate_blob_custody_proof, materialized_blob_reference_id,
            cleanup_completed_at_unix_millis, completed, review_id, rejected,
            received_at_unix_millis
     FROM makosh_data.review_note_candidate_submissions
     WHERE logical_owner_id=$1
       AND (NOT completed OR (materialized_blob_reference_id IS NOT NULL
            AND cleanup_completed_at_unix_millis IS NULL))
     ORDER BY received_at_unix_millis, submission_message_id LIMIT $2";

pub(crate) fn review_from_row(
    row: PgRow,
) -> Result<ReviewNoteCandidateV1, ReviewNoteCandidatePersistenceErrorV1> {
    let decided_seconds: Option<i64> = field(&row, "decided_at_unix_seconds")?;
    let decided_nanos: Option<i32> = field(&row, "decided_at_nanos")?;
    let decided_at = match (decided_seconds, decided_nanos) {
        (Some(unix_seconds), Some(nanos)) => Some(ReviewNoteCandidateTimestampV1 {
            unix_seconds,
            nanos,
        }),
        (None, None) => None,
        _ => return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidRow),
    };
    let review = ReviewNoteCandidateV1 {
        review_id: array(field::<Vec<u8>>(&row, "review_id")?)?,
        logical_owner_id: field(&row, "logical_owner_id")?,
        candidate_id: array(field::<Vec<u8>>(&row, "candidate_id")?)?,
        candidate_digest: array(field::<Vec<u8>>(&row, "candidate_digest")?)?,
        source_evidence_id: array(field::<Vec<u8>>(&row, "source_evidence_id")?)?,
        source_evidence_revision: unsigned(field(&row, "source_evidence_revision")?)?,
        title: field(&row, "title")?,
        excerpt: field(&row, "excerpt")?,
        topic_hints: topic_hints_from_codes(field(&row, "topic_hints")?)?,
        source_basis: source_basis_from_code(field(&row, "source_basis")?)?,
        confidence_basis_points: unsigned_i32(field(&row, "confidence_basis_points")?)?,
        state: state_from_code(field(&row, "state")?)?,
        promotion_status: promotion_from_code(field(&row, "promotion_status")?)?,
        review_revision: unsigned(field(&row, "review_revision")?)?,
        decided_by_owner_device_id: optional_array(field(&row, "decided_by_owner_device_id")?)?,
        decided_at,
        promoted_note_id: optional_array(field(&row, "promoted_note_id")?)?,
        updated_at: ReviewNoteCandidateTimestampV1 {
            unix_seconds: field(&row, "updated_at_unix_seconds")?,
            nanos: field(&row, "updated_at_nanos")?,
        },
    };
    validate_review_note_candidate_v1(&review)
        .map_err(|_| ReviewNoteCandidatePersistenceErrorV1::InvalidRow)?;
    Ok(review)
}

pub(crate) fn submission_from_row(
    row: PgRow,
) -> Result<PersistedReviewNoteCandidateSubmissionV1, ReviewNoteCandidatePersistenceErrorV1> {
    let materialized_reference_id: Option<[u8; 16]> =
        optional_array(field(&row, "materialized_blob_reference_id")?)?;
    let materialization = match materialized_reference_id {
        Some(reference_id) => Some(ReviewNoteCandidateBlobCleanupV1 {
            reference_id,
            declared_bytes: unsigned(field(&row, "candidate_blob_declared_bytes")?)?,
            sha256: array(field::<Vec<u8>>(&row, "candidate_blob_sha256")?)?,
            custody_proof: field(&row, "candidate_blob_custody_proof")?,
        }),
        None => None,
    };
    Ok(PersistedReviewNoteCandidateSubmissionV1 {
        logical_owner_id: field(&row, "logical_owner_id")?,
        submission_message_id: array(field::<Vec<u8>>(&row, "submission_message_id")?)?,
        submission_envelope_sha256: array(field::<Vec<u8>>(&row, "submission_envelope_sha256")?)?,
        submission_id: array(field::<Vec<u8>>(&row, "submission_id")?)?,
        candidate_id: array(field::<Vec<u8>>(&row, "candidate_id")?)?,
        candidate_digest: array(field::<Vec<u8>>(&row, "candidate_digest")?)?,
        source_evidence_id: array(field::<Vec<u8>>(&row, "source_evidence_id")?)?,
        source_evidence_revision: unsigned(field(&row, "source_evidence_revision")?)?,
        candidate_content: ReviewNoteCandidateBlobReceiptV1 {
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
    review: &ReviewNoteCandidateV1,
) -> Result<(), ReviewNoteCandidatePersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.review_note_candidate_state (
           logical_owner_id, review_id, candidate_id, candidate_digest,
           source_evidence_id, source_evidence_revision, title, excerpt,
           topic_hints, source_basis, confidence_basis_points, state, promotion_status, review_revision,
           decided_by_owner_device_id, decided_at_unix_seconds, decided_at_nanos,
           promoted_note_id, updated_at_unix_seconds, updated_at_nanos
         ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20)",
    )
    .bind(&review.logical_owner_id)
    .bind(review.review_id.as_slice())
    .bind(review.candidate_id.as_slice())
    .bind(review.candidate_digest.as_slice())
    .bind(review.source_evidence_id.as_slice())
    .bind(signed(review.source_evidence_revision)?)
    .bind(&review.title)
    .bind(&review.excerpt)
    .bind(review.topic_hints.iter().copied().map(topic_hint_code).collect::<Vec<_>>())
    .bind(source_basis_code(review.source_basis))
    .bind(signed_u32(review.confidence_basis_points)?)
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
    .bind(review.promoted_note_id.as_ref().map(<[u8; 16]>::as_slice))
    .bind(review.updated_at.unix_seconds)
    .bind(review.updated_at.nanos)
    .execute(connection)
    .await
    .map_err(storage_error)?;
    Ok(())
}

pub(crate) async fn update_review(
    connection: &mut PgConnection,
    current_revision: u64,
    review: &ReviewNoteCandidateV1,
) -> Result<(), ReviewNoteCandidatePersistenceErrorV1> {
    let affected = sqlx::query(
        "UPDATE makosh_data.review_note_candidate_state
         SET state=$1, promotion_status=$2, review_revision=$3,
             decided_by_owner_device_id=$4, decided_at_unix_seconds=$5,
             decided_at_nanos=$6, promoted_note_id=$7,
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
    .bind(review.promoted_note_id.as_ref().map(<[u8; 16]>::as_slice))
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
        return Err(ReviewNoteCandidatePersistenceErrorV1::RevisionConflict);
    }
    Ok(())
}

pub(crate) const fn state_code(value: ReviewNoteCandidateStateV1) -> i16 {
    match value {
        ReviewNoteCandidateStateV1::Pending => 1,
        ReviewNoteCandidateStateV1::Approved => 2,
        ReviewNoteCandidateStateV1::Rejected => 3,
    }
}

pub(crate) const fn promotion_code(value: ReviewNoteCandidatePromotionStatusV1) -> i16 {
    match value {
        ReviewNoteCandidatePromotionStatusV1::NotRequested => 1,
        ReviewNoteCandidatePromotionStatusV1::Pending => 2,
        ReviewNoteCandidatePromotionStatusV1::Succeeded => 3,
        ReviewNoteCandidatePromotionStatusV1::Failed => 4,
    }
}

pub(crate) const fn source_basis_code(value: ReviewNoteSourceBasisV1) -> i16 {
    match value {
        ReviewNoteSourceBasisV1::Subject => 1,
        ReviewNoteSourceBasisV1::Body => 2,
        ReviewNoteSourceBasisV1::Combined => 3,
    }
}

pub(crate) const fn topic_hint_code(value: ReviewNoteTopicHintV1) -> i16 {
    match value {
        ReviewNoteTopicHintV1::Financial => 1,
        ReviewNoteTopicHintV1::Legal => 2,
        ReviewNoteTopicHintV1::DecisionStatement => 3,
        ReviewNoteTopicHintV1::DeadlineStatement => 4,
    }
}

pub(crate) fn timestamp_millis(
    value: ReviewNoteCandidateTimestampV1,
) -> Result<i64, ReviewNoteCandidatePersistenceErrorV1> {
    value
        .unix_seconds
        .checked_mul(1_000)
        .and_then(|millis| millis.checked_add(i64::from(value.nanos / 1_000_000)))
        .filter(|millis| *millis > 0)
        .ok_or(ReviewNoteCandidatePersistenceErrorV1::InvalidInput)
}

pub(crate) fn signed(value: u64) -> Result<i64, ReviewNoteCandidatePersistenceErrorV1> {
    i64::try_from(value).map_err(|_| ReviewNoteCandidatePersistenceErrorV1::InvalidInput)
}

pub(crate) fn unsigned(value: i64) -> Result<u64, ReviewNoteCandidatePersistenceErrorV1> {
    u64::try_from(value).map_err(|_| ReviewNoteCandidatePersistenceErrorV1::InvalidRow)
}

fn signed_u32(value: u32) -> Result<i32, ReviewNoteCandidatePersistenceErrorV1> {
    i32::try_from(value).map_err(|_| ReviewNoteCandidatePersistenceErrorV1::InvalidInput)
}

fn unsigned_i32(value: i32) -> Result<u32, ReviewNoteCandidatePersistenceErrorV1> {
    u32::try_from(value).map_err(|_| ReviewNoteCandidatePersistenceErrorV1::InvalidRow)
}

pub(crate) fn storage_error(_: sqlx::Error) -> ReviewNoteCandidatePersistenceErrorV1 {
    ReviewNoteCandidatePersistenceErrorV1::StorageUnavailable
}

fn state_from_code(
    value: i16,
) -> Result<ReviewNoteCandidateStateV1, ReviewNoteCandidatePersistenceErrorV1> {
    match value {
        1 => Ok(ReviewNoteCandidateStateV1::Pending),
        2 => Ok(ReviewNoteCandidateStateV1::Approved),
        3 => Ok(ReviewNoteCandidateStateV1::Rejected),
        _ => Err(ReviewNoteCandidatePersistenceErrorV1::InvalidRow),
    }
}

fn promotion_from_code(
    value: i16,
) -> Result<ReviewNoteCandidatePromotionStatusV1, ReviewNoteCandidatePersistenceErrorV1> {
    match value {
        1 => Ok(ReviewNoteCandidatePromotionStatusV1::NotRequested),
        2 => Ok(ReviewNoteCandidatePromotionStatusV1::Pending),
        3 => Ok(ReviewNoteCandidatePromotionStatusV1::Succeeded),
        4 => Ok(ReviewNoteCandidatePromotionStatusV1::Failed),
        _ => Err(ReviewNoteCandidatePersistenceErrorV1::InvalidRow),
    }
}

fn source_basis_from_code(
    value: i16,
) -> Result<ReviewNoteSourceBasisV1, ReviewNoteCandidatePersistenceErrorV1> {
    match value {
        1 => Ok(ReviewNoteSourceBasisV1::Subject),
        2 => Ok(ReviewNoteSourceBasisV1::Body),
        3 => Ok(ReviewNoteSourceBasisV1::Combined),
        _ => Err(ReviewNoteCandidatePersistenceErrorV1::InvalidRow),
    }
}

fn topic_hints_from_codes(
    values: Vec<i16>,
) -> Result<Vec<ReviewNoteTopicHintV1>, ReviewNoteCandidatePersistenceErrorV1> {
    values
        .into_iter()
        .map(|value| match value {
            1 => Ok(ReviewNoteTopicHintV1::Financial),
            2 => Ok(ReviewNoteTopicHintV1::Legal),
            3 => Ok(ReviewNoteTopicHintV1::DecisionStatement),
            4 => Ok(ReviewNoteTopicHintV1::DeadlineStatement),
            _ => Err(ReviewNoteCandidatePersistenceErrorV1::InvalidRow),
        })
        .collect()
}

fn field<T>(row: &PgRow, name: &str) -> Result<T, ReviewNoteCandidatePersistenceErrorV1>
where
    for<'r> T: sqlx::Decode<'r, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
{
    row.try_get(name)
        .map_err(|_| ReviewNoteCandidatePersistenceErrorV1::InvalidRow)
}

fn array<const N: usize>(value: Vec<u8>) -> Result<[u8; N], ReviewNoteCandidatePersistenceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; N]| value.iter().any(|byte| *byte != 0))
        .ok_or(ReviewNoteCandidatePersistenceErrorV1::InvalidRow)
}

fn optional_array<const N: usize>(
    value: Option<Vec<u8>>,
) -> Result<Option<[u8; N]>, ReviewNoteCandidatePersistenceErrorV1> {
    value.map(array).transpose()
}
