use makosh_attachment_translation_core::{
    AttachmentTranslationArtifactV1, AttachmentTranslationCompletenessV1,
    AttachmentTranslationDetectedLanguageV1, AttachmentTranslationDraftV1,
    AttachmentTranslationLanguageV1, AttachmentTranslationPendingResultV1,
    AttachmentTranslationRejectionCodeV1, AttachmentTranslationStateV1,
    AttachmentTranslationStatusV1, AttachmentTranslationTransitionV1,
    accepted_attachment_translation_status_v1, transition_attachment_translation_v1,
    validate_attachment_translation_draft_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions, PgRow},
};

use crate::{
    model::{
        ATTACHMENT_TRANSLATION_MAX_CUSTODY_PROOF_BYTES_V1,
        ATTACHMENT_TRANSLATION_MAX_EVENT_BYTES_V1,
        ATTACHMENT_TRANSLATION_MAX_INFERENCE_REQUEST_BYTES_V1,
        ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1, ATTACHMENT_TRANSLATION_RECOVERY_LIMIT_V1,
        AttachmentTranslationInboxResultV1, AttachmentTranslationInferenceResultV1,
        AttachmentTranslationMaterializationResultV1, AttachmentTranslationPersistenceErrorV1,
        AttachmentTranslationSourceAuthorityV1, AttachmentTranslationSourceResultV1,
        CreateAttachmentTranslationOutcomeV1, CreateAttachmentTranslationRunV1,
        PersistedAttachmentTranslationRunV1, nonzero, rejection_code, request_fingerprint,
        target_language_code, valid_identity, valid_timestamp,
    },
    realtime::insert_realtime_transition,
};

#[derive(Clone)]
pub struct AttachmentTranslationPersistenceV1 {
    pub(crate) pool: PgPool,
}

impl AttachmentTranslationPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, AttachmentTranslationPersistenceErrorV1> {
        if pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || database_id.is_empty()
            || database_id != binding.identity().database_id()
            || binding.access().runtime_principal().is_empty()
        {
            return Err(AttachmentTranslationPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| AttachmentTranslationPersistenceErrorV1::StorageUnavailable)?;
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
    ) -> Result<(), AttachmentTranslationPersistenceErrorV1> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(storage_error)
    }

    pub async fn create_run(
        &self,
        input: CreateAttachmentTranslationRunV1,
    ) -> Result<CreateAttachmentTranslationOutcomeV1, AttachmentTranslationPersistenceErrorV1> {
        validate_create(&input)?;
        let fingerprint = request_fingerprint(&input.draft);
        let awaiting_source = transition_attachment_translation_v1(
            &accepted_attachment_translation_status_v1(),
            AttachmentTranslationTransitionV1::RequestSource,
        )
        .map_err(|_| AttachmentTranslationPersistenceErrorV1::InvalidTransition)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.attachment_translation_runs (
               logical_owner_id, run_id, operation_id, request_fingerprint,
               source_extraction_run_id, expected_source_revision, target_language,
               state, state_revision, created_at_unix_millis, updated_at_unix_millis
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10)
             ON CONFLICT (logical_owner_id, operation_id) DO NOTHING",
        )
        .bind(&input.logical_owner_id)
        .bind(input.draft.run_id.as_slice())
        .bind(input.draft.operation_id.as_slice())
        .bind(fingerprint.as_slice())
        .bind(input.draft.source_extraction_run_id.as_slice())
        .bind(signed(input.draft.expected_source_revision)?)
        .bind(i16::from(target_language_code(input.draft.target_language)))
        .bind(state_code(awaiting_source.state))
        .bind(signed(awaiting_source.state_revision)?)
        .bind(input.created_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if inserted == 1 {
            sqlx::query(
                "INSERT INTO makosh_data.attachment_translation_outbox (
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
            return Err(AttachmentTranslationPersistenceErrorV1::RequestConflict);
        }
        Ok(if inserted == 1 {
            CreateAttachmentTranslationOutcomeV1::Created(persisted)
        } else {
            CreateAttachmentTranslationOutcomeV1::Existing(persisted)
        })
    }

    pub async fn load_run(
        &self,
        logical_owner_id: &str,
        run_id: &[u8; 16],
    ) -> Result<PersistedAttachmentTranslationRunV1, AttachmentTranslationPersistenceErrorV1> {
        if !valid_identity(logical_owner_id) || !nonzero(run_id) {
            return Err(AttachmentTranslationPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(SELECT_RUN)
            .bind(logical_owner_id)
            .bind(run_id.as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_error)?
            .ok_or(AttachmentTranslationPersistenceErrorV1::NotFound)?;
        persisted_from_row(row)
    }

    pub async fn load_recoverable_runs(
        &self,
        logical_owner_id: &str,
    ) -> Result<Vec<PersistedAttachmentTranslationRunV1>, AttachmentTranslationPersistenceErrorV1>
    {
        if !valid_identity(logical_owner_id) {
            return Err(AttachmentTranslationPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(SELECT_RECOVERABLE_RUNS)
            .bind(logical_owner_id)
            .bind(i64::from(ATTACHMENT_TRANSLATION_RECOVERY_LIMIT_V1))
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(persisted_from_row)
            .collect()
    }

    pub async fn persist_source_result(
        &self,
        input: AttachmentTranslationSourceResultV1,
    ) -> Result<AttachmentTranslationInboxResultV1, AttachmentTranslationPersistenceErrorV1> {
        validate_source_result(&input)?;
        self.persist_event_transition(EventTransitionWriteV1 {
            message_id: input.result_message_id,
            envelope_sha256: input.envelope_sha256,
            logical_owner_id: input.logical_owner_id,
            run_id: input.run_id,
            transition: input.transition,
            source_materialization: input
                .source_authority
                .zip(input.inference_request_bytes)
                .map(|(authority, request)| SourceMaterialization { authority, request }),
            artifact_fence: None,
            occurred_at_unix_millis: input.occurred_at_unix_millis,
        })
        .await
    }

    pub async fn persist_inference_result(
        &self,
        input: AttachmentTranslationInferenceResultV1,
    ) -> Result<AttachmentTranslationInboxResultV1, AttachmentTranslationPersistenceErrorV1> {
        validate_event_identity(
            &input.logical_owner_id,
            &input.run_id,
            &input.message_id,
            &input.envelope_sha256,
            input.occurred_at_unix_millis,
        )?;
        if !matches!(
            input.transition,
            AttachmentTranslationTransitionV1::InferenceCompleted(_)
                | AttachmentTranslationTransitionV1::Reject(
                    AttachmentTranslationRejectionCodeV1::InferenceRejected
                        | AttachmentTranslationRejectionCodeV1::ResultRejected
                        | AttachmentTranslationRejectionCodeV1::Policy
                )
        ) {
            return Err(AttachmentTranslationPersistenceErrorV1::InvalidInput);
        }
        self.persist_event_transition(EventTransitionWriteV1 {
            message_id: input.message_id,
            envelope_sha256: input.envelope_sha256,
            logical_owner_id: input.logical_owner_id,
            run_id: input.run_id,
            transition: input.transition,
            source_materialization: None,
            artifact_fence: None,
            occurred_at_unix_millis: input.occurred_at_unix_millis,
        })
        .await
    }

    pub async fn persist_materialization_result(
        &self,
        input: AttachmentTranslationMaterializationResultV1,
    ) -> Result<AttachmentTranslationInboxResultV1, AttachmentTranslationPersistenceErrorV1> {
        validate_event_identity(
            &input.logical_owner_id,
            &input.run_id,
            &input.message_id,
            &input.result_sha256,
            input.occurred_at_unix_millis,
        )?;
        if !matches!(
            &input.transition,
            AttachmentTranslationTransitionV1::ResultMaterialized { .. }
                | AttachmentTranslationTransitionV1::Reject(
                    AttachmentTranslationRejectionCodeV1::ResultRejected
                        | AttachmentTranslationRejectionCodeV1::Policy
                )
        ) {
            return Err(AttachmentTranslationPersistenceErrorV1::InvalidInput);
        }
        let artifact_fence = matches!(
            input.transition,
            AttachmentTranslationTransitionV1::ResultMaterialized { .. }
        )
        .then_some((input.runtime_generation, input.grant_epoch));
        if artifact_fence.is_some() && (input.runtime_generation == 0 || input.grant_epoch == 0) {
            return Err(AttachmentTranslationPersistenceErrorV1::InvalidInput);
        }
        self.persist_event_transition(EventTransitionWriteV1 {
            message_id: input.message_id,
            envelope_sha256: input.result_sha256,
            logical_owner_id: input.logical_owner_id,
            run_id: input.run_id,
            transition: input.transition,
            source_materialization: None,
            artifact_fence,
            occurred_at_unix_millis: input.occurred_at_unix_millis,
        })
        .await
    }

    pub async fn complete_source_cleanup(
        &self,
        logical_owner_id: &str,
        run_id: &[u8; 16],
        authority: &AttachmentTranslationSourceAuthorityV1,
        completed_at_unix_millis: i64,
    ) -> Result<PersistedAttachmentTranslationRunV1, AttachmentTranslationPersistenceErrorV1> {
        if !valid_identity(logical_owner_id)
            || !nonzero(run_id)
            || !valid_source_authority(authority)
            || !valid_timestamp(completed_at_unix_millis)
        {
            return Err(AttachmentTranslationPersistenceErrorV1::InvalidInput);
        }
        let updated = sqlx::query(
            "UPDATE makosh_data.attachment_translation_runs
             SET inference_request_bytes = NULL, source_reference_id = NULL,
                 source_declared_bytes = NULL, source_receipt_sha256 = NULL,
                 source_custody_proof = NULL, cleanup_completed_at_unix_millis = $1,
                 updated_at_unix_millis = $1
             WHERE logical_owner_id = $2 AND run_id = $3 AND state IN (5, 6)
               AND source_reference_id = $4 AND source_declared_bytes = $5
               AND source_receipt_sha256 = $6 AND source_custody_proof = $7
               AND cleanup_completed_at_unix_millis IS NULL
               AND updated_at_unix_millis <= $1",
        )
        .bind(completed_at_unix_millis)
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .bind(authority.reference_id.as_slice())
        .bind(signed(authority.declared_bytes)?)
        .bind(authority.sha256.as_slice())
        .bind(&authority.custody_proof)
        .execute(&self.pool)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if updated != 1 {
            return Err(AttachmentTranslationPersistenceErrorV1::RevisionConflict);
        }
        self.load_run(logical_owner_id, run_id).await
    }

    async fn persist_event_transition(
        &self,
        input: EventTransitionWriteV1,
    ) -> Result<AttachmentTranslationInboxResultV1, AttachmentTranslationPersistenceErrorV1> {
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        if inbox_duplicate(
            &mut transaction,
            &input.logical_owner_id,
            &input.message_id,
            &input.envelope_sha256,
            &input.run_id,
        )
        .await?
        {
            transaction.commit().await.map_err(storage_error)?;
            return self
                .load_run(&input.logical_owner_id, &input.run_id)
                .await
                .map(AttachmentTranslationInboxResultV1::Duplicate);
        }
        let current =
            load_for_update(&mut transaction, &input.logical_owner_id, &input.run_id).await?;
        let next = transition_attachment_translation_v1(&current.status, input.transition)
            .map_err(|_| AttachmentTranslationPersistenceErrorV1::InvalidTransition)?;
        persist_status(
            &mut transaction,
            StatusTransitionWriteV1 {
                logical_owner_id: &input.logical_owner_id,
                run_id: &input.run_id,
                current_revision: current.status.state_revision,
                next: &next,
                source_materialization: input.source_materialization.as_ref(),
                artifact_fence: input.artifact_fence,
                occurred_at_unix_millis: input.occurred_at_unix_millis,
            },
        )
        .await?;
        sqlx::query(
            "INSERT INTO makosh_data.attachment_translation_inbox (
               logical_owner_id, message_id, envelope_sha256, run_id,
               processed_at_unix_millis
             ) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.message_id.as_slice())
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
            .map(AttachmentTranslationInboxResultV1::Applied)
    }

    async fn load_by_operation(
        &self,
        logical_owner_id: &str,
        operation_id: &[u8; 16],
    ) -> Result<PersistedAttachmentTranslationRunV1, AttachmentTranslationPersistenceErrorV1> {
        let row = sqlx::query(SELECT_RUN_BY_OPERATION)
            .bind(logical_owner_id)
            .bind(operation_id.as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_error)?
            .ok_or(AttachmentTranslationPersistenceErrorV1::NotFound)?;
        persisted_from_row(row)
    }
}

#[derive(Clone)]
struct SourceMaterialization {
    authority: AttachmentTranslationSourceAuthorityV1,
    request: Vec<u8>,
}

struct EventTransitionWriteV1 {
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
    logical_owner_id: String,
    run_id: [u8; 16],
    transition: AttachmentTranslationTransitionV1,
    source_materialization: Option<SourceMaterialization>,
    artifact_fence: Option<(u64, u64)>,
    occurred_at_unix_millis: i64,
}

struct StatusTransitionWriteV1<'a> {
    logical_owner_id: &'a str,
    run_id: &'a [u8; 16],
    current_revision: u64,
    next: &'a AttachmentTranslationStatusV1,
    source_materialization: Option<&'a SourceMaterialization>,
    artifact_fence: Option<(u64, u64)>,
    occurred_at_unix_millis: i64,
}

macro_rules! select_run {
    ($tail:literal) => {
        concat!(
            "SELECT logical_owner_id, run_id, operation_id, request_fingerprint, ",
            "source_extraction_run_id, expected_source_revision, target_language, state, ",
            "state_revision, source_sha256, inference_request_digest, inference_request_bytes, ",
            "source_reference_id, source_declared_bytes, source_receipt_sha256, ",
            "source_custody_proof, cleanup_completed_at_unix_millis, ",
            "pending_translated_sha256, pending_translated_size_bytes, ",
            "pending_detected_source_language, pending_target_language, pending_completeness, ",
            "pending_confidence_basis_points, artifact_id, artifact_translated_sha256, ",
            "artifact_translated_size_bytes, artifact_detected_source_language, ",
            "artifact_target_language, artifact_completeness, ",
            "artifact_confidence_basis_points, rejection_code, created_at_unix_millis, ",
            "updated_at_unix_millis FROM makosh_data.attachment_translation_runs ",
            $tail
        )
    };
}

const SELECT_RUN: &str = select_run!("WHERE logical_owner_id = $1 AND run_id = $2");
const SELECT_RUN_FOR_UPDATE: &str =
    select_run!("WHERE logical_owner_id = $1 AND run_id = $2 FOR UPDATE");
const SELECT_RUN_BY_OPERATION: &str =
    select_run!("WHERE logical_owner_id = $1 AND operation_id = $2");
const SELECT_RECOVERABLE_RUNS: &str = select_run!(
    "WHERE logical_owner_id = $1 AND state IN (2, 3, 4) \
     ORDER BY state_revision, run_id LIMIT $2"
);

async fn inbox_duplicate(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    message_id: &[u8; 16],
    envelope_sha256: &[u8; 32],
    run_id: &[u8; 16],
) -> Result<bool, AttachmentTranslationPersistenceErrorV1> {
    let Some(row) = sqlx::query(
        "SELECT envelope_sha256, run_id FROM makosh_data.attachment_translation_inbox
         WHERE logical_owner_id = $1 AND message_id = $2",
    )
    .bind(logical_owner_id)
    .bind(message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_error)?
    else {
        return Ok(false);
    };
    let existing_hash: Vec<u8> = column(&row, "envelope_sha256")?;
    let existing_run: Vec<u8> = column(&row, "run_id")?;
    if existing_hash.as_slice() != envelope_sha256 || existing_run.as_slice() != run_id {
        return Err(AttachmentTranslationPersistenceErrorV1::InboxConflict);
    }
    Ok(true)
}

async fn load_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: &[u8; 16],
) -> Result<PersistedAttachmentTranslationRunV1, AttachmentTranslationPersistenceErrorV1> {
    let row = sqlx::query(SELECT_RUN_FOR_UPDATE)
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_error)?
        .ok_or(AttachmentTranslationPersistenceErrorV1::NotFound)?;
    persisted_from_row(row)
}

async fn persist_status(
    transaction: &mut Transaction<'_, Postgres>,
    input: StatusTransitionWriteV1<'_>,
) -> Result<(), AttachmentTranslationPersistenceErrorV1> {
    if !valid_status(input.next) || !valid_timestamp(input.occurred_at_unix_millis) {
        return Err(AttachmentTranslationPersistenceErrorV1::InvalidTransition);
    }
    let pending = input.next.pending_result.as_ref();
    let artifact = input.next.artifact.as_ref();
    let updated = sqlx::query(
        "UPDATE makosh_data.attachment_translation_runs SET
           state=$1, state_revision=$2, source_sha256=$3, inference_request_digest=$4,
           inference_request_bytes=COALESCE($5,inference_request_bytes),
           source_reference_id=COALESCE($6,source_reference_id),
           source_declared_bytes=COALESCE($7,source_declared_bytes),
           source_receipt_sha256=COALESCE($8,source_receipt_sha256),
           source_custody_proof=COALESCE($9,source_custody_proof),
           pending_translated_sha256=$10, pending_translated_size_bytes=$11,
           pending_detected_source_language=$12, pending_target_language=$13,
           pending_completeness=$14, pending_confidence_basis_points=$15,
           artifact_id=$16, artifact_translated_sha256=$17,
           artifact_translated_size_bytes=$18, artifact_detected_source_language=$19,
           artifact_target_language=$20, artifact_completeness=$21,
           artifact_confidence_basis_points=$22, rejection_code=$23,
           artifact_runtime_generation=$24, artifact_grant_epoch=$25,
           updated_at_unix_millis=$26
         WHERE logical_owner_id=$27 AND run_id=$28 AND state_revision=$29",
    )
    .bind(state_code(input.next.state))
    .bind(signed(input.next.state_revision)?)
    .bind(input.next.source_sha256.map(|value| value.to_vec()))
    .bind(
        input
            .next
            .inference_request_digest
            .map(|value| value.to_vec()),
    )
    .bind(
        input
            .source_materialization
            .map(|value| value.request.as_slice()),
    )
    .bind(
        input
            .source_materialization
            .map(|value| value.authority.reference_id.to_vec()),
    )
    .bind(
        input
            .source_materialization
            .map(|value| signed(value.authority.declared_bytes))
            .transpose()?,
    )
    .bind(
        input
            .source_materialization
            .map(|value| value.authority.sha256.to_vec()),
    )
    .bind(
        input
            .source_materialization
            .map(|value| value.authority.custody_proof.as_slice()),
    )
    .bind(pending.map(|value| value.translated_sha256.to_vec()))
    .bind(
        pending
            .map(|value| signed(value.translated_size_bytes))
            .transpose()?,
    )
    .bind(pending.map(|value| detected_language_code(value.detected_source_language)))
    .bind(pending.map(|value| i16::from(target_language_code(value.target_language))))
    .bind(pending.map(|value| completeness_code(value.completeness)))
    .bind(
        pending
            .map(|value| i32::try_from(value.confidence_basis_points))
            .transpose()
            .map_err(|_| AttachmentTranslationPersistenceErrorV1::InvalidInput)?,
    )
    .bind(artifact.map(|value| value.artifact_id.to_vec()))
    .bind(artifact.map(|value| value.translated_sha256.to_vec()))
    .bind(
        artifact
            .map(|value| signed(value.translated_size_bytes))
            .transpose()?,
    )
    .bind(artifact.map(|value| detected_language_code(value.detected_source_language)))
    .bind(artifact.map(|value| i16::from(target_language_code(value.target_language))))
    .bind(artifact.map(|value| completeness_code(value.completeness)))
    .bind(
        artifact
            .map(|value| i32::try_from(value.confidence_basis_points))
            .transpose()
            .map_err(|_| AttachmentTranslationPersistenceErrorV1::InvalidInput)?,
    )
    .bind(input.next.rejection.map(rejection_code))
    .bind(
        input
            .artifact_fence
            .map(|(runtime_generation, _)| signed(runtime_generation))
            .transpose()?,
    )
    .bind(
        input
            .artifact_fence
            .map(|(_, grant_epoch)| signed(grant_epoch))
            .transpose()?,
    )
    .bind(input.occurred_at_unix_millis)
    .bind(input.logical_owner_id)
    .bind(input.run_id.as_slice())
    .bind(signed(input.current_revision)?)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?
    .rows_affected();
    if updated == 1 {
        Ok(())
    } else {
        Err(AttachmentTranslationPersistenceErrorV1::RevisionConflict)
    }
}

fn persisted_from_row(
    row: PgRow,
) -> Result<PersistedAttachmentTranslationRunV1, AttachmentTranslationPersistenceErrorV1> {
    let source_sha256 = optional_array32(column(&row, "source_sha256")?)?;
    let inference_request_digest = optional_array32(column(&row, "inference_request_digest")?)?;
    let pending = pending_from_row(&row, source_sha256, inference_request_digest)?;
    let artifact = artifact_from_row(&row)?;
    let status = AttachmentTranslationStatusV1 {
        state: state_from_code(column(&row, "state")?)?,
        state_revision: positive_u64(column(&row, "state_revision")?)?,
        source_sha256,
        inference_request_digest,
        pending_result: pending,
        artifact,
        rejection: optional_rejection_from_code(column(&row, "rejection_code")?)?,
    };
    if !valid_status(&status) {
        return Err(AttachmentTranslationPersistenceErrorV1::InvalidRow);
    }
    let inference_request_bytes: Option<Vec<u8>> = column(&row, "inference_request_bytes")?;
    let source_authority = source_authority_from_row(&row)?;
    let persisted = PersistedAttachmentTranslationRunV1 {
        logical_owner_id: column(&row, "logical_owner_id")?,
        draft: AttachmentTranslationDraftV1 {
            run_id: array16(column(&row, "run_id")?)?,
            operation_id: array16(column(&row, "operation_id")?)?,
            source_extraction_run_id: array16(column(&row, "source_extraction_run_id")?)?,
            expected_source_revision: positive_u64(column(&row, "expected_source_revision")?)?,
            target_language: target_language_from_code(column(&row, "target_language")?)?,
        },
        request_fingerprint: array32(column(&row, "request_fingerprint")?)?,
        status,
        inference_request_bytes,
        source_authority,
        cleanup_completed_at_unix_millis: column(&row, "cleanup_completed_at_unix_millis")?,
        created_at_unix_millis: column(&row, "created_at_unix_millis")?,
        updated_at_unix_millis: column(&row, "updated_at_unix_millis")?,
    };
    if !valid_persisted(&persisted) {
        return Err(AttachmentTranslationPersistenceErrorV1::InvalidRow);
    }
    Ok(persisted)
}

fn pending_from_row(
    row: &PgRow,
    source_sha256: Option<[u8; 32]>,
    request_digest: Option<[u8; 32]>,
) -> Result<Option<AttachmentTranslationPendingResultV1>, AttachmentTranslationPersistenceErrorV1> {
    let sha: Option<Vec<u8>> = column(row, "pending_translated_sha256")?;
    let size: Option<i64> = column(row, "pending_translated_size_bytes")?;
    let detected: Option<i16> = column(row, "pending_detected_source_language")?;
    let target: Option<i16> = column(row, "pending_target_language")?;
    let completeness: Option<i16> = column(row, "pending_completeness")?;
    let confidence: Option<i32> = column(row, "pending_confidence_basis_points")?;
    match (
        sha,
        size,
        detected,
        target,
        completeness,
        confidence,
        request_digest,
        source_sha256,
    ) {
        (
            Some(sha),
            Some(size),
            Some(detected),
            Some(target),
            Some(completeness),
            Some(confidence),
            Some(request_digest),
            Some(source_sha256),
        ) => Ok(Some(AttachmentTranslationPendingResultV1 {
            translated_sha256: array32(sha)?,
            translated_size_bytes: positive_u64(size)?,
            detected_source_language: detected_language_from_code(detected)?,
            target_language: target_language_from_code(target)?,
            completeness: completeness_from_code(completeness)?,
            confidence_basis_points: u32::try_from(confidence)
                .map_err(|_| AttachmentTranslationPersistenceErrorV1::InvalidRow)?,
            inference_request_digest: request_digest,
            source_sha256,
        })),
        (None, None, None, None, None, None, _, _) => Ok(None),
        _ => Err(AttachmentTranslationPersistenceErrorV1::InvalidRow),
    }
}

fn artifact_from_row(
    row: &PgRow,
) -> Result<Option<AttachmentTranslationArtifactV1>, AttachmentTranslationPersistenceErrorV1> {
    let id: Option<Vec<u8>> = column(row, "artifact_id")?;
    let sha: Option<Vec<u8>> = column(row, "artifact_translated_sha256")?;
    let size: Option<i64> = column(row, "artifact_translated_size_bytes")?;
    let detected: Option<i16> = column(row, "artifact_detected_source_language")?;
    let target: Option<i16> = column(row, "artifact_target_language")?;
    let completeness: Option<i16> = column(row, "artifact_completeness")?;
    let confidence: Option<i32> = column(row, "artifact_confidence_basis_points")?;
    match (id, sha, size, detected, target, completeness, confidence) {
        (
            Some(id),
            Some(sha),
            Some(size),
            Some(detected),
            Some(target),
            Some(completeness),
            Some(confidence),
        ) => Ok(Some(AttachmentTranslationArtifactV1 {
            artifact_id: array16(id)?,
            translated_sha256: array32(sha)?,
            translated_size_bytes: positive_u64(size)?,
            detected_source_language: detected_language_from_code(detected)?,
            target_language: target_language_from_code(target)?,
            completeness: completeness_from_code(completeness)?,
            confidence_basis_points: u32::try_from(confidence)
                .map_err(|_| AttachmentTranslationPersistenceErrorV1::InvalidRow)?,
        })),
        (None, None, None, None, None, None, None) => Ok(None),
        _ => Err(AttachmentTranslationPersistenceErrorV1::InvalidRow),
    }
}

fn source_authority_from_row(
    row: &PgRow,
) -> Result<Option<AttachmentTranslationSourceAuthorityV1>, AttachmentTranslationPersistenceErrorV1>
{
    let reference: Option<Vec<u8>> = column(row, "source_reference_id")?;
    let declared: Option<i64> = column(row, "source_declared_bytes")?;
    let sha: Option<Vec<u8>> = column(row, "source_receipt_sha256")?;
    let proof: Option<Vec<u8>> = column(row, "source_custody_proof")?;
    match (reference, declared, sha, proof) {
        (Some(reference), Some(declared), Some(sha), Some(custody_proof)) => {
            let value = AttachmentTranslationSourceAuthorityV1 {
                reference_id: array16(reference)?,
                declared_bytes: positive_u64(declared)?,
                sha256: array32(sha)?,
                custody_proof,
            };
            valid_source_authority(&value)
                .then_some(Some(value))
                .ok_or(AttachmentTranslationPersistenceErrorV1::InvalidRow)
        }
        (None, None, None, None) => Ok(None),
        _ => Err(AttachmentTranslationPersistenceErrorV1::InvalidRow),
    }
}

fn validate_create(
    input: &CreateAttachmentTranslationRunV1,
) -> Result<(), AttachmentTranslationPersistenceErrorV1> {
    if !valid_identity(&input.logical_owner_id)
        || !valid_timestamp(input.created_at_unix_millis)
        || !nonzero(&input.source_prepare_message_id)
        || !nonzero(&input.source_prepare_envelope_sha256)
        || input.source_prepare_envelope_bytes.is_empty()
        || input.source_prepare_envelope_bytes.len() > ATTACHMENT_TRANSLATION_MAX_EVENT_BYTES_V1
        || validate_attachment_translation_draft_v1(&input.draft).is_err()
    {
        return Err(AttachmentTranslationPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

fn validate_source_result(
    input: &AttachmentTranslationSourceResultV1,
) -> Result<(), AttachmentTranslationPersistenceErrorV1> {
    validate_event_identity(
        &input.logical_owner_id,
        &input.run_id,
        &input.result_message_id,
        &input.envelope_sha256,
        input.occurred_at_unix_millis,
    )?;
    match (
        &input.transition,
        &input.inference_request_bytes,
        &input.source_authority,
    ) {
        (
            AttachmentTranslationTransitionV1::SourcePrepared {
                source_sha256,
                source_size_bytes,
                ..
            },
            Some(request),
            Some(authority),
        ) if *source_sha256 == authority.sha256
            && *source_size_bytes == authority.declared_bytes
            && valid_materialization(request, authority) =>
        {
            Ok(())
        }
        (
            AttachmentTranslationTransitionV1::Reject(
                AttachmentTranslationRejectionCodeV1::SourceRejected
                | AttachmentTranslationRejectionCodeV1::InvalidRequest
                | AttachmentTranslationRejectionCodeV1::Policy,
            ),
            None,
            None,
        ) => Ok(()),
        _ => Err(AttachmentTranslationPersistenceErrorV1::InvalidInput),
    }
}

fn validate_event_identity(
    logical_owner_id: &str,
    run_id: &[u8; 16],
    message_id: &[u8; 16],
    sha256: &[u8; 32],
    occurred_at_unix_millis: i64,
) -> Result<(), AttachmentTranslationPersistenceErrorV1> {
    if valid_identity(logical_owner_id)
        && nonzero(run_id)
        && nonzero(message_id)
        && nonzero(sha256)
        && valid_timestamp(occurred_at_unix_millis)
    {
        Ok(())
    } else {
        Err(AttachmentTranslationPersistenceErrorV1::InvalidInput)
    }
}

fn valid_materialization(
    request: &[u8],
    authority: &AttachmentTranslationSourceAuthorityV1,
) -> bool {
    !request.is_empty()
        && request.len() <= ATTACHMENT_TRANSLATION_MAX_INFERENCE_REQUEST_BYTES_V1
        && valid_source_authority(authority)
}

fn valid_source_authority(authority: &AttachmentTranslationSourceAuthorityV1) -> bool {
    nonzero(&authority.reference_id)
        && (1..=ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1).contains(&authority.declared_bytes)
        && nonzero(&authority.sha256)
        && !authority.custody_proof.is_empty()
        && authority.custody_proof.len() <= ATTACHMENT_TRANSLATION_MAX_CUSTODY_PROOF_BYTES_V1
}

fn valid_persisted(run: &PersistedAttachmentTranslationRunV1) -> bool {
    if !valid_identity(&run.logical_owner_id)
        || !valid_timestamp(run.created_at_unix_millis)
        || run.updated_at_unix_millis < run.created_at_unix_millis
        || validate_attachment_translation_draft_v1(&run.draft).is_err()
        || run.inference_request_bytes.is_some() != run.source_authority.is_some()
        || run
            .inference_request_bytes
            .as_ref()
            .zip(run.source_authority.as_ref())
            .is_some_and(|(request, authority)| !valid_materialization(request, authority))
        || run
            .cleanup_completed_at_unix_millis
            .is_some_and(|value| value < run.created_at_unix_millis)
    {
        return false;
    }
    match run.status.state {
        AttachmentTranslationStateV1::Accepted | AttachmentTranslationStateV1::AwaitingSource => {
            run.source_authority.is_none() && run.cleanup_completed_at_unix_millis.is_none()
        }
        AttachmentTranslationStateV1::AwaitingInference
        | AttachmentTranslationStateV1::MaterializingResult => {
            run.source_authority.is_some() && run.cleanup_completed_at_unix_millis.is_none()
        }
        AttachmentTranslationStateV1::Ready => {
            run.source_authority.is_some() != run.cleanup_completed_at_unix_millis.is_some()
        }
        AttachmentTranslationStateV1::Rejected if run.status.source_sha256.is_none() => {
            run.source_authority.is_none() && run.cleanup_completed_at_unix_millis.is_none()
        }
        AttachmentTranslationStateV1::Rejected => {
            run.source_authority.is_some() != run.cleanup_completed_at_unix_millis.is_some()
        }
    }
}

fn valid_status(status: &AttachmentTranslationStatusV1) -> bool {
    if status.state_revision == 0 {
        return false;
    }
    match status.state {
        AttachmentTranslationStateV1::Accepted | AttachmentTranslationStateV1::AwaitingSource => {
            status.source_sha256.is_none()
                && status.inference_request_digest.is_none()
                && status.pending_result.is_none()
                && status.artifact.is_none()
                && status.rejection.is_none()
        }
        AttachmentTranslationStateV1::AwaitingInference => {
            status.source_sha256.is_some()
                && status.inference_request_digest.is_some()
                && status.pending_result.is_none()
                && status.artifact.is_none()
                && status.rejection.is_none()
        }
        AttachmentTranslationStateV1::MaterializingResult => {
            status.source_sha256.is_some()
                && status.inference_request_digest.is_some()
                && status.pending_result.is_some()
                && status.artifact.is_none()
                && status.rejection.is_none()
        }
        AttachmentTranslationStateV1::Ready => {
            status.source_sha256.is_some()
                && status.inference_request_digest.is_some()
                && status.pending_result.is_none()
                && status.artifact.is_some()
                && status.rejection.is_none()
        }
        AttachmentTranslationStateV1::Rejected => {
            status.pending_result.is_none()
                && status.artifact.is_none()
                && status.rejection.is_some()
        }
    }
}

fn state_code(value: AttachmentTranslationStateV1) -> i16 {
    match value {
        AttachmentTranslationStateV1::Accepted => 1,
        AttachmentTranslationStateV1::AwaitingSource => 2,
        AttachmentTranslationStateV1::AwaitingInference => 3,
        AttachmentTranslationStateV1::MaterializingResult => 4,
        AttachmentTranslationStateV1::Ready => 5,
        AttachmentTranslationStateV1::Rejected => 6,
    }
}

pub(crate) fn state_from_code(
    value: i16,
) -> Result<AttachmentTranslationStateV1, AttachmentTranslationPersistenceErrorV1> {
    match value {
        1 => Ok(AttachmentTranslationStateV1::Accepted),
        2 => Ok(AttachmentTranslationStateV1::AwaitingSource),
        3 => Ok(AttachmentTranslationStateV1::AwaitingInference),
        4 => Ok(AttachmentTranslationStateV1::MaterializingResult),
        5 => Ok(AttachmentTranslationStateV1::Ready),
        6 => Ok(AttachmentTranslationStateV1::Rejected),
        _ => Err(AttachmentTranslationPersistenceErrorV1::InvalidRow),
    }
}

fn detected_language_code(value: AttachmentTranslationDetectedLanguageV1) -> i16 {
    match value {
        AttachmentTranslationDetectedLanguageV1::Unknown => 1,
        AttachmentTranslationDetectedLanguageV1::English => 2,
        AttachmentTranslationDetectedLanguageV1::Russian => 3,
        AttachmentTranslationDetectedLanguageV1::Spanish => 4,
    }
}
fn detected_language_from_code(
    value: i16,
) -> Result<AttachmentTranslationDetectedLanguageV1, AttachmentTranslationPersistenceErrorV1> {
    match value {
        1 => Ok(AttachmentTranslationDetectedLanguageV1::Unknown),
        2 => Ok(AttachmentTranslationDetectedLanguageV1::English),
        3 => Ok(AttachmentTranslationDetectedLanguageV1::Russian),
        4 => Ok(AttachmentTranslationDetectedLanguageV1::Spanish),
        _ => Err(AttachmentTranslationPersistenceErrorV1::InvalidRow),
    }
}
fn target_language_from_code(
    value: i16,
) -> Result<AttachmentTranslationLanguageV1, AttachmentTranslationPersistenceErrorV1> {
    match value {
        1 => Ok(AttachmentTranslationLanguageV1::English),
        2 => Ok(AttachmentTranslationLanguageV1::Russian),
        3 => Ok(AttachmentTranslationLanguageV1::Spanish),
        _ => Err(AttachmentTranslationPersistenceErrorV1::InvalidRow),
    }
}
fn completeness_code(value: AttachmentTranslationCompletenessV1) -> i16 {
    match value {
        AttachmentTranslationCompletenessV1::Complete => 1,
        AttachmentTranslationCompletenessV1::Partial => 2,
    }
}
fn completeness_from_code(
    value: i16,
) -> Result<AttachmentTranslationCompletenessV1, AttachmentTranslationPersistenceErrorV1> {
    match value {
        1 => Ok(AttachmentTranslationCompletenessV1::Complete),
        2 => Ok(AttachmentTranslationCompletenessV1::Partial),
        _ => Err(AttachmentTranslationPersistenceErrorV1::InvalidRow),
    }
}
fn optional_rejection_from_code(
    value: Option<i16>,
) -> Result<Option<AttachmentTranslationRejectionCodeV1>, AttachmentTranslationPersistenceErrorV1> {
    value
        .map(|code| match code {
            1 => Ok(AttachmentTranslationRejectionCodeV1::InvalidRequest),
            2 => Ok(AttachmentTranslationRejectionCodeV1::SourceRejected),
            3 => Ok(AttachmentTranslationRejectionCodeV1::InferenceRejected),
            4 => Ok(AttachmentTranslationRejectionCodeV1::ResultRejected),
            5 => Ok(AttachmentTranslationRejectionCodeV1::Policy),
            _ => Err(AttachmentTranslationPersistenceErrorV1::InvalidRow),
        })
        .transpose()
}

fn column<T>(row: &PgRow, name: &'static str) -> Result<T, AttachmentTranslationPersistenceErrorV1>
where
    T: for<'row> sqlx::Decode<'row, Postgres> + sqlx::Type<Postgres>,
{
    row.try_get(name)
        .map_err(|_| AttachmentTranslationPersistenceErrorV1::InvalidRow)
}
fn array16(value: Vec<u8>) -> Result<[u8; 16], AttachmentTranslationPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| AttachmentTranslationPersistenceErrorV1::InvalidRow)
}
fn array32(value: Vec<u8>) -> Result<[u8; 32], AttachmentTranslationPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| AttachmentTranslationPersistenceErrorV1::InvalidRow)
}
fn optional_array32(
    value: Option<Vec<u8>>,
) -> Result<Option<[u8; 32]>, AttachmentTranslationPersistenceErrorV1> {
    value.map(array32).transpose()
}
fn signed(value: u64) -> Result<i64, AttachmentTranslationPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| AttachmentTranslationPersistenceErrorV1::InvalidInput)
}
fn positive_u64(value: i64) -> Result<u64, AttachmentTranslationPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(AttachmentTranslationPersistenceErrorV1::InvalidRow)
}
fn storage_error(_: sqlx::Error) -> AttachmentTranslationPersistenceErrorV1 {
    AttachmentTranslationPersistenceErrorV1::StorageUnavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_validation_requires_metadata_only_result_lifecycle() {
        let mut status = accepted_attachment_translation_status_v1();
        assert!(valid_status(&status));
        status.state = AttachmentTranslationStateV1::Ready;
        assert!(!valid_status(&status));
    }

    #[test]
    fn source_authority_is_bounded_and_digest_bearing() {
        let authority = AttachmentTranslationSourceAuthorityV1 {
            reference_id: [1; 16],
            declared_bytes: ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1,
            sha256: [2; 32],
            custody_proof: vec![3; 64],
        };
        assert!(valid_source_authority(&authority));
        let mut oversized = authority;
        oversized.declared_bytes += 1;
        assert!(!valid_source_authority(&oversized));
    }
}
