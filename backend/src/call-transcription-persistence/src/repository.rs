use makosh_call_transcription_api::run_id_v1;
use makosh_call_transcription_core::{
    CallTranscriptionCompletenessV1, CallTranscriptionDraftV1, CallTranscriptionLanguageV1,
    CallTranscriptionRejectionV1, CallTranscriptionStateV1, CallTranscriptionStatusV1,
    CallTranscriptionTransitionV1, PendingTranscriptV1, RecordingSourceV1, TranscriptArtifactV1,
    accepted_status_v1, request_fingerprint_v1, transition_v1, validate_draft_v1,
};
use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions, PgRow},
};

use crate::{
    CallTranscriptionInboxOutcomeV1, CallTranscriptionPersistenceErrorV1,
    CreateCallTranscriptionRunOutcomeV1, CreateCallTranscriptionRunV1, PersistRecordingIngressV1,
    PersistedCallTranscriptionRunV1, PersistedRecordingSourceV1, PersistedTranscriptBlobV1,
    RecordingIngressOutcomeV1,
    jobs::enqueue_stt_job,
    model::{
        CALL_TRANSCRIPTION_RECOVERY_LIMIT_V1, valid_id16, valid_outbox, valid_owner, valid_sha256,
        valid_timestamp_millis,
    },
    outbox::insert_outbox,
    realtime::append_realtime,
};

#[derive(Clone)]
pub struct CallTranscriptionPersistenceV1 {
    pub(crate) pool: PgPool,
}

impl CallTranscriptionPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, CallTranscriptionPersistenceErrorV1> {
        if database_id.is_empty()
            || database_id != binding.identity().database_id()
            || pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || binding.access().runtime_principal().is_empty()
        {
            return Err(CallTranscriptionPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port).map_err(|_| storage_unavailable())?;
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
            .map_err(|_| storage_unavailable())?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), CallTranscriptionPersistenceErrorV1> {
        sqlx::query(
            "SELECT 1 FROM makosh_data.call_transcription_runs,
             makosh_data.call_transcription_inbox,
             makosh_data.call_transcription_jobs,
             makosh_data.call_transcription_outbox,
             makosh_data.call_transcription_realtime,
             makosh_data.call_transcription_read_tickets LIMIT 0",
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(storage_error)
    }

    pub async fn create_run(
        &self,
        input: CreateCallTranscriptionRunV1,
    ) -> Result<CreateCallTranscriptionRunOutcomeV1, CallTranscriptionPersistenceErrorV1> {
        if !valid_owner(&input.logical_owner_id)
            || !valid_timestamp_millis(input.created_at_unix_millis)
            || validate_draft_v1(&input.draft).is_err()
        {
            return Err(CallTranscriptionPersistenceErrorV1::InvalidInput);
        }
        let fingerprint = request_fingerprint_v1(&input.draft)
            .map_err(|_| CallTranscriptionPersistenceErrorV1::InvalidInput)?;
        let run_id = run_id_v1(input.draft.operation_id);
        let awaiting = transition_v1(
            &input.draft,
            &accepted_status_v1(),
            CallTranscriptionTransitionV1::AwaitRecording,
        )
        .map_err(|_| CallTranscriptionPersistenceErrorV1::InvalidTransition)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.call_transcription_runs (
               logical_owner_id,run_id,operation_id,request_fingerprint,
               call_evidence_id,call_evidence_revision,recording_evidence_id,
               recording_revision,consent_receipt_id,consent_policy_revision,
               requested_language,state,state_revision,created_at_unix_millis,
               updated_at_unix_millis
             ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$14)
             ON CONFLICT (logical_owner_id,operation_id) DO NOTHING",
        )
        .bind(&input.logical_owner_id)
        .bind(run_id.as_slice())
        .bind(input.draft.operation_id.as_slice())
        .bind(fingerprint.as_slice())
        .bind(input.draft.call_evidence_id.as_slice())
        .bind(signed(input.draft.call_evidence_revision)?)
        .bind(input.draft.recording_evidence_id.as_slice())
        .bind(signed(input.draft.recording_revision)?)
        .bind(input.draft.consent_receipt_id.as_slice())
        .bind(i32::try_from(input.draft.consent_policy_revision).map_err(invalid_input)?)
        .bind(language_code(input.draft.requested_language))
        .bind(state_code(awaiting.state))
        .bind(signed(awaiting.state_revision)?)
        .bind(input.created_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected();
        if inserted == 1 {
            append_realtime(
                &mut transaction,
                &input.logical_owner_id,
                run_id,
                &awaiting,
                input.created_at_unix_millis,
            )
            .await?;
        }
        transaction.commit().await.map_err(storage_error)?;
        let persisted = self.load_run(&input.logical_owner_id, run_id).await?;
        if persisted.request_fingerprint != fingerprint || persisted.draft != input.draft {
            return Err(CallTranscriptionPersistenceErrorV1::RequestConflict);
        }
        Ok(if inserted == 1 {
            CreateCallTranscriptionRunOutcomeV1::Created(persisted)
        } else {
            CreateCallTranscriptionRunOutcomeV1::Existing(persisted)
        })
    }

    pub async fn load_run(
        &self,
        logical_owner_id: &str,
        run_id: [u8; 16],
    ) -> Result<PersistedCallTranscriptionRunV1, CallTranscriptionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || !valid_id16(&run_id) {
            return Err(CallTranscriptionPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(SELECT_RUN)
            .bind(logical_owner_id)
            .bind(run_id.as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_error)?
            .ok_or(CallTranscriptionPersistenceErrorV1::NotFound)
            .and_then(|row| persisted_from_row(&row))
    }

    pub async fn load_recoverable_runs(
        &self,
        logical_owner_id: &str,
        limit: u32,
    ) -> Result<Vec<PersistedCallTranscriptionRunV1>, CallTranscriptionPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !(1..=CALL_TRANSCRIPTION_RECOVERY_LIMIT_V1).contains(&limit)
        {
            return Err(CallTranscriptionPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(SELECT_RECOVERABLE)
            .bind(logical_owner_id)
            .bind(i64::from(limit))
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .iter()
            .map(persisted_from_row)
            .collect()
    }

    pub async fn persist_recording_ingress(
        &self,
        input: PersistRecordingIngressV1,
    ) -> Result<CallTranscriptionInboxOutcomeV1, CallTranscriptionPersistenceErrorV1> {
        validate_ingress(&input)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        if inbox_duplicate(
            &mut transaction,
            &input.logical_owner_id,
            input.message_id,
            input.envelope_sha256,
            input.run_id,
        )
        .await?
        {
            transaction.commit().await.map_err(storage_error)?;
            return self
                .load_run(&input.logical_owner_id, input.run_id)
                .await
                .map(CallTranscriptionInboxOutcomeV1::Duplicate);
        }
        let current = load_run_for_update(&mut transaction, &input.logical_owner_id, input.run_id)
            .await?
            .ok_or(CallTranscriptionPersistenceErrorV1::NotFound)?;
        let (transition, ready) = match input.outcome {
            RecordingIngressOutcomeV1::Ready {
                source,
                source_receipt_sha256,
                stt_request_id,
                stt_request_digest,
            } => (
                CallTranscriptionTransitionV1::RecordingReady {
                    source: (*source).clone(),
                    stt_request_digest,
                },
                Some((
                    *source,
                    source_receipt_sha256,
                    stt_request_id,
                    stt_request_digest,
                )),
            ),
            RecordingIngressOutcomeV1::Rejected(rejection) => {
                (CallTranscriptionTransitionV1::Reject(rejection), None)
            }
        };
        let next = transition_v1(&current.draft, &current.status, transition)
            .map_err(|_| CallTranscriptionPersistenceErrorV1::InvalidTransition)?;
        match ready.as_ref() {
            Some((source, receipt, request_id, request_digest)) => {
                update_recording_ready(
                    &mut transaction,
                    &input.logical_owner_id,
                    input.run_id,
                    current.status.state_revision,
                    &next,
                    source,
                    *receipt,
                    *request_id,
                    *request_digest,
                    input.occurred_at_unix_millis,
                )
                .await?;
                enqueue_stt_job(
                    &mut transaction,
                    &input.logical_owner_id,
                    input.run_id,
                    *request_id,
                    *request_digest,
                    input.occurred_at_unix_millis,
                )
                .await?;
            }
            None => {
                update_rejection(
                    &mut transaction,
                    &input.logical_owner_id,
                    input.run_id,
                    current.status.state_revision,
                    &next,
                    input.occurred_at_unix_millis,
                )
                .await?;
            }
        }
        sqlx::query(
            "INSERT INTO makosh_data.call_transcription_inbox
             (logical_owner_id,message_id,envelope_sha256,run_id,processed_at_unix_millis)
             VALUES ($1,$2,$3,$4,$5)",
        )
        .bind(&input.logical_owner_id)
        .bind(input.message_id.as_slice())
        .bind(input.envelope_sha256.as_slice())
        .bind(input.run_id.as_slice())
        .bind(input.occurred_at_unix_millis)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
        if let Some(outbox) = input.outbox.as_ref() {
            insert_outbox(
                &mut transaction,
                &input.logical_owner_id,
                outbox,
                input.occurred_at_unix_millis,
            )
            .await?;
        }
        append_realtime(
            &mut transaction,
            &input.logical_owner_id,
            input.run_id,
            &next,
            input.occurred_at_unix_millis,
        )
        .await?;
        transaction.commit().await.map_err(storage_error)?;
        self.load_run(&input.logical_owner_id, input.run_id)
            .await
            .map(CallTranscriptionInboxOutcomeV1::Applied)
    }
}

pub(crate) async fn load_run_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
) -> Result<Option<PersistedCallTranscriptionRunV1>, CallTranscriptionPersistenceErrorV1> {
    sqlx::query(SELECT_RUN_FOR_UPDATE)
        .bind(logical_owner_id)
        .bind(run_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_error)?
        .map(|row| persisted_from_row(&row))
        .transpose()
}

async fn inbox_duplicate(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    message_id: [u8; 16],
    envelope_sha256: [u8; 32],
    run_id: [u8; 16],
) -> Result<bool, CallTranscriptionPersistenceErrorV1> {
    let row = sqlx::query(
        "SELECT envelope_sha256,run_id FROM makosh_data.call_transcription_inbox
         WHERE logical_owner_id=$1 AND message_id=$2 FOR UPDATE",
    )
    .bind(logical_owner_id)
    .bind(message_id.as_slice())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_error)?;
    let Some(row) = row else {
        return Ok(false);
    };
    if id32(row.try_get("envelope_sha256").map_err(row_error)?)? != envelope_sha256
        || id16(row.try_get("run_id").map_err(row_error)?)? != run_id
    {
        return Err(CallTranscriptionPersistenceErrorV1::InboxConflict);
    }
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
async fn update_recording_ready(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    current_revision: u64,
    next: &CallTranscriptionStatusV1,
    source: &RecordingSourceV1,
    source_receipt_sha256: [u8; 32],
    stt_request_id: [u8; 16],
    stt_request_digest: [u8; 32],
    occurred_at_unix_millis: i64,
) -> Result<(), CallTranscriptionPersistenceErrorV1> {
    let changed = sqlx::query(
        "UPDATE makosh_data.call_transcription_runs SET state=$1,state_revision=$2,
         source_reference_id=$3,source_sha256=$4,source_declared_bytes=$5,
         source_duration_millis=$6,source_receipt_sha256=$7,stt_request_id=$8,
         stt_request_digest=$9,updated_at_unix_millis=$10
         WHERE logical_owner_id=$11 AND run_id=$12 AND state_revision=$13",
    )
    .bind(state_code(next.state))
    .bind(signed(next.state_revision)?)
    .bind(source.audio_reference_id.as_slice())
    .bind(source.audio_sha256.as_slice())
    .bind(signed(source.declared_bytes)?)
    .bind(signed(source.duration_millis)?)
    .bind(source_receipt_sha256.as_slice())
    .bind(stt_request_id.as_slice())
    .bind(stt_request_digest.as_slice())
    .bind(occurred_at_unix_millis)
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .bind(signed(current_revision)?)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?
    .rows_affected();
    exact_update(changed)
}

pub(crate) async fn update_rejection(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    run_id: [u8; 16],
    current_revision: u64,
    next: &CallTranscriptionStatusV1,
    occurred_at_unix_millis: i64,
) -> Result<(), CallTranscriptionPersistenceErrorV1> {
    let rejection = next
        .rejection
        .ok_or(CallTranscriptionPersistenceErrorV1::InvalidTransition)?;
    let changed = sqlx::query(
        "UPDATE makosh_data.call_transcription_runs SET state=$1,state_revision=$2,
         pending_transcript_reference_id=NULL,pending_transcript_sha256=NULL,
         pending_transcript_size_bytes=NULL,pending_detected_language=NULL,
         pending_duration_millis=NULL,pending_segment_count=NULL,
         pending_completeness=NULL,pending_confidence_basis_points=NULL,
         rejection_code=$3,updated_at_unix_millis=$4
         WHERE logical_owner_id=$5 AND run_id=$6 AND state_revision=$7",
    )
    .bind(state_code(next.state))
    .bind(signed(next.state_revision)?)
    .bind(rejection_code(rejection))
    .bind(occurred_at_unix_millis)
    .bind(logical_owner_id)
    .bind(run_id.as_slice())
    .bind(signed(current_revision)?)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?
    .rows_affected();
    exact_update(changed)
}

fn validate_ingress(
    input: &PersistRecordingIngressV1,
) -> Result<(), CallTranscriptionPersistenceErrorV1> {
    if !valid_owner(&input.logical_owner_id)
        || !valid_id16(&input.run_id)
        || !valid_id16(&input.message_id)
        || !valid_sha256(&input.envelope_sha256)
        || !valid_timestamp_millis(input.occurred_at_unix_millis)
        || input
            .outbox
            .as_ref()
            .is_some_and(|value| !valid_outbox(value))
    {
        return Err(CallTranscriptionPersistenceErrorV1::InvalidInput);
    }
    match &input.outcome {
        RecordingIngressOutcomeV1::Ready {
            source_receipt_sha256,
            stt_request_id,
            stt_request_digest,
            ..
        } if valid_sha256(source_receipt_sha256)
            && valid_id16(stt_request_id)
            && valid_sha256(stt_request_digest) =>
        {
            Ok(())
        }
        RecordingIngressOutcomeV1::Rejected(
            CallTranscriptionRejectionV1::RecordingRejected
            | CallTranscriptionRejectionV1::StaleAuthority
            | CallTranscriptionRejectionV1::Policy,
        ) => Ok(()),
        _ => Err(CallTranscriptionPersistenceErrorV1::InvalidInput),
    }
}

#[cfg(test)]
const SELECT_COLUMNS: &str = "logical_owner_id,run_id,operation_id,request_fingerprint,
 call_evidence_id,call_evidence_revision,recording_evidence_id,recording_revision,
 consent_receipt_id,consent_policy_revision,requested_language,state,state_revision,
 source_reference_id,source_sha256,source_declared_bytes,source_duration_millis,
 source_receipt_sha256,source_cleanup_completed_at_unix_millis,stt_request_id,
 stt_request_digest,stt_result_receipt_sha256,pending_transcript_reference_id,
 pending_transcript_sha256,pending_transcript_size_bytes,pending_detected_language,
 pending_duration_millis,pending_segment_count,pending_completeness,
 pending_confidence_basis_points,artifact_id,artifact_reference_id,
 artifact_receipt_sha256,artifact_transcript_sha256,artifact_transcript_size_bytes,
 artifact_detected_language,artifact_duration_millis,artifact_segment_count,
 artifact_completeness,artifact_confidence_basis_points,artifact_runtime_generation,
 artifact_grant_epoch,rejection_code";

const SELECT_RUN: &str = concat!(
    "SELECT ",
    "logical_owner_id,run_id,operation_id,request_fingerprint,call_evidence_id,call_evidence_revision,recording_evidence_id,recording_revision,consent_receipt_id,consent_policy_revision,requested_language,state,state_revision,source_reference_id,source_sha256,source_declared_bytes,source_duration_millis,source_receipt_sha256,source_cleanup_completed_at_unix_millis,stt_request_id,stt_request_digest,stt_result_receipt_sha256,pending_transcript_reference_id,pending_transcript_sha256,pending_transcript_size_bytes,pending_detected_language,pending_duration_millis,pending_segment_count,pending_completeness,pending_confidence_basis_points,artifact_id,artifact_reference_id,artifact_receipt_sha256,artifact_transcript_sha256,artifact_transcript_size_bytes,artifact_detected_language,artifact_duration_millis,artifact_segment_count,artifact_completeness,artifact_confidence_basis_points,artifact_runtime_generation,artifact_grant_epoch,rejection_code ",
    "FROM makosh_data.call_transcription_runs WHERE logical_owner_id=$1 AND run_id=$2"
);
const SELECT_RUN_FOR_UPDATE: &str = concat!(
    "SELECT ",
    "logical_owner_id,run_id,operation_id,request_fingerprint,call_evidence_id,call_evidence_revision,recording_evidence_id,recording_revision,consent_receipt_id,consent_policy_revision,requested_language,state,state_revision,source_reference_id,source_sha256,source_declared_bytes,source_duration_millis,source_receipt_sha256,source_cleanup_completed_at_unix_millis,stt_request_id,stt_request_digest,stt_result_receipt_sha256,pending_transcript_reference_id,pending_transcript_sha256,pending_transcript_size_bytes,pending_detected_language,pending_duration_millis,pending_segment_count,pending_completeness,pending_confidence_basis_points,artifact_id,artifact_reference_id,artifact_receipt_sha256,artifact_transcript_sha256,artifact_transcript_size_bytes,artifact_detected_language,artifact_duration_millis,artifact_segment_count,artifact_completeness,artifact_confidence_basis_points,artifact_runtime_generation,artifact_grant_epoch,rejection_code ",
    "FROM makosh_data.call_transcription_runs WHERE logical_owner_id=$1 AND run_id=$2 FOR UPDATE"
);
const SELECT_RECOVERABLE: &str = concat!(
    "SELECT ",
    "logical_owner_id,run_id,operation_id,request_fingerprint,call_evidence_id,call_evidence_revision,recording_evidence_id,recording_revision,consent_receipt_id,consent_policy_revision,requested_language,state,state_revision,source_reference_id,source_sha256,source_declared_bytes,source_duration_millis,source_receipt_sha256,source_cleanup_completed_at_unix_millis,stt_request_id,stt_request_digest,stt_result_receipt_sha256,pending_transcript_reference_id,pending_transcript_sha256,pending_transcript_size_bytes,pending_detected_language,pending_duration_millis,pending_segment_count,pending_completeness,pending_confidence_basis_points,artifact_id,artifact_reference_id,artifact_receipt_sha256,artifact_transcript_sha256,artifact_transcript_size_bytes,artifact_detected_language,artifact_duration_millis,artifact_segment_count,artifact_completeness,artifact_confidence_basis_points,artifact_runtime_generation,artifact_grant_epoch,rejection_code ",
    "FROM makosh_data.call_transcription_runs WHERE logical_owner_id=$1 AND (state IN (2,3,4) OR (state IN (5,6) AND source_reference_id IS NOT NULL AND source_cleanup_completed_at_unix_millis IS NULL)) ORDER BY state_revision,run_id LIMIT $2"
);

fn persisted_from_row(
    row: &PgRow,
) -> Result<PersistedCallTranscriptionRunV1, CallTranscriptionPersistenceErrorV1> {
    let draft = CallTranscriptionDraftV1 {
        operation_id: id16(row.try_get("operation_id").map_err(row_error)?)?,
        call_evidence_id: id16(row.try_get("call_evidence_id").map_err(row_error)?)?,
        call_evidence_revision: unsigned(
            row.try_get("call_evidence_revision").map_err(row_error)?,
        )?,
        recording_evidence_id: id16(row.try_get("recording_evidence_id").map_err(row_error)?)?,
        recording_revision: unsigned(row.try_get("recording_revision").map_err(row_error)?)?,
        consent_receipt_id: id16(row.try_get("consent_receipt_id").map_err(row_error)?)?,
        consent_policy_revision: positive_u32(
            row.try_get("consent_policy_revision").map_err(row_error)?,
        )?,
        requested_language: language_from_code(
            row.try_get("requested_language").map_err(row_error)?,
        )?,
    };
    validate_draft_v1(&draft).map_err(|_| CallTranscriptionPersistenceErrorV1::InvalidRow)?;
    let state = state_from_code(row.try_get("state").map_err(row_error)?)?;
    let source = optional_id16(row, "source_reference_id")?
        .map(|reference_id| {
            Ok(PersistedRecordingSourceV1 {
                source: RecordingSourceV1 {
                    recording_evidence_id: draft.recording_evidence_id,
                    recording_revision: draft.recording_revision,
                    call_evidence_id: draft.call_evidence_id,
                    call_evidence_revision: draft.call_evidence_revision,
                    consent_receipt_id: draft.consent_receipt_id,
                    consent_policy_revision: draft.consent_policy_revision,
                    audio_reference_id: reference_id,
                    audio_sha256: required_id32(row, "source_sha256")?,
                    declared_bytes: required_unsigned(row, "source_declared_bytes")?,
                    duration_millis: required_unsigned(row, "source_duration_millis")?,
                },
                source_receipt_sha256: required_id32(row, "source_receipt_sha256")?,
            })
        })
        .transpose()?;
    let stt_request_digest = optional_id32(row, "stt_request_digest")?;
    let pending = optional_id16(row, "pending_transcript_reference_id")?
        .map(|reference_id| {
            Ok(PendingTranscriptV1 {
                transcript_reference_id: reference_id,
                transcript_sha256: required_id32(row, "pending_transcript_sha256")?,
                transcript_size_bytes: required_unsigned(row, "pending_transcript_size_bytes")?,
                detected_language: language_from_code(required_i16(
                    row,
                    "pending_detected_language",
                )?)?,
                duration_millis: required_unsigned(row, "pending_duration_millis")?,
                segment_count: required_u32(row, "pending_segment_count")?,
                completeness: completeness_from_code(required_i16(row, "pending_completeness")?)?,
                confidence_basis_points: required_u32(row, "pending_confidence_basis_points")?,
                stt_request_digest: stt_request_digest
                    .ok_or(CallTranscriptionPersistenceErrorV1::InvalidRow)?,
            })
        })
        .transpose()?;
    let artifact = optional_id16(row, "artifact_id")?
        .map(|artifact_id| {
            Ok(TranscriptArtifactV1 {
                artifact_id,
                transcript_sha256: required_id32(row, "artifact_transcript_sha256")?,
                transcript_size_bytes: required_unsigned(row, "artifact_transcript_size_bytes")?,
                detected_language: language_from_code(required_i16(
                    row,
                    "artifact_detected_language",
                )?)?,
                duration_millis: required_unsigned(row, "artifact_duration_millis")?,
                segment_count: required_u32(row, "artifact_segment_count")?,
                completeness: completeness_from_code(required_i16(row, "artifact_completeness")?)?,
                confidence_basis_points: required_u32(row, "artifact_confidence_basis_points")?,
            })
        })
        .transpose()?;
    let artifact_blob = artifact
        .as_ref()
        .map(|_| {
            Ok(PersistedTranscriptBlobV1 {
                reference_id: required_id16(row, "artifact_reference_id")?,
                receipt_sha256: required_id32(row, "artifact_receipt_sha256")?,
                runtime_generation: required_unsigned(row, "artifact_runtime_generation")?,
                grant_epoch: required_unsigned(row, "artifact_grant_epoch")?,
            })
        })
        .transpose()?;
    let rejection = optional_i16(row, "rejection_code")?
        .map(rejection_from_code)
        .transpose()?;
    Ok(PersistedCallTranscriptionRunV1 {
        logical_owner_id: row.try_get("logical_owner_id").map_err(row_error)?,
        run_id: id16(row.try_get("run_id").map_err(row_error)?)?,
        request_fingerprint: id32(row.try_get("request_fingerprint").map_err(row_error)?)?,
        draft,
        status: CallTranscriptionStatusV1 {
            state,
            state_revision: unsigned(row.try_get("state_revision").map_err(row_error)?)?,
            source_sha256: source.as_ref().map(|value| value.source.audio_sha256),
            stt_request_digest,
            pending_transcript: pending,
            artifact,
            rejection,
        },
        recording_source: source,
        source_cleanup_completed_at_unix_millis: row
            .try_get("source_cleanup_completed_at_unix_millis")
            .map_err(row_error)?,
        stt_request_id: optional_id16(row, "stt_request_id")?,
        stt_result_receipt_sha256: optional_id32(row, "stt_result_receipt_sha256")?,
        artifact_blob,
    })
}

pub(crate) fn state_code(value: CallTranscriptionStateV1) -> i16 {
    match value {
        CallTranscriptionStateV1::Accepted => 1,
        CallTranscriptionStateV1::AwaitingRecording => 2,
        CallTranscriptionStateV1::AwaitingStt => 3,
        CallTranscriptionStateV1::MaterializingTranscript => 4,
        CallTranscriptionStateV1::Ready => 5,
        CallTranscriptionStateV1::Rejected => 6,
    }
}

pub(crate) fn state_from_code(
    value: i16,
) -> Result<CallTranscriptionStateV1, CallTranscriptionPersistenceErrorV1> {
    match value {
        1 => Ok(CallTranscriptionStateV1::Accepted),
        2 => Ok(CallTranscriptionStateV1::AwaitingRecording),
        3 => Ok(CallTranscriptionStateV1::AwaitingStt),
        4 => Ok(CallTranscriptionStateV1::MaterializingTranscript),
        5 => Ok(CallTranscriptionStateV1::Ready),
        6 => Ok(CallTranscriptionStateV1::Rejected),
        _ => Err(CallTranscriptionPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) fn language_code(value: CallTranscriptionLanguageV1) -> i16 {
    match value {
        CallTranscriptionLanguageV1::Auto => 1,
        CallTranscriptionLanguageV1::English => 2,
        CallTranscriptionLanguageV1::Russian => 3,
        CallTranscriptionLanguageV1::Spanish => 4,
    }
}

fn language_from_code(
    value: i16,
) -> Result<CallTranscriptionLanguageV1, CallTranscriptionPersistenceErrorV1> {
    match value {
        1 => Ok(CallTranscriptionLanguageV1::Auto),
        2 => Ok(CallTranscriptionLanguageV1::English),
        3 => Ok(CallTranscriptionLanguageV1::Russian),
        4 => Ok(CallTranscriptionLanguageV1::Spanish),
        _ => Err(CallTranscriptionPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) fn completeness_code(value: CallTranscriptionCompletenessV1) -> i16 {
    match value {
        CallTranscriptionCompletenessV1::Complete => 1,
        CallTranscriptionCompletenessV1::Partial => 2,
    }
}

fn completeness_from_code(
    value: i16,
) -> Result<CallTranscriptionCompletenessV1, CallTranscriptionPersistenceErrorV1> {
    match value {
        1 => Ok(CallTranscriptionCompletenessV1::Complete),
        2 => Ok(CallTranscriptionCompletenessV1::Partial),
        _ => Err(CallTranscriptionPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) fn rejection_code(value: CallTranscriptionRejectionV1) -> i16 {
    match value {
        CallTranscriptionRejectionV1::RecordingRejected => 1,
        CallTranscriptionRejectionV1::SttRejected => 2,
        CallTranscriptionRejectionV1::ResultRejected => 3,
        CallTranscriptionRejectionV1::StaleAuthority => 4,
        CallTranscriptionRejectionV1::Policy => 5,
    }
}

pub(crate) fn rejection_from_code(
    value: i16,
) -> Result<CallTranscriptionRejectionV1, CallTranscriptionPersistenceErrorV1> {
    match value {
        1 => Ok(CallTranscriptionRejectionV1::RecordingRejected),
        2 => Ok(CallTranscriptionRejectionV1::SttRejected),
        3 => Ok(CallTranscriptionRejectionV1::ResultRejected),
        4 => Ok(CallTranscriptionRejectionV1::StaleAuthority),
        5 => Ok(CallTranscriptionRejectionV1::Policy),
        _ => Err(CallTranscriptionPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) fn id16(value: Vec<u8>) -> Result<[u8; 16], CallTranscriptionPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| CallTranscriptionPersistenceErrorV1::InvalidRow)
}

pub(crate) fn id32(value: Vec<u8>) -> Result<[u8; 32], CallTranscriptionPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| CallTranscriptionPersistenceErrorV1::InvalidRow)
}

fn optional_id16(
    row: &PgRow,
    name: &str,
) -> Result<Option<[u8; 16]>, CallTranscriptionPersistenceErrorV1> {
    row.try_get::<Option<Vec<u8>>, _>(name)
        .map_err(row_error)?
        .map(id16)
        .transpose()
}

fn optional_id32(
    row: &PgRow,
    name: &str,
) -> Result<Option<[u8; 32]>, CallTranscriptionPersistenceErrorV1> {
    row.try_get::<Option<Vec<u8>>, _>(name)
        .map_err(row_error)?
        .map(id32)
        .transpose()
}

fn optional_i16(
    row: &PgRow,
    name: &str,
) -> Result<Option<i16>, CallTranscriptionPersistenceErrorV1> {
    row.try_get(name).map_err(row_error)
}

fn required_id16(row: &PgRow, name: &str) -> Result<[u8; 16], CallTranscriptionPersistenceErrorV1> {
    optional_id16(row, name)?.ok_or(CallTranscriptionPersistenceErrorV1::InvalidRow)
}

fn required_id32(row: &PgRow, name: &str) -> Result<[u8; 32], CallTranscriptionPersistenceErrorV1> {
    optional_id32(row, name)?.ok_or(CallTranscriptionPersistenceErrorV1::InvalidRow)
}

fn required_i16(row: &PgRow, name: &str) -> Result<i16, CallTranscriptionPersistenceErrorV1> {
    optional_i16(row, name)?.ok_or(CallTranscriptionPersistenceErrorV1::InvalidRow)
}

fn required_unsigned(row: &PgRow, name: &str) -> Result<u64, CallTranscriptionPersistenceErrorV1> {
    unsigned(
        row.try_get::<Option<i64>, _>(name)
            .map_err(row_error)?
            .ok_or(CallTranscriptionPersistenceErrorV1::InvalidRow)?,
    )
}

fn required_u32(row: &PgRow, name: &str) -> Result<u32, CallTranscriptionPersistenceErrorV1> {
    u32::try_from(
        row.try_get::<Option<i32>, _>(name)
            .map_err(row_error)?
            .ok_or(CallTranscriptionPersistenceErrorV1::InvalidRow)?,
    )
    .map_err(row_error)
}

pub(crate) fn signed(value: u64) -> Result<i64, CallTranscriptionPersistenceErrorV1> {
    i64::try_from(value).map_err(invalid_input)
}

pub(crate) fn unsigned(value: i64) -> Result<u64, CallTranscriptionPersistenceErrorV1> {
    u64::try_from(value).map_err(row_error)
}

fn positive_u32(value: i32) -> Result<u32, CallTranscriptionPersistenceErrorV1> {
    let value = u32::try_from(value).map_err(row_error)?;
    if value == 0 {
        Err(CallTranscriptionPersistenceErrorV1::InvalidRow)
    } else {
        Ok(value)
    }
}

pub(crate) fn exact_update(changed: u64) -> Result<(), CallTranscriptionPersistenceErrorV1> {
    if changed == 1 {
        Ok(())
    } else {
        Err(CallTranscriptionPersistenceErrorV1::RevisionConflict)
    }
}

pub(crate) fn storage_error<T>(_: T) -> CallTranscriptionPersistenceErrorV1 {
    CallTranscriptionPersistenceErrorV1::StorageUnavailable
}

pub(crate) fn storage_unavailable() -> CallTranscriptionPersistenceErrorV1 {
    CallTranscriptionPersistenceErrorV1::StorageUnavailable
}

pub(crate) fn row_error<T>(_: T) -> CallTranscriptionPersistenceErrorV1 {
    CallTranscriptionPersistenceErrorV1::InvalidRow
}

pub(crate) fn invalid_input<T>(_: T) -> CallTranscriptionPersistenceErrorV1 {
    CallTranscriptionPersistenceErrorV1::InvalidInput
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_inventory_tracks_every_durable_metadata_column() {
        for required in [
            "request_fingerprint",
            "source_receipt_sha256",
            "stt_result_receipt_sha256",
            "artifact_receipt_sha256",
            "artifact_runtime_generation",
        ] {
            assert!(SELECT_COLUMNS.contains(required));
            assert!(SELECT_RUN.contains(required));
        }
    }
}
