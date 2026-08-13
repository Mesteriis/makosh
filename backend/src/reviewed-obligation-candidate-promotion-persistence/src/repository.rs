use makosh_reviewed_obligation_candidate_promotion_core::{
    derive_reviewed_obligation_candidate_command_id_v1,
    derive_reviewed_obligation_candidate_result_id_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    PersistPromotionApprovalOutcomeV1, PersistPromotionApprovalV1, PersistPromotionResultOutcomeV1,
    PersistPromotionTerminalResultV1, PromotionCorrelationV1,
    ReviewedObligationCandidatePromotionOutcomeV1,
    ReviewedObligationCandidatePromotionPersistenceErrorV1,
    model::{nonzero, valid_outbox, valid_owner, valid_timestamp},
    outbox::{insert_exact_outbox, verify_exact_outbox},
};

#[derive(Clone)]
pub struct ReviewedObligationCandidatePromotionPersistenceV1 {
    pub(crate) pool: PgPool,
}

impl ReviewedObligationCandidatePromotionPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, ReviewedObligationCandidatePromotionPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(ReviewedObligationCandidatePromotionPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port).map_err(|_| {
            ReviewedObligationCandidatePromotionPersistenceErrorV1::StorageUnavailable
        })?;
        let options = PgConnectOptions::new()
            .host(pgbouncer_host)
            .port(port)
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

    pub async fn verify_storage_ready(
        &self,
    ) -> Result<(), ReviewedObligationCandidatePromotionPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(storage_error)
    }

    pub async fn load_correlation(
        &self,
        logical_owner_id: &str,
        obligations_id: &[u8; 16],
    ) -> Result<PromotionCorrelationV1, ReviewedObligationCandidatePromotionPersistenceErrorV1>
    {
        if !valid_owner(logical_owner_id) || !nonzero(obligations_id) {
            return Err(ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let row = sqlx::query(
            "SELECT review_id, candidate_id, decision_revision, obligations_result_message_id
             FROM makosh_data.reviewed_obligation_candidate_promotion_requests
             WHERE logical_owner_id = $1 AND obligations_id = $2",
        )
        .bind(logical_owner_id)
        .bind(obligations_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_error)?
        .ok_or(ReviewedObligationCandidatePromotionPersistenceErrorV1::NotFound)?;
        let result = PromotionCorrelationV1 {
            review_id: fixed(row_value(&row, "review_id")?)?,
            candidate_id: fixed(row_value(&row, "candidate_id")?)?,
            decision_revision: unsigned_revision(row_value(&row, "decision_revision")?)?,
            completed: row_value::<Option<Vec<u8>>>(&row, "obligations_result_message_id")?
                .is_some(),
        };
        transaction.commit().await.map_err(storage_error)?;
        Ok(result)
    }

    pub async fn persist_approval_and_obligations(
        &self,
        input: &PersistPromotionApprovalV1,
    ) -> Result<
        PersistPromotionApprovalOutcomeV1,
        ReviewedObligationCandidatePromotionPersistenceErrorV1,
    > {
        validate_approval(input)?;
        let decision_revision = signed_revision(input.decision_revision)?;
        let mut transaction = self
            .begin_owner_transaction(&input.logical_owner_id)
            .await?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.reviewed_obligation_candidate_promotion_requests (
               logical_owner_id, approval_message_id, approval_envelope_sha256,
               review_id, candidate_id, decision_revision,
               obligations_id, obligations_message_id,
               created_at_unix_millis, updated_at_unix_millis
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $7, $8, $8)
             ON CONFLICT DO NOTHING",
        )
        .bind(&input.logical_owner_id)
        .bind(input.approval_message_id.as_slice())
        .bind(input.approval_envelope_sha256.as_slice())
        .bind(input.review_id.as_slice())
        .bind(input.candidate_id.as_slice())
        .bind(decision_revision)
        .bind(input.obligations_id.as_slice())
        .bind(input.occurred_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if inserted == 1 {
            insert_exact_outbox(
                &mut transaction,
                &input.logical_owner_id,
                &input.obligations_outbox,
                input.occurred_at_unix_millis,
            )
            .await?;
            transaction.commit().await.map_err(storage_error)?;
            return Ok(PersistPromotionApprovalOutcomeV1::Applied);
        }
        let existing = load_request_by_approval(
            &mut transaction,
            &input.logical_owner_id,
            &input.approval_message_id,
        )
        .await?
        .ok_or(ReviewedObligationCandidatePromotionPersistenceErrorV1::ApprovalConflict)?;
        if !existing.matches_approval(input)? {
            return Err(ReviewedObligationCandidatePromotionPersistenceErrorV1::ApprovalConflict);
        }
        verify_exact_outbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.obligations_outbox,
        )
        .await?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(PersistPromotionApprovalOutcomeV1::Duplicate)
    }

    pub async fn persist_obligations_result_and_review_result(
        &self,
        input: &PersistPromotionTerminalResultV1,
    ) -> Result<
        PersistPromotionResultOutcomeV1,
        ReviewedObligationCandidatePromotionPersistenceErrorV1,
    > {
        validate_terminal_result(input)?;
        let mut transaction = self
            .begin_owner_transaction(&input.logical_owner_id)
            .await?;
        let request = load_request_by_command_for_update(
            &mut transaction,
            &input.logical_owner_id,
            &input.obligations_id,
        )
        .await?
        .ok_or(ReviewedObligationCandidatePromotionPersistenceErrorV1::NotFound)?;
        if request.review_id != input.review_id || request.candidate_id != input.candidate_id {
            return Err(ReviewedObligationCandidatePromotionPersistenceErrorV1::ResultConflict);
        }
        if request.obligations_result_message_id.is_some() {
            if !request.matches_terminal_result(input)? {
                return Err(ReviewedObligationCandidatePromotionPersistenceErrorV1::ResultConflict);
            }
            verify_result_inbox(&mut transaction, input).await?;
            verify_exact_outbox(
                &mut transaction,
                &input.logical_owner_id,
                &input.review_result_outbox,
            )
            .await?;
            transaction.commit().await.map_err(storage_error)?;
            return Ok(PersistPromotionResultOutcomeV1::Duplicate);
        }
        let inbox_inserted = sqlx::query(
            "INSERT INTO makosh_data.reviewed_obligation_candidate_promotion_result_inbox (
               logical_owner_id, result_message_id, envelope_sha256,
               obligations_id, review_id, processed_at_unix_millis
             ) VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT DO NOTHING",
        )
        .bind(&input.logical_owner_id)
        .bind(input.obligations_result_message_id.as_slice())
        .bind(input.obligations_result_envelope_sha256.as_slice())
        .bind(input.obligations_id.as_slice())
        .bind(input.review_id.as_slice())
        .bind(input.occurred_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if inbox_inserted != 1 {
            return Err(ReviewedObligationCandidatePromotionPersistenceErrorV1::ResultConflict);
        }
        insert_exact_outbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.review_result_outbox,
            input.occurred_at_unix_millis,
        )
        .await?;
        let (outcome, obligation_id, failure_code) = encoded_outcome(input.outcome);
        let updated = sqlx::query(
            "UPDATE makosh_data.reviewed_obligation_candidate_promotion_requests
             SET obligations_result_message_id = $1, promotion_outcome = $2,
                 obligation_id = $3, failure_code = $4, updated_at_unix_millis = $5
             WHERE logical_owner_id = $6 AND obligations_id = $7
               AND obligations_result_message_id IS NULL",
        )
        .bind(input.obligations_result_message_id.as_slice())
        .bind(outcome)
        .bind(obligation_id.map(|value| value.to_vec()))
        .bind(failure_code)
        .bind(input.occurred_at_unix_millis)
        .bind(&input.logical_owner_id)
        .bind(input.obligations_id.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if updated != 1 {
            return Err(ReviewedObligationCandidatePromotionPersistenceErrorV1::ResultConflict);
        }
        transaction.commit().await.map_err(storage_error)?;
        Ok(PersistPromotionResultOutcomeV1::Applied)
    }

    pub(crate) async fn begin_owner_transaction(
        &self,
        logical_owner_id: &str,
    ) -> Result<Transaction<'_, Postgres>, ReviewedObligationCandidatePromotionPersistenceErrorV1>
    {
        if !valid_owner(logical_owner_id) {
            return Err(ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidInput);
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

#[derive(Debug)]
struct PromotionRequestRowV1 {
    approval_envelope_sha256: [u8; 32],
    review_id: [u8; 16],
    candidate_id: [u8; 16],
    decision_revision: u64,
    obligations_id: [u8; 16],
    obligations_message_id: [u8; 16],
    obligations_result_message_id: Option<[u8; 16]>,
    promotion_outcome: Option<i16>,
    obligation_id: Option<[u8; 16]>,
    failure_code: Option<u16>,
}

impl PromotionRequestRowV1 {
    fn matches_approval(
        &self,
        input: &PersistPromotionApprovalV1,
    ) -> Result<bool, ReviewedObligationCandidatePromotionPersistenceErrorV1> {
        Ok(
            self.approval_envelope_sha256 == input.approval_envelope_sha256
                && self.review_id == input.review_id
                && self.candidate_id == input.candidate_id
                && self.decision_revision == input.decision_revision
                && self.obligations_id == input.obligations_id
                && self.obligations_message_id == *input.obligations_outbox.message_id(),
        )
    }

    fn matches_terminal_result(
        &self,
        input: &PersistPromotionTerminalResultV1,
    ) -> Result<bool, ReviewedObligationCandidatePromotionPersistenceErrorV1> {
        let (outcome, obligation_id, failure_code) = encoded_outcome(input.outcome);
        Ok(
            self.obligations_result_message_id == Some(input.obligations_result_message_id)
                && self.promotion_outcome == Some(outcome)
                && self.obligation_id == obligation_id
                && self.failure_code.map(i32::from) == failure_code,
        )
    }
}

async fn load_request_by_approval(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    approval_message_id: &[u8; 16],
) -> Result<Option<PromotionRequestRowV1>, ReviewedObligationCandidatePromotionPersistenceErrorV1> {
    load_request(
        sqlx::query(REQUEST_BY_APPROVAL)
            .bind(logical_owner_id)
            .bind(approval_message_id.as_slice())
            .fetch_optional(&mut **transaction)
            .await
            .map_err(storage_error)?,
    )
}

async fn load_request_by_command_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    obligations_id: &[u8; 16],
) -> Result<Option<PromotionRequestRowV1>, ReviewedObligationCandidatePromotionPersistenceErrorV1> {
    load_request(
        sqlx::query(REQUEST_BY_COMMAND_FOR_UPDATE)
            .bind(logical_owner_id)
            .bind(obligations_id.as_slice())
            .fetch_optional(&mut **transaction)
            .await
            .map_err(storage_error)?,
    )
}

fn load_request(
    row: Option<sqlx::postgres::PgRow>,
) -> Result<Option<PromotionRequestRowV1>, ReviewedObligationCandidatePromotionPersistenceErrorV1> {
    row.map(request_from_row).transpose()
}

fn request_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<PromotionRequestRowV1, ReviewedObligationCandidatePromotionPersistenceErrorV1> {
    Ok(PromotionRequestRowV1 {
        approval_envelope_sha256: fixed(row_value(&row, "approval_envelope_sha256")?)?,
        review_id: fixed(row_value(&row, "review_id")?)?,
        candidate_id: fixed(row_value(&row, "candidate_id")?)?,
        decision_revision: unsigned_revision(row_value(&row, "decision_revision")?)?,
        obligations_id: fixed(row_value(&row, "obligations_id")?)?,
        obligations_message_id: fixed(row_value(&row, "obligations_message_id")?)?,
        obligations_result_message_id: optional_fixed(row_value(
            &row,
            "obligations_result_message_id",
        )?)?,
        promotion_outcome: row_value(&row, "promotion_outcome")?,
        obligation_id: optional_fixed(row_value(&row, "obligation_id")?)?,
        failure_code: row_value::<Option<i32>>(&row, "failure_code")?
            .map(u16::try_from)
            .transpose()
            .map_err(|_| ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidRow)?,
    })
}

async fn verify_result_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    input: &PersistPromotionTerminalResultV1,
) -> Result<(), ReviewedObligationCandidatePromotionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT envelope_sha256, obligations_id, review_id
         FROM makosh_data.reviewed_obligation_candidate_promotion_result_inbox
         WHERE logical_owner_id = $1 AND result_message_id = $2",
    )
    .bind(&input.logical_owner_id)
    .bind(input.obligations_result_message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_error)?
    .ok_or(ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidRow)?;
    let hash: [u8; 32] = fixed(row_value(&row, "envelope_sha256")?)?;
    let command: [u8; 16] = fixed(row_value(&row, "obligations_id")?)?;
    let review: [u8; 16] = fixed(row_value(&row, "review_id")?)?;
    if hash != input.obligations_result_envelope_sha256
        || command != input.obligations_id
        || review != input.review_id
    {
        return Err(ReviewedObligationCandidatePromotionPersistenceErrorV1::ResultConflict);
    }
    Ok(())
}

fn validate_approval(
    input: &PersistPromotionApprovalV1,
) -> Result<(), ReviewedObligationCandidatePromotionPersistenceErrorV1> {
    let expected = derive_reviewed_obligation_candidate_command_id_v1(
        input.approval_message_id,
        input.review_id,
        input.candidate_id,
        input.decision_revision,
    )
    .map_err(|_| ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidInput)?;
    if !valid_owner(&input.logical_owner_id)
        || !nonzero(&input.approval_envelope_sha256)
        || input.obligations_id != expected
        || input.obligations_outbox.message_id() != &expected
        || !valid_outbox(&input.obligations_outbox)
        || !valid_timestamp(input.occurred_at_unix_millis)
    {
        return Err(ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_terminal_result(
    input: &PersistPromotionTerminalResultV1,
) -> Result<(), ReviewedObligationCandidatePromotionPersistenceErrorV1> {
    let expected_result_id = derive_reviewed_obligation_candidate_result_id_v1(
        input.obligations_result_message_id,
        input.obligations_id,
        input.review_id,
    )
    .map_err(|_| ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidInput)?;
    let outcome_valid = match input.outcome {
        ReviewedObligationCandidatePromotionOutcomeV1::Succeeded { obligation_id } => {
            nonzero(&obligation_id)
        }
        ReviewedObligationCandidatePromotionOutcomeV1::Failed { failure_code } => failure_code > 0,
    };
    if !valid_owner(&input.logical_owner_id)
        || !nonzero(&input.obligations_result_envelope_sha256)
        || !nonzero(&input.candidate_id)
        || !outcome_valid
        || input.review_result_outbox.message_id() != &expected_result_id
        || !valid_outbox(&input.review_result_outbox)
        || !valid_timestamp(input.occurred_at_unix_millis)
    {
        return Err(ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn encoded_outcome(
    outcome: ReviewedObligationCandidatePromotionOutcomeV1,
) -> (i16, Option<[u8; 16]>, Option<i32>) {
    match outcome {
        ReviewedObligationCandidatePromotionOutcomeV1::Succeeded { obligation_id } => {
            (1, Some(obligation_id), None)
        }
        ReviewedObligationCandidatePromotionOutcomeV1::Failed { failure_code } => {
            (2, None, Some(i32::from(failure_code)))
        }
    }
}

fn fixed<const N: usize>(
    value: Vec<u8>,
) -> Result<[u8; N], ReviewedObligationCandidatePromotionPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidRow)
}

fn optional_fixed<const N: usize>(
    value: Option<Vec<u8>>,
) -> Result<Option<[u8; N]>, ReviewedObligationCandidatePromotionPersistenceErrorV1> {
    value.map(fixed).transpose()
}

fn row_value<T>(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<T, ReviewedObligationCandidatePromotionPersistenceErrorV1>
where
    for<'r> T: sqlx::Decode<'r, Postgres> + sqlx::Type<Postgres>,
{
    row.try_get(column)
        .map_err(|_| ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidRow)
}

fn signed_revision(
    value: u64,
) -> Result<i64, ReviewedObligationCandidatePromotionPersistenceErrorV1> {
    i64::try_from(value)
        .map_err(|_| ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidInput)
}

fn unsigned_revision(
    value: i64,
) -> Result<u64, ReviewedObligationCandidatePromotionPersistenceErrorV1> {
    u64::try_from(value)
        .map_err(|_| ReviewedObligationCandidatePromotionPersistenceErrorV1::InvalidRow)
}

fn storage_error(_: sqlx::Error) -> ReviewedObligationCandidatePromotionPersistenceErrorV1 {
    ReviewedObligationCandidatePromotionPersistenceErrorV1::StorageUnavailable
}

const REQUEST_BY_APPROVAL: &str = "SELECT approval_envelope_sha256, review_id, candidate_id, decision_revision, \
     obligations_id, obligations_message_id, obligations_result_message_id, \
     promotion_outcome, obligation_id, failure_code \
     FROM makosh_data.reviewed_obligation_candidate_promotion_requests \
     WHERE logical_owner_id = $1 AND approval_message_id = $2";
const REQUEST_BY_COMMAND_FOR_UPDATE: &str = "SELECT approval_envelope_sha256, review_id, candidate_id, decision_revision, \
     obligations_id, obligations_message_id, obligations_result_message_id, \
     promotion_outcome, obligation_id, failure_code \
     FROM makosh_data.reviewed_obligation_candidate_promotion_requests \
     WHERE logical_owner_id = $1 AND obligations_id = $2 FOR UPDATE";
