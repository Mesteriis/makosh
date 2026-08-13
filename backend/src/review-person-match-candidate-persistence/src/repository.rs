use makosh_review_person_match_candidate_core::{
    DecidePersonMatchCandidateV1, PersonMatchCandidateApprovedActionV1,
    PersonMatchCandidateCoreErrorV1, PersonMatchCandidateDecisionV1,
    PersonMatchCandidateEvidenceV1, PersonMatchCandidatePromotionStatusV1,
    PersonMatchCandidateReviewV1, PersonMatchCandidateStateV1, PersonMatchKindV1,
    PublicPersonSourceIdentityV1, SplitProfileFactKindV1, SplitSourceSelectionV1,
    create_person_match_candidate_review_v1, decide_person_match_candidate_v1,
    record_person_match_candidate_promotion_v1, validate_review,
};
use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

const OUTBOX_LIMIT: i64 = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewPersonMatchCandidateEnvelopeRecordV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

impl ReviewPersonMatchCandidateEnvelopeRecordV1 {
    pub fn validate(&self) -> Result<(), ReviewPersonMatchCandidatePersistenceErrorV1> {
        if !nonzero(&self.message_id)
            || !nonzero(&self.envelope_sha256)
            || self.envelope_bytes.is_empty()
            || self.envelope_bytes.len() > 65_536
            || <[u8; 32]>::from(Sha256::digest(&self.envelope_bytes)) != self.envelope_sha256
        {
            Err(ReviewPersonMatchCandidatePersistenceErrorV1::InvalidInput)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewPersonMatchCandidateOutboxRecordV1 {
    pub record: ReviewPersonMatchCandidateEnvelopeRecordV1,
    pub review_id: [u8; 16],
    pub review_revision: u64,
    pub semantic_kind: u8,
    pub created_at_unix_millis: i64,
    pub published_at_unix_millis: Option<i64>,
}

pub struct ReviewPersonMatchCandidateOutboxPublishClaimV1 {
    transaction: Transaction<'static, Postgres>,
    logical_owner_id: String,
    record: ReviewPersonMatchCandidateOutboxRecordV1,
}

impl ReviewPersonMatchCandidateOutboxPublishClaimV1 {
    #[must_use]
    pub fn record(&self) -> &ReviewPersonMatchCandidateOutboxRecordV1 {
        &self.record
    }

    pub async fn mark_published(
        mut self,
        expected_sha256: [u8; 32],
        published_at: i64,
    ) -> Result<(), ReviewPersonMatchCandidatePersistenceErrorV1> {
        if expected_sha256 != self.record.record.envelope_sha256
            || published_at < self.record.created_at_unix_millis
        {
            return Err(ReviewPersonMatchCandidatePersistenceErrorV1::HashMismatch);
        }
        let affected = sqlx::query("UPDATE makosh_data.review_person_match_candidate_outbox SET published_at_unix_millis=$3 WHERE logical_owner_id=$1 AND message_id=$2 AND envelope_sha256=$4 AND published_at_unix_millis IS NULL")
            .bind(&self.logical_owner_id).bind(self.record.record.message_id.as_slice()).bind(published_at).bind(expected_sha256.as_slice()).execute(&mut *self.transaction).await.map_err(|_| storage())?.rows_affected();
        if affected != 1 {
            return Err(ReviewPersonMatchCandidatePersistenceErrorV1::Conflict);
        }
        self.transaction.commit().await.map_err(|_| storage())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubmitPersonMatchCandidateOperationV1 {
    pub command: ReviewPersonMatchCandidateEnvelopeRecordV1,
    pub evidence: PersonMatchCandidateEvidenceV1,
    pub submitted_result: ReviewPersonMatchCandidateEnvelopeRecordV1,
    pub expected_existing_revision: Option<u64>,
    pub received_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecidePersonMatchCandidateOperationV1 {
    pub logical_owner_id: String,
    pub command: ReviewPersonMatchCandidateEnvelopeRecordV1,
    pub review_id: [u8; 16],
    pub expected_review_revision: u64,
    pub decision: PersonMatchCandidateDecisionV1,
    pub decided_by_owner_device_id: [u8; 16],
    pub decided_at_unix_millis: i64,
    pub approved_event: Option<ReviewPersonMatchCandidateEnvelopeRecordV1>,
    pub received_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistPersonMatchCandidatePromotionResultV1 {
    pub logical_owner_id: String,
    pub result: ReviewPersonMatchCandidateEnvelopeRecordV1,
    pub review_id: [u8; 16],
    pub candidate_id: [u8; 16],
    pub decision_id: [u8; 16],
    pub persons_command_id: Option<[u8; 16]>,
    pub expected_review_revision: u64,
    pub succeeded: bool,
    pub completed_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReviewPersonMatchCandidateReplayOutcomeV1 {
    Applied(PersonMatchCandidateReviewV1),
    Replayed(PersonMatchCandidateReviewV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewPersonMatchCandidatePersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    Conflict,
    RevisionConflict,
    NotFound,
    HashMismatch,
    StorageUnavailable,
}

#[derive(Clone)]
pub struct ReviewPersonMatchCandidatePersistenceV1 {
    pool: PgPool,
}

impl ReviewPersonMatchCandidatePersistenceV1 {
    pub async fn replay_submission_if_completed(
        &self,
        logical_owner_id: &str,
        command: &ReviewPersonMatchCandidateEnvelopeRecordV1,
    ) -> Result<
        Option<ReviewPersonMatchCandidateReplayOutcomeV1>,
        ReviewPersonMatchCandidatePersistenceErrorV1,
    > {
        validate_owner(logical_owner_id)?;
        command.validate()?;
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, logical_owner_id).await?;
        let existing = load_inbox(&mut tx, logical_owner_id, command.message_id).await?;
        let Some(existing) = existing else {
            tx.rollback().await.map_err(|_| storage())?;
            return Ok(None);
        };
        if existing.envelope_sha256 != command.envelope_sha256
            || existing.envelope_bytes != command.envelope_bytes
        {
            return Err(ReviewPersonMatchCandidatePersistenceErrorV1::Conflict);
        }
        let current = load_review(&mut tx, logical_owner_id, existing.review_id, false).await?;
        tx.rollback().await.map_err(|_| storage())?;
        Ok(Some(ReviewPersonMatchCandidateReplayOutcomeV1::Replayed(
            current,
        )))
    }

    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        host: &str,
        port: u32,
        password: &str,
    ) -> Result<Self, ReviewPersonMatchCandidatePersistenceErrorV1> {
        if host.is_empty()
            || port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(ReviewPersonMatchCandidatePersistenceErrorV1::StorageUnavailable);
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
    ) -> Result<(), ReviewPersonMatchCandidatePersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| storage())
    }

    pub async fn submit_once(
        &self,
        input: &SubmitPersonMatchCandidateOperationV1,
    ) -> Result<
        ReviewPersonMatchCandidateReplayOutcomeV1,
        ReviewPersonMatchCandidatePersistenceErrorV1,
    > {
        input.command.validate()?;
        input.submitted_result.validate()?;
        let review =
            create_person_match_candidate_review_v1(input.evidence.clone()).map_err(core_error)?;
        if input.received_at_unix_millis < review.evidence.observed_at_unix_millis {
            return Err(ReviewPersonMatchCandidatePersistenceErrorV1::InvalidInput);
        }
        let fingerprint = submission_fingerprint(input);
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, &review.evidence.logical_owner_id).await?;
        if let Some(existing) = load_inbox(
            &mut tx,
            &review.evidence.logical_owner_id,
            input.command.message_id,
        )
        .await?
        {
            if existing.envelope_sha256 != input.command.envelope_sha256
                || existing.envelope_bytes != input.command.envelope_bytes
                || existing.fingerprint != fingerprint
                || existing.review_id != review.review_id
            {
                return Err(ReviewPersonMatchCandidatePersistenceErrorV1::Conflict);
            }
            let current = load_review(
                &mut tx,
                &review.evidence.logical_owner_id,
                review.review_id,
                false,
            )
            .await?;
            tx.rollback().await.map_err(|_| storage())?;
            return Ok(ReviewPersonMatchCandidateReplayOutcomeV1::Replayed(current));
        }
        let applied = match load_review(
            &mut tx,
            &review.evidence.logical_owner_id,
            review.review_id,
            true,
        )
        .await
        {
            Ok(current) => {
                if input.expected_existing_revision != Some(current.review_revision)
                    || current.state != PersonMatchCandidateStateV1::Pending
                    || review.evidence.resulting_owner_revision
                        <= current.evidence.resulting_owner_revision
                    || review.evidence.observed_at_unix_millis < current.updated_at_unix_millis
                {
                    return Err(ReviewPersonMatchCandidatePersistenceErrorV1::Conflict);
                }
                let mut updated = review;
                updated.review_revision = current
                    .review_revision
                    .checked_add(1)
                    .ok_or(ReviewPersonMatchCandidatePersistenceErrorV1::InvalidInput)?;
                update_review(&mut tx, &updated, current.review_revision).await?;
                updated
            }
            Err(ReviewPersonMatchCandidatePersistenceErrorV1::NotFound) => {
                if input.expected_existing_revision.is_some() {
                    return Err(ReviewPersonMatchCandidatePersistenceErrorV1::Conflict);
                }
                insert_review(&mut tx, &review).await?;
                review
            }
            Err(error) => return Err(error),
        };
        insert_inbox(
            &mut tx,
            &applied.evidence.logical_owner_id,
            &input.command,
            1,
            fingerprint,
            applied.review_id,
            applied.review_revision,
            input.received_at_unix_millis,
            input.received_at_unix_millis,
        )
        .await?;
        insert_outbox(
            &mut tx,
            &applied.evidence.logical_owner_id,
            &input.submitted_result,
            applied.review_id,
            applied.review_revision,
            1,
            input.received_at_unix_millis,
        )
        .await?;
        tx.commit().await.map_err(|_| storage())?;
        Ok(ReviewPersonMatchCandidateReplayOutcomeV1::Applied(applied))
    }

    pub async fn decide_once(
        &self,
        input: &DecidePersonMatchCandidateOperationV1,
    ) -> Result<
        ReviewPersonMatchCandidateReplayOutcomeV1,
        ReviewPersonMatchCandidatePersistenceErrorV1,
    > {
        validate_owner(&input.logical_owner_id)?;
        input.command.validate()?;
        let fingerprint = decision_fingerprint(input)?;
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, &input.logical_owner_id).await?;
        if let Some(existing) =
            load_inbox(&mut tx, &input.logical_owner_id, input.command.message_id).await?
        {
            if existing.envelope_sha256 != input.command.envelope_sha256
                || existing.envelope_bytes != input.command.envelope_bytes
                || existing.fingerprint != fingerprint
                || existing.review_id != input.review_id
            {
                return Err(ReviewPersonMatchCandidatePersistenceErrorV1::Conflict);
            }
            let current =
                load_review(&mut tx, &input.logical_owner_id, input.review_id, false).await?;
            tx.rollback().await.map_err(|_| storage())?;
            return Ok(ReviewPersonMatchCandidateReplayOutcomeV1::Replayed(current));
        }
        let current = load_review(&mut tx, &input.logical_owner_id, input.review_id, true).await?;
        let next = decide_person_match_candidate_v1(
            &current,
            DecidePersonMatchCandidateV1 {
                decision_id: input.command.message_id,
                expected_review_revision: input.expected_review_revision,
                decision: input.decision.clone(),
                decided_by_owner_device_id: input.decided_by_owner_device_id,
                decided_at_unix_millis: input.decided_at_unix_millis,
            },
        )
        .map_err(core_error)?;
        match (&input.decision, &input.approved_event) {
            (PersonMatchCandidateDecisionV1::Approve { .. }, Some(event)) => event.validate()?,
            (PersonMatchCandidateDecisionV1::Reject, None) => {}
            _ => return Err(ReviewPersonMatchCandidatePersistenceErrorV1::InvalidInput),
        }
        update_review(&mut tx, &next, current.review_revision).await?;
        insert_inbox(
            &mut tx,
            &input.logical_owner_id,
            &input.command,
            2,
            fingerprint,
            next.review_id,
            next.review_revision,
            input.received_at_unix_millis,
            input.received_at_unix_millis,
        )
        .await?;
        if let Some(event) = &input.approved_event {
            insert_outbox(
                &mut tx,
                &input.logical_owner_id,
                event,
                next.review_id,
                next.review_revision,
                2,
                input.decided_at_unix_millis,
            )
            .await?;
        }
        tx.commit().await.map_err(|_| storage())?;
        Ok(ReviewPersonMatchCandidateReplayOutcomeV1::Applied(next))
    }

    pub async fn persist_promotion_result_once(
        &self,
        input: &PersistPersonMatchCandidatePromotionResultV1,
    ) -> Result<
        ReviewPersonMatchCandidateReplayOutcomeV1,
        ReviewPersonMatchCandidatePersistenceErrorV1,
    > {
        validate_owner(&input.logical_owner_id)?;
        input.result.validate()?;
        let fingerprint = promotion_fingerprint(input);
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, &input.logical_owner_id).await?;
        if let Some(existing) =
            load_inbox(&mut tx, &input.logical_owner_id, input.result.message_id).await?
        {
            if existing.envelope_sha256 != input.result.envelope_sha256
                || existing.envelope_bytes != input.result.envelope_bytes
                || existing.fingerprint != fingerprint
                || existing.review_id != input.review_id
            {
                return Err(ReviewPersonMatchCandidatePersistenceErrorV1::Conflict);
            }
            let current =
                load_review(&mut tx, &input.logical_owner_id, input.review_id, false).await?;
            tx.rollback().await.map_err(|_| storage())?;
            return Ok(ReviewPersonMatchCandidateReplayOutcomeV1::Replayed(current));
        }
        let current = load_review(&mut tx, &input.logical_owner_id, input.review_id, true).await?;
        if current.evidence.candidate_id != input.candidate_id
            || current.decision_id != Some(input.decision_id)
            || current.review_revision != input.expected_review_revision
        {
            return Err(ReviewPersonMatchCandidatePersistenceErrorV1::RevisionConflict);
        }
        let next = record_person_match_candidate_promotion_v1(
            &current,
            input.succeeded,
            input.completed_at_unix_millis,
        )
        .map_err(core_error)?;
        update_review(&mut tx, &next, current.review_revision).await?;
        insert_inbox(
            &mut tx,
            &input.logical_owner_id,
            &input.result,
            3,
            fingerprint,
            next.review_id,
            next.review_revision,
            input.completed_at_unix_millis,
            input.completed_at_unix_millis,
        )
        .await?;
        tx.commit().await.map_err(|_| storage())?;
        Ok(ReviewPersonMatchCandidateReplayOutcomeV1::Applied(next))
    }

    pub async fn load_review(
        &self,
        logical_owner_id: &str,
        review_id: [u8; 16],
    ) -> Result<PersonMatchCandidateReviewV1, ReviewPersonMatchCandidatePersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, logical_owner_id).await?;
        let review = load_review(&mut tx, logical_owner_id, review_id, false).await?;
        tx.commit().await.map_err(|_| storage())?;
        Ok(review)
    }

    pub async fn list_reviews(
        &self,
        logical_owner_id: &str,
        after_review_id: Option<[u8; 16]>,
        limit: u32,
    ) -> Result<Vec<PersonMatchCandidateReviewV1>, ReviewPersonMatchCandidatePersistenceErrorV1>
    {
        validate_owner(logical_owner_id)?;
        // Callers request at most 200 visible rows and may ask for one
        // additional row solely to derive the next-page cursor.
        if !(1..=201).contains(&limit) {
            return Err(ReviewPersonMatchCandidatePersistenceErrorV1::InvalidInput);
        }
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, logical_owner_id).await?;
        let rows = sqlx::query(
            "SELECT * FROM makosh_data.review_person_match_candidate_state \
             WHERE logical_owner_id=$1 AND ($2::bytea IS NULL OR review_id > $2) \
             ORDER BY review_id LIMIT $3",
        )
        .bind(logical_owner_id)
        .bind(after_review_id.map(|value| value.to_vec()))
        .bind(i64::from(limit))
        .fetch_all(&mut *tx)
        .await
        .map_err(|_| storage())?;
        let reviews = rows
            .iter()
            .map(review_from_row)
            .collect::<Result<Vec<_>, _>>()?;
        tx.commit().await.map_err(|_| storage())?;
        Ok(reviews)
    }

    pub async fn load_pending_outbox(
        &self,
        logical_owner_id: &str,
    ) -> Result<
        Vec<ReviewPersonMatchCandidateOutboxRecordV1>,
        ReviewPersonMatchCandidatePersistenceErrorV1,
    > {
        validate_owner(logical_owner_id)?;
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, logical_owner_id).await?;
        let rows = sqlx::query(
            "SELECT message_id,envelope_sha256,envelope_bytes,review_id,review_revision,semantic_kind,created_at_unix_millis,published_at_unix_millis \
             FROM makosh_data.review_person_match_candidate_outbox WHERE logical_owner_id=$1 \
             AND published_at_unix_millis IS NULL ORDER BY review_revision,semantic_kind,message_id LIMIT $2",
        )
        .bind(logical_owner_id)
        .bind(OUTBOX_LIMIT)
        .fetch_all(&mut *tx)
        .await
        .map_err(|_| storage())?;
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
        Option<ReviewPersonMatchCandidateOutboxPublishClaimV1>,
        ReviewPersonMatchCandidatePersistenceErrorV1,
    > {
        validate_owner(logical_owner_id)?;
        let mut transaction = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut transaction, logical_owner_id).await?;
        let row = sqlx::query("SELECT message_id,envelope_sha256,envelope_bytes,review_id,review_revision,semantic_kind,created_at_unix_millis,published_at_unix_millis FROM makosh_data.review_person_match_candidate_outbox WHERE logical_owner_id=$1 AND published_at_unix_millis IS NULL ORDER BY outbox_sequence FOR UPDATE SKIP LOCKED LIMIT 1")
            .bind(logical_owner_id).fetch_optional(&mut *transaction).await.map_err(|_| storage())?;
        let Some(row) = row else {
            transaction.rollback().await.map_err(|_| storage())?;
            return Ok(None);
        };
        let record = outbox_from_row(row)?;
        Ok(Some(ReviewPersonMatchCandidateOutboxPublishClaimV1 {
            transaction,
            logical_owner_id: logical_owner_id.to_owned(),
            record,
        }))
    }

    pub async fn mark_outbox_published(
        &self,
        logical_owner_id: &str,
        message_id: [u8; 16],
        expected_sha256: [u8; 32],
        published_at_unix_millis: i64,
    ) -> Result<(), ReviewPersonMatchCandidatePersistenceErrorV1> {
        validate_owner(logical_owner_id)?;
        let mut tx = self.pool.begin().await.map_err(|_| storage())?;
        set_owner(&mut tx, logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT envelope_sha256,envelope_bytes,created_at_unix_millis,published_at_unix_millis \
             FROM makosh_data.review_person_match_candidate_outbox \
             WHERE logical_owner_id=$1 AND message_id=$2 FOR UPDATE",
        )
        .bind(logical_owner_id)
        .bind(message_id.as_slice())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| storage())?
        .ok_or(ReviewPersonMatchCandidatePersistenceErrorV1::NotFound)?;
        let stored_sha = bytes::<32>(&row, "envelope_sha256")?;
        let stored_bytes: Vec<u8> = row.try_get("envelope_bytes").map_err(|_| storage())?;
        let created: i64 = row
            .try_get("created_at_unix_millis")
            .map_err(|_| storage())?;
        let published: Option<i64> = row
            .try_get("published_at_unix_millis")
            .map_err(|_| storage())?;
        if stored_sha != expected_sha256
            || <[u8; 32]>::from(Sha256::digest(&stored_bytes)) != stored_sha
            || published_at_unix_millis < created
        {
            return Err(ReviewPersonMatchCandidatePersistenceErrorV1::HashMismatch);
        }
        if published.is_none() {
            sqlx::query(
                "UPDATE makosh_data.review_person_match_candidate_outbox SET published_at_unix_millis=$3 \
                 WHERE logical_owner_id=$1 AND message_id=$2 AND published_at_unix_millis IS NULL",
            )
            .bind(logical_owner_id)
            .bind(message_id.as_slice())
            .bind(published_at_unix_millis)
            .execute(&mut *tx)
            .await
            .map_err(|_| storage())?;
        }
        tx.commit().await.map_err(|_| storage())
    }
}

struct InboxRow {
    envelope_sha256: [u8; 32],
    envelope_bytes: Vec<u8>,
    fingerprint: [u8; 32],
    review_id: [u8; 16],
}

async fn load_inbox(
    tx: &mut Transaction<'_, Postgres>,
    owner: &str,
    message_id: [u8; 16],
) -> Result<Option<InboxRow>, ReviewPersonMatchCandidatePersistenceErrorV1> {
    sqlx::query(
        "SELECT envelope_sha256,envelope_bytes,request_fingerprint,review_id \
         FROM makosh_data.review_person_match_candidate_inbox \
         WHERE logical_owner_id=$1 AND message_id=$2 FOR UPDATE",
    )
    .bind(owner)
    .bind(message_id.as_slice())
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| storage())?
    .map(|row| {
        Ok(InboxRow {
            envelope_sha256: bytes::<32>(&row, "envelope_sha256")?,
            envelope_bytes: row.try_get("envelope_bytes").map_err(|_| storage())?,
            fingerprint: bytes::<32>(&row, "request_fingerprint")?,
            review_id: bytes::<16>(&row, "review_id")?,
        })
    })
    .transpose()
}

#[allow(clippy::too_many_arguments)]
async fn insert_inbox(
    tx: &mut Transaction<'_, Postgres>,
    owner: &str,
    record: &ReviewPersonMatchCandidateEnvelopeRecordV1,
    kind: i16,
    fingerprint: [u8; 32],
    review_id: [u8; 16],
    revision: u64,
    received_at: i64,
    completed_at: i64,
) -> Result<(), ReviewPersonMatchCandidatePersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.review_person_match_candidate_inbox \
         (logical_owner_id,message_id,envelope_sha256,envelope_bytes,message_kind,request_fingerprint,review_id,resulting_review_revision,received_at_unix_millis,completed_at_unix_millis) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
    )
    .bind(owner)
    .bind(record.message_id.as_slice())
    .bind(record.envelope_sha256.as_slice())
    .bind(&record.envelope_bytes)
    .bind(kind)
    .bind(fingerprint.as_slice())
    .bind(review_id.as_slice())
    .bind(i64::try_from(revision).map_err(|_| invalid())?)
    .bind(received_at)
    .bind(completed_at)
    .execute(&mut **tx)
    .await
    .map_err(|_| storage())?;
    Ok(())
}

async fn insert_outbox(
    tx: &mut Transaction<'_, Postgres>,
    owner: &str,
    record: &ReviewPersonMatchCandidateEnvelopeRecordV1,
    review_id: [u8; 16],
    revision: u64,
    semantic_kind: i16,
    created_at: i64,
) -> Result<(), ReviewPersonMatchCandidatePersistenceErrorV1> {
    sqlx::query(
        "INSERT INTO makosh_data.review_person_match_candidate_outbox \
         (logical_owner_id,message_id,envelope_sha256,envelope_bytes,review_id,review_revision,semantic_kind,created_at_unix_millis,published_at_unix_millis) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,NULL)",
    )
    .bind(owner)
    .bind(record.message_id.as_slice())
    .bind(record.envelope_sha256.as_slice())
    .bind(&record.envelope_bytes)
    .bind(review_id.as_slice())
    .bind(i64::try_from(revision).map_err(|_| invalid())?)
    .bind(semantic_kind)
    .bind(created_at)
    .execute(&mut **tx)
    .await
    .map_err(|_| storage())?;
    Ok(())
}

async fn insert_review(
    tx: &mut Transaction<'_, Postgres>,
    review: &PersonMatchCandidateReviewV1,
) -> Result<(), ReviewPersonMatchCandidatePersistenceErrorV1> {
    let values = review_values(review)?;
    sqlx::query(
        "INSERT INTO makosh_data.review_person_match_candidate_state \
         (logical_owner_id,review_id,evidence_event_id,candidate_id,candidate_digest,first_person_id,second_person_id, \
          first_integration_public_id,first_account_public_id,first_source_public_id,second_integration_public_id,second_account_public_id,second_source_public_id, \
          match_kind,observed_at_unix_millis,resulting_owner_revision,state,promotion_status,review_revision,decision_id,decided_by_owner_device_id,decided_at_unix_millis, \
          approved_action_kind,approved_action_bytes,approved_action_digest,updated_at_unix_millis) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26)",
    )
    .bind(&review.evidence.logical_owner_id)
    .bind(review.review_id.as_slice())
    .bind(review.evidence.evidence_event_id.as_slice())
    .bind(review.evidence.candidate_id.as_slice())
    .bind(review.evidence.candidate_digest.as_slice())
    .bind(review.evidence.first_person_id.as_slice())
    .bind(review.evidence.second_person_id.as_slice())
    .bind(review.evidence.first_source.integration_public_id.as_slice())
    .bind(review.evidence.first_source.account_public_id.as_slice())
    .bind(review.evidence.first_source.provider_source_contact_public_id.as_slice())
    .bind(review.evidence.second_source.integration_public_id.as_slice())
    .bind(review.evidence.second_source.account_public_id.as_slice())
    .bind(review.evidence.second_source.provider_source_contact_public_id.as_slice())
    .bind(match_kind(review.evidence.match_kind))
    .bind(review.evidence.observed_at_unix_millis)
    .bind(i64::try_from(review.evidence.resulting_owner_revision).map_err(|_| invalid())?)
    .bind(values.state)
    .bind(values.promotion)
    .bind(i64::try_from(review.review_revision).map_err(|_| invalid())?)
    .bind(values.decision_id.as_deref())
    .bind(values.device_id.as_deref())
    .bind(values.decided_at)
    .bind(values.action_kind)
    .bind(values.action_bytes.as_deref())
    .bind(values.action_digest.as_deref())
    .bind(review.updated_at_unix_millis)
    .execute(&mut **tx)
    .await
    .map_err(|_| storage())?;
    Ok(())
}

async fn update_review(
    tx: &mut Transaction<'_, Postgres>,
    review: &PersonMatchCandidateReviewV1,
    expected_revision: u64,
) -> Result<(), ReviewPersonMatchCandidatePersistenceErrorV1> {
    let values = review_values(review)?;
    let affected = sqlx::query(
        "UPDATE makosh_data.review_person_match_candidate_state SET \
         evidence_event_id=$4,candidate_id=$5,candidate_digest=$6,first_person_id=$7,second_person_id=$8, \
         first_integration_public_id=$9,first_account_public_id=$10,first_source_public_id=$11, \
         second_integration_public_id=$12,second_account_public_id=$13,second_source_public_id=$14, \
         match_kind=$15,observed_at_unix_millis=$16,resulting_owner_revision=$17, \
         state=$18,promotion_status=$19,review_revision=$20,decision_id=$21,decided_by_owner_device_id=$22,decided_at_unix_millis=$23, \
         approved_action_kind=$24,approved_action_bytes=$25,approved_action_digest=$26,updated_at_unix_millis=$27 \
         WHERE logical_owner_id=$1 AND review_id=$2 AND review_revision=$3",
    )
    .bind(&review.evidence.logical_owner_id)
    .bind(review.review_id.as_slice())
    .bind(i64::try_from(expected_revision).map_err(|_| invalid())?)
    .bind(review.evidence.evidence_event_id.as_slice())
    .bind(review.evidence.candidate_id.as_slice())
    .bind(review.evidence.candidate_digest.as_slice())
    .bind(review.evidence.first_person_id.as_slice())
    .bind(review.evidence.second_person_id.as_slice())
    .bind(review.evidence.first_source.integration_public_id.as_slice())
    .bind(review.evidence.first_source.account_public_id.as_slice())
    .bind(
        review
            .evidence
            .first_source
            .provider_source_contact_public_id
            .as_slice(),
    )
    .bind(review.evidence.second_source.integration_public_id.as_slice())
    .bind(review.evidence.second_source.account_public_id.as_slice())
    .bind(
        review
            .evidence
            .second_source
            .provider_source_contact_public_id
            .as_slice(),
    )
    .bind(match_kind(review.evidence.match_kind))
    .bind(review.evidence.observed_at_unix_millis)
    .bind(i64::try_from(review.evidence.resulting_owner_revision).map_err(|_| invalid())?)
    .bind(values.state)
    .bind(values.promotion)
    .bind(i64::try_from(review.review_revision).map_err(|_| invalid())?)
    .bind(values.decision_id.as_deref())
    .bind(values.device_id.as_deref())
    .bind(values.decided_at)
    .bind(values.action_kind)
    .bind(values.action_bytes.as_deref())
    .bind(values.action_digest.as_deref())
    .bind(review.updated_at_unix_millis)
    .execute(&mut **tx)
    .await
    .map_err(|_| storage())?
    .rows_affected();
    if affected != 1 {
        return Err(ReviewPersonMatchCandidatePersistenceErrorV1::RevisionConflict);
    }
    Ok(())
}

async fn load_review(
    tx: &mut Transaction<'_, Postgres>,
    owner: &str,
    review_id: [u8; 16],
    for_update: bool,
) -> Result<PersonMatchCandidateReviewV1, ReviewPersonMatchCandidatePersistenceErrorV1> {
    let row = if for_update {
        sqlx::query(
            "SELECT * FROM makosh_data.review_person_match_candidate_state \
             WHERE logical_owner_id=$1 AND review_id=$2 FOR UPDATE",
        )
        .bind(owner)
        .bind(review_id.as_slice())
        .fetch_optional(&mut **tx)
        .await
        .map_err(|_| storage())?
    } else {
        sqlx::query(
            "SELECT * FROM makosh_data.review_person_match_candidate_state \
             WHERE logical_owner_id=$1 AND review_id=$2",
        )
        .bind(owner)
        .bind(review_id.as_slice())
        .fetch_optional(&mut **tx)
        .await
        .map_err(|_| storage())?
    };
    let row = row.ok_or(ReviewPersonMatchCandidatePersistenceErrorV1::NotFound)?;
    review_from_row(&row)
}

fn review_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<PersonMatchCandidateReviewV1, ReviewPersonMatchCandidatePersistenceErrorV1> {
    let state = match row.try_get::<i16, _>("state").map_err(|_| invalid())? {
        1 => PersonMatchCandidateStateV1::Pending,
        2 => PersonMatchCandidateStateV1::Approved,
        3 => PersonMatchCandidateStateV1::Rejected,
        _ => return Err(ReviewPersonMatchCandidatePersistenceErrorV1::InvalidRow),
    };
    let promotion_status = match row
        .try_get::<i16, _>("promotion_status")
        .map_err(|_| invalid())?
    {
        1 => PersonMatchCandidatePromotionStatusV1::NotRequested,
        2 => PersonMatchCandidatePromotionStatusV1::Pending,
        3 => PersonMatchCandidatePromotionStatusV1::Succeeded,
        4 => PersonMatchCandidatePromotionStatusV1::Failed,
        _ => return Err(ReviewPersonMatchCandidatePersistenceErrorV1::InvalidRow),
    };
    let action_kind: Option<i16> = row.try_get("approved_action_kind").map_err(|_| invalid())?;
    let action_bytes: Option<Vec<u8>> = row
        .try_get("approved_action_bytes")
        .map_err(|_| invalid())?;
    let approved_action = match (action_kind, action_bytes) {
        (None, None) => None,
        (Some(kind), Some(bytes)) => Some(decode_action(kind, &bytes)?),
        _ => return Err(ReviewPersonMatchCandidatePersistenceErrorV1::InvalidRow),
    };
    let review = PersonMatchCandidateReviewV1 {
        review_id: bytes::<16>(row, "review_id")?,
        evidence: PersonMatchCandidateEvidenceV1 {
            evidence_event_id: bytes::<16>(row, "evidence_event_id")?,
            candidate_id: bytes::<16>(row, "candidate_id")?,
            logical_owner_id: row.try_get("logical_owner_id").map_err(|_| invalid())?,
            first_person_id: bytes::<16>(row, "first_person_id")?,
            second_person_id: bytes::<16>(row, "second_person_id")?,
            first_source: PublicPersonSourceIdentityV1 {
                integration_public_id: bytes::<16>(row, "first_integration_public_id")?,
                account_public_id: bytes::<16>(row, "first_account_public_id")?,
                provider_source_contact_public_id: bytes::<16>(row, "first_source_public_id")?,
            },
            second_source: PublicPersonSourceIdentityV1 {
                integration_public_id: bytes::<16>(row, "second_integration_public_id")?,
                account_public_id: bytes::<16>(row, "second_account_public_id")?,
                provider_source_contact_public_id: bytes::<16>(row, "second_source_public_id")?,
            },
            match_kind: match row.try_get::<i16, _>("match_kind").map_err(|_| invalid())? {
                1 => PersonMatchKindV1::NormalizedEmail,
                2 => PersonMatchKindV1::NormalizedPhone,
                _ => return Err(ReviewPersonMatchCandidatePersistenceErrorV1::InvalidRow),
            },
            observed_at_unix_millis: row
                .try_get("observed_at_unix_millis")
                .map_err(|_| invalid())?,
            resulting_owner_revision: unsigned(row, "resulting_owner_revision")?,
            candidate_digest: bytes::<32>(row, "candidate_digest")?,
        },
        state,
        promotion_status,
        review_revision: unsigned(row, "review_revision")?,
        decision_id: optional_bytes::<16>(row, "decision_id")?,
        decided_by_owner_device_id: optional_bytes::<16>(row, "decided_by_owner_device_id")?,
        decided_at_unix_millis: row
            .try_get("decided_at_unix_millis")
            .map_err(|_| invalid())?,
        approved_action,
        approved_action_digest: optional_bytes::<32>(row, "approved_action_digest")?,
        updated_at_unix_millis: row
            .try_get("updated_at_unix_millis")
            .map_err(|_| invalid())?,
    };
    validate_review(&review).map_err(core_error)?;
    Ok(review)
}

struct ReviewValues {
    state: i16,
    promotion: i16,
    decision_id: Option<Vec<u8>>,
    device_id: Option<Vec<u8>>,
    decided_at: Option<i64>,
    action_kind: Option<i16>,
    action_bytes: Option<Vec<u8>>,
    action_digest: Option<Vec<u8>>,
}

fn review_values(
    review: &PersonMatchCandidateReviewV1,
) -> Result<ReviewValues, ReviewPersonMatchCandidatePersistenceErrorV1> {
    validate_review(review).map_err(core_error)?;
    let (action_kind, action_bytes) = review
        .approved_action
        .as_ref()
        .map(encode_action)
        .transpose()?
        .map_or((None, None), |(kind, bytes)| (Some(kind), Some(bytes)));
    Ok(ReviewValues {
        state: match review.state {
            PersonMatchCandidateStateV1::Pending => 1,
            PersonMatchCandidateStateV1::Approved => 2,
            PersonMatchCandidateStateV1::Rejected => 3,
        },
        promotion: match review.promotion_status {
            PersonMatchCandidatePromotionStatusV1::NotRequested => 1,
            PersonMatchCandidatePromotionStatusV1::Pending => 2,
            PersonMatchCandidatePromotionStatusV1::Succeeded => 3,
            PersonMatchCandidatePromotionStatusV1::Failed => 4,
        },
        decision_id: review.decision_id.map(|value| value.to_vec()),
        device_id: review
            .decided_by_owner_device_id
            .map(|value| value.to_vec()),
        decided_at: review.decided_at_unix_millis,
        action_kind,
        action_bytes,
        action_digest: review.approved_action_digest.map(|value| value.to_vec()),
    })
}

fn encode_action(
    action: &PersonMatchCandidateApprovedActionV1,
) -> Result<(i16, Vec<u8>), ReviewPersonMatchCandidatePersistenceErrorV1> {
    let mut out = Vec::with_capacity(512);
    match action {
        PersonMatchCandidateApprovedActionV1::Attach {
            from_person_id,
            expected_from_person_revision,
            to_person_id,
            expected_to_person_revision,
            source,
            expected_source_revision,
        } => {
            out.extend_from_slice(from_person_id);
            put_u64(&mut out, *expected_from_person_revision);
            out.extend_from_slice(to_person_id);
            put_u64(&mut out, *expected_to_person_revision);
            put_source(&mut out, *source);
            put_u64(&mut out, *expected_source_revision);
            Ok((1, out))
        }
        PersonMatchCandidateApprovedActionV1::Merge {
            source_person_id,
            expected_source_person_revision,
            target_person_id,
            expected_target_person_revision,
        } => {
            out.extend_from_slice(source_person_id);
            put_u64(&mut out, *expected_source_person_revision);
            out.extend_from_slice(target_person_id);
            put_u64(&mut out, *expected_target_person_revision);
            Ok((2, out))
        }
        PersonMatchCandidateApprovedActionV1::Split {
            merged_person_id,
            expected_merged_person_revision,
            target_person_id,
            expected_target_person_revision,
            source_selection,
            profile_fact_selection,
        } => {
            out.extend_from_slice(merged_person_id);
            put_u64(&mut out, *expected_merged_person_revision);
            out.extend_from_slice(target_person_id);
            put_u64(&mut out, *expected_target_person_revision);
            put_u16(
                &mut out,
                u16::try_from(source_selection.len()).map_err(|_| invalid())?,
            );
            for selected in source_selection {
                put_source(&mut out, selected.source);
                put_u64(&mut out, selected.expected_source_revision);
            }
            put_u16(
                &mut out,
                u16::try_from(profile_fact_selection.len()).map_err(|_| invalid())?,
            );
            for fact in profile_fact_selection {
                out.push(match fact {
                    SplitProfileFactKindV1::DisplayName => 1,
                    SplitProfileFactKindV1::GivenName => 2,
                    SplitProfileFactKindV1::FamilyName => 3,
                    SplitProfileFactKindV1::Emails => 4,
                    SplitProfileFactKindV1::Phones => 5,
                });
            }
            Ok((3, out))
        }
    }
}

fn decode_action(
    kind: i16,
    bytes: &[u8],
) -> Result<PersonMatchCandidateApprovedActionV1, ReviewPersonMatchCandidatePersistenceErrorV1> {
    let mut cursor = Cursor::new(bytes);
    let action = match kind {
        1 => PersonMatchCandidateApprovedActionV1::Attach {
            from_person_id: cursor.id()?,
            expected_from_person_revision: cursor.u64()?,
            to_person_id: cursor.id()?,
            expected_to_person_revision: cursor.u64()?,
            source: cursor.source()?,
            expected_source_revision: cursor.u64()?,
        },
        2 => PersonMatchCandidateApprovedActionV1::Merge {
            source_person_id: cursor.id()?,
            expected_source_person_revision: cursor.u64()?,
            target_person_id: cursor.id()?,
            expected_target_person_revision: cursor.u64()?,
        },
        3 => {
            let merged_person_id = cursor.id()?;
            let expected_merged_person_revision = cursor.u64()?;
            let target_person_id = cursor.id()?;
            let expected_target_person_revision = cursor.u64()?;
            let source_count = usize::from(cursor.u16()?);
            let mut source_selection = Vec::with_capacity(source_count);
            for _ in 0..source_count {
                source_selection.push(SplitSourceSelectionV1 {
                    source: cursor.source()?,
                    expected_source_revision: cursor.u64()?,
                });
            }
            let fact_count = usize::from(cursor.u16()?);
            let mut profile_fact_selection = Vec::with_capacity(fact_count);
            for _ in 0..fact_count {
                profile_fact_selection.push(match cursor.byte()? {
                    1 => SplitProfileFactKindV1::DisplayName,
                    2 => SplitProfileFactKindV1::GivenName,
                    3 => SplitProfileFactKindV1::FamilyName,
                    4 => SplitProfileFactKindV1::Emails,
                    5 => SplitProfileFactKindV1::Phones,
                    _ => return Err(ReviewPersonMatchCandidatePersistenceErrorV1::InvalidRow),
                });
            }
            PersonMatchCandidateApprovedActionV1::Split {
                merged_person_id,
                expected_merged_person_revision,
                target_person_id,
                expected_target_person_revision,
                source_selection,
                profile_fact_selection,
            }
        }
        _ => return Err(ReviewPersonMatchCandidatePersistenceErrorV1::InvalidRow),
    };
    if cursor.position != bytes.len() {
        return Err(ReviewPersonMatchCandidatePersistenceErrorV1::InvalidRow);
    }
    Ok(action)
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }
    fn take<const N: usize>(
        &mut self,
    ) -> Result<[u8; N], ReviewPersonMatchCandidatePersistenceErrorV1> {
        let end = self.position.checked_add(N).ok_or_else(invalid)?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or_else(invalid)?
            .try_into()
            .map_err(|_| invalid())?;
        self.position = end;
        Ok(value)
    }
    fn id(&mut self) -> Result<[u8; 16], ReviewPersonMatchCandidatePersistenceErrorV1> {
        self.take()
    }
    fn u64(&mut self) -> Result<u64, ReviewPersonMatchCandidatePersistenceErrorV1> {
        Ok(u64::from_be_bytes(self.take()?))
    }
    fn u16(&mut self) -> Result<u16, ReviewPersonMatchCandidatePersistenceErrorV1> {
        Ok(u16::from_be_bytes(self.take()?))
    }
    fn byte(&mut self) -> Result<u8, ReviewPersonMatchCandidatePersistenceErrorV1> {
        Ok(self.take::<1>()?[0])
    }
    fn source(
        &mut self,
    ) -> Result<PublicPersonSourceIdentityV1, ReviewPersonMatchCandidatePersistenceErrorV1> {
        Ok(PublicPersonSourceIdentityV1 {
            integration_public_id: self.id()?,
            account_public_id: self.id()?,
            provider_source_contact_public_id: self.id()?,
        })
    }
}

fn put_source(out: &mut Vec<u8>, source: PublicPersonSourceIdentityV1) {
    out.extend_from_slice(&source.integration_public_id);
    out.extend_from_slice(&source.account_public_id);
    out.extend_from_slice(&source.provider_source_contact_public_id);
}
fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_be_bytes());
}
fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn submission_fingerprint(input: &SubmitPersonMatchCandidateOperationV1) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.review.person-match-candidate.submit.v1");
    hash.update(input.evidence.candidate_digest);
    hash.update(input.submitted_result.envelope_sha256);
    hash.finalize().into()
}

fn decision_fingerprint(
    input: &DecidePersonMatchCandidateOperationV1,
) -> Result<[u8; 32], ReviewPersonMatchCandidatePersistenceErrorV1> {
    let mut hash = Sha256::new();
    hash.update(b"makosh.review.person-match-candidate.decide.v1");
    hash.update(input.review_id);
    hash.update(input.expected_review_revision.to_be_bytes());
    hash.update(input.decided_by_owner_device_id);
    hash.update(input.decided_at_unix_millis.to_be_bytes());
    match &input.decision {
        PersonMatchCandidateDecisionV1::Approve {
            action,
            approved_action_digest,
        } => {
            hash.update([1]);
            let (kind, bytes) = encode_action(action)?;
            hash.update(kind.to_be_bytes());
            hash.update((bytes.len() as u64).to_be_bytes());
            hash.update(bytes);
            hash.update(approved_action_digest);
        }
        PersonMatchCandidateDecisionV1::Reject => hash.update([2]),
    }
    Ok(hash.finalize().into())
}

fn promotion_fingerprint(input: &PersistPersonMatchCandidatePromotionResultV1) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.review.person-match-candidate.promotion-result.v1");
    hash.update(input.review_id);
    hash.update(input.candidate_id);
    hash.update(input.decision_id);
    match input.persons_command_id {
        Some(persons_command_id) => {
            hash.update([1]);
            hash.update(persons_command_id);
        }
        None => hash.update([0]),
    }
    hash.update(input.expected_review_revision.to_be_bytes());
    hash.update([u8::from(input.succeeded)]);
    hash.finalize().into()
}

fn outbox_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<ReviewPersonMatchCandidateOutboxRecordV1, ReviewPersonMatchCandidatePersistenceErrorV1>
{
    let record = ReviewPersonMatchCandidateEnvelopeRecordV1 {
        message_id: bytes::<16>(&row, "message_id")?,
        envelope_sha256: bytes::<32>(&row, "envelope_sha256")?,
        envelope_bytes: row.try_get("envelope_bytes").map_err(|_| invalid())?,
    };
    record
        .validate()
        .map_err(|_| ReviewPersonMatchCandidatePersistenceErrorV1::InvalidRow)?;
    Ok(ReviewPersonMatchCandidateOutboxRecordV1 {
        record,
        review_id: bytes::<16>(&row, "review_id")?,
        review_revision: unsigned(&row, "review_revision")?,
        semantic_kind: u8::try_from(
            row.try_get::<i16, _>("semantic_kind")
                .map_err(|_| invalid())?,
        )
        .map_err(|_| invalid())?,
        created_at_unix_millis: row
            .try_get("created_at_unix_millis")
            .map_err(|_| invalid())?,
        published_at_unix_millis: row
            .try_get("published_at_unix_millis")
            .map_err(|_| invalid())?,
    })
}

async fn set_owner(
    tx: &mut Transaction<'_, Postgres>,
    owner: &str,
) -> Result<(), ReviewPersonMatchCandidatePersistenceErrorV1> {
    sqlx::query("SELECT set_config('makosh.logical_owner_id',$1,true)")
        .bind(owner)
        .execute(&mut **tx)
        .await
        .map_err(|_| storage())?;
    Ok(())
}

fn match_kind(value: PersonMatchKindV1) -> i16 {
    match value {
        PersonMatchKindV1::NormalizedEmail => 1,
        PersonMatchKindV1::NormalizedPhone => 2,
    }
}

fn validate_owner(owner: &str) -> Result<(), ReviewPersonMatchCandidatePersistenceErrorV1> {
    if !owner.is_empty()
        && owner.len() <= 128
        && owner.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        Ok(())
    } else {
        Err(ReviewPersonMatchCandidatePersistenceErrorV1::InvalidInput)
    }
}

fn core_error(
    value: PersonMatchCandidateCoreErrorV1,
) -> ReviewPersonMatchCandidatePersistenceErrorV1 {
    match value {
        PersonMatchCandidateCoreErrorV1::RevisionConflict => {
            ReviewPersonMatchCandidatePersistenceErrorV1::RevisionConflict
        }
        PersonMatchCandidateCoreErrorV1::TerminalDecision => {
            ReviewPersonMatchCandidatePersistenceErrorV1::Conflict
        }
        _ => ReviewPersonMatchCandidatePersistenceErrorV1::InvalidInput,
    }
}

fn optional_bytes<const N: usize>(
    row: &sqlx::postgres::PgRow,
    name: &str,
) -> Result<Option<[u8; N]>, ReviewPersonMatchCandidatePersistenceErrorV1> {
    row.try_get::<Option<Vec<u8>>, _>(name)
        .map_err(|_| invalid())?
        .map(|value| value.try_into().map_err(|_| invalid()))
        .transpose()
}

fn bytes<const N: usize>(
    row: &sqlx::postgres::PgRow,
    name: &str,
) -> Result<[u8; N], ReviewPersonMatchCandidatePersistenceErrorV1> {
    row.try_get::<Vec<u8>, _>(name)
        .map_err(|_| invalid())?
        .try_into()
        .map_err(|_| invalid())
}

fn unsigned(
    row: &sqlx::postgres::PgRow,
    name: &str,
) -> Result<u64, ReviewPersonMatchCandidatePersistenceErrorV1> {
    u64::try_from(row.try_get::<i64, _>(name).map_err(|_| invalid())?).map_err(|_| invalid())
}

fn nonzero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().any(|byte| *byte != 0)
}
const fn invalid() -> ReviewPersonMatchCandidatePersistenceErrorV1 {
    ReviewPersonMatchCandidatePersistenceErrorV1::InvalidInput
}
const fn storage() -> ReviewPersonMatchCandidatePersistenceErrorV1 {
    ReviewPersonMatchCandidatePersistenceErrorV1::StorageUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(seed: u8) -> PublicPersonSourceIdentityV1 {
        PublicPersonSourceIdentityV1 {
            integration_public_id: [seed; 16],
            account_public_id: [seed + 1; 16],
            provider_source_contact_public_id: [seed + 2; 16],
        }
    }

    #[test]
    fn action_codec_round_trips_attach_merge_and_selective_split() {
        let actions = [
            PersonMatchCandidateApprovedActionV1::Attach {
                from_person_id: [1; 16],
                expected_from_person_revision: 2,
                to_person_id: [3; 16],
                expected_to_person_revision: 4,
                source: source(5),
                expected_source_revision: 6,
            },
            PersonMatchCandidateApprovedActionV1::Merge {
                source_person_id: [7; 16],
                expected_source_person_revision: 8,
                target_person_id: [9; 16],
                expected_target_person_revision: 10,
            },
            PersonMatchCandidateApprovedActionV1::Split {
                merged_person_id: [11; 16],
                expected_merged_person_revision: 12,
                target_person_id: [13; 16],
                expected_target_person_revision: 14,
                source_selection: vec![SplitSourceSelectionV1 {
                    source: source(15),
                    expected_source_revision: 16,
                }],
                profile_fact_selection: vec![SplitProfileFactKindV1::Emails],
            },
        ];
        for action in actions {
            let (kind, bytes) = encode_action(&action).expect("encode");
            assert_eq!(decode_action(kind, &bytes).expect("decode"), action);
        }
    }

    #[test]
    fn envelope_record_rejects_hash_mismatch_and_private_sized_payloads() {
        let bytes = vec![1, 2, 3];
        let mut record = ReviewPersonMatchCandidateEnvelopeRecordV1 {
            message_id: [1; 16],
            envelope_sha256: Sha256::digest(&bytes).into(),
            envelope_bytes: bytes,
        };
        assert_eq!(record.validate(), Ok(()));
        record.envelope_bytes.push(4);
        assert_eq!(
            record.validate(),
            Err(ReviewPersonMatchCandidatePersistenceErrorV1::InvalidInput)
        );
    }
}
