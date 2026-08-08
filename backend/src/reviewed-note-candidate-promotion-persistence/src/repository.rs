use makosh_reviewed_note_candidate_promotion_core::{
    derive_reviewed_note_candidate_command_id_v1, derive_reviewed_note_candidate_result_id_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{
    PersistPromotionApprovalOutcomeV1, PersistPromotionApprovalV1,
    PersistPromotionMaterializationV1, PersistPromotionResultOutcomeV1,
    PersistPromotionTerminalResultV1, PersistPromotionWorkflowFailureV1,
    PersistedPromotionApprovalV1, PromotionBlobReceiptV1, PromotionCorrelationV1,
    ReservePromotionApprovalOutcomeV1, ReservePromotionApprovalV1,
    ReviewedNoteCandidatePromotionOutcomeV1, ReviewedNoteCandidatePromotionPersistenceErrorV1,
    model::{nonzero, valid_blob, valid_outbox, valid_owner, valid_timestamp},
    outbox::{insert_exact_outbox, verify_exact_outbox},
};

#[derive(Clone)]
pub struct ReviewedNoteCandidatePromotionPersistenceV1 {
    pub(crate) pool: PgPool,
}

impl ReviewedNoteCandidatePromotionPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, ReviewedNoteCandidatePromotionPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| ReviewedNoteCandidatePromotionPersistenceErrorV1::StorageUnavailable)?;
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
    ) -> Result<(), ReviewedNoteCandidatePromotionPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(storage_error)
    }

    pub async fn reserve_approval(
        &self,
        input: &ReservePromotionApprovalV1,
    ) -> Result<ReservePromotionApprovalOutcomeV1, ReviewedNoteCandidatePromotionPersistenceErrorV1>
    {
        validate_reservation(input)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.reviewed_note_candidate_promotion_requests (
               logical_owner_id, approval_message_id, approval_envelope_sha256,
               review_id, candidate_id, decision_revision,
               source_blob_reference_id, source_blob_declared_bytes,
               source_blob_sha256, source_blob_custody_proof,
               knowledge_command_id, created_at_unix_millis, updated_at_unix_millis
             ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$12)
             ON CONFLICT DO NOTHING",
        )
        .bind(&input.logical_owner_id)
        .bind(input.approval_message_id.as_slice())
        .bind(input.approval_envelope_sha256.as_slice())
        .bind(input.review_id.as_slice())
        .bind(input.candidate_id.as_slice())
        .bind(signed_revision(input.decision_revision)?)
        .bind(input.source_blob.reference_id.as_slice())
        .bind(signed_revision(input.source_blob.declared_bytes)?)
        .bind(input.source_blob.sha256.as_slice())
        .bind(&input.source_blob.custody_proof)
        .bind(input.knowledge_command_id.as_slice())
        .bind(input.occurred_at_unix_millis)
        .execute(&self.pool)
        .await
        .map_err(storage_error)?
        .rows_affected();
        let persisted = self
            .load_approval(&input.logical_owner_id, &input.approval_message_id)
            .await?;
        if !same_reservation(&persisted, input) {
            return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::ApprovalConflict);
        }
        if inserted == 1 {
            Ok(ReservePromotionApprovalOutcomeV1::Reserved(persisted))
        } else {
            Ok(ReservePromotionApprovalOutcomeV1::Existing(persisted))
        }
    }

    pub async fn persist_materialization(
        &self,
        input: &PersistPromotionMaterializationV1,
    ) -> Result<PersistedPromotionApprovalV1, ReviewedNoteCandidatePromotionPersistenceErrorV1>
    {
        if !valid_owner(&input.logical_owner_id)
            || !nonzero(&input.approval_message_id)
            || !nonzero(&input.materialized_reference_id)
            || !valid_timestamp(input.materialized_at_unix_millis)
        {
            return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::InvalidInput);
        }
        let affected = sqlx::query(
            "UPDATE makosh_data.reviewed_note_candidate_promotion_requests
             SET materialized_blob_reference_id=$1, updated_at_unix_millis=$2
             WHERE logical_owner_id=$3 AND approval_message_id=$4
               AND materialized_blob_reference_id IS NULL",
        )
        .bind(input.materialized_reference_id.as_slice())
        .bind(input.materialized_at_unix_millis)
        .bind(&input.logical_owner_id)
        .bind(input.approval_message_id.as_slice())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?
        .rows_affected();
        let persisted = self
            .load_approval(&input.logical_owner_id, &input.approval_message_id)
            .await?;
        if persisted.materialized_reference_id != Some(input.materialized_reference_id) {
            return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::ApprovalConflict);
        }
        if affected > 1 {
            return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::InvalidRow);
        }
        Ok(persisted)
    }

    pub async fn complete_source_cleanup(
        &self,
        logical_owner_id: &str,
        approval_message_id: &[u8; 16],
        materialized_reference_id: &[u8; 16],
        completed_at_unix_millis: i64,
    ) -> Result<(), ReviewedNoteCandidatePromotionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !nonzero(approval_message_id)
            || !nonzero(materialized_reference_id)
            || !valid_timestamp(completed_at_unix_millis)
        {
            return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::InvalidInput);
        }
        let affected = sqlx::query(
            "UPDATE makosh_data.reviewed_note_candidate_promotion_requests
             SET cleanup_completed_at_unix_millis=$1, updated_at_unix_millis=$1
             WHERE logical_owner_id=$2 AND approval_message_id=$3
               AND materialized_blob_reference_id=$4
               AND (
                 knowledge_command_message_id IS NOT NULL
                 OR workflow_failure_result_id IS NOT NULL
               )
               AND cleanup_completed_at_unix_millis IS NULL",
        )
        .bind(completed_at_unix_millis)
        .bind(logical_owner_id)
        .bind(approval_message_id.as_slice())
        .bind(materialized_reference_id.as_slice())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if affected == 1 {
            return Ok(());
        }
        let current = self
            .load_approval(logical_owner_id, approval_message_id)
            .await?;
        if current.materialized_reference_id == Some(*materialized_reference_id)
            && (current.command_completed || current.workflow_failure_result_id.is_some())
            && current.cleanup_completed_at_unix_millis.is_some()
        {
            Ok(())
        } else {
            Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::ApprovalConflict)
        }
    }

    async fn load_approval(
        &self,
        logical_owner_id: &str,
        approval_message_id: &[u8; 16],
    ) -> Result<PersistedPromotionApprovalV1, ReviewedNoteCandidatePromotionPersistenceErrorV1>
    {
        let row = sqlx::query(REQUEST_BY_APPROVAL)
            .bind(logical_owner_id)
            .bind(approval_message_id.as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_error)?
            .ok_or(ReviewedNoteCandidatePromotionPersistenceErrorV1::NotFound)?;
        persisted_approval_from_row(&row)
    }

    pub async fn load_correlation(
        &self,
        logical_owner_id: &str,
        knowledge_command_id: &[u8; 16],
    ) -> Result<PromotionCorrelationV1, ReviewedNoteCandidatePromotionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || !nonzero(knowledge_command_id) {
            return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT review_id, candidate_id, decision_revision, knowledge_result_message_id
             FROM makosh_data.reviewed_note_candidate_promotion_requests
             WHERE logical_owner_id = $1 AND knowledge_command_id = $2
               AND knowledge_command_message_id IS NOT NULL",
        )
        .bind(logical_owner_id)
        .bind(knowledge_command_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .ok_or(ReviewedNoteCandidatePromotionPersistenceErrorV1::NotFound)?;
        Ok(PromotionCorrelationV1 {
            review_id: fixed(row_value(&row, "review_id")?)?,
            candidate_id: fixed(row_value(&row, "candidate_id")?)?,
            decision_revision: unsigned_revision(row_value(&row, "decision_revision")?)?,
            completed: row_value::<Option<Vec<u8>>>(&row, "knowledge_result_message_id")?.is_some(),
        })
    }

    pub async fn persist_approval_and_knowledge_command(
        &self,
        input: &PersistPromotionApprovalV1,
    ) -> Result<PersistPromotionApprovalOutcomeV1, ReviewedNoteCandidatePromotionPersistenceErrorV1>
    {
        validate_approval(input)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let existing = load_request_by_approval(
            &mut transaction,
            &input.logical_owner_id,
            &input.approval_message_id,
        )
        .await?
        .ok_or(ReviewedNoteCandidatePromotionPersistenceErrorV1::ApprovalConflict)?;
        if !existing.matches_approval(input)? {
            return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::ApprovalConflict);
        }
        if existing.workflow_failure_result_id.is_some() {
            return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::ResultConflict);
        }
        if existing.knowledge_command_message_id.is_none() {
            insert_exact_outbox(
                &mut transaction,
                &input.logical_owner_id,
                &input.knowledge_command_outbox,
                input.occurred_at_unix_millis,
            )
            .await?;
            let updated = sqlx::query(
                "UPDATE makosh_data.reviewed_note_candidate_promotion_requests
                 SET knowledge_command_message_id=$1, updated_at_unix_millis=$2
                 WHERE logical_owner_id=$3 AND approval_message_id=$4
                   AND knowledge_command_message_id IS NULL",
            )
            .bind(input.knowledge_command_id.as_slice())
            .bind(input.occurred_at_unix_millis)
            .bind(&input.logical_owner_id)
            .bind(input.approval_message_id.as_slice())
            .execute(&mut *transaction)
            .await
            .map_err(storage_error)?
            .rows_affected();
            if updated != 1 {
                return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::ApprovalConflict);
            }
            transaction.commit().await.map_err(storage_error)?;
            return Ok(PersistPromotionApprovalOutcomeV1::Applied);
        }
        verify_exact_outbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.knowledge_command_outbox,
        )
        .await?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(PersistPromotionApprovalOutcomeV1::Duplicate)
    }

    pub async fn persist_workflow_failure(
        &self,
        input: &PersistPromotionWorkflowFailureV1,
    ) -> Result<PersistPromotionResultOutcomeV1, ReviewedNoteCandidatePromotionPersistenceErrorV1>
    {
        validate_workflow_failure(input)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let existing = load_request_by_approval(
            &mut transaction,
            &input.logical_owner_id,
            &input.approval_message_id,
        )
        .await?
        .ok_or(ReviewedNoteCandidatePromotionPersistenceErrorV1::NotFound)?;
        let result_id = *input.review_result_outbox.message_id();
        if existing.workflow_failure_result_id.is_some() {
            if existing.workflow_failure_result_id != Some(result_id)
                || existing.promotion_outcome != Some(2)
                || existing.failure_code != Some(input.failure_code)
            {
                return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::ResultConflict);
            }
            verify_exact_outbox(
                &mut transaction,
                &input.logical_owner_id,
                &input.review_result_outbox,
            )
            .await?;
            transaction.commit().await.map_err(storage_error)?;
            return Ok(PersistPromotionResultOutcomeV1::Duplicate);
        }
        if existing.knowledge_command_message_id.is_some()
            || existing.knowledge_result_message_id.is_some()
            || existing.review_id != input.review_id
            || existing.candidate_id != input.candidate_id
            || existing.knowledge_command_id != input.knowledge_command_id
        {
            return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::ResultConflict);
        }
        insert_exact_outbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.review_result_outbox,
            input.occurred_at_unix_millis,
        )
        .await?;
        let updated = sqlx::query(
            "UPDATE makosh_data.reviewed_note_candidate_promotion_requests
             SET workflow_failure_result_id=$1, promotion_outcome=2,
                 failure_code=$2, updated_at_unix_millis=$3
             WHERE logical_owner_id=$4 AND approval_message_id=$5
               AND knowledge_command_message_id IS NULL
               AND workflow_failure_result_id IS NULL
               AND knowledge_result_message_id IS NULL",
        )
        .bind(result_id.as_slice())
        .bind(i32::from(input.failure_code))
        .bind(input.occurred_at_unix_millis)
        .bind(&input.logical_owner_id)
        .bind(input.approval_message_id.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if updated != 1 {
            return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::ResultConflict);
        }
        transaction.commit().await.map_err(storage_error)?;
        Ok(PersistPromotionResultOutcomeV1::Applied)
    }

    pub async fn persist_knowledge_result_and_review_result(
        &self,
        input: &PersistPromotionTerminalResultV1,
    ) -> Result<PersistPromotionResultOutcomeV1, ReviewedNoteCandidatePromotionPersistenceErrorV1>
    {
        validate_terminal_result(input)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let request = load_request_by_command_for_update(
            &mut transaction,
            &input.logical_owner_id,
            &input.knowledge_command_id,
        )
        .await?
        .ok_or(ReviewedNoteCandidatePromotionPersistenceErrorV1::NotFound)?;
        if request.review_id != input.review_id || request.candidate_id != input.candidate_id {
            return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::ResultConflict);
        }
        if request.knowledge_result_message_id.is_some() {
            if !request.matches_terminal_result(input)? {
                return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::ResultConflict);
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
            "INSERT INTO makosh_data.reviewed_note_candidate_promotion_result_inbox (
               logical_owner_id, result_message_id, envelope_sha256,
               knowledge_command_id, review_id, processed_at_unix_millis
             ) VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT DO NOTHING",
        )
        .bind(&input.logical_owner_id)
        .bind(input.knowledge_result_message_id.as_slice())
        .bind(input.knowledge_result_envelope_sha256.as_slice())
        .bind(input.knowledge_command_id.as_slice())
        .bind(input.review_id.as_slice())
        .bind(input.occurred_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if inbox_inserted != 1 {
            return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::ResultConflict);
        }
        insert_exact_outbox(
            &mut transaction,
            &input.logical_owner_id,
            &input.review_result_outbox,
            input.occurred_at_unix_millis,
        )
        .await?;
        let (outcome, note_id, failure_code) = encoded_outcome(input.outcome);
        let updated = sqlx::query(
            "UPDATE makosh_data.reviewed_note_candidate_promotion_requests
             SET knowledge_result_message_id = $1, promotion_outcome = $2,
                 note_id = $3, failure_code = $4, updated_at_unix_millis = $5
             WHERE logical_owner_id = $6 AND knowledge_command_id = $7
               AND knowledge_result_message_id IS NULL",
        )
        .bind(input.knowledge_result_message_id.as_slice())
        .bind(outcome)
        .bind(note_id.map(|value| value.to_vec()))
        .bind(failure_code)
        .bind(input.occurred_at_unix_millis)
        .bind(&input.logical_owner_id)
        .bind(input.knowledge_command_id.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if updated != 1 {
            return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::ResultConflict);
        }
        transaction.commit().await.map_err(storage_error)?;
        Ok(PersistPromotionResultOutcomeV1::Applied)
    }
}

#[derive(Debug)]
struct PromotionRequestRowV1 {
    logical_owner_id: String,
    approval_message_id: [u8; 16],
    approval_envelope_sha256: [u8; 32],
    review_id: [u8; 16],
    candidate_id: [u8; 16],
    decision_revision: u64,
    source_blob: PromotionBlobReceiptV1,
    materialized_reference_id: Option<[u8; 16]>,
    cleanup_completed_at_unix_millis: Option<i64>,
    knowledge_command_id: [u8; 16],
    knowledge_command_message_id: Option<[u8; 16]>,
    workflow_failure_result_id: Option<[u8; 16]>,
    knowledge_result_message_id: Option<[u8; 16]>,
    promotion_outcome: Option<i16>,
    note_id: Option<[u8; 16]>,
    failure_code: Option<u16>,
}

impl PromotionRequestRowV1 {
    fn matches_approval(
        &self,
        input: &PersistPromotionApprovalV1,
    ) -> Result<bool, ReviewedNoteCandidatePromotionPersistenceErrorV1> {
        Ok(
            self.approval_envelope_sha256 == input.approval_envelope_sha256
                && self.review_id == input.review_id
                && self.candidate_id == input.candidate_id
                && self.decision_revision == input.decision_revision
                && self.knowledge_command_id == input.knowledge_command_id
                && self
                    .knowledge_command_message_id
                    .is_none_or(|value| value == *input.knowledge_command_outbox.message_id()),
        )
    }

    fn matches_terminal_result(
        &self,
        input: &PersistPromotionTerminalResultV1,
    ) -> Result<bool, ReviewedNoteCandidatePromotionPersistenceErrorV1> {
        let (outcome, note_id, failure_code) = encoded_outcome(input.outcome);
        Ok(
            self.knowledge_result_message_id == Some(input.knowledge_result_message_id)
                && self.promotion_outcome == Some(outcome)
                && self.note_id == note_id
                && self.failure_code.map(i32::from) == failure_code,
        )
    }
}

async fn load_request_by_approval(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    approval_message_id: &[u8; 16],
) -> Result<Option<PromotionRequestRowV1>, ReviewedNoteCandidatePromotionPersistenceErrorV1> {
    load_request(
        sqlx::query(REQUEST_BY_APPROVAL_FOR_UPDATE)
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
    knowledge_command_id: &[u8; 16],
) -> Result<Option<PromotionRequestRowV1>, ReviewedNoteCandidatePromotionPersistenceErrorV1> {
    load_request(
        sqlx::query(REQUEST_BY_COMMAND_FOR_UPDATE)
            .bind(logical_owner_id)
            .bind(knowledge_command_id.as_slice())
            .fetch_optional(&mut **transaction)
            .await
            .map_err(storage_error)?,
    )
}

fn load_request(
    row: Option<sqlx::postgres::PgRow>,
) -> Result<Option<PromotionRequestRowV1>, ReviewedNoteCandidatePromotionPersistenceErrorV1> {
    row.as_ref().map(request_from_row).transpose()
}

fn request_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<PromotionRequestRowV1, ReviewedNoteCandidatePromotionPersistenceErrorV1> {
    Ok(PromotionRequestRowV1 {
        logical_owner_id: row_value(row, "logical_owner_id")?,
        approval_message_id: fixed(row_value(row, "approval_message_id")?)?,
        approval_envelope_sha256: fixed(row_value(row, "approval_envelope_sha256")?)?,
        review_id: fixed(row_value(row, "review_id")?)?,
        candidate_id: fixed(row_value(row, "candidate_id")?)?,
        decision_revision: unsigned_revision(row_value(row, "decision_revision")?)?,
        source_blob: PromotionBlobReceiptV1 {
            reference_id: fixed(row_value(row, "source_blob_reference_id")?)?,
            declared_bytes: unsigned_revision(row_value(row, "source_blob_declared_bytes")?)?,
            sha256: fixed(row_value(row, "source_blob_sha256")?)?,
            custody_proof: row_value(row, "source_blob_custody_proof")?,
        },
        materialized_reference_id: optional_fixed(row_value(
            row,
            "materialized_blob_reference_id",
        )?)?,
        cleanup_completed_at_unix_millis: row_value(row, "cleanup_completed_at_unix_millis")?,
        knowledge_command_id: fixed(row_value(row, "knowledge_command_id")?)?,
        knowledge_command_message_id: optional_fixed(row_value(
            row,
            "knowledge_command_message_id",
        )?)?,
        workflow_failure_result_id: optional_fixed(row_value(row, "workflow_failure_result_id")?)?,
        knowledge_result_message_id: optional_fixed(row_value(
            row,
            "knowledge_result_message_id",
        )?)?,
        promotion_outcome: row_value(row, "promotion_outcome")?,
        note_id: optional_fixed(row_value(row, "note_id")?)?,
        failure_code: row_value::<Option<i32>>(row, "failure_code")?
            .map(u16::try_from)
            .transpose()
            .map_err(|_| ReviewedNoteCandidatePromotionPersistenceErrorV1::InvalidRow)?,
    })
}

async fn verify_result_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    input: &PersistPromotionTerminalResultV1,
) -> Result<(), ReviewedNoteCandidatePromotionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT envelope_sha256, knowledge_command_id, review_id
         FROM makosh_data.reviewed_note_candidate_promotion_result_inbox
         WHERE logical_owner_id = $1 AND result_message_id = $2",
    )
    .bind(&input.logical_owner_id)
    .bind(input.knowledge_result_message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_error)?
    .ok_or(ReviewedNoteCandidatePromotionPersistenceErrorV1::InvalidRow)?;
    let hash: [u8; 32] = fixed(row_value(&row, "envelope_sha256")?)?;
    let command: [u8; 16] = fixed(row_value(&row, "knowledge_command_id")?)?;
    let review: [u8; 16] = fixed(row_value(&row, "review_id")?)?;
    if hash != input.knowledge_result_envelope_sha256
        || command != input.knowledge_command_id
        || review != input.review_id
    {
        return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::ResultConflict);
    }
    Ok(())
}

fn validate_approval(
    input: &PersistPromotionApprovalV1,
) -> Result<(), ReviewedNoteCandidatePromotionPersistenceErrorV1> {
    let expected = derive_reviewed_note_candidate_command_id_v1(
        input.approval_message_id,
        input.review_id,
        input.candidate_id,
        input.decision_revision,
    )
    .map_err(|_| ReviewedNoteCandidatePromotionPersistenceErrorV1::InvalidInput)?;
    if !valid_owner(&input.logical_owner_id)
        || !nonzero(&input.approval_envelope_sha256)
        || input.knowledge_command_id != expected
        || input.knowledge_command_outbox.message_id() != &expected
        || !valid_outbox(&input.knowledge_command_outbox)
        || !valid_timestamp(input.occurred_at_unix_millis)
    {
        return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_reservation(
    input: &ReservePromotionApprovalV1,
) -> Result<(), ReviewedNoteCandidatePromotionPersistenceErrorV1> {
    let expected = derive_reviewed_note_candidate_command_id_v1(
        input.approval_message_id,
        input.review_id,
        input.candidate_id,
        input.decision_revision,
    )
    .map_err(|_| ReviewedNoteCandidatePromotionPersistenceErrorV1::InvalidInput)?;
    if !valid_owner(&input.logical_owner_id)
        || !nonzero(&input.approval_envelope_sha256)
        || !valid_blob(&input.source_blob)
        || input.knowledge_command_id != expected
        || !valid_timestamp(input.occurred_at_unix_millis)
    {
        return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn persisted_approval_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<PersistedPromotionApprovalV1, ReviewedNoteCandidatePromotionPersistenceErrorV1> {
    let value = request_from_row(row)?;
    Ok(PersistedPromotionApprovalV1 {
        logical_owner_id: value.logical_owner_id,
        approval_message_id: value.approval_message_id,
        approval_envelope_sha256: value.approval_envelope_sha256,
        review_id: value.review_id,
        candidate_id: value.candidate_id,
        decision_revision: value.decision_revision,
        source_blob: value.source_blob,
        materialized_reference_id: value.materialized_reference_id,
        cleanup_completed_at_unix_millis: value.cleanup_completed_at_unix_millis,
        knowledge_command_id: value.knowledge_command_id,
        command_completed: value.knowledge_command_message_id.is_some(),
        workflow_failure_result_id: value.workflow_failure_result_id,
    })
}

fn same_reservation(
    persisted: &PersistedPromotionApprovalV1,
    input: &ReservePromotionApprovalV1,
) -> bool {
    persisted.logical_owner_id == input.logical_owner_id
        && persisted.approval_message_id == input.approval_message_id
        && persisted.approval_envelope_sha256 == input.approval_envelope_sha256
        && persisted.review_id == input.review_id
        && persisted.candidate_id == input.candidate_id
        && persisted.decision_revision == input.decision_revision
        && persisted.source_blob == input.source_blob
        && persisted.knowledge_command_id == input.knowledge_command_id
}

fn validate_terminal_result(
    input: &PersistPromotionTerminalResultV1,
) -> Result<(), ReviewedNoteCandidatePromotionPersistenceErrorV1> {
    let expected_result_id = derive_reviewed_note_candidate_result_id_v1(
        input.knowledge_result_message_id,
        input.knowledge_command_id,
        input.review_id,
    )
    .map_err(|_| ReviewedNoteCandidatePromotionPersistenceErrorV1::InvalidInput)?;
    let outcome_valid = match input.outcome {
        ReviewedNoteCandidatePromotionOutcomeV1::Succeeded { note_id } => nonzero(&note_id),
        ReviewedNoteCandidatePromotionOutcomeV1::Failed { failure_code } => failure_code > 0,
    };
    if !valid_owner(&input.logical_owner_id)
        || !nonzero(&input.knowledge_result_envelope_sha256)
        || !nonzero(&input.candidate_id)
        || !outcome_valid
        || input.review_result_outbox.message_id() != &expected_result_id
        || !valid_outbox(&input.review_result_outbox)
        || !valid_timestamp(input.occurred_at_unix_millis)
    {
        return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_workflow_failure(
    input: &PersistPromotionWorkflowFailureV1,
) -> Result<(), ReviewedNoteCandidatePromotionPersistenceErrorV1> {
    let expected_result_id = derive_reviewed_note_candidate_result_id_v1(
        input.approval_message_id,
        input.knowledge_command_id,
        input.review_id,
    )
    .map_err(|_| ReviewedNoteCandidatePromotionPersistenceErrorV1::InvalidInput)?;
    if !valid_owner(&input.logical_owner_id)
        || !nonzero(&input.candidate_id)
        || input.failure_code == 0
        || input.review_result_outbox.message_id() != &expected_result_id
        || !valid_outbox(&input.review_result_outbox)
        || !valid_timestamp(input.occurred_at_unix_millis)
    {
        return Err(ReviewedNoteCandidatePromotionPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn encoded_outcome(
    outcome: ReviewedNoteCandidatePromotionOutcomeV1,
) -> (i16, Option<[u8; 16]>, Option<i32>) {
    match outcome {
        ReviewedNoteCandidatePromotionOutcomeV1::Succeeded { note_id } => (1, Some(note_id), None),
        ReviewedNoteCandidatePromotionOutcomeV1::Failed { failure_code } => {
            (2, None, Some(i32::from(failure_code)))
        }
    }
}

fn fixed<const N: usize>(
    value: Vec<u8>,
) -> Result<[u8; N], ReviewedNoteCandidatePromotionPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| ReviewedNoteCandidatePromotionPersistenceErrorV1::InvalidRow)
}

fn optional_fixed<const N: usize>(
    value: Option<Vec<u8>>,
) -> Result<Option<[u8; N]>, ReviewedNoteCandidatePromotionPersistenceErrorV1> {
    value.map(fixed).transpose()
}

fn row_value<T>(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<T, ReviewedNoteCandidatePromotionPersistenceErrorV1>
where
    for<'r> T: sqlx::Decode<'r, Postgres> + sqlx::Type<Postgres>,
{
    row.try_get(column)
        .map_err(|_| ReviewedNoteCandidatePromotionPersistenceErrorV1::InvalidRow)
}

fn signed_revision(value: u64) -> Result<i64, ReviewedNoteCandidatePromotionPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| ReviewedNoteCandidatePromotionPersistenceErrorV1::InvalidInput)
}

fn unsigned_revision(value: i64) -> Result<u64, ReviewedNoteCandidatePromotionPersistenceErrorV1> {
    u64::try_from(value).map_err(|_| ReviewedNoteCandidatePromotionPersistenceErrorV1::InvalidRow)
}

fn storage_error(_: sqlx::Error) -> ReviewedNoteCandidatePromotionPersistenceErrorV1 {
    ReviewedNoteCandidatePromotionPersistenceErrorV1::StorageUnavailable
}

const REQUEST_BY_APPROVAL: &str = "SELECT logical_owner_id, approval_message_id, approval_envelope_sha256, review_id, candidate_id, decision_revision, \
     source_blob_reference_id, source_blob_declared_bytes, source_blob_sha256, source_blob_custody_proof, \
     materialized_blob_reference_id, cleanup_completed_at_unix_millis, \
     knowledge_command_id, knowledge_command_message_id, workflow_failure_result_id, knowledge_result_message_id, \
     promotion_outcome, note_id, failure_code \
     FROM makosh_data.reviewed_note_candidate_promotion_requests \
     WHERE logical_owner_id = $1 AND approval_message_id = $2";
const REQUEST_BY_APPROVAL_FOR_UPDATE: &str = "SELECT logical_owner_id, approval_message_id, approval_envelope_sha256, review_id, candidate_id, decision_revision, \
     source_blob_reference_id, source_blob_declared_bytes, source_blob_sha256, source_blob_custody_proof, \
     materialized_blob_reference_id, cleanup_completed_at_unix_millis, \
     knowledge_command_id, knowledge_command_message_id, workflow_failure_result_id, knowledge_result_message_id, \
     promotion_outcome, note_id, failure_code \
     FROM makosh_data.reviewed_note_candidate_promotion_requests \
     WHERE logical_owner_id = $1 AND approval_message_id = $2 FOR UPDATE";
const REQUEST_BY_COMMAND_FOR_UPDATE: &str = "SELECT logical_owner_id, approval_message_id, approval_envelope_sha256, review_id, candidate_id, decision_revision, \
     source_blob_reference_id, source_blob_declared_bytes, source_blob_sha256, source_blob_custody_proof, \
     materialized_blob_reference_id, cleanup_completed_at_unix_millis, \
     knowledge_command_id, knowledge_command_message_id, workflow_failure_result_id, knowledge_result_message_id, \
     promotion_outcome, note_id, failure_code \
     FROM makosh_data.reviewed_note_candidate_promotion_requests \
     WHERE logical_owner_id = $1 AND knowledge_command_id = $2 FOR UPDATE";
