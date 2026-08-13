use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

const OUTBOX_LIMIT: i64 = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewedPersonMatchCandidatePromotionEnvelopeV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

impl ReviewedPersonMatchCandidatePromotionEnvelopeV1 {
    pub fn validate(&self) -> Result<(), ReviewedPersonMatchCandidatePromotionPersistenceErrorV1> {
        if !nonzero(&self.message_id)
            || !nonzero(&self.envelope_sha256)
            || self.envelope_bytes.is_empty()
            || self.envelope_bytes.len() > 65_536
            || <[u8; 32]>::from(Sha256::digest(&self.envelope_bytes)) != self.envelope_sha256
        {
            Err(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::InvalidInput)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistReviewedPersonMatchApprovalV1 {
    pub logical_owner_id: String,
    pub approval: ReviewedPersonMatchCandidatePromotionEnvelopeV1,
    pub review_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub candidate_digest: [u8; 32],
    pub decision_id: [u8; 16],
    pub decision_revision: u64,
    pub approved_action_digest: [u8; 32],
    pub persons_command_id: [u8; 16],
    pub persons_command_fingerprint: [u8; 32],
    pub persons_command: ReviewedPersonMatchCandidatePromotionEnvelopeV1,
    pub occurred_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistReviewedPersonMatchApprovalFailureV1 {
    pub logical_owner_id: String,
    pub approval: ReviewedPersonMatchCandidatePromotionEnvelopeV1,
    pub review_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub candidate_digest: [u8; 32],
    pub decision_id: [u8; 16],
    pub decision_revision: u64,
    pub approved_action_digest: [u8; 32],
    pub review_result: ReviewedPersonMatchCandidatePromotionEnvelopeV1,
    pub completed_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistReviewedPersonMatchTerminalV1 {
    pub logical_owner_id: String,
    pub persons_result: ReviewedPersonMatchCandidatePromotionEnvelopeV1,
    pub persons_command_id: [u8; 16],
    pub review_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub succeeded: bool,
    pub failure_code: Option<u8>,
    pub review_result: ReviewedPersonMatchCandidatePromotionEnvelopeV1,
    pub completed_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewedPersonMatchCandidatePromotionOutboxV1 {
    pub record: ReviewedPersonMatchCandidatePromotionEnvelopeV1,
    pub semantic_kind: u8,
    pub created_at_unix_millis: i64,
    pub published_at_unix_millis: Option<i64>,
}

pub struct ReviewedPersonMatchCandidatePromotionOutboxPublishClaimV1 {
    transaction: Transaction<'static, Postgres>,
    logical_owner_id: String,
    record: ReviewedPersonMatchCandidatePromotionOutboxV1,
}

impl ReviewedPersonMatchCandidatePromotionOutboxPublishClaimV1 {
    #[must_use]
    pub fn record(&self) -> &ReviewedPersonMatchCandidatePromotionOutboxV1 {
        &self.record
    }
    pub async fn mark_published(
        mut self,
        expected_sha256: [u8; 32],
        published_at: i64,
    ) -> Result<(), ReviewedPersonMatchCandidatePromotionPersistenceErrorV1> {
        if expected_sha256 != self.record.record.envelope_sha256
            || published_at < self.record.created_at_unix_millis
        {
            return Err(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::HashMismatch);
        }
        let affected = sqlx::query("UPDATE makosh_data.reviewed_person_match_candidate_promotion_outbox SET published_at_unix_millis=$3 WHERE logical_owner_id=$1 AND message_id=$2 AND envelope_sha256=$4 AND published_at_unix_millis IS NULL")
            .bind(&self.logical_owner_id).bind(self.record.record.message_id.as_slice()).bind(published_at).bind(expected_sha256.as_slice()).execute(&mut *self.transaction).await.map_err(|_| storage())?.rows_affected();
        if affected != 1 {
            return Err(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::Conflict);
        }
        self.transaction.commit().await.map_err(|_| storage())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewedPersonMatchCandidatePromotionCorrelationV1 {
    pub review_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub decision_id: [u8; 16],
    pub decision_revision: u64,
    pub persons_command_id: [u8; 16],
    pub persons_command_fingerprint: [u8; 32],
    pub completed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewedPersonMatchCandidatePromotionReplayV1 {
    Applied,
    Replayed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewedPersonMatchCandidatePromotionPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    Conflict,
    NotFound,
    HashMismatch,
    StorageUnavailable,
}

#[derive(Clone)]
pub struct ReviewedPersonMatchCandidatePromotionPersistenceV1 {
    pool: PgPool,
}

impl ReviewedPersonMatchCandidatePromotionPersistenceV1 {
    pub async fn replay_approval_if_completed(
        &self,
        logical_owner_id: &str,
        approval: &ReviewedPersonMatchCandidatePromotionEnvelopeV1,
    ) -> Result<
        Option<ReviewedPersonMatchCandidatePromotionReplayV1>,
        ReviewedPersonMatchCandidatePromotionPersistenceErrorV1,
    > {
        validate_owner(logical_owner_id)?;
        approval.validate()?;
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, logical_owner_id).await?;
        let row = sqlx::query("SELECT approval_envelope_sha256,approval_envelope_bytes FROM makosh_data.reviewed_person_match_candidate_promotion_requests WHERE logical_owner_id=$1 AND approval_message_id=$2 FOR UPDATE")
            .bind(logical_owner_id).bind(approval.message_id.as_slice()).fetch_optional(&mut *tx).await.map_err(|_| storage())?;
        let Some(row) = row else {
            tx.rollback().await.map_err(|_| storage())?;
            return Ok(None);
        };
        if bytes::<32>(&row, "approval_envelope_sha256")? != approval.envelope_sha256
            || row
                .try_get::<Vec<u8>, _>("approval_envelope_bytes")
                .map_err(|_| storage())?
                != approval.envelope_bytes
        {
            return Err(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::Conflict);
        }
        tx.rollback().await.map_err(|_| storage())?;
        Ok(Some(
            ReviewedPersonMatchCandidatePromotionReplayV1::Replayed,
        ))
    }

    pub async fn replay_terminal_if_completed(
        &self,
        logical_owner_id: &str,
        result: &ReviewedPersonMatchCandidatePromotionEnvelopeV1,
    ) -> Result<
        Option<ReviewedPersonMatchCandidatePromotionReplayV1>,
        ReviewedPersonMatchCandidatePromotionPersistenceErrorV1,
    > {
        validate_owner(logical_owner_id)?;
        result.validate()?;
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, logical_owner_id).await?;
        let row = sqlx::query("SELECT envelope_sha256,envelope_bytes FROM makosh_data.reviewed_person_match_candidate_promotion_result_inbox WHERE logical_owner_id=$1 AND result_message_id=$2 FOR UPDATE")
            .bind(logical_owner_id).bind(result.message_id.as_slice()).fetch_optional(&mut *tx).await.map_err(|_| storage())?;
        let Some(row) = row else {
            tx.rollback().await.map_err(|_| storage())?;
            return Ok(None);
        };
        if bytes::<32>(&row, "envelope_sha256")? != result.envelope_sha256
            || row
                .try_get::<Vec<u8>, _>("envelope_bytes")
                .map_err(|_| storage())?
                != result.envelope_bytes
        {
            return Err(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::Conflict);
        }
        tx.rollback().await.map_err(|_| storage())?;
        Ok(Some(
            ReviewedPersonMatchCandidatePromotionReplayV1::Replayed,
        ))
    }

    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        host: &str,
        port: u32,
        password: &str,
    ) -> Result<Self, ReviewedPersonMatchCandidatePromotionPersistenceErrorV1> {
        if host.is_empty()
            || port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(storage());
        }
        let options = PgConnectOptions::new()
            .host(host)
            .port(u16::try_from(port).map_err(|_| storage())?)
            .username(binding.access().runtime_principal())
            .password(password)
            .database(binding.access().pool_alias());
        let pool = PgPoolOptions::new()
            .max_connections(u32::from(
                binding.access().effective_budgets().max_connections(),
            ))
            .connect_with(options)
            .await
            .map_err(|_| storage())?;
        Ok(Self { pool })
    }

    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[cfg(feature = "conformance-test-support")]
    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn verify_storage_ready(
        &self,
    ) -> Result<(), ReviewedPersonMatchCandidatePromotionPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| storage())
    }

    pub async fn persist_approval_once(
        &self,
        input: &PersistReviewedPersonMatchApprovalV1,
    ) -> Result<
        ReviewedPersonMatchCandidatePromotionReplayV1,
        ReviewedPersonMatchCandidatePromotionPersistenceErrorV1,
    > {
        validate_approval(input)?;
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, &input.logical_owner_id).await?;
        if let Some(row) = sqlx::query(
            "SELECT approval_envelope_sha256,approval_envelope_bytes,review_id,candidate_id,candidate_digest,decision_id,decision_revision,approved_action_digest,persons_command_id,persons_command_fingerprint,persons_command_message_id FROM makosh_data.reviewed_person_match_candidate_promotion_requests WHERE logical_owner_id=$1 AND approval_message_id=$2 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.approval.message_id.as_slice())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| storage())?
        {
            let exact = bytes::<32>(&row, "approval_envelope_sha256")? == input.approval.envelope_sha256
                && row.try_get::<Vec<u8>, _>("approval_envelope_bytes").map_err(|_| storage())? == input.approval.envelope_bytes
                && bytes::<16>(&row, "review_id")? == input.review_id
                && bytes::<16>(&row, "candidate_id")? == input.candidate_id
                && bytes::<32>(&row, "candidate_digest")? == input.candidate_digest
                && bytes::<16>(&row, "decision_id")? == input.decision_id
                && u64_from_i64(row.try_get("decision_revision").map_err(|_| storage())?)? == input.decision_revision
                && bytes::<32>(&row, "approved_action_digest")? == input.approved_action_digest
                && bytes::<16>(&row, "persons_command_id")? == input.persons_command_id
                && bytes::<32>(&row, "persons_command_fingerprint")? == input.persons_command_fingerprint
                && bytes::<16>(&row, "persons_command_message_id")? == input.persons_command.message_id;
            if !exact {
                return Err(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::Conflict);
            }
            verify_outbox(&mut tx, &input.logical_owner_id, &input.persons_command).await?;
            tx.rollback().await.map_err(|_| storage())?;
            return Ok(ReviewedPersonMatchCandidatePromotionReplayV1::Replayed);
        }
        sqlx::query(
            "INSERT INTO makosh_data.reviewed_person_match_candidate_promotion_requests (logical_owner_id,approval_message_id,approval_envelope_sha256,approval_envelope_bytes,review_id,candidate_id,candidate_digest,decision_id,decision_revision,approved_action_digest,persons_command_id,persons_command_fingerprint,persons_command_message_id,created_at_unix_millis,updated_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$14)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.approval.message_id.as_slice())
        .bind(input.approval.envelope_sha256.as_slice())
        .bind(&input.approval.envelope_bytes)
        .bind(input.review_id.as_slice())
        .bind(input.candidate_id.as_slice())
        .bind(input.candidate_digest.as_slice())
        .bind(input.decision_id.as_slice())
        .bind(i64::try_from(input.decision_revision).map_err(|_| invalid())?)
        .bind(input.approved_action_digest.as_slice())
        .bind(input.persons_command_id.as_slice())
        .bind(input.persons_command_fingerprint.as_slice())
        .bind(input.persons_command.message_id.as_slice())
        .bind(input.occurred_at_unix_millis)
        .execute(&mut *tx)
        .await
        .map_err(|error| if database_conflict(&error) { ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::Conflict } else { storage() })?;
        insert_outbox(
            &mut tx,
            &input.logical_owner_id,
            &input.persons_command,
            1,
            input.occurred_at_unix_millis,
        )
        .await?;
        tx.commit().await.map_err(|_| storage())?;
        Ok(ReviewedPersonMatchCandidatePromotionReplayV1::Applied)
    }

    pub async fn persist_approval_failure_once(
        &self,
        input: &PersistReviewedPersonMatchApprovalFailureV1,
    ) -> Result<
        ReviewedPersonMatchCandidatePromotionReplayV1,
        ReviewedPersonMatchCandidatePromotionPersistenceErrorV1,
    > {
        validate_approval_failure(input)?;
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, &input.logical_owner_id).await?;
        if let Some(row) = sqlx::query(
            "SELECT approval_envelope_sha256,approval_envelope_bytes,review_id,candidate_id,candidate_digest,decision_id,decision_revision,approved_action_digest,persons_command_id,persons_result_message_id,promotion_outcome,failure_code FROM makosh_data.reviewed_person_match_candidate_promotion_requests WHERE logical_owner_id=$1 AND approval_message_id=$2 FOR UPDATE",
        )
        .bind(&input.logical_owner_id)
        .bind(input.approval.message_id.as_slice())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| storage())?
        {
            let exact = bytes::<32>(&row, "approval_envelope_sha256")?
                == input.approval.envelope_sha256
                && row
                    .try_get::<Vec<u8>, _>("approval_envelope_bytes")
                    .map_err(|_| storage())?
                    == input.approval.envelope_bytes
                && bytes::<16>(&row, "review_id")? == input.review_id
                && bytes::<16>(&row, "candidate_id")? == input.candidate_id
                && bytes::<32>(&row, "candidate_digest")? == input.candidate_digest
                && bytes::<16>(&row, "decision_id")? == input.decision_id
                && u64_from_i64(
                    row.try_get("decision_revision").map_err(|_| storage())?,
                )? == input.decision_revision
                && bytes::<32>(&row, "approved_action_digest")? == input.approved_action_digest
                && row
                    .try_get::<Option<Vec<u8>>, _>("persons_command_id")
                    .map_err(|_| storage())?
                    .is_none()
                && row
                    .try_get::<Option<Vec<u8>>, _>("persons_result_message_id")
                    .map_err(|_| storage())?
                    .is_none()
                && row.try_get::<Option<i16>, _>("promotion_outcome").map_err(|_| storage())?
                    == Some(2)
                && row.try_get::<Option<i16>, _>("failure_code").map_err(|_| storage())?
                    == Some(2);
            if !exact {
                return Err(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::Conflict);
            }
            verify_outbox(&mut tx, &input.logical_owner_id, &input.review_result).await?;
            tx.rollback().await.map_err(|_| storage())?;
            return Ok(ReviewedPersonMatchCandidatePromotionReplayV1::Replayed);
        }
        sqlx::query(
            "INSERT INTO makosh_data.reviewed_person_match_candidate_promotion_requests (logical_owner_id,approval_message_id,approval_envelope_sha256,approval_envelope_bytes,review_id,candidate_id,candidate_digest,decision_id,decision_revision,approved_action_digest,promotion_outcome,failure_code,created_at_unix_millis,updated_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,2,2,$11,$11)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.approval.message_id.as_slice())
        .bind(input.approval.envelope_sha256.as_slice())
        .bind(&input.approval.envelope_bytes)
        .bind(input.review_id.as_slice())
        .bind(input.candidate_id.as_slice())
        .bind(input.candidate_digest.as_slice())
        .bind(input.decision_id.as_slice())
        .bind(i64::try_from(input.decision_revision).map_err(|_| invalid())?)
        .bind(input.approved_action_digest.as_slice())
        .bind(input.completed_at_unix_millis)
        .execute(&mut *tx)
        .await
        .map_err(|error| {
            if database_conflict(&error) {
                ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::Conflict
            } else {
                storage()
            }
        })?;
        insert_outbox(
            &mut tx,
            &input.logical_owner_id,
            &input.review_result,
            2,
            input.completed_at_unix_millis,
        )
        .await?;
        tx.commit().await.map_err(|_| storage())?;
        Ok(ReviewedPersonMatchCandidatePromotionReplayV1::Applied)
    }

    pub async fn load_correlation(
        &self,
        logical_owner_id: &str,
        persons_command_id: [u8; 16],
    ) -> Result<
        ReviewedPersonMatchCandidatePromotionCorrelationV1,
        ReviewedPersonMatchCandidatePromotionPersistenceErrorV1,
    > {
        validate_owner(logical_owner_id)?;
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, logical_owner_id).await?;
        let row = sqlx::query("SELECT review_id,candidate_id,decision_id,decision_revision,persons_command_id,persons_command_fingerprint,persons_result_message_id FROM makosh_data.reviewed_person_match_candidate_promotion_requests WHERE logical_owner_id=$1 AND persons_command_id=$2")
            .bind(logical_owner_id)
            .bind(persons_command_id.as_slice())
            .fetch_optional(&mut *tx).await.map_err(|_| storage())?
            .ok_or(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::NotFound)?;
        let correlation = ReviewedPersonMatchCandidatePromotionCorrelationV1 {
            review_id: bytes::<16>(&row, "review_id")?,
            candidate_id: bytes::<16>(&row, "candidate_id")?,
            decision_id: bytes::<16>(&row, "decision_id")?,
            decision_revision: u64_from_i64(
                row.try_get("decision_revision").map_err(|_| storage())?,
            )?,
            persons_command_id: bytes::<16>(&row, "persons_command_id")?,
            persons_command_fingerprint: bytes::<32>(&row, "persons_command_fingerprint")?,
            completed: row
                .try_get::<Option<Vec<u8>>, _>("persons_result_message_id")
                .map_err(|_| storage())?
                .is_some(),
        };
        tx.commit().await.map_err(|_| storage())?;
        Ok(correlation)
    }

    pub async fn persist_terminal_once(
        &self,
        input: &PersistReviewedPersonMatchTerminalV1,
    ) -> Result<
        ReviewedPersonMatchCandidatePromotionReplayV1,
        ReviewedPersonMatchCandidatePromotionPersistenceErrorV1,
    > {
        validate_terminal(input)?;
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, &input.logical_owner_id).await?;
        let request = sqlx::query("SELECT review_id,candidate_id,persons_result_message_id,promotion_outcome,failure_code,updated_at_unix_millis FROM makosh_data.reviewed_person_match_candidate_promotion_requests WHERE logical_owner_id=$1 AND persons_command_id=$2 FOR UPDATE")
            .bind(&input.logical_owner_id)
            .bind(input.persons_command_id.as_slice())
            .fetch_optional(&mut *tx).await.map_err(|_| storage())?
            .ok_or(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::NotFound)?;
        if bytes::<16>(&request, "review_id")? != input.review_id
            || bytes::<16>(&request, "candidate_id")? != input.candidate_id
        {
            return Err(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::Conflict);
        }
        if let Some(existing) = sqlx::query("SELECT envelope_sha256,envelope_bytes,persons_command_id,review_id FROM makosh_data.reviewed_person_match_candidate_promotion_result_inbox WHERE logical_owner_id=$1 AND result_message_id=$2 FOR UPDATE")
            .bind(&input.logical_owner_id)
            .bind(input.persons_result.message_id.as_slice())
            .fetch_optional(&mut *tx).await.map_err(|_| storage())?
        {
            let exact = bytes::<32>(&existing, "envelope_sha256")? == input.persons_result.envelope_sha256
                && existing.try_get::<Vec<u8>, _>("envelope_bytes").map_err(|_| storage())? == input.persons_result.envelope_bytes
                && bytes::<16>(&existing, "persons_command_id")? == input.persons_command_id
                && bytes::<16>(&existing, "review_id")? == input.review_id;
            if !exact {
                return Err(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::Conflict);
            }
            verify_outbox(&mut tx, &input.logical_owner_id, &input.review_result).await?;
            tx.rollback().await.map_err(|_| storage())?;
            return Ok(ReviewedPersonMatchCandidatePromotionReplayV1::Replayed);
        }
        let updated_at: i64 = request
            .try_get("updated_at_unix_millis")
            .map_err(|_| storage())?;
        if request
            .try_get::<Option<Vec<u8>>, _>("persons_result_message_id")
            .map_err(|_| storage())?
            .is_some()
            || input.completed_at_unix_millis < updated_at
        {
            return Err(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::Conflict);
        }
        sqlx::query("INSERT INTO makosh_data.reviewed_person_match_candidate_promotion_result_inbox (logical_owner_id,result_message_id,envelope_sha256,envelope_bytes,persons_command_id,review_id,processed_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,$7)")
            .bind(&input.logical_owner_id)
            .bind(input.persons_result.message_id.as_slice())
            .bind(input.persons_result.envelope_sha256.as_slice())
            .bind(&input.persons_result.envelope_bytes)
            .bind(input.persons_command_id.as_slice())
            .bind(input.review_id.as_slice())
            .bind(input.completed_at_unix_millis)
            .execute(&mut *tx).await.map_err(|error| if database_conflict(&error) { ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::Conflict } else { storage() })?;
        let outcome = if input.succeeded { 1_i16 } else { 2_i16 };
        let failure = input.failure_code.map(i16::from);
        sqlx::query("UPDATE makosh_data.reviewed_person_match_candidate_promotion_requests SET persons_result_message_id=$3,promotion_outcome=$4,failure_code=$5,updated_at_unix_millis=$6 WHERE logical_owner_id=$1 AND persons_command_id=$2 AND persons_result_message_id IS NULL")
            .bind(&input.logical_owner_id)
            .bind(input.persons_command_id.as_slice())
            .bind(input.persons_result.message_id.as_slice())
            .bind(outcome)
            .bind(failure)
            .bind(input.completed_at_unix_millis)
            .execute(&mut *tx).await.map_err(|_| storage())?;
        insert_outbox(
            &mut tx,
            &input.logical_owner_id,
            &input.review_result,
            2,
            input.completed_at_unix_millis,
        )
        .await?;
        tx.commit().await.map_err(|_| storage())?;
        Ok(ReviewedPersonMatchCandidatePromotionReplayV1::Applied)
    }

    pub async fn load_pending_outbox(
        &self,
        logical_owner_id: &str,
    ) -> Result<
        Vec<ReviewedPersonMatchCandidatePromotionOutboxV1>,
        ReviewedPersonMatchCandidatePromotionPersistenceErrorV1,
    > {
        validate_owner(logical_owner_id)?;
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, logical_owner_id).await?;
        let rows = sqlx::query("SELECT message_id,envelope_sha256,envelope_bytes,semantic_kind,created_at_unix_millis,published_at_unix_millis FROM makosh_data.reviewed_person_match_candidate_promotion_outbox WHERE logical_owner_id=$1 AND published_at_unix_millis IS NULL ORDER BY semantic_kind,created_at_unix_millis,message_id LIMIT $2")
            .bind(logical_owner_id).bind(OUTBOX_LIMIT).fetch_all(&mut *tx).await.map_err(|_| storage())?;
        let out = rows
            .into_iter()
            .map(outbox_from_row)
            .collect::<Result<Vec<_>, _>>()?;
        tx.commit().await.map_err(|_| storage())?;
        Ok(out)
    }

    pub async fn claim_next_pending_outbox(
        &self,
        logical_owner_id: &str,
    ) -> Result<
        Option<ReviewedPersonMatchCandidatePromotionOutboxPublishClaimV1>,
        ReviewedPersonMatchCandidatePromotionPersistenceErrorV1,
    > {
        validate_owner(logical_owner_id)?;
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let row = sqlx::query("SELECT message_id,envelope_sha256,envelope_bytes,semantic_kind,created_at_unix_millis,published_at_unix_millis FROM makosh_data.reviewed_person_match_candidate_promotion_outbox WHERE logical_owner_id=$1 AND published_at_unix_millis IS NULL ORDER BY outbox_sequence FOR UPDATE SKIP LOCKED LIMIT 1")
            .bind(logical_owner_id).fetch_optional(&mut *transaction).await.map_err(|_| storage())?;
        let Some(row) = row else {
            transaction.rollback().await.map_err(|_| storage())?;
            return Ok(None);
        };
        let record = outbox_from_row(row)?;
        Ok(Some(
            ReviewedPersonMatchCandidatePromotionOutboxPublishClaimV1 {
                transaction,
                logical_owner_id: logical_owner_id.to_owned(),
                record,
            },
        ))
    }

    pub async fn mark_outbox_published(
        &self,
        logical_owner_id: &str,
        message_id: [u8; 16],
        expected_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), ReviewedPersonMatchCandidatePromotionPersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, logical_owner_id).await?;
        let row = sqlx::query("SELECT envelope_sha256,envelope_bytes,created_at_unix_millis,published_at_unix_millis FROM makosh_data.reviewed_person_match_candidate_promotion_outbox WHERE logical_owner_id=$1 AND message_id=$2 FOR UPDATE")
            .bind(logical_owner_id).bind(message_id.as_slice()).fetch_optional(&mut *tx).await.map_err(|_| storage())?
            .ok_or(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::NotFound)?;
        let sha = bytes::<32>(&row, "envelope_sha256")?;
        let raw: Vec<u8> = row.try_get("envelope_bytes").map_err(|_| storage())?;
        let created: i64 = row
            .try_get("created_at_unix_millis")
            .map_err(|_| storage())?;
        if sha != expected_sha256
            || <[u8; 32]>::from(Sha256::digest(&raw)) != sha
            || published_at_unix_millis < created
        {
            return Err(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::HashMismatch);
        }
        if row
            .try_get::<Option<i64>, _>("published_at_unix_millis")
            .map_err(|_| storage())?
            .is_none()
        {
            sqlx::query("UPDATE makosh_data.reviewed_person_match_candidate_promotion_outbox SET published_at_unix_millis=$3 WHERE logical_owner_id=$1 AND message_id=$2 AND published_at_unix_millis IS NULL")
                .bind(logical_owner_id).bind(message_id.as_slice()).bind(published_at_unix_millis).execute(&mut *tx).await.map_err(|_| storage())?;
        }
        tx.commit().await.map_err(|_| storage())
    }
}

fn validate_approval(
    input: &PersistReviewedPersonMatchApprovalV1,
) -> Result<(), ReviewedPersonMatchCandidatePromotionPersistenceErrorV1> {
    validate_owner(&input.logical_owner_id)?;
    input.approval.validate()?;
    input.persons_command.validate()?;
    if input.decision_revision == 0
        || input.occurred_at_unix_millis <= 0
        || input.persons_command_id != input.persons_command.message_id
        || [
            input.review_id.as_slice(),
            input.candidate_id.as_slice(),
            input.candidate_digest.as_slice(),
            input.decision_id.as_slice(),
            input.approved_action_digest.as_slice(),
            input.persons_command_fingerprint.as_slice(),
        ]
        .iter()
        .any(|value| !nonzero(value))
    {
        return Err(invalid());
    }
    Ok(())
}

fn validate_approval_failure(
    input: &PersistReviewedPersonMatchApprovalFailureV1,
) -> Result<(), ReviewedPersonMatchCandidatePromotionPersistenceErrorV1> {
    validate_owner(&input.logical_owner_id)?;
    input.approval.validate()?;
    input.review_result.validate()?;
    if input.decision_revision == 0
        || input.completed_at_unix_millis <= 0
        || [
            input.review_id.as_slice(),
            input.candidate_id.as_slice(),
            input.candidate_digest.as_slice(),
            input.decision_id.as_slice(),
            input.approved_action_digest.as_slice(),
        ]
        .iter()
        .any(|value| !nonzero(value))
    {
        return Err(invalid());
    }
    Ok(())
}

fn validate_terminal(
    input: &PersistReviewedPersonMatchTerminalV1,
) -> Result<(), ReviewedPersonMatchCandidatePromotionPersistenceErrorV1> {
    validate_owner(&input.logical_owner_id)?;
    input.persons_result.validate()?;
    input.review_result.validate()?;
    if input.completed_at_unix_millis <= 0
        || !nonzero(&input.persons_command_id)
        || !nonzero(&input.review_id)
        || !nonzero(&input.candidate_id)
        || (input.succeeded && input.failure_code.is_some())
        || (!input.succeeded && input.failure_code != Some(3))
    {
        return Err(invalid());
    }
    Ok(())
}

async fn set_owner(
    tx: &mut Transaction<'_, Postgres>,
    owner: &str,
) -> Result<(), ReviewedPersonMatchCandidatePromotionPersistenceErrorV1> {
    sqlx::query("SELECT set_config('makosh.logical_owner_id',$1,true)")
        .bind(owner)
        .execute(&mut **tx)
        .await
        .map_err(|_| storage())?;
    Ok(())
}

async fn insert_outbox(
    tx: &mut Transaction<'_, Postgres>,
    owner: &str,
    record: &ReviewedPersonMatchCandidatePromotionEnvelopeV1,
    semantic_kind: i16,
    created_at: i64,
) -> Result<(), ReviewedPersonMatchCandidatePromotionPersistenceErrorV1> {
    sqlx::query("INSERT INTO makosh_data.reviewed_person_match_candidate_promotion_outbox (logical_owner_id,message_id,envelope_sha256,envelope_bytes,semantic_kind,created_at_unix_millis,published_at_unix_millis) VALUES ($1,$2,$3,$4,$5,$6,NULL)")
        .bind(owner).bind(record.message_id.as_slice()).bind(record.envelope_sha256.as_slice()).bind(&record.envelope_bytes).bind(semantic_kind).bind(created_at)
        .execute(&mut **tx).await.map_err(|error| if database_conflict(&error) { ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::Conflict } else { storage() })?;
    Ok(())
}

async fn verify_outbox(
    tx: &mut Transaction<'_, Postgres>,
    owner: &str,
    expected: &ReviewedPersonMatchCandidatePromotionEnvelopeV1,
) -> Result<(), ReviewedPersonMatchCandidatePromotionPersistenceErrorV1> {
    let row = sqlx::query("SELECT envelope_sha256,envelope_bytes FROM makosh_data.reviewed_person_match_candidate_promotion_outbox WHERE logical_owner_id=$1 AND message_id=$2")
        .bind(owner).bind(expected.message_id.as_slice()).fetch_optional(&mut **tx).await.map_err(|_| storage())?
        .ok_or(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::InvalidRow)?;
    let sha = bytes::<32>(&row, "envelope_sha256")?;
    let raw: Vec<u8> = row.try_get("envelope_bytes").map_err(|_| storage())?;
    if sha != expected.envelope_sha256
        || raw != expected.envelope_bytes
        || <[u8; 32]>::from(Sha256::digest(&raw)) != sha
    {
        return Err(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::HashMismatch);
    }
    Ok(())
}

fn outbox_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<
    ReviewedPersonMatchCandidatePromotionOutboxV1,
    ReviewedPersonMatchCandidatePromotionPersistenceErrorV1,
> {
    let record = ReviewedPersonMatchCandidatePromotionEnvelopeV1 {
        message_id: bytes::<16>(&row, "message_id")?,
        envelope_sha256: bytes::<32>(&row, "envelope_sha256")?,
        envelope_bytes: row.try_get("envelope_bytes").map_err(|_| storage())?,
    };
    record
        .validate()
        .map_err(|_| ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::HashMismatch)?;
    let semantic: i16 = row.try_get("semantic_kind").map_err(|_| storage())?;
    Ok(ReviewedPersonMatchCandidatePromotionOutboxV1 {
        record,
        semantic_kind: u8::try_from(semantic).map_err(|_| invalid())?,
        created_at_unix_millis: row
            .try_get("created_at_unix_millis")
            .map_err(|_| storage())?,
        published_at_unix_millis: row
            .try_get("published_at_unix_millis")
            .map_err(|_| storage())?,
    })
}

fn bytes<const N: usize>(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<[u8; N], ReviewedPersonMatchCandidatePromotionPersistenceErrorV1> {
    row.try_get::<Vec<u8>, _>(column)
        .map_err(|_| storage())?
        .try_into()
        .map_err(|_| ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::InvalidRow)
}

fn validate_owner(
    owner: &str,
) -> Result<(), ReviewedPersonMatchCandidatePromotionPersistenceErrorV1> {
    if owner.is_empty()
        || owner.len() > 128
        || !owner.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        })
    {
        Err(invalid())
    } else {
        Ok(())
    }
}

fn database_conflict(error: &sqlx::Error) -> bool {
    matches!(error, sqlx::Error::Database(db) if db.is_unique_violation() || db.is_check_violation())
}

fn u64_from_i64(
    value: i64,
) -> Result<u64, ReviewedPersonMatchCandidatePromotionPersistenceErrorV1> {
    u64::try_from(value)
        .map_err(|_| ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::InvalidRow)
}

fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}
fn invalid() -> ReviewedPersonMatchCandidatePromotionPersistenceErrorV1 {
    ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::InvalidInput
}
fn storage() -> ReviewedPersonMatchCandidatePromotionPersistenceErrorV1 {
    ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::StorageUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_hash_and_bounds_fail_closed() {
        let bytes = b"sanitized".to_vec();
        let valid = ReviewedPersonMatchCandidatePromotionEnvelopeV1 {
            message_id: [1; 16],
            envelope_sha256: Sha256::digest(&bytes).into(),
            envelope_bytes: bytes,
        };
        valid.validate().expect("valid");
        let mut corrupt = valid.clone();
        corrupt.envelope_bytes.push(1);
        assert_eq!(
            corrupt.validate(),
            Err(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::InvalidInput)
        );
        let mut private = valid;
        private.envelope_bytes = vec![0; 65_537];
        private.envelope_sha256 = Sha256::digest(&private.envelope_bytes).into();
        assert_eq!(
            private.validate(),
            Err(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::InvalidInput)
        );
    }

    #[test]
    fn terminal_outcome_shape_is_exact() {
        let record = || {
            let bytes = vec![1];
            ReviewedPersonMatchCandidatePromotionEnvelopeV1 {
                message_id: [1; 16],
                envelope_sha256: Sha256::digest(&bytes).into(),
                envelope_bytes: bytes,
            }
        };
        let mut input = PersistReviewedPersonMatchTerminalV1 {
            logical_owner_id: "owner.1".into(),
            persons_result: record(),
            persons_command_id: [2; 16],
            review_id: [3; 16],
            candidate_id: [4; 16],
            succeeded: true,
            failure_code: None,
            review_result: record(),
            completed_at_unix_millis: 1,
        };
        validate_terminal(&input).expect("success");
        input.failure_code = Some(3);
        assert_eq!(
            validate_terminal(&input),
            Err(ReviewedPersonMatchCandidatePromotionPersistenceErrorV1::InvalidInput)
        );
        input.succeeded = false;
        validate_terminal(&input).expect("failure");
    }
}
