use makosh_communication_reply_suggestion_core::{
    ReplySuggestionCandidateV1, ReplySuggestionCompletenessV1, ReplySuggestionDraftV1,
    ReplySuggestionLanguageV1, ReplySuggestionRejectionCodeV1, ReplySuggestionStateV1,
    ReplySuggestionStatusV1, ReplySuggestionToneV1, ReplySuggestionTransitionV1,
    accepted_reply_suggestion_status_v1, transition_reply_suggestion_v1,
    validate_reply_suggestion_draft_v1, validate_reply_suggestion_status_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions, PgRow},
};

use crate::{
    model::{
        CreateReplySuggestionOutcomeV1, CreateReplySuggestionRunV1, PersistedReplySuggestionRunV1,
        REPLY_SUGGESTION_MAX_CUSTODY_PROOF_BYTES_V1, REPLY_SUGGESTION_MAX_EVENT_BYTES_V1,
        REPLY_SUGGESTION_MAX_INFERENCE_REQUEST_BYTES_V1, REPLY_SUGGESTION_RECOVERY_LIMIT_V1,
        ReplySuggestionBlobCleanupV1, ReplySuggestionInboxResultV1,
        ReplySuggestionPersistenceErrorV1, ReplySuggestionSourceResultV1, language_code, nonzero,
        rejection_code, request_fingerprint, tone_code, valid_identity, valid_timestamp,
    },
    realtime::insert_realtime_transition,
};

#[derive(Clone)]
pub struct CommunicationReplySuggestionPersistenceV1 {
    pub(crate) pool: PgPool,
}

impl CommunicationReplySuggestionPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, ReplySuggestionPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(ReplySuggestionPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| ReplySuggestionPersistenceErrorV1::StorageUnavailable)?;
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
            .map_err(|_| ReplySuggestionPersistenceErrorV1::StorageUnavailable)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), ReplySuggestionPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| ReplySuggestionPersistenceErrorV1::StorageUnavailable)
    }

    pub async fn create_run(
        &self,
        input: CreateReplySuggestionRunV1,
    ) -> Result<CreateReplySuggestionOutcomeV1, ReplySuggestionPersistenceErrorV1> {
        validate_create(&input)?;
        let fingerprint = request_fingerprint(&input.draft);
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.communication_reply_suggestion_runs (
               logical_owner_id, run_id, operation_id, request_fingerprint,
               source_message_id, expected_source_revision, requested_tone,
               requested_language, state, state_revision,
               created_at_unix_millis, updated_at_unix_millis
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 1, 1, $9, $9)
             ON CONFLICT (logical_owner_id, operation_id) DO NOTHING",
        )
        .bind(&input.logical_owner_id)
        .bind(input.draft.run_id.as_slice())
        .bind(input.draft.operation_id.as_slice())
        .bind(fingerprint.as_slice())
        .bind(input.draft.source_message_id.as_slice())
        .bind(signed(input.draft.expected_source_revision)?)
        .bind(i16::from(tone_code(input.draft.tone)))
        .bind(i16::from(language_code(input.draft.language)))
        .bind(input.created_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if inserted == 1 {
            sqlx::query(
                "INSERT INTO makosh_data.communication_reply_suggestion_outbox (
                   logical_owner_id, message_id, envelope_sha256, envelope_bytes,
                   created_at_unix_millis
                 ) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(&input.logical_owner_id)
            .bind(input.source_prepare_message_id.as_slice())
            .bind(input.source_prepare_envelope_sha256.as_slice())
            .bind(&input.source_prepare_envelope_bytes)
            .bind(input.created_at_unix_millis)
            .execute(&mut *transaction)
            .await
            .map_err(storage_error)?;
            insert_realtime_transition(
                &mut transaction,
                &input.logical_owner_id,
                &input.draft.run_id,
                input.created_at_unix_millis,
            )
            .await?;
        }
        transaction.commit().await.map_err(storage_error)?;
        let persisted = self
            .load_by_operation(&input.logical_owner_id, &input.draft.operation_id)
            .await?;
        if persisted.request_fingerprint != fingerprint {
            return Err(ReplySuggestionPersistenceErrorV1::RequestConflict);
        }
        if inserted == 1 {
            Ok(CreateReplySuggestionOutcomeV1::Created(persisted))
        } else {
            Ok(CreateReplySuggestionOutcomeV1::Existing(persisted))
        }
    }

    pub async fn load_run(
        &self,
        logical_owner_id: &str,
        run_id: &[u8; 16],
    ) -> Result<PersistedReplySuggestionRunV1, ReplySuggestionPersistenceErrorV1> {
        if !valid_identity(logical_owner_id) || !nonzero(run_id) {
            return Err(ReplySuggestionPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(SELECT_RUN)
            .bind(logical_owner_id)
            .bind(run_id.as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_error)?
            .ok_or(ReplySuggestionPersistenceErrorV1::NotFound)?;
        persisted_from_row(row)
    }

    pub async fn load_recoverable_runs(
        &self,
        logical_owner_id: &str,
    ) -> Result<Vec<PersistedReplySuggestionRunV1>, ReplySuggestionPersistenceErrorV1> {
        if !valid_identity(logical_owner_id) {
            return Err(ReplySuggestionPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(SELECT_RECOVERABLE_RUNS)
            .bind(logical_owner_id)
            .bind(i64::from(REPLY_SUGGESTION_RECOVERY_LIMIT_V1))
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(persisted_from_row)
            .collect()
    }

    pub async fn persist_source_result(
        &self,
        input: ReplySuggestionSourceResultV1,
    ) -> Result<ReplySuggestionInboxResultV1, ReplySuggestionPersistenceErrorV1> {
        validate_source_result(&input)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        if let Some(row) = sqlx::query(
            "SELECT envelope_sha256, run_id
             FROM makosh_data.communication_reply_suggestion_inbox
             WHERE logical_owner_id = $1 AND result_message_id = $2",
        )
        .bind(&input.logical_owner_id)
        .bind(input.result_message_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_error)?
        {
            let existing_hash: Vec<u8> = row
                .try_get("envelope_sha256")
                .map_err(|_| ReplySuggestionPersistenceErrorV1::InvalidRow)?;
            let existing_run: Vec<u8> = row
                .try_get("run_id")
                .map_err(|_| ReplySuggestionPersistenceErrorV1::InvalidRow)?;
            if existing_hash.as_slice() != input.envelope_sha256
                || existing_run.as_slice() != input.run_id
            {
                return Err(ReplySuggestionPersistenceErrorV1::InboxConflict);
            }
            transaction.commit().await.map_err(storage_error)?;
            return self
                .load_run(&input.logical_owner_id, &input.run_id)
                .await
                .map(ReplySuggestionInboxResultV1::Duplicate);
        }
        let current =
            load_for_update(&mut transaction, &input.logical_owner_id, &input.run_id).await?;
        let next = transition_reply_suggestion_v1(&current.status, input.transition.clone())
            .map_err(|_| ReplySuggestionPersistenceErrorV1::InvalidTransition)?;
        persist_status(
            &mut transaction,
            &input.logical_owner_id,
            &input.run_id,
            current.status.state_revision,
            &next,
            input.occurred_at_unix_millis,
            materialization(&input)?,
        )
        .await?;
        sqlx::query(
            "INSERT INTO makosh_data.communication_reply_suggestion_inbox (
               logical_owner_id, result_message_id, envelope_sha256, run_id,
               processed_at_unix_millis
             ) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.result_message_id.as_slice())
        .bind(input.envelope_sha256.as_slice())
        .bind(input.run_id.as_slice())
        .bind(input.occurred_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
        insert_realtime_transition(
            &mut transaction,
            &input.logical_owner_id,
            &input.run_id,
            input.occurred_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_error)?;
        self.load_run(&input.logical_owner_id, &input.run_id)
            .await
            .map(ReplySuggestionInboxResultV1::Applied)
    }

    pub async fn persist_inference_transition(
        &self,
        logical_owner_id: &str,
        run_id: &[u8; 16],
        transition: ReplySuggestionTransitionV1,
        occurred_at_unix_millis: i64,
    ) -> Result<PersistedReplySuggestionRunV1, ReplySuggestionPersistenceErrorV1> {
        if !valid_identity(logical_owner_id)
            || !nonzero(run_id)
            || !valid_timestamp(occurred_at_unix_millis)
            || !matches!(
                transition,
                ReplySuggestionTransitionV1::Complete(_) | ReplySuggestionTransitionV1::Reject(_)
            )
        {
            return Err(ReplySuggestionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let current = load_for_update(&mut transaction, logical_owner_id, run_id).await?;
        let next = transition_reply_suggestion_v1(&current.status, transition)
            .map_err(|_| ReplySuggestionPersistenceErrorV1::InvalidTransition)?;
        persist_status(
            &mut transaction,
            logical_owner_id,
            run_id,
            current.status.state_revision,
            &next,
            occurred_at_unix_millis,
            None,
        )
        .await?;
        insert_realtime_transition(
            &mut transaction,
            logical_owner_id,
            run_id,
            occurred_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_error)?;
        self.load_run(logical_owner_id, run_id).await
    }

    pub async fn begin_source_preparation(
        &self,
        logical_owner_id: &str,
        run_id: &[u8; 16],
        occurred_at_unix_millis: i64,
    ) -> Result<PersistedReplySuggestionRunV1, ReplySuggestionPersistenceErrorV1> {
        if !valid_identity(logical_owner_id)
            || !nonzero(run_id)
            || !valid_timestamp(occurred_at_unix_millis)
        {
            return Err(ReplySuggestionPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let current = load_for_update(&mut transaction, logical_owner_id, run_id).await?;
        if current.status.state == ReplySuggestionStateV1::PreparingSource {
            transaction.commit().await.map_err(storage_error)?;
            return Ok(current);
        }
        let next = transition_reply_suggestion_v1(
            &current.status,
            ReplySuggestionTransitionV1::BeginSourcePreparation,
        )
        .map_err(|_| ReplySuggestionPersistenceErrorV1::InvalidTransition)?;
        persist_status(
            &mut transaction,
            logical_owner_id,
            run_id,
            current.status.state_revision,
            &next,
            occurred_at_unix_millis,
            None,
        )
        .await?;
        insert_realtime_transition(
            &mut transaction,
            logical_owner_id,
            run_id,
            occurred_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_error)?;
        self.load_run(logical_owner_id, run_id).await
    }

    pub async fn refresh_inference_request(
        &self,
        logical_owner_id: &str,
        run_id: &[u8; 16],
        expected_request_digest: &[u8; 32],
        source_cleanup: &ReplySuggestionBlobCleanupV1,
        inference_request_bytes: &[u8],
        updated_at_unix_millis: i64,
    ) -> Result<PersistedReplySuggestionRunV1, ReplySuggestionPersistenceErrorV1> {
        if !valid_identity(logical_owner_id)
            || !nonzero(run_id)
            || !nonzero(expected_request_digest)
            || !valid_materialization(inference_request_bytes, source_cleanup)
            || !valid_timestamp(updated_at_unix_millis)
        {
            return Err(ReplySuggestionPersistenceErrorV1::InvalidInput);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.communication_reply_suggestion_runs
             SET inference_request_bytes = $1, updated_at_unix_millis = $2
             WHERE logical_owner_id = $3 AND run_id = $4 AND state = 3
               AND inference_request_digest = $5
               AND source_cleanup_reference_id = $6
               AND source_cleanup_declared_bytes = $7
               AND source_cleanup_sha256 = $8
               AND source_cleanup_custody_proof = $9
               AND updated_at_unix_millis <= $2",
        )
        .bind(inference_request_bytes)
        .bind(updated_at_unix_millis)
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .bind(expected_request_digest.as_slice())
        .bind(source_cleanup.reference_id.as_slice())
        .bind(signed(source_cleanup.declared_bytes)?)
        .bind(source_cleanup.sha256.as_slice())
        .bind(&source_cleanup.custody_proof)
        .execute(&self.pool)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if updated != 1 {
            return Err(ReplySuggestionPersistenceErrorV1::RevisionConflict);
        }
        self.load_run(logical_owner_id, run_id).await
    }

    pub async fn complete_blob_cleanup(
        &self,
        logical_owner_id: &str,
        run_id: &[u8; 16],
        source_cleanup: &ReplySuggestionBlobCleanupV1,
        completed_at_unix_millis: i64,
    ) -> Result<PersistedReplySuggestionRunV1, ReplySuggestionPersistenceErrorV1> {
        if !valid_identity(logical_owner_id)
            || !nonzero(run_id)
            || !valid_cleanup(source_cleanup)
            || !valid_timestamp(completed_at_unix_millis)
        {
            return Err(ReplySuggestionPersistenceErrorV1::InvalidInput);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.communication_reply_suggestion_runs
             SET inference_request_bytes = NULL,
                 source_cleanup_reference_id = NULL,
                 source_cleanup_declared_bytes = NULL,
                 source_cleanup_sha256 = NULL,
                 source_cleanup_custody_proof = NULL,
                 cleanup_completed_at_unix_millis = $1,
                 updated_at_unix_millis = $1
             WHERE logical_owner_id = $2 AND run_id = $3 AND state IN (4, 5)
               AND source_cleanup_reference_id = $4
               AND source_cleanup_declared_bytes = $5
               AND source_cleanup_sha256 = $6
               AND source_cleanup_custody_proof = $7
               AND cleanup_completed_at_unix_millis IS NULL
               AND updated_at_unix_millis <= $1",
        )
        .bind(completed_at_unix_millis)
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .bind(source_cleanup.reference_id.as_slice())
        .bind(signed(source_cleanup.declared_bytes)?)
        .bind(source_cleanup.sha256.as_slice())
        .bind(&source_cleanup.custody_proof)
        .execute(&self.pool)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if updated != 1 {
            return Err(ReplySuggestionPersistenceErrorV1::RevisionConflict);
        }
        self.load_run(logical_owner_id, run_id).await
    }

    async fn load_by_operation(
        &self,
        logical_owner_id: &str,
        operation_id: &[u8; 16],
    ) -> Result<PersistedReplySuggestionRunV1, ReplySuggestionPersistenceErrorV1> {
        let row = sqlx::query(SELECT_RUN_BY_OPERATION)
            .bind(logical_owner_id)
            .bind(operation_id.as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_error)?
            .ok_or(ReplySuggestionPersistenceErrorV1::NotFound)?;
        persisted_from_row(row)
    }
}

const SELECT_RUN: &str = "
SELECT logical_owner_id, run_id, operation_id, request_fingerprint,
       source_message_id, expected_source_revision, requested_tone,
       requested_language, state, state_revision, source_evidence_id,
       source_evidence_revision, source_sha256, inference_request_digest,
       inference_request_bytes, source_cleanup_reference_id,
       source_cleanup_declared_bytes, source_cleanup_sha256,
       source_cleanup_custody_proof, cleanup_completed_at_unix_millis,
       candidate_subject_utf8, candidate_body_utf8, candidate_resolved_tone,
       candidate_resolved_language, candidate_completeness,
       candidate_confidence_basis_points, rejection_code,
       created_at_unix_millis, updated_at_unix_millis
FROM makosh_data.communication_reply_suggestion_runs
WHERE logical_owner_id = $1 AND run_id = $2";

const SELECT_RUN_FOR_UPDATE: &str = "
SELECT logical_owner_id, run_id, operation_id, request_fingerprint,
       source_message_id, expected_source_revision, requested_tone,
       requested_language, state, state_revision, source_evidence_id,
       source_evidence_revision, source_sha256, inference_request_digest,
       inference_request_bytes, source_cleanup_reference_id,
       source_cleanup_declared_bytes, source_cleanup_sha256,
       source_cleanup_custody_proof, cleanup_completed_at_unix_millis,
       candidate_subject_utf8, candidate_body_utf8, candidate_resolved_tone,
       candidate_resolved_language, candidate_completeness,
       candidate_confidence_basis_points, rejection_code,
       created_at_unix_millis, updated_at_unix_millis
FROM makosh_data.communication_reply_suggestion_runs
WHERE logical_owner_id = $1 AND run_id = $2
FOR UPDATE";

const SELECT_RUN_BY_OPERATION: &str = "
SELECT logical_owner_id, run_id, operation_id, request_fingerprint,
       source_message_id, expected_source_revision, requested_tone,
       requested_language, state, state_revision, source_evidence_id,
       source_evidence_revision, source_sha256, inference_request_digest,
       inference_request_bytes, source_cleanup_reference_id,
       source_cleanup_declared_bytes, source_cleanup_sha256,
       source_cleanup_custody_proof, cleanup_completed_at_unix_millis,
       candidate_subject_utf8, candidate_body_utf8, candidate_resolved_tone,
       candidate_resolved_language, candidate_completeness,
       candidate_confidence_basis_points, rejection_code,
       created_at_unix_millis, updated_at_unix_millis
FROM makosh_data.communication_reply_suggestion_runs
WHERE logical_owner_id = $1 AND operation_id = $2";

const SELECT_RECOVERABLE_RUNS: &str = "
SELECT logical_owner_id, run_id, operation_id, request_fingerprint,
       source_message_id, expected_source_revision, requested_tone,
       requested_language, state, state_revision, source_evidence_id,
       source_evidence_revision, source_sha256, inference_request_digest,
       inference_request_bytes, source_cleanup_reference_id,
       source_cleanup_declared_bytes, source_cleanup_sha256,
       source_cleanup_custody_proof, cleanup_completed_at_unix_millis,
       candidate_subject_utf8, candidate_body_utf8, candidate_resolved_tone,
       candidate_resolved_language, candidate_completeness,
       candidate_confidence_basis_points, rejection_code,
       created_at_unix_millis, updated_at_unix_millis
FROM makosh_data.communication_reply_suggestion_runs
WHERE logical_owner_id = $1 AND state IN (1, 2, 3)
ORDER BY state_revision, run_id
LIMIT $2";

async fn load_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: &[u8; 16],
) -> Result<PersistedReplySuggestionRunV1, ReplySuggestionPersistenceErrorV1> {
    let row = sqlx::query(SELECT_RUN_FOR_UPDATE)
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_error)?
        .ok_or(ReplySuggestionPersistenceErrorV1::NotFound)?;
    persisted_from_row(row)
}

async fn persist_status(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: &[u8; 16],
    current_revision: u64,
    next: &ReplySuggestionStatusV1,
    occurred_at_unix_millis: i64,
    materialization: Option<(&[u8], &ReplySuggestionBlobCleanupV1)>,
) -> Result<(), ReplySuggestionPersistenceErrorV1> {
    validate_reply_suggestion_status_v1(next)
        .map_err(|_| ReplySuggestionPersistenceErrorV1::InvalidTransition)?;
    let candidate = next.candidate.as_ref();
    let updated = sqlx::query(
        "UPDATE makosh_data.communication_reply_suggestion_runs
         SET state = $1, state_revision = $2,
             source_evidence_id = $3, source_evidence_revision = $4,
             source_sha256 = $5, inference_request_digest = $6,
             inference_request_bytes = COALESCE($7, inference_request_bytes),
             source_cleanup_reference_id = COALESCE($8, source_cleanup_reference_id),
             source_cleanup_declared_bytes = COALESCE($9, source_cleanup_declared_bytes),
             source_cleanup_sha256 = COALESCE($10, source_cleanup_sha256),
             source_cleanup_custody_proof = COALESCE($11, source_cleanup_custody_proof),
             candidate_subject_utf8 = $12, candidate_body_utf8 = $13,
             candidate_resolved_tone = $14, candidate_resolved_language = $15,
             candidate_completeness = $16,
             candidate_confidence_basis_points = $17,
             rejection_code = $18, updated_at_unix_millis = $19
         WHERE logical_owner_id = $20 AND run_id = $21 AND state_revision = $22",
    )
    .bind(state_code(next.state))
    .bind(signed(next.state_revision)?)
    .bind(next.source_evidence_id.map(|value| value.to_vec()))
    .bind(optional_signed(next.source_evidence_revision)?)
    .bind(next.source_sha256.map(|value| value.to_vec()))
    .bind(next.inference_request_digest.map(|value| value.to_vec()))
    .bind(materialization.map(|(bytes, _)| bytes))
    .bind(materialization.map(|(_, cleanup)| cleanup.reference_id.to_vec()))
    .bind(
        materialization
            .map(|(_, cleanup)| signed(cleanup.declared_bytes))
            .transpose()?,
    )
    .bind(materialization.map(|(_, cleanup)| cleanup.sha256.to_vec()))
    .bind(materialization.map(|(_, cleanup)| cleanup.custody_proof.as_slice()))
    .bind(candidate.map(|value| value.subject_utf8.clone()))
    .bind(candidate.map(|value| value.body_utf8.clone()))
    .bind(candidate.map(|value| i16::from(tone_code(value.resolved_tone))))
    .bind(candidate.map(|value| i16::from(language_code(value.resolved_language))))
    .bind(candidate.map(|value| completeness_code(value.completeness)))
    .bind(
        candidate
            .map(|value| i32::try_from(value.confidence_basis_points))
            .transpose()
            .map_err(|_| ReplySuggestionPersistenceErrorV1::InvalidInput)?,
    )
    .bind(next.rejection.map(rejection_code))
    .bind(occurred_at_unix_millis)
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .bind(signed(current_revision)?)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?
    .rows_affected();
    if updated == 1 {
        Ok(())
    } else {
        Err(ReplySuggestionPersistenceErrorV1::RevisionConflict)
    }
}

fn persisted_from_row(
    row: PgRow,
) -> Result<PersistedReplySuggestionRunV1, ReplySuggestionPersistenceErrorV1> {
    let candidate_subject: Option<Vec<u8>> = column(&row, "candidate_subject_utf8")?;
    let candidate_body: Option<Vec<u8>> = column(&row, "candidate_body_utf8")?;
    let candidate_tone: Option<i16> = column(&row, "candidate_resolved_tone")?;
    let candidate_language: Option<i16> = column(&row, "candidate_resolved_language")?;
    let candidate_completeness: Option<i16> = column(&row, "candidate_completeness")?;
    let candidate_confidence: Option<i32> = column(&row, "candidate_confidence_basis_points")?;
    let source_sha256 = optional_array32(column(&row, "source_sha256")?)?;
    let inference_request_digest = optional_array32(column(&row, "inference_request_digest")?)?;
    let inference_request_bytes: Option<Vec<u8>> = column(&row, "inference_request_bytes")?;
    let cleanup_reference: Option<Vec<u8>> = column(&row, "source_cleanup_reference_id")?;
    let cleanup_declared: Option<i64> = column(&row, "source_cleanup_declared_bytes")?;
    let cleanup_sha256: Option<Vec<u8>> = column(&row, "source_cleanup_sha256")?;
    let cleanup_proof: Option<Vec<u8>> = column(&row, "source_cleanup_custody_proof")?;
    let source_cleanup = match (
        cleanup_reference,
        cleanup_declared,
        cleanup_sha256,
        cleanup_proof,
    ) {
        (Some(reference), Some(declared), Some(sha256), Some(custody_proof)) => {
            Some(ReplySuggestionBlobCleanupV1 {
                reference_id: array16(reference)?,
                declared_bytes: positive_u64(declared)?,
                sha256: array32(sha256)?,
                custody_proof,
            })
        }
        (None, None, None, None) => None,
        _ => return Err(ReplySuggestionPersistenceErrorV1::InvalidRow),
    };
    let cleanup_completed_at_unix_millis: Option<i64> =
        column(&row, "cleanup_completed_at_unix_millis")?;
    let candidate = match (
        candidate_subject,
        candidate_body,
        candidate_tone,
        candidate_language,
        candidate_completeness,
        candidate_confidence,
        inference_request_digest,
        source_sha256,
    ) {
        (
            Some(subject_utf8),
            Some(body_utf8),
            Some(tone),
            Some(language),
            Some(completeness),
            Some(confidence),
            Some(request_digest),
            Some(source_sha256),
        ) => Some(ReplySuggestionCandidateV1 {
            subject_utf8,
            body_utf8,
            resolved_tone: tone_from_code(tone)?,
            resolved_language: language_from_code(language)?,
            completeness: completeness_from_code(completeness)?,
            confidence_basis_points: u32::try_from(confidence)
                .map_err(|_| ReplySuggestionPersistenceErrorV1::InvalidRow)?,
            request_digest,
            source_sha256,
        }),
        (None, None, None, None, None, None, _, _) => None,
        _ => return Err(ReplySuggestionPersistenceErrorV1::InvalidRow),
    };
    let status = ReplySuggestionStatusV1 {
        state: state_from_code(column(&row, "state")?)?,
        state_revision: positive_u64(column(&row, "state_revision")?)?,
        source_evidence_id: optional_array16(column(&row, "source_evidence_id")?)?,
        source_evidence_revision: optional_positive_u64(column(&row, "source_evidence_revision")?)?,
        source_sha256,
        inference_request_digest,
        candidate,
        rejection: optional_rejection_from_code(column(&row, "rejection_code")?)?,
    };
    validate_reply_suggestion_status_v1(&status)
        .map_err(|_| ReplySuggestionPersistenceErrorV1::InvalidRow)?;
    let persisted = PersistedReplySuggestionRunV1 {
        logical_owner_id: column(&row, "logical_owner_id")?,
        draft: ReplySuggestionDraftV1 {
            run_id: array16(column(&row, "run_id")?)?,
            operation_id: array16(column(&row, "operation_id")?)?,
            source_message_id: array16(column(&row, "source_message_id")?)?,
            expected_source_revision: positive_u64(column(&row, "expected_source_revision")?)?,
            tone: tone_from_code(column(&row, "requested_tone")?)?,
            language: language_from_code(column(&row, "requested_language")?)?,
        },
        request_fingerprint: array32(column(&row, "request_fingerprint")?)?,
        status,
        inference_request_bytes,
        source_cleanup,
        cleanup_completed_at_unix_millis,
        created_at_unix_millis: column(&row, "created_at_unix_millis")?,
        updated_at_unix_millis: column(&row, "updated_at_unix_millis")?,
    };
    if !valid_identity(&persisted.logical_owner_id)
        || !valid_timestamp(persisted.created_at_unix_millis)
        || persisted.updated_at_unix_millis < persisted.created_at_unix_millis
        || validate_reply_suggestion_draft_v1(&persisted.draft).is_err()
        || !valid_materialization_state(&persisted)
    {
        return Err(ReplySuggestionPersistenceErrorV1::InvalidRow);
    }
    Ok(persisted)
}

fn validate_create(
    input: &CreateReplySuggestionRunV1,
) -> Result<(), ReplySuggestionPersistenceErrorV1> {
    if !valid_identity(&input.logical_owner_id)
        || !valid_timestamp(input.created_at_unix_millis)
        || !nonzero(&input.source_prepare_message_id)
        || !nonzero(&input.source_prepare_envelope_sha256)
        || input.source_prepare_envelope_bytes.is_empty()
        || input.source_prepare_envelope_bytes.len() > REPLY_SUGGESTION_MAX_EVENT_BYTES_V1
        || validate_reply_suggestion_draft_v1(&input.draft).is_err()
        || validate_reply_suggestion_status_v1(&accepted_reply_suggestion_status_v1()).is_err()
    {
        return Err(ReplySuggestionPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_source_result(
    input: &ReplySuggestionSourceResultV1,
) -> Result<(), ReplySuggestionPersistenceErrorV1> {
    if materialization(input).is_err()
        || !valid_identity(&input.logical_owner_id)
        || !valid_timestamp(input.occurred_at_unix_millis)
        || !nonzero(&input.result_message_id)
        || !nonzero(&input.envelope_sha256)
        || !nonzero(&input.run_id)
        || !matches!(
            input.transition,
            ReplySuggestionTransitionV1::SourcePrepared { .. }
                | ReplySuggestionTransitionV1::Reject(
                    ReplySuggestionRejectionCodeV1::SourceRejected
                        | ReplySuggestionRejectionCodeV1::InvalidRequest
                        | ReplySuggestionRejectionCodeV1::Policy
                )
        )
    {
        return Err(ReplySuggestionPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn materialization(
    input: &ReplySuggestionSourceResultV1,
) -> Result<Option<(&[u8], &ReplySuggestionBlobCleanupV1)>, ReplySuggestionPersistenceErrorV1> {
    match (
        &input.transition,
        input.inference_request_bytes.as_deref(),
        input.source_cleanup.as_ref(),
    ) {
        (ReplySuggestionTransitionV1::SourcePrepared { .. }, Some(request), Some(cleanup))
            if valid_materialization(request, cleanup) =>
        {
            Ok(Some((request, cleanup)))
        }
        (ReplySuggestionTransitionV1::Reject(_), None, None) => Ok(None),
        _ => Err(ReplySuggestionPersistenceErrorV1::InvalidInput),
    }
}

fn valid_materialization(request: &[u8], cleanup: &ReplySuggestionBlobCleanupV1) -> bool {
    !request.is_empty()
        && request.len() <= REPLY_SUGGESTION_MAX_INFERENCE_REQUEST_BYTES_V1
        && valid_cleanup(cleanup)
}

fn valid_cleanup(cleanup: &ReplySuggestionBlobCleanupV1) -> bool {
    nonzero(&cleanup.reference_id)
        && (1..=256 * 1024).contains(&cleanup.declared_bytes)
        && nonzero(&cleanup.sha256)
        && !cleanup.custody_proof.is_empty()
        && cleanup.custody_proof.len() <= REPLY_SUGGESTION_MAX_CUSTODY_PROOF_BYTES_V1
}

fn valid_materialization_state(run: &PersistedReplySuggestionRunV1) -> bool {
    let materialization = run
        .inference_request_bytes
        .as_deref()
        .zip(run.source_cleanup.as_ref());
    if materialization.is_some_and(|(request, cleanup)| !valid_materialization(request, cleanup))
        || run.inference_request_bytes.is_some() != run.source_cleanup.is_some()
        || run
            .cleanup_completed_at_unix_millis
            .is_some_and(|value| value < run.created_at_unix_millis)
    {
        return false;
    }
    match run.status.state {
        ReplySuggestionStateV1::Accepted | ReplySuggestionStateV1::PreparingSource => {
            materialization.is_none() && run.cleanup_completed_at_unix_millis.is_none()
        }
        ReplySuggestionStateV1::AwaitingInference => {
            materialization.is_some() && run.cleanup_completed_at_unix_millis.is_none()
        }
        ReplySuggestionStateV1::Ready => {
            (materialization.is_some() && run.cleanup_completed_at_unix_millis.is_none())
                || (materialization.is_none() && run.cleanup_completed_at_unix_millis.is_some())
        }
        ReplySuggestionStateV1::Rejected if run.status.source_evidence_id.is_none() => {
            materialization.is_none() && run.cleanup_completed_at_unix_millis.is_none()
        }
        ReplySuggestionStateV1::Rejected => {
            (materialization.is_some() && run.cleanup_completed_at_unix_millis.is_none())
                || (materialization.is_none() && run.cleanup_completed_at_unix_millis.is_some())
        }
    }
}

fn state_code(value: ReplySuggestionStateV1) -> i16 {
    match value {
        ReplySuggestionStateV1::Accepted => 1,
        ReplySuggestionStateV1::PreparingSource => 2,
        ReplySuggestionStateV1::AwaitingInference => 3,
        ReplySuggestionStateV1::Ready => 4,
        ReplySuggestionStateV1::Rejected => 5,
    }
}

pub(crate) fn state_from_code(
    value: i16,
) -> Result<ReplySuggestionStateV1, ReplySuggestionPersistenceErrorV1> {
    match value {
        1 => Ok(ReplySuggestionStateV1::Accepted),
        2 => Ok(ReplySuggestionStateV1::PreparingSource),
        3 => Ok(ReplySuggestionStateV1::AwaitingInference),
        4 => Ok(ReplySuggestionStateV1::Ready),
        5 => Ok(ReplySuggestionStateV1::Rejected),
        _ => Err(ReplySuggestionPersistenceErrorV1::InvalidRow),
    }
}

fn tone_from_code(value: i16) -> Result<ReplySuggestionToneV1, ReplySuggestionPersistenceErrorV1> {
    match value {
        1 => Ok(ReplySuggestionToneV1::Professional),
        2 => Ok(ReplySuggestionToneV1::Friendly),
        3 => Ok(ReplySuggestionToneV1::Concise),
        4 => Ok(ReplySuggestionToneV1::Formal),
        _ => Err(ReplySuggestionPersistenceErrorV1::InvalidRow),
    }
}

fn language_from_code(
    value: i16,
) -> Result<ReplySuggestionLanguageV1, ReplySuggestionPersistenceErrorV1> {
    match value {
        1 => Ok(ReplySuggestionLanguageV1::Source),
        2 => Ok(ReplySuggestionLanguageV1::English),
        3 => Ok(ReplySuggestionLanguageV1::Russian),
        4 => Ok(ReplySuggestionLanguageV1::Spanish),
        _ => Err(ReplySuggestionPersistenceErrorV1::InvalidRow),
    }
}

fn completeness_code(value: ReplySuggestionCompletenessV1) -> i16 {
    match value {
        ReplySuggestionCompletenessV1::Complete => 1,
        ReplySuggestionCompletenessV1::Partial => 2,
    }
}

fn completeness_from_code(
    value: i16,
) -> Result<ReplySuggestionCompletenessV1, ReplySuggestionPersistenceErrorV1> {
    match value {
        1 => Ok(ReplySuggestionCompletenessV1::Complete),
        2 => Ok(ReplySuggestionCompletenessV1::Partial),
        _ => Err(ReplySuggestionPersistenceErrorV1::InvalidRow),
    }
}

fn optional_rejection_from_code(
    value: Option<i16>,
) -> Result<Option<ReplySuggestionRejectionCodeV1>, ReplySuggestionPersistenceErrorV1> {
    value
        .map(|code| match code {
            1 => Ok(ReplySuggestionRejectionCodeV1::InvalidRequest),
            2 => Ok(ReplySuggestionRejectionCodeV1::SourceRejected),
            3 => Ok(ReplySuggestionRejectionCodeV1::InferenceRejected),
            4 => Ok(ReplySuggestionRejectionCodeV1::Policy),
            _ => Err(ReplySuggestionPersistenceErrorV1::InvalidRow),
        })
        .transpose()
}

fn column<T>(row: &PgRow, name: &'static str) -> Result<T, ReplySuggestionPersistenceErrorV1>
where
    T: for<'row> sqlx::Decode<'row, Postgres> + sqlx::Type<Postgres>,
{
    row.try_get(name)
        .map_err(|_| ReplySuggestionPersistenceErrorV1::InvalidRow)
}

fn array16(value: Vec<u8>) -> Result<[u8; 16], ReplySuggestionPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| ReplySuggestionPersistenceErrorV1::InvalidRow)
}

fn array32(value: Vec<u8>) -> Result<[u8; 32], ReplySuggestionPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| ReplySuggestionPersistenceErrorV1::InvalidRow)
}

fn optional_array16(
    value: Option<Vec<u8>>,
) -> Result<Option<[u8; 16]>, ReplySuggestionPersistenceErrorV1> {
    value.map(array16).transpose()
}

fn optional_array32(
    value: Option<Vec<u8>>,
) -> Result<Option<[u8; 32]>, ReplySuggestionPersistenceErrorV1> {
    value.map(array32).transpose()
}

fn signed(value: u64) -> Result<i64, ReplySuggestionPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| ReplySuggestionPersistenceErrorV1::InvalidInput)
}

fn optional_signed(value: Option<u64>) -> Result<Option<i64>, ReplySuggestionPersistenceErrorV1> {
    value.map(signed).transpose()
}

fn positive_u64(value: i64) -> Result<u64, ReplySuggestionPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(ReplySuggestionPersistenceErrorV1::InvalidRow)
}

fn optional_positive_u64(
    value: Option<i64>,
) -> Result<Option<u64>, ReplySuggestionPersistenceErrorV1> {
    value.map(positive_u64).transpose()
}

fn storage_error(_: sqlx::Error) -> ReplySuggestionPersistenceErrorV1 {
    ReplySuggestionPersistenceErrorV1::StorageUnavailable
}
