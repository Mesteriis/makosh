use makosh_review_note_candidate_core::{
    ReviewNoteCandidateDecisionV1, ReviewNoteCandidatePromotionStatusV1,
    ReviewNoteCandidateStateV1, ReviewNoteCandidateV1, create_review_note_candidate_v1,
    decide_review_note_candidate_v1, record_review_note_candidate_promotion_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    CheckReviewNoteCandidateDecisionReplayV1, CompleteReviewNoteCandidateSubmissionV1,
    DecideReviewNoteCandidateOperationV1, ListReviewNoteCandidatesV1,
    PersistReviewNoteCandidateMaterializationV1, PersistReviewNoteCandidatePromotionResultV1,
    PersistedReviewNoteCandidateSubmissionV1, RejectReviewNoteCandidateSubmissionV1,
    ReserveReviewNoteCandidateSubmissionOutcomeV1, ReserveReviewNoteCandidateSubmissionV1,
    ReviewNoteCandidateDecisionOutcomeV1, ReviewNoteCandidateInboxOutcomeV1,
    ReviewNoteCandidateOutboxRecordV1, ReviewNoteCandidatePageV1,
    ReviewNoteCandidatePersistenceErrorV1, ReviewNoteCandidateRealtimeTransitionV1,
    model::{
        REVIEW_NOTE_CANDIDATE_MAX_PAGE_SIZE_V1, REVIEW_NOTE_CANDIDATE_OUTBOX_LIMIT_V1,
        REVIEW_NOTE_CANDIDATE_REALTIME_LIMIT_V1, REVIEW_NOTE_CANDIDATE_RECOVERY_LIMIT_V1,
        decision_fingerprint, decision_replay_fingerprint, nonzero, valid_blob, valid_cleanup,
        valid_identity, valid_outbox,
    },
    row_codec::{
        SELECT_PENDING_PROMOTIONS, SELECT_RECOVERABLE_SUBMISSIONS, SELECT_REVIEW_BY_ID,
        SELECT_REVIEW_FOR_UPDATE, SELECT_SUBMISSION_BY_MESSAGE_ID, SELECT_SUBMISSION_FOR_UPDATE,
        insert_review, promotion_code, review_from_row, signed, state_code, storage_error,
        submission_from_row, timestamp_millis, unsigned, update_review,
    },
};

#[derive(Clone)]
pub struct ReviewNoteCandidatePersistenceV1 {
    pool: PgPool,
}

impl ReviewNoteCandidatePersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, ReviewNoteCandidatePersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(ReviewNoteCandidatePersistenceErrorV1::StorageUnavailable);
        }
        let options = PgConnectOptions::new()
            .host(pgbouncer_host)
            .port(
                u16::try_from(pgbouncer_port)
                    .map_err(|_| ReviewNoteCandidatePersistenceErrorV1::StorageUnavailable)?,
            )
            .username(binding.access().runtime_principal())
            .password(password)
            .database(binding.access().pool_alias());
        let pool = PgPoolOptions::new()
            .max_connections(u32::from(
                binding.access().effective_budgets().max_connections(),
            ))
            .connect_with(options)
            .await
            .map_err(storage_error)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), ReviewNoteCandidatePersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(storage_error)
    }

    pub async fn reserve_submission(
        &self,
        input: ReserveReviewNoteCandidateSubmissionV1,
    ) -> Result<ReserveReviewNoteCandidateSubmissionOutcomeV1, ReviewNoteCandidatePersistenceErrorV1>
    {
        validate_reservation(&input)?;
        let mut transaction = self
            .begin_owner_transaction(&input.logical_owner_id)
            .await?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.review_note_candidate_submissions (
               logical_owner_id, submission_message_id, submission_envelope_sha256,
               submission_id, candidate_id, candidate_digest, source_evidence_id,
               source_evidence_revision, candidate_blob_reference_id,
               candidate_blob_declared_bytes, candidate_blob_sha256,
               candidate_blob_custody_proof, received_at_unix_millis
             ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
             ON CONFLICT (logical_owner_id, submission_message_id) DO NOTHING",
        )
        .bind(&input.logical_owner_id)
        .bind(input.submission_message_id.as_slice())
        .bind(input.submission_envelope_sha256.as_slice())
        .bind(input.submission_id.as_slice())
        .bind(input.candidate_id.as_slice())
        .bind(input.candidate_digest.as_slice())
        .bind(input.source_evidence_id.as_slice())
        .bind(signed(input.source_evidence_revision)?)
        .bind(input.candidate_content.reference_id.as_slice())
        .bind(signed(input.candidate_content.declared_bytes)?)
        .bind(input.candidate_content.sha256.as_slice())
        .bind(&input.candidate_content.custody_transfer_source_proof)
        .bind(input.received_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        let persisted = load_submission_in_transaction(
            &mut transaction,
            &input.logical_owner_id,
            &input.submission_message_id,
        )
        .await?;
        if !same_submission(&persisted, &input) {
            return Err(ReviewNoteCandidatePersistenceErrorV1::SubmissionConflict);
        }
        let outcome = if inserted == 1 {
            Ok(ReserveReviewNoteCandidateSubmissionOutcomeV1::Reserved(
                persisted,
            ))
        } else {
            Ok(ReserveReviewNoteCandidateSubmissionOutcomeV1::Existing(
                persisted,
            ))
        };
        transaction.commit().await.map_err(storage_error)?;
        outcome
    }

    pub async fn load_recoverable_submissions(
        &self,
        logical_owner_id: &str,
    ) -> Result<Vec<PersistedReviewNoteCandidateSubmissionV1>, ReviewNoteCandidatePersistenceErrorV1>
    {
        if !valid_identity(logical_owner_id) {
            return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let submissions = sqlx::query(SELECT_RECOVERABLE_SUBMISSIONS)
            .bind(logical_owner_id)
            .bind(i64::from(REVIEW_NOTE_CANDIDATE_RECOVERY_LIMIT_V1))
            .fetch_all(&mut *transaction)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(submission_from_row)
            .collect::<Result<Vec<_>, _>>()?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(submissions)
    }

    pub async fn persist_materialization(
        &self,
        input: PersistReviewNoteCandidateMaterializationV1,
    ) -> Result<PersistedReviewNoteCandidateSubmissionV1, ReviewNoteCandidatePersistenceErrorV1>
    {
        if !valid_identity(&input.logical_owner_id)
            || !nonzero(&input.submission_message_id)
            || !valid_cleanup(&input.materialization)
            || input.materialized_at_unix_millis <= 0
        {
            return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .begin_owner_transaction(&input.logical_owner_id)
            .await?;
        let current = load_submission_for_update(
            &mut transaction,
            &input.logical_owner_id,
            &input.submission_message_id,
        )
        .await?;
        if let Some(existing) = &current.materialization {
            if existing != &input.materialization {
                return Err(ReviewNoteCandidatePersistenceErrorV1::SubmissionConflict);
            }
            transaction.commit().await.map_err(storage_error)?;
            return Ok(current);
        }
        let affected = sqlx::query(
            "UPDATE makosh_data.review_note_candidate_submissions
             SET materialized_blob_reference_id=$1
             WHERE logical_owner_id=$2 AND submission_message_id=$3
               AND materialized_blob_reference_id IS NULL",
        )
        .bind(input.materialization.reference_id.as_slice())
        .bind(&input.logical_owner_id)
        .bind(input.submission_message_id.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if affected != 1 {
            return Err(ReviewNoteCandidatePersistenceErrorV1::RevisionConflict);
        }
        let persisted = load_submission_in_transaction(
            &mut transaction,
            &input.logical_owner_id,
            &input.submission_message_id,
        )
        .await?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(persisted)
    }

    pub async fn complete_blob_cleanup(
        &self,
        logical_owner_id: &str,
        submission_message_id: &[u8; 16],
        materialization: &crate::ReviewNoteCandidateBlobCleanupV1,
        completed_at_unix_millis: i64,
    ) -> Result<(), ReviewNoteCandidatePersistenceErrorV1> {
        if !valid_identity(logical_owner_id)
            || !nonzero(submission_message_id)
            || !valid_cleanup(materialization)
            || completed_at_unix_millis <= 0
        {
            return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let affected = sqlx::query(
            "UPDATE makosh_data.review_note_candidate_submissions
             SET cleanup_completed_at_unix_millis=$1
             WHERE logical_owner_id=$2 AND submission_message_id=$3
               AND materialized_blob_reference_id=$4
               AND candidate_blob_declared_bytes=$5
               AND candidate_blob_sha256=$6
               AND candidate_blob_custody_proof=$7
               AND cleanup_completed_at_unix_millis IS NULL",
        )
        .bind(completed_at_unix_millis)
        .bind(logical_owner_id)
        .bind(submission_message_id.as_slice())
        .bind(materialization.reference_id.as_slice())
        .bind(signed(materialization.declared_bytes)?)
        .bind(materialization.sha256.as_slice())
        .bind(&materialization.custody_proof)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if affected == 1 {
            transaction.commit().await.map_err(storage_error)?;
            return Ok(());
        }
        let current = load_submission_in_transaction(
            &mut transaction,
            logical_owner_id,
            submission_message_id,
        )
        .await?;
        if current.materialization.as_ref() == Some(materialization)
            && current.cleanup_completed_at_unix_millis.is_some()
        {
            transaction.commit().await.map_err(storage_error)?;
            Ok(())
        } else {
            Err(ReviewNoteCandidatePersistenceErrorV1::RevisionConflict)
        }
    }

    pub async fn complete_submission(
        &self,
        input: CompleteReviewNoteCandidateSubmissionV1,
    ) -> Result<ReviewNoteCandidateV1, ReviewNoteCandidatePersistenceErrorV1> {
        if !valid_identity(&input.logical_owner_id)
            || !nonzero(&input.submission_message_id)
            || !valid_outbox(&input.submitted_result)
            || input.occurred_at_unix_millis <= 0
            || input.logical_owner_id != input.draft.logical_owner_id
        {
            return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidInput);
        }
        let review = create_review_note_candidate_v1(input.draft.clone())
            .map_err(|_| ReviewNoteCandidatePersistenceErrorV1::InvalidTransition)?;
        let mut transaction = self
            .begin_owner_transaction(&input.logical_owner_id)
            .await?;
        let submission = load_submission_for_update(
            &mut transaction,
            &input.logical_owner_id,
            &input.submission_message_id,
        )
        .await?;
        if submission.completed {
            let review_id = submission
                .review_id
                .ok_or(ReviewNoteCandidatePersistenceErrorV1::SubmissionConflict)?;
            let review =
                load_review_in_transaction(&mut transaction, &input.logical_owner_id, &review_id)
                    .await?;
            transaction.commit().await.map_err(storage_error)?;
            return Ok(review);
        }
        if submission.candidate_id != review.candidate_id
            || submission.candidate_digest != review.candidate_digest
            || submission.source_evidence_id != review.source_evidence_id
            || submission.source_evidence_revision != review.source_evidence_revision
        {
            return Err(ReviewNoteCandidatePersistenceErrorV1::SubmissionConflict);
        }
        insert_review(&mut transaction, &review).await?;
        let affected = sqlx::query(
            "UPDATE makosh_data.review_note_candidate_submissions
             SET completed=TRUE, review_id=$1, completed_at_unix_millis=$2
             WHERE logical_owner_id=$3 AND submission_message_id=$4 AND NOT completed",
        )
        .bind(review.review_id.as_slice())
        .bind(input.occurred_at_unix_millis)
        .bind(&input.logical_owner_id)
        .bind(input.submission_message_id.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if affected != 1 {
            return Err(ReviewNoteCandidatePersistenceErrorV1::RevisionConflict);
        }
        insert_outbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.submitted_result,
            input.occurred_at_unix_millis,
        )
        .await?;
        insert_realtime(&mut transaction, &review, input.occurred_at_unix_millis).await?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(review)
    }

    pub async fn reject_submission(
        &self,
        input: RejectReviewNoteCandidateSubmissionV1,
    ) -> Result<(), ReviewNoteCandidatePersistenceErrorV1> {
        if !valid_identity(&input.logical_owner_id)
            || !nonzero(&input.submission_message_id)
            || !valid_outbox(&input.rejected_result)
            || input.occurred_at_unix_millis <= 0
        {
            return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self
            .begin_owner_transaction(&input.logical_owner_id)
            .await?;
        let submission = load_submission_for_update(
            &mut transaction,
            &input.logical_owner_id,
            &input.submission_message_id,
        )
        .await?;
        if submission.completed {
            transaction.commit().await.map_err(storage_error)?;
            return if submission.rejected {
                Ok(())
            } else {
                Err(ReviewNoteCandidatePersistenceErrorV1::SubmissionConflict)
            };
        }
        sqlx::query(
            "UPDATE makosh_data.review_note_candidate_submissions
             SET completed=TRUE, rejected=TRUE, completed_at_unix_millis=$1
             WHERE logical_owner_id=$2 AND submission_message_id=$3 AND NOT completed",
        )
        .bind(input.occurred_at_unix_millis)
        .bind(&input.logical_owner_id)
        .bind(input.submission_message_id.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
        insert_outbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.rejected_result,
            input.occurred_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(())
    }

    pub async fn load_review(
        &self,
        logical_owner_id: &str,
        review_id: &[u8; 16],
    ) -> Result<ReviewNoteCandidateV1, ReviewNoteCandidatePersistenceErrorV1> {
        if !valid_identity(logical_owner_id) || !nonzero(review_id) {
            return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let review = sqlx::query(SELECT_REVIEW_BY_ID)
            .bind(logical_owner_id)
            .bind(review_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(storage_error)?
            .ok_or(ReviewNoteCandidatePersistenceErrorV1::NotFound)
            .and_then(review_from_row)?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(review)
    }

    pub async fn list_reviews(
        &self,
        logical_owner_id: &str,
        input: ListReviewNoteCandidatesV1,
    ) -> Result<ReviewNoteCandidatePageV1, ReviewNoteCandidatePersistenceErrorV1> {
        if !valid_identity(logical_owner_id)
            || input.limit == 0
            || input.limit > REVIEW_NOTE_CANDIDATE_MAX_PAGE_SIZE_V1
            || input
                .after_review_id
                .is_some_and(|review_id| !nonzero(&review_id))
        {
            return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let rows = sqlx::query(
            "SELECT logical_owner_id, review_id, candidate_id, candidate_digest,
                    source_evidence_id, source_evidence_revision, title, excerpt,
                    topic_hints, source_basis, confidence_basis_points, state, promotion_status,
                    review_revision, decided_by_owner_device_id, decided_at_unix_seconds,
                    decided_at_nanos, promoted_note_id, updated_at_unix_seconds, updated_at_nanos
             FROM makosh_data.review_note_candidate_state
             WHERE logical_owner_id=$1
               AND ($2::BYTEA IS NULL OR review_id > $2)
               AND ($3::SMALLINT IS NULL OR state=$3)
             ORDER BY review_id ASC LIMIT $4",
        )
        .bind(logical_owner_id)
        .bind(input.after_review_id.map(|value| value.to_vec()))
        .bind(input.state.map(state_code))
        .bind(i64::from(input.limit) + 1)
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage_error)?;
        let mut reviews = rows
            .into_iter()
            .map(review_from_row)
            .collect::<Result<Vec<_>, _>>()?;
        let has_more = reviews.len() > usize::from(input.limit);
        reviews.truncate(usize::from(input.limit));
        let next_after_review_id = has_more
            .then(|| reviews.last().map(|review| review.review_id))
            .flatten();
        transaction.commit().await.map_err(storage_error)?;
        Ok(ReviewNoteCandidatePageV1 {
            reviews,
            next_after_review_id,
        })
    }

    pub async fn decide(
        &self,
        input: DecideReviewNoteCandidateOperationV1,
    ) -> Result<ReviewNoteCandidateDecisionOutcomeV1, ReviewNoteCandidatePersistenceErrorV1> {
        validate_decision(&input)?;
        let fingerprint = decision_fingerprint(&input);
        let mut transaction = self
            .begin_owner_transaction(&input.logical_owner_id)
            .await?;
        if let Some(row) = sqlx::query(
            "SELECT request_sha256, decision_fingerprint, review_id, result_review_revision
             FROM makosh_data.review_note_candidate_operations
             WHERE logical_owner_id=$1 AND operation_id=$2",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_error)?
        {
            let request_hash: Vec<u8> = row.try_get("request_sha256").map_err(invalid_row)?;
            let stored_fingerprint: Vec<u8> =
                row.try_get("decision_fingerprint").map_err(invalid_row)?;
            let review_id: Vec<u8> = row.try_get("review_id").map_err(invalid_row)?;
            if request_hash.as_slice() != input.request_sha256
                || stored_fingerprint.as_slice() != fingerprint
                || review_id.as_slice() != input.review_id
            {
                return Err(ReviewNoteCandidatePersistenceErrorV1::OperationConflict);
            }
            let review = load_review_in_transaction(
                &mut transaction,
                &input.logical_owner_id,
                &input.review_id,
            )
            .await?;
            transaction.commit().await.map_err(storage_error)?;
            return Ok(ReviewNoteCandidateDecisionOutcomeV1::Replayed(review));
        }
        let current =
            load_review_for_update(&mut transaction, &input.logical_owner_id, &input.review_id)
                .await?;
        let next = decide_review_note_candidate_v1(
            &current,
            input.expected_review_revision,
            input.decision,
            input.owner_device_id,
            input.decided_at,
        )
        .map_err(transition_error)?;
        update_review(&mut transaction, current.review_revision, &next).await?;
        if let Some(event) = &input.approved_event {
            insert_outbox(
                &mut transaction,
                &input.logical_owner_id,
                event,
                timestamp_millis(input.decided_at)?,
            )
            .await?;
        }
        sqlx::query(
            "INSERT INTO makosh_data.review_note_candidate_operations (
               logical_owner_id, operation_id, request_sha256, decision_fingerprint,
               review_id, expected_review_revision, result_review_revision,
               completed_at_unix_millis
             ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .bind(input.request_sha256.as_slice())
        .bind(fingerprint.as_slice())
        .bind(input.review_id.as_slice())
        .bind(signed(input.expected_review_revision)?)
        .bind(signed(next.review_revision)?)
        .bind(timestamp_millis(input.decided_at)?)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
        insert_realtime(&mut transaction, &next, timestamp_millis(input.decided_at)?).await?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(ReviewNoteCandidateDecisionOutcomeV1::Applied(next))
    }

    pub async fn load_decision_replay(
        &self,
        input: &CheckReviewNoteCandidateDecisionReplayV1,
    ) -> Result<Option<ReviewNoteCandidateV1>, ReviewNoteCandidatePersistenceErrorV1> {
        validate_decision_replay(input)?;
        let fingerprint = decision_replay_fingerprint(input);
        let mut transaction = self
            .begin_owner_transaction(&input.logical_owner_id)
            .await?;
        let Some(row) = sqlx::query(
            "SELECT request_sha256, decision_fingerprint, review_id
             FROM makosh_data.review_note_candidate_operations
             WHERE logical_owner_id=$1 AND operation_id=$2",
        )
        .bind(&input.logical_owner_id)
        .bind(input.operation_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_error)?
        else {
            transaction.commit().await.map_err(storage_error)?;
            return Ok(None);
        };
        let request_hash: Vec<u8> = row.try_get("request_sha256").map_err(invalid_row)?;
        let stored_fingerprint: Vec<u8> =
            row.try_get("decision_fingerprint").map_err(invalid_row)?;
        let review_id: Vec<u8> = row.try_get("review_id").map_err(invalid_row)?;
        if request_hash.as_slice() != input.request_sha256
            || stored_fingerprint.as_slice() != fingerprint
            || review_id.as_slice() != input.review_id
        {
            return Err(ReviewNoteCandidatePersistenceErrorV1::OperationConflict);
        }
        let review =
            load_review_in_transaction(&mut transaction, &input.logical_owner_id, &input.review_id)
                .await?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(Some(review))
    }

    pub async fn persist_promotion_result(
        &self,
        input: PersistReviewNoteCandidatePromotionResultV1,
    ) -> Result<ReviewNoteCandidateInboxOutcomeV1, ReviewNoteCandidatePersistenceErrorV1> {
        validate_promotion_result(&input)?;
        let mut transaction = self
            .begin_owner_transaction(&input.logical_owner_id)
            .await?;
        let current =
            load_review_for_update(&mut transaction, &input.logical_owner_id, &input.review_id)
                .await?;
        if current.candidate_id != input.candidate_id {
            return Err(ReviewNoteCandidatePersistenceErrorV1::InboxConflict);
        }
        if let Some(row) = sqlx::query(
            "SELECT result_envelope_sha256, review_id
             FROM makosh_data.review_note_candidate_promotion_inbox
             WHERE logical_owner_id=$1 AND result_message_id=$2",
        )
        .bind(&input.logical_owner_id)
        .bind(input.result_message_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_error)?
        {
            let hash: Vec<u8> = row.try_get("result_envelope_sha256").map_err(invalid_row)?;
            let review_id: Vec<u8> = row.try_get("review_id").map_err(invalid_row)?;
            if hash.as_slice() != input.result_envelope_sha256
                || review_id.as_slice() != input.review_id
            {
                return Err(ReviewNoteCandidatePersistenceErrorV1::InboxConflict);
            }
            transaction.commit().await.map_err(storage_error)?;
            return Ok(ReviewNoteCandidateInboxOutcomeV1::Duplicate(current));
        }
        let next = record_review_note_candidate_promotion_v1(
            &current,
            input.expected_review_revision,
            input.result,
            input.occurred_at,
        )
        .map_err(transition_error)?;
        update_review(&mut transaction, current.review_revision, &next).await?;
        sqlx::query(
            "INSERT INTO makosh_data.review_note_candidate_promotion_inbox (
               logical_owner_id, result_message_id, result_envelope_sha256,
               review_id, result_review_revision, processed_at_unix_millis
             ) VALUES ($1,$2,$3,$4,$5,$6)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.result_message_id.as_slice())
        .bind(input.result_envelope_sha256.as_slice())
        .bind(input.review_id.as_slice())
        .bind(signed(next.review_revision)?)
        .bind(timestamp_millis(input.occurred_at)?)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
        insert_realtime(
            &mut transaction,
            &next,
            timestamp_millis(input.occurred_at)?,
        )
        .await?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(ReviewNoteCandidateInboxOutcomeV1::Applied(next))
    }

    pub async fn load_pending_promotions(
        &self,
        logical_owner_id: &str,
    ) -> Result<Vec<ReviewNoteCandidateV1>, ReviewNoteCandidatePersistenceErrorV1> {
        if !valid_identity(logical_owner_id) {
            return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let reviews = sqlx::query(SELECT_PENDING_PROMOTIONS)
            .bind(logical_owner_id)
            .bind(i64::from(REVIEW_NOTE_CANDIDATE_RECOVERY_LIMIT_V1))
            .fetch_all(&mut *transaction)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(review_from_row)
            .collect::<Result<Vec<_>, _>>()?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(reviews)
    }

    pub async fn realtime_after(
        &self,
        logical_owner_id: &str,
        after_sequence: u64,
        limit: u16,
    ) -> Result<Vec<ReviewNoteCandidateRealtimeTransitionV1>, ReviewNoteCandidatePersistenceErrorV1>
    {
        if !valid_identity(logical_owner_id)
            || limit == 0
            || limit > REVIEW_NOTE_CANDIDATE_REALTIME_LIMIT_V1
        {
            return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let transitions = sqlx::query(
            "SELECT realtime_sequence, review_id, candidate_id, state,
                    promotion_status, review_revision, occurred_at_unix_millis
             FROM makosh_data.review_note_candidate_realtime
             WHERE logical_owner_id=$1 AND realtime_sequence>$2
             ORDER BY realtime_sequence LIMIT $3",
        )
        .bind(logical_owner_id)
        .bind(signed(after_sequence)?)
        .bind(i64::from(limit))
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(realtime_from_row)
        .collect::<Result<Vec<_>, _>>()?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(transitions)
    }

    pub async fn unpublished_outbox(
        &self,
        logical_owner_id: &str,
        limit: u16,
    ) -> Result<Vec<ReviewNoteCandidateOutboxRecordV1>, ReviewNoteCandidatePersistenceErrorV1> {
        if !valid_identity(logical_owner_id)
            || limit == 0
            || limit > REVIEW_NOTE_CANDIDATE_OUTBOX_LIMIT_V1
        {
            return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let records = sqlx::query(
            "SELECT message_id, envelope_sha256, envelope_bytes
             FROM makosh_data.review_note_candidate_outbox
             WHERE logical_owner_id=$1 AND published_at_unix_millis IS NULL
             ORDER BY created_at_unix_millis, message_id LIMIT $2",
        )
        .bind(logical_owner_id)
        .bind(i64::from(limit))
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(outbox_from_row)
        .collect::<Result<Vec<_>, _>>()?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(records)
    }

    pub async fn mark_outbox_published(
        &self,
        logical_owner_id: &str,
        message_id: &[u8; 16],
        envelope_sha256: &[u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), ReviewNoteCandidatePersistenceErrorV1> {
        if !valid_identity(logical_owner_id)
            || !nonzero(message_id)
            || !nonzero(envelope_sha256)
            || published_at_unix_millis <= 0
        {
            return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let affected = sqlx::query(
            "UPDATE makosh_data.review_note_candidate_outbox
             SET published_at_unix_millis=$1
             WHERE logical_owner_id=$2 AND message_id=$3 AND envelope_sha256=$4
               AND published_at_unix_millis IS NULL",
        )
        .bind(published_at_unix_millis)
        .bind(logical_owner_id)
        .bind(message_id.as_slice())
        .bind(envelope_sha256.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if affected != 1 {
            return Err(ReviewNoteCandidatePersistenceErrorV1::NotFound);
        }
        transaction.commit().await.map_err(storage_error)?;
        Ok(())
    }

    async fn begin_owner_transaction(
        &self,
        logical_owner_id: &str,
    ) -> Result<Transaction<'_, Postgres>, ReviewNoteCandidatePersistenceErrorV1> {
        if !valid_identity(logical_owner_id) {
            return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage_error)?;
        Ok(transaction)
    }
}

async fn load_submission_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    submission_message_id: &[u8; 16],
) -> Result<PersistedReviewNoteCandidateSubmissionV1, ReviewNoteCandidatePersistenceErrorV1> {
    sqlx::query(SELECT_SUBMISSION_BY_MESSAGE_ID)
        .bind(logical_owner_id)
        .bind(submission_message_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_error)?
        .ok_or(ReviewNoteCandidatePersistenceErrorV1::NotFound)
        .and_then(submission_from_row)
}

async fn load_review_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    review_id: &[u8; 16],
) -> Result<ReviewNoteCandidateV1, ReviewNoteCandidatePersistenceErrorV1> {
    sqlx::query(SELECT_REVIEW_BY_ID)
        .bind(logical_owner_id)
        .bind(review_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_error)?
        .ok_or(ReviewNoteCandidatePersistenceErrorV1::NotFound)
        .and_then(review_from_row)
}

async fn load_submission_for_update(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
    submission_message_id: &[u8; 16],
) -> Result<PersistedReviewNoteCandidateSubmissionV1, ReviewNoteCandidatePersistenceErrorV1> {
    sqlx::query(SELECT_SUBMISSION_FOR_UPDATE)
        .bind(logical_owner_id)
        .bind(submission_message_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_error)?
        .ok_or(ReviewNoteCandidatePersistenceErrorV1::NotFound)
        .and_then(submission_from_row)
}

async fn load_review_for_update(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
    review_id: &[u8; 16],
) -> Result<ReviewNoteCandidateV1, ReviewNoteCandidatePersistenceErrorV1> {
    sqlx::query(SELECT_REVIEW_FOR_UPDATE)
        .bind(logical_owner_id)
        .bind(review_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_error)?
        .ok_or(ReviewNoteCandidatePersistenceErrorV1::NotFound)
        .and_then(review_from_row)
}

async fn insert_outbox(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    logical_owner_id: &str,
    record: &ReviewNoteCandidateOutboxRecordV1,
    created_at_unix_millis: i64,
) -> Result<(), ReviewNoteCandidatePersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.review_note_candidate_outbox (
           logical_owner_id, message_id, envelope_sha256, envelope_bytes,
           created_at_unix_millis
         ) VALUES ($1,$2,$3,$4,$5)",
    )
    .bind(logical_owner_id)
    .bind(record.message_id.as_slice())
    .bind(record.envelope_sha256.as_slice())
    .bind(&record.envelope_bytes)
    .bind(created_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?;
    Ok(())
}

async fn insert_realtime(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    review: &ReviewNoteCandidateV1,
    occurred_at_unix_millis: i64,
) -> Result<(), ReviewNoteCandidatePersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.review_note_candidate_realtime (
           logical_owner_id, review_id, candidate_id, state, promotion_status,
           review_revision, occurred_at_unix_millis
         ) VALUES ($1,$2,$3,$4,$5,$6,$7)",
    )
    .bind(&review.logical_owner_id)
    .bind(review.review_id.as_slice())
    .bind(review.candidate_id.as_slice())
    .bind(state_code(review.state))
    .bind(promotion_code(review.promotion_status))
    .bind(signed(review.review_revision)?)
    .bind(occurred_at_unix_millis)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?;
    Ok(())
}

fn validate_reservation(
    input: &ReserveReviewNoteCandidateSubmissionV1,
) -> Result<(), ReviewNoteCandidatePersistenceErrorV1> {
    if !valid_identity(&input.logical_owner_id)
        || !nonzero(&input.submission_message_id)
        || !nonzero(&input.submission_envelope_sha256)
        || !nonzero(&input.submission_id)
        || !nonzero(&input.candidate_id)
        || !nonzero(&input.candidate_digest)
        || !nonzero(&input.source_evidence_id)
        || input.source_evidence_revision == 0
        || !valid_blob(&input.candidate_content)
        || input.received_at_unix_millis <= 0
    {
        return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_decision(
    input: &DecideReviewNoteCandidateOperationV1,
) -> Result<(), ReviewNoteCandidatePersistenceErrorV1> {
    let event_valid = match input.decision {
        ReviewNoteCandidateDecisionV1::Approve => {
            input.approved_event.as_ref().is_some_and(valid_outbox)
        }
        ReviewNoteCandidateDecisionV1::Reject => input.approved_event.is_none(),
    };
    if !valid_identity(&input.logical_owner_id)
        || !nonzero(&input.operation_id)
        || !nonzero(&input.request_sha256)
        || !nonzero(&input.review_id)
        || input.expected_review_revision == 0
        || !nonzero(&input.owner_device_id)
        || timestamp_millis(input.decided_at).is_err()
        || !event_valid
    {
        return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_decision_replay(
    input: &CheckReviewNoteCandidateDecisionReplayV1,
) -> Result<(), ReviewNoteCandidatePersistenceErrorV1> {
    if !valid_identity(&input.logical_owner_id)
        || !nonzero(&input.operation_id)
        || !nonzero(&input.request_sha256)
        || !nonzero(&input.review_id)
        || input.expected_review_revision == 0
        || !nonzero(&input.owner_device_id)
    {
        return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_promotion_result(
    input: &PersistReviewNoteCandidatePromotionResultV1,
) -> Result<(), ReviewNoteCandidatePersistenceErrorV1> {
    if !valid_identity(&input.logical_owner_id)
        || !nonzero(&input.result_message_id)
        || !nonzero(&input.result_envelope_sha256)
        || !nonzero(&input.review_id)
        || !nonzero(&input.candidate_id)
        || input.expected_review_revision == 0
        || timestamp_millis(input.occurred_at).is_err()
    {
        return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn same_submission(
    persisted: &PersistedReviewNoteCandidateSubmissionV1,
    input: &ReserveReviewNoteCandidateSubmissionV1,
) -> bool {
    persisted.submission_envelope_sha256 == input.submission_envelope_sha256
        && persisted.submission_id == input.submission_id
        && persisted.candidate_id == input.candidate_id
        && persisted.candidate_digest == input.candidate_digest
        && persisted.source_evidence_id == input.source_evidence_id
        && persisted.source_evidence_revision == input.source_evidence_revision
        && persisted.candidate_content == input.candidate_content
}

fn realtime_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<ReviewNoteCandidateRealtimeTransitionV1, ReviewNoteCandidatePersistenceErrorV1> {
    let state = match row.try_get::<i16, _>("state").map_err(invalid_row)? {
        1 => ReviewNoteCandidateStateV1::Pending,
        2 => ReviewNoteCandidateStateV1::Approved,
        3 => ReviewNoteCandidateStateV1::Rejected,
        _ => return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidRow),
    };
    let promotion_status = match row
        .try_get::<i16, _>("promotion_status")
        .map_err(invalid_row)?
    {
        1 => ReviewNoteCandidatePromotionStatusV1::NotRequested,
        2 => ReviewNoteCandidatePromotionStatusV1::Pending,
        3 => ReviewNoteCandidatePromotionStatusV1::Succeeded,
        4 => ReviewNoteCandidatePromotionStatusV1::Failed,
        _ => return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidRow),
    };
    Ok(ReviewNoteCandidateRealtimeTransitionV1 {
        sequence: unsigned(row.try_get("realtime_sequence").map_err(invalid_row)?)?,
        review_id: array(row.try_get("review_id").map_err(invalid_row)?)?,
        candidate_id: array(row.try_get("candidate_id").map_err(invalid_row)?)?,
        state,
        promotion_status,
        review_revision: unsigned(row.try_get("review_revision").map_err(invalid_row)?)?,
        occurred_at_unix_millis: row
            .try_get("occurred_at_unix_millis")
            .map_err(invalid_row)?,
    })
}

fn outbox_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<ReviewNoteCandidateOutboxRecordV1, ReviewNoteCandidatePersistenceErrorV1> {
    let record = ReviewNoteCandidateOutboxRecordV1 {
        message_id: array(row.try_get("message_id").map_err(invalid_row)?)?,
        envelope_sha256: array(row.try_get("envelope_sha256").map_err(invalid_row)?)?,
        envelope_bytes: row.try_get("envelope_bytes").map_err(invalid_row)?,
    };
    if !valid_outbox(&record) {
        return Err(ReviewNoteCandidatePersistenceErrorV1::InvalidRow);
    }
    Ok(record)
}

fn array<const N: usize>(value: Vec<u8>) -> Result<[u8; N], ReviewNoteCandidatePersistenceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(nonzero)
        .ok_or(ReviewNoteCandidatePersistenceErrorV1::InvalidRow)
}

fn transition_error(
    error: makosh_review_note_candidate_core::ReviewNoteCandidateTransitionErrorV1,
) -> ReviewNoteCandidatePersistenceErrorV1 {
    match error {
        makosh_review_note_candidate_core::ReviewNoteCandidateTransitionErrorV1::RevisionConflict => {
            ReviewNoteCandidatePersistenceErrorV1::RevisionConflict
        }
        _ => ReviewNoteCandidatePersistenceErrorV1::InvalidTransition,
    }
}

fn invalid_row(_: sqlx::Error) -> ReviewNoteCandidatePersistenceErrorV1 {
    ReviewNoteCandidatePersistenceErrorV1::InvalidRow
}
