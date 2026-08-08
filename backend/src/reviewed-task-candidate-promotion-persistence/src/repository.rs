use makosh_reviewed_task_candidate_promotion_core::{
    derive_reviewed_task_candidate_command_id_v1, derive_reviewed_task_candidate_result_id_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    PersistPromotionApprovalOutcomeV1, PersistPromotionApprovalV1, PersistPromotionResultOutcomeV1,
    PersistPromotionTerminalResultV1, PromotionCorrelationV1,
    ReviewedTaskCandidatePromotionOutcomeV1, ReviewedTaskCandidatePromotionPersistenceErrorV1,
    model::{nonzero, valid_outbox, valid_owner, valid_timestamp},
    outbox::{insert_exact_outbox, verify_exact_outbox},
};

#[derive(Clone)]
pub struct ReviewedTaskCandidatePromotionPersistenceV1 {
    pub(crate) pool: PgPool,
}

impl ReviewedTaskCandidatePromotionPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, ReviewedTaskCandidatePromotionPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(ReviewedTaskCandidatePromotionPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| ReviewedTaskCandidatePromotionPersistenceErrorV1::StorageUnavailable)?;
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
    ) -> Result<(), ReviewedTaskCandidatePromotionPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(storage_error)
    }

    pub async fn load_correlation(
        &self,
        logical_owner_id: &str,
        tasks_command_id: &[u8; 16],
    ) -> Result<PromotionCorrelationV1, ReviewedTaskCandidatePromotionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || !nonzero(tasks_command_id) {
            return Err(ReviewedTaskCandidatePromotionPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT review_id, candidate_id, decision_revision, tasks_result_message_id
             FROM makosh_data.reviewed_task_candidate_promotion_requests
             WHERE logical_owner_id = $1 AND tasks_command_id = $2",
        )
        .bind(logical_owner_id)
        .bind(tasks_command_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .ok_or(ReviewedTaskCandidatePromotionPersistenceErrorV1::NotFound)?;
        Ok(PromotionCorrelationV1 {
            review_id: fixed(row_value(&row, "review_id")?)?,
            candidate_id: fixed(row_value(&row, "candidate_id")?)?,
            decision_revision: unsigned_revision(row_value(&row, "decision_revision")?)?,
            completed: row_value::<Option<Vec<u8>>>(&row, "tasks_result_message_id")?.is_some(),
        })
    }

    pub async fn persist_approval_and_tasks_command(
        &self,
        input: &PersistPromotionApprovalV1,
    ) -> Result<PersistPromotionApprovalOutcomeV1, ReviewedTaskCandidatePromotionPersistenceErrorV1>
    {
        validate_approval(input)?;
        let decision_revision = signed_revision(input.decision_revision)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.reviewed_task_candidate_promotion_requests (
               logical_owner_id, approval_message_id, approval_envelope_sha256,
               review_id, candidate_id, decision_revision,
               tasks_command_id, tasks_command_message_id,
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
        .bind(input.tasks_command_id.as_slice())
        .bind(input.occurred_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if inserted == 1 {
            insert_exact_outbox(
                &mut transaction,
                &input.logical_owner_id,
                &input.tasks_command_outbox,
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
        .ok_or(ReviewedTaskCandidatePromotionPersistenceErrorV1::ApprovalConflict)?;
        if !existing.matches_approval(input)? {
            return Err(ReviewedTaskCandidatePromotionPersistenceErrorV1::ApprovalConflict);
        }
        verify_exact_outbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.tasks_command_outbox,
        )
        .await?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(PersistPromotionApprovalOutcomeV1::Duplicate)
    }

    pub async fn persist_tasks_result_and_review_result(
        &self,
        input: &PersistPromotionTerminalResultV1,
    ) -> Result<PersistPromotionResultOutcomeV1, ReviewedTaskCandidatePromotionPersistenceErrorV1>
    {
        validate_terminal_result(input)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let request = load_request_by_command_for_update(
            &mut transaction,
            &input.logical_owner_id,
            &input.tasks_command_id,
        )
        .await?
        .ok_or(ReviewedTaskCandidatePromotionPersistenceErrorV1::NotFound)?;
        if request.review_id != input.review_id || request.candidate_id != input.candidate_id {
            return Err(ReviewedTaskCandidatePromotionPersistenceErrorV1::ResultConflict);
        }
        if request.tasks_result_message_id.is_some() {
            if !request.matches_terminal_result(input)? {
                return Err(ReviewedTaskCandidatePromotionPersistenceErrorV1::ResultConflict);
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
            "INSERT INTO makosh_data.reviewed_task_candidate_promotion_result_inbox (
               logical_owner_id, result_message_id, envelope_sha256,
               tasks_command_id, review_id, processed_at_unix_millis
             ) VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT DO NOTHING",
        )
        .bind(&input.logical_owner_id)
        .bind(input.tasks_result_message_id.as_slice())
        .bind(input.tasks_result_envelope_sha256.as_slice())
        .bind(input.tasks_command_id.as_slice())
        .bind(input.review_id.as_slice())
        .bind(input.occurred_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if inbox_inserted != 1 {
            return Err(ReviewedTaskCandidatePromotionPersistenceErrorV1::ResultConflict);
        }
        insert_exact_outbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.review_result_outbox,
            input.occurred_at_unix_millis,
        )
        .await?;
        let (outcome, task_id, failure_code) = encoded_outcome(input.outcome);
        let updated = sqlx::query(
            "UPDATE makosh_data.reviewed_task_candidate_promotion_requests
             SET tasks_result_message_id = $1, promotion_outcome = $2,
                 task_id = $3, failure_code = $4, updated_at_unix_millis = $5
             WHERE logical_owner_id = $6 AND tasks_command_id = $7
               AND tasks_result_message_id IS NULL",
        )
        .bind(input.tasks_result_message_id.as_slice())
        .bind(outcome)
        .bind(task_id.map(|value| value.to_vec()))
        .bind(failure_code)
        .bind(input.occurred_at_unix_millis)
        .bind(&input.logical_owner_id)
        .bind(input.tasks_command_id.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if updated != 1 {
            return Err(ReviewedTaskCandidatePromotionPersistenceErrorV1::ResultConflict);
        }
        transaction.commit().await.map_err(storage_error)?;
        Ok(PersistPromotionResultOutcomeV1::Applied)
    }
}

#[derive(Debug)]
struct PromotionRequestRowV1 {
    approval_envelope_sha256: [u8; 32],
    review_id: [u8; 16],
    candidate_id: [u8; 16],
    decision_revision: u64,
    tasks_command_id: [u8; 16],
    tasks_command_message_id: [u8; 16],
    tasks_result_message_id: Option<[u8; 16]>,
    promotion_outcome: Option<i16>,
    task_id: Option<[u8; 16]>,
    failure_code: Option<u16>,
}

impl PromotionRequestRowV1 {
    fn matches_approval(
        &self,
        input: &PersistPromotionApprovalV1,
    ) -> Result<bool, ReviewedTaskCandidatePromotionPersistenceErrorV1> {
        Ok(
            self.approval_envelope_sha256 == input.approval_envelope_sha256
                && self.review_id == input.review_id
                && self.candidate_id == input.candidate_id
                && self.decision_revision == input.decision_revision
                && self.tasks_command_id == input.tasks_command_id
                && self.tasks_command_message_id == *input.tasks_command_outbox.message_id(),
        )
    }

    fn matches_terminal_result(
        &self,
        input: &PersistPromotionTerminalResultV1,
    ) -> Result<bool, ReviewedTaskCandidatePromotionPersistenceErrorV1> {
        let (outcome, task_id, failure_code) = encoded_outcome(input.outcome);
        Ok(
            self.tasks_result_message_id == Some(input.tasks_result_message_id)
                && self.promotion_outcome == Some(outcome)
                && self.task_id == task_id
                && self.failure_code.map(i32::from) == failure_code,
        )
    }
}

async fn load_request_by_approval(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    approval_message_id: &[u8; 16],
) -> Result<Option<PromotionRequestRowV1>, ReviewedTaskCandidatePromotionPersistenceErrorV1> {
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
    tasks_command_id: &[u8; 16],
) -> Result<Option<PromotionRequestRowV1>, ReviewedTaskCandidatePromotionPersistenceErrorV1> {
    load_request(
        sqlx::query(REQUEST_BY_COMMAND_FOR_UPDATE)
            .bind(logical_owner_id)
            .bind(tasks_command_id.as_slice())
            .fetch_optional(&mut **transaction)
            .await
            .map_err(storage_error)?,
    )
}

fn load_request(
    row: Option<sqlx::postgres::PgRow>,
) -> Result<Option<PromotionRequestRowV1>, ReviewedTaskCandidatePromotionPersistenceErrorV1> {
    row.map(request_from_row).transpose()
}

fn request_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<PromotionRequestRowV1, ReviewedTaskCandidatePromotionPersistenceErrorV1> {
    Ok(PromotionRequestRowV1 {
        approval_envelope_sha256: fixed(row_value(&row, "approval_envelope_sha256")?)?,
        review_id: fixed(row_value(&row, "review_id")?)?,
        candidate_id: fixed(row_value(&row, "candidate_id")?)?,
        decision_revision: unsigned_revision(row_value(&row, "decision_revision")?)?,
        tasks_command_id: fixed(row_value(&row, "tasks_command_id")?)?,
        tasks_command_message_id: fixed(row_value(&row, "tasks_command_message_id")?)?,
        tasks_result_message_id: optional_fixed(row_value(&row, "tasks_result_message_id")?)?,
        promotion_outcome: row_value(&row, "promotion_outcome")?,
        task_id: optional_fixed(row_value(&row, "task_id")?)?,
        failure_code: row_value::<Option<i32>>(&row, "failure_code")?
            .map(u16::try_from)
            .transpose()
            .map_err(|_| ReviewedTaskCandidatePromotionPersistenceErrorV1::InvalidRow)?,
    })
}

async fn verify_result_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    input: &PersistPromotionTerminalResultV1,
) -> Result<(), ReviewedTaskCandidatePromotionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT envelope_sha256, tasks_command_id, review_id
         FROM makosh_data.reviewed_task_candidate_promotion_result_inbox
         WHERE logical_owner_id = $1 AND result_message_id = $2",
    )
    .bind(&input.logical_owner_id)
    .bind(input.tasks_result_message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_error)?
    .ok_or(ReviewedTaskCandidatePromotionPersistenceErrorV1::InvalidRow)?;
    let hash: [u8; 32] = fixed(row_value(&row, "envelope_sha256")?)?;
    let command: [u8; 16] = fixed(row_value(&row, "tasks_command_id")?)?;
    let review: [u8; 16] = fixed(row_value(&row, "review_id")?)?;
    if hash != input.tasks_result_envelope_sha256
        || command != input.tasks_command_id
        || review != input.review_id
    {
        return Err(ReviewedTaskCandidatePromotionPersistenceErrorV1::ResultConflict);
    }
    Ok(())
}

fn validate_approval(
    input: &PersistPromotionApprovalV1,
) -> Result<(), ReviewedTaskCandidatePromotionPersistenceErrorV1> {
    let expected = derive_reviewed_task_candidate_command_id_v1(
        input.approval_message_id,
        input.review_id,
        input.candidate_id,
        input.decision_revision,
    )
    .map_err(|_| ReviewedTaskCandidatePromotionPersistenceErrorV1::InvalidInput)?;
    if !valid_owner(&input.logical_owner_id)
        || !nonzero(&input.approval_envelope_sha256)
        || input.tasks_command_id != expected
        || input.tasks_command_outbox.message_id() != &expected
        || !valid_outbox(&input.tasks_command_outbox)
        || !valid_timestamp(input.occurred_at_unix_millis)
    {
        return Err(ReviewedTaskCandidatePromotionPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_terminal_result(
    input: &PersistPromotionTerminalResultV1,
) -> Result<(), ReviewedTaskCandidatePromotionPersistenceErrorV1> {
    let expected_result_id = derive_reviewed_task_candidate_result_id_v1(
        input.tasks_result_message_id,
        input.tasks_command_id,
        input.review_id,
    )
    .map_err(|_| ReviewedTaskCandidatePromotionPersistenceErrorV1::InvalidInput)?;
    let outcome_valid = match input.outcome {
        ReviewedTaskCandidatePromotionOutcomeV1::Succeeded { task_id } => nonzero(&task_id),
        ReviewedTaskCandidatePromotionOutcomeV1::Failed { failure_code } => failure_code > 0,
    };
    if !valid_owner(&input.logical_owner_id)
        || !nonzero(&input.tasks_result_envelope_sha256)
        || !nonzero(&input.candidate_id)
        || !outcome_valid
        || input.review_result_outbox.message_id() != &expected_result_id
        || !valid_outbox(&input.review_result_outbox)
        || !valid_timestamp(input.occurred_at_unix_millis)
    {
        return Err(ReviewedTaskCandidatePromotionPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn encoded_outcome(
    outcome: ReviewedTaskCandidatePromotionOutcomeV1,
) -> (i16, Option<[u8; 16]>, Option<i32>) {
    match outcome {
        ReviewedTaskCandidatePromotionOutcomeV1::Succeeded { task_id } => (1, Some(task_id), None),
        ReviewedTaskCandidatePromotionOutcomeV1::Failed { failure_code } => {
            (2, None, Some(i32::from(failure_code)))
        }
    }
}

fn fixed<const N: usize>(
    value: Vec<u8>,
) -> Result<[u8; N], ReviewedTaskCandidatePromotionPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| ReviewedTaskCandidatePromotionPersistenceErrorV1::InvalidRow)
}

fn optional_fixed<const N: usize>(
    value: Option<Vec<u8>>,
) -> Result<Option<[u8; N]>, ReviewedTaskCandidatePromotionPersistenceErrorV1> {
    value.map(fixed).transpose()
}

fn row_value<T>(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<T, ReviewedTaskCandidatePromotionPersistenceErrorV1>
where
    for<'r> T: sqlx::Decode<'r, Postgres> + sqlx::Type<Postgres>,
{
    row.try_get(column)
        .map_err(|_| ReviewedTaskCandidatePromotionPersistenceErrorV1::InvalidRow)
}

fn signed_revision(value: u64) -> Result<i64, ReviewedTaskCandidatePromotionPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| ReviewedTaskCandidatePromotionPersistenceErrorV1::InvalidInput)
}

fn unsigned_revision(value: i64) -> Result<u64, ReviewedTaskCandidatePromotionPersistenceErrorV1> {
    u64::try_from(value).map_err(|_| ReviewedTaskCandidatePromotionPersistenceErrorV1::InvalidRow)
}

fn storage_error(_: sqlx::Error) -> ReviewedTaskCandidatePromotionPersistenceErrorV1 {
    ReviewedTaskCandidatePromotionPersistenceErrorV1::StorageUnavailable
}

const REQUEST_BY_APPROVAL: &str = "SELECT approval_envelope_sha256, review_id, candidate_id, decision_revision, \
     tasks_command_id, tasks_command_message_id, tasks_result_message_id, \
     promotion_outcome, task_id, failure_code \
     FROM makosh_data.reviewed_task_candidate_promotion_requests \
     WHERE logical_owner_id = $1 AND approval_message_id = $2";
const REQUEST_BY_COMMAND_FOR_UPDATE: &str = "SELECT approval_envelope_sha256, review_id, candidate_id, decision_revision, \
     tasks_command_id, tasks_command_message_id, tasks_result_message_id, \
     promotion_outcome, task_id, failure_code \
     FROM makosh_data.reviewed_task_candidate_promotion_requests \
     WHERE logical_owner_id = $1 AND tasks_command_id = $2 FOR UPDATE";
