use makosh_ai_contracts::wire::{
    AiContextReceiptV1, AiInferenceReceiptV1, AiPrivateSourceReceiptV1, AiUseCaseV1,
    AttachmentTranslationInferenceRequestV1, AttachmentTranslationInferenceResultV1,
};
use makosh_ai_inference_core::{AiAttachmentTranslationRunV1, AiInferenceRunStateV1};
use sqlx::{Row, postgres::PgRow};

use crate::{
    AI_INFERENCE_RECOVERY_LIMIT_V1, AiAttachmentTranslationPersistenceOutcomeV1,
    AiAttachmentTranslationTransitionV1, AiInferencePersistenceErrorV1, AiInferencePersistenceV1,
    PersistedAiAttachmentTranslationRunV1,
    attachment_translation_model::{
        validate_attachment_translation_accepted, validate_attachment_translation_persisted,
        validate_attachment_translation_run, validate_attachment_translation_transition,
    },
};

impl AiInferencePersistenceV1 {
    pub async fn accept_attachment_translation_run(
        &self,
        run: AiAttachmentTranslationRunV1,
    ) -> Result<AiAttachmentTranslationPersistenceOutcomeV1, AiInferencePersistenceErrorV1> {
        validate_attachment_translation_accepted(&run)?;
        let request = &run.request;
        let context = request.context.as_ref().ok_or(invalid_input())?;
        let source = request.source.as_ref().ok_or(invalid_input())?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.ai_attachment_translation_runs (
               logical_owner_id, run_id, request_digest, context_id, source_evidence_id,
               source_evidence_revision, contract_major, contract_revision,
               contract_schema_sha256, source_reference_id, source_declared_bytes,
               source_sha256, source_custody_proof, requested_target_language,
               maximum_output_bytes, maximum_output_tokens, egress_policy,
               egress_policy_revision, state_revision, run_state
             ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20)
             ON CONFLICT (logical_owner_id, run_id) DO NOTHING",
        )
        .bind(&request.logical_owner_id)
        .bind(&request.run_id)
        .bind(&context.request_digest)
        .bind(&context.context_id)
        .bind(&context.source_evidence_id)
        .bind(signed(context.source_evidence_revision)?)
        .bind(i32::try_from(context.contract_major).map_err(|_| invalid_input())?)
        .bind(i32::try_from(context.contract_revision).map_err(|_| invalid_input())?)
        .bind(&context.contract_schema_sha256)
        .bind(&source.reference_id)
        .bind(signed(source.declared_bytes)?)
        .bind(&source.sha256)
        .bind(&source.custody_transfer_source_proof)
        .bind(i16::try_from(request.target_language).map_err(|_| invalid_input())?)
        .bind(i32::try_from(request.maximum_output_bytes).map_err(|_| invalid_input())?)
        .bind(i32::try_from(request.maximum_output_tokens).map_err(|_| invalid_input())?)
        .bind(i16::try_from(request.egress_policy).map_err(|_| invalid_input())?)
        .bind(i32::try_from(request.egress_policy_revision).map_err(|_| invalid_input())?)
        .bind(signed(run.revision)?)
        .bind(run_state_code(run.state))
        .execute(&self.pool)
        .await
        .map_err(storage_error)?
        .rows_affected()
            == 1;
        let mut persisted = self
            .load_attachment_translation_run(&request.logical_owner_id, id16(&request.run_id)?)
            .await?
            .ok_or(AiInferencePersistenceErrorV1::InvalidRow)?;
        if !inserted && persisted.run.request != run.request {
            if !same_semantic_request(&persisted.run.request, &run.request) {
                return Err(AiInferencePersistenceErrorV1::RequestConflict);
            }
            if matches!(
                persisted.run.state,
                AiInferenceRunStateV1::Accepted | AiInferenceRunStateV1::Executing
            ) {
                let updated = sqlx::query(
                    "UPDATE makosh_data.ai_attachment_translation_runs SET source_custody_proof=$1
                     WHERE logical_owner_id=$2 AND run_id=$3 AND request_digest=$4 AND run_state IN (1,2)",
                )
                .bind(&source.custody_transfer_source_proof)
                .bind(&request.logical_owner_id)
                .bind(&request.run_id)
                .bind(&context.request_digest)
                .execute(&self.pool)
                .await
                .map_err(storage_error)?
                .rows_affected();
                if updated != 1 {
                    return Err(AiInferencePersistenceErrorV1::RevisionConflict);
                }
                persisted = self
                    .load_attachment_translation_run(
                        &request.logical_owner_id,
                        id16(&request.run_id)?,
                    )
                    .await?
                    .ok_or(AiInferencePersistenceErrorV1::InvalidRow)?;
            }
        }
        Ok(AiAttachmentTranslationPersistenceOutcomeV1 {
            persisted,
            replayed: !inserted,
        })
    }

    pub async fn load_attachment_translation_run(
        &self,
        logical_owner_id: &str,
        run_id: [u8; 16],
    ) -> Result<Option<PersistedAiAttachmentTranslationRunV1>, AiInferencePersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || run_id == [0; 16] {
            return Err(invalid_input());
        }
        sqlx::query(SELECT_TRANSLATION_RUN)
            .bind(logical_owner_id)
            .bind(run_id.as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_error)?
            .map(|row| persisted_from_row(&row))
            .transpose()
    }

    pub async fn load_recoverable_attachment_translation_runs(
        &self,
        logical_owner_id: &str,
        limit: u32,
    ) -> Result<Vec<PersistedAiAttachmentTranslationRunV1>, AiInferencePersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || !(1..=AI_INFERENCE_RECOVERY_LIMIT_V1).contains(&limit)
        {
            return Err(invalid_input());
        }
        sqlx::query(SELECT_RECOVERABLE_TRANSLATION_RUNS)
            .bind(logical_owner_id)
            .bind(i64::from(limit))
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .iter()
            .map(persisted_from_row)
            .collect()
    }

    pub async fn persist_attachment_translation_transition(
        &self,
        transition: AiAttachmentTranslationTransitionV1,
    ) -> Result<PersistedAiAttachmentTranslationRunV1, AiInferencePersistenceErrorV1> {
        validate_attachment_translation_run(&transition.next_run)?;
        let request = &transition.next_run.request;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let current_row = sqlx::query(SELECT_TRANSLATION_RUN_FOR_UPDATE)
            .bind(&request.logical_owner_id)
            .bind(&request.run_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(storage_error)?
            .ok_or(AiInferencePersistenceErrorV1::RevisionConflict)?;
        let current = persisted_from_row(&current_row)?;
        let selected = validate_attachment_translation_transition(&current, &transition)?;
        let result = transition.next_run.terminal_result.as_ref();
        let receipt = result.and_then(|result| result.inference_receipt.as_ref());
        let updated = sqlx::query(
            "UPDATE makosh_data.ai_attachment_translation_runs SET state_revision=$4, run_state=$5,
             selected_provider_settings_revision=$6, result_translated_text_utf8=$7,
             result_detected_source_language=$8, result_target_language=$9,
             result_model_revision_sha256=$10, result_prompt_policy_sha256=$11,
             result_provider_policy_revision=$12, result_completeness=$13,
             result_confidence_basis_points=$14, result_terminal_status=$15
             WHERE logical_owner_id=$1 AND run_id=$2 AND state_revision=$3",
        )
        .bind(&request.logical_owner_id)
        .bind(&request.run_id)
        .bind(signed(transition.current_revision)?)
        .bind(signed(transition.next_run.revision)?)
        .bind(run_state_code(transition.next_run.state))
        .bind(optional_signed(selected)?)
        .bind(result.map(|value| value.translated_text_utf8.as_slice()))
        .bind(optional_i16(
            result.map(|value| value.detected_source_language),
        )?)
        .bind(optional_i16(result.map(|value| value.target_language))?)
        .bind(receipt.map(|value| value.model_revision_sha256.as_slice()))
        .bind(receipt.map(|value| value.prompt_policy_sha256.as_slice()))
        .bind(optional_i32(
            receipt.map(|value| value.provider_policy_revision),
        )?)
        .bind(optional_i16(result.map(|value| value.completeness))?)
        .bind(optional_i32(
            result.map(|value| value.confidence_basis_points),
        )?)
        .bind(optional_i16(result.map(|value| value.terminal_status))?)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
        if updated.rows_affected() != 1 {
            return Err(AiInferencePersistenceErrorV1::RevisionConflict);
        }
        transaction.commit().await.map_err(storage_error)?;
        Ok(PersistedAiAttachmentTranslationRunV1 {
            run: transition.next_run,
            selected_provider_settings_revision: selected,
        })
    }
}

const SELECT_TRANSLATION_RUN: &str = "SELECT logical_owner_id, run_id, request_digest, context_id,
 source_evidence_id, source_evidence_revision, contract_major, contract_revision,
 contract_schema_sha256, source_reference_id, source_declared_bytes, source_sha256,
 source_custody_proof, requested_target_language, maximum_output_bytes,
 maximum_output_tokens, egress_policy, egress_policy_revision, state_revision, run_state,
 selected_provider_settings_revision, result_translated_text_utf8,
 result_detected_source_language, result_target_language, result_model_revision_sha256,
 result_prompt_policy_sha256, result_provider_policy_revision, result_completeness,
 result_confidence_basis_points, result_terminal_status
 FROM makosh_data.ai_attachment_translation_runs WHERE logical_owner_id=$1 AND run_id=$2";
const SELECT_TRANSLATION_RUN_FOR_UPDATE: &str =
    "SELECT logical_owner_id, run_id, request_digest, context_id,
 source_evidence_id, source_evidence_revision, contract_major, contract_revision,
 contract_schema_sha256, source_reference_id, source_declared_bytes, source_sha256,
 source_custody_proof, requested_target_language, maximum_output_bytes,
 maximum_output_tokens, egress_policy, egress_policy_revision, state_revision, run_state,
 selected_provider_settings_revision, result_translated_text_utf8,
 result_detected_source_language, result_target_language, result_model_revision_sha256,
 result_prompt_policy_sha256, result_provider_policy_revision, result_completeness,
 result_confidence_basis_points, result_terminal_status
 FROM makosh_data.ai_attachment_translation_runs
 WHERE logical_owner_id=$1 AND run_id=$2 FOR UPDATE";
const SELECT_RECOVERABLE_TRANSLATION_RUNS: &str =
    "SELECT logical_owner_id, run_id, request_digest, context_id,
 source_evidence_id, source_evidence_revision, contract_major, contract_revision,
 contract_schema_sha256, source_reference_id, source_declared_bytes, source_sha256,
 source_custody_proof, requested_target_language, maximum_output_bytes,
 maximum_output_tokens, egress_policy, egress_policy_revision, state_revision, run_state,
 selected_provider_settings_revision, result_translated_text_utf8,
 result_detected_source_language, result_target_language, result_model_revision_sha256,
 result_prompt_policy_sha256, result_provider_policy_revision, result_completeness,
 result_confidence_basis_points, result_terminal_status
 FROM makosh_data.ai_attachment_translation_runs
 WHERE logical_owner_id=$1 AND run_state IN (1,2)
 ORDER BY state_revision, run_id LIMIT $2";

fn persisted_from_row(
    row: &PgRow,
) -> Result<PersistedAiAttachmentTranslationRunV1, AiInferencePersistenceErrorV1> {
    let selected = row
        .try_get::<Option<i64>, _>("selected_provider_settings_revision")
        .map_err(row_error)?
        .map(positive_u64)
        .transpose()?;
    let status: Option<i16> = row.try_get("result_terminal_status").map_err(row_error)?;
    let result = status
        .map(|status| {
            let model: Option<Vec<u8>> = row
                .try_get("result_model_revision_sha256")
                .map_err(row_error)?;
            let prompt: Option<Vec<u8>> = row
                .try_get("result_prompt_policy_sha256")
                .map_err(row_error)?;
            let policy: Option<i32> = row
                .try_get("result_provider_policy_revision")
                .map_err(row_error)?;
            let receipt = match (model, prompt, selected, policy) {
                (Some(model), Some(prompt), Some(settings), Some(policy)) => {
                    Some(AiInferenceReceiptV1 {
                        model_revision_sha256: model,
                        prompt_policy_sha256: prompt,
                        provider_settings_revision: settings,
                        provider_policy_revision: positive_u32(policy)?,
                    })
                }
                (None, None, _, None) => None,
                _ => return Err(AiInferencePersistenceErrorV1::InvalidRow),
            };
            Ok(AttachmentTranslationInferenceResultV1 {
                run_id: row.try_get("run_id").map_err(row_error)?,
                request_digest: row.try_get("request_digest").map_err(row_error)?,
                source_sha256: row.try_get("source_sha256").map_err(row_error)?,
                translated_text_utf8: row
                    .try_get("result_translated_text_utf8")
                    .map_err(row_error)?,
                detected_source_language: i32::from(
                    row.try_get::<i16, _>("result_detected_source_language")
                        .map_err(row_error)?,
                ),
                target_language: i32::from(
                    row.try_get::<i16, _>("result_target_language")
                        .map_err(row_error)?,
                ),
                inference_receipt: receipt,
                completeness: i32::from(
                    row.try_get::<i16, _>("result_completeness")
                        .map_err(row_error)?,
                ),
                confidence_basis_points: nonnegative_u32(
                    row.try_get("result_confidence_basis_points")
                        .map_err(row_error)?,
                )?,
                terminal_status: i32::from(status),
            })
        })
        .transpose()?;
    let request = AttachmentTranslationInferenceRequestV1 {
        run_id: row.try_get("run_id").map_err(row_error)?,
        context: Some(AiContextReceiptV1 {
            context_id: row.try_get("context_id").map_err(row_error)?,
            use_case: AiUseCaseV1::AiUseCaseAttachmentTranslation as i32,
            source_evidence_id: row.try_get("source_evidence_id").map_err(row_error)?,
            source_evidence_revision: positive_u64(
                row.try_get("source_evidence_revision").map_err(row_error)?,
            )?,
            contract_major: positive_u32(row.try_get("contract_major").map_err(row_error)?)?,
            contract_revision: positive_u32(row.try_get("contract_revision").map_err(row_error)?)?,
            contract_schema_sha256: row.try_get("contract_schema_sha256").map_err(row_error)?,
            request_digest: row.try_get("request_digest").map_err(row_error)?,
        }),
        source: Some(AiPrivateSourceReceiptV1 {
            reference_id: row.try_get("source_reference_id").map_err(row_error)?,
            declared_bytes: positive_u64(row.try_get("source_declared_bytes").map_err(row_error)?)?,
            sha256: row.try_get("source_sha256").map_err(row_error)?,
            custody_transfer_source_proof: row
                .try_get("source_custody_proof")
                .map_err(row_error)?,
        }),
        target_language: i32::from(
            row.try_get::<i16, _>("requested_target_language")
                .map_err(row_error)?,
        ),
        maximum_output_bytes: positive_u32(
            row.try_get("maximum_output_bytes").map_err(row_error)?,
        )?,
        maximum_output_tokens: positive_u32(
            row.try_get("maximum_output_tokens").map_err(row_error)?,
        )?,
        egress_policy: i32::from(row.try_get::<i16, _>("egress_policy").map_err(row_error)?),
        egress_policy_revision: positive_u32(
            row.try_get("egress_policy_revision").map_err(row_error)?,
        )?,
        logical_owner_id: row.try_get("logical_owner_id").map_err(row_error)?,
    };
    let persisted = PersistedAiAttachmentTranslationRunV1 {
        run: AiAttachmentTranslationRunV1 {
            request,
            revision: positive_u64(row.try_get("state_revision").map_err(row_error)?)?,
            state: run_state(row.try_get("run_state").map_err(row_error)?)?,
            terminal_result: result,
        },
        selected_provider_settings_revision: selected,
    };
    validate_attachment_translation_run(&persisted.run)?;
    validate_attachment_translation_persisted(&persisted)?;
    Ok(persisted)
}

fn same_semantic_request(
    left: &AttachmentTranslationInferenceRequestV1,
    right: &AttachmentTranslationInferenceRequestV1,
) -> bool {
    let (mut left, mut right) = (left.clone(), right.clone());
    let (Some(left_source), Some(right_source)) = (left.source.as_mut(), right.source.as_mut())
    else {
        return false;
    };
    left_source.custody_transfer_source_proof.clear();
    right_source.custody_transfer_source_proof.clear();
    left == right
}

fn run_state_code(state: AiInferenceRunStateV1) -> i16 {
    match state {
        AiInferenceRunStateV1::Accepted => 1,
        AiInferenceRunStateV1::Executing => 2,
        AiInferenceRunStateV1::Ready => 3,
        AiInferenceRunStateV1::Rejected => 4,
    }
}
fn run_state(value: i16) -> Result<AiInferenceRunStateV1, AiInferencePersistenceErrorV1> {
    match value {
        1 => Ok(AiInferenceRunStateV1::Accepted),
        2 => Ok(AiInferenceRunStateV1::Executing),
        3 => Ok(AiInferenceRunStateV1::Ready),
        4 => Ok(AiInferenceRunStateV1::Rejected),
        _ => Err(AiInferencePersistenceErrorV1::InvalidRow),
    }
}
fn valid_owner(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}
fn id16(value: &[u8]) -> Result<[u8; 16], AiInferencePersistenceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| *id != [0; 16])
        .ok_or(invalid_input())
}
fn signed(value: u64) -> Result<i64, AiInferencePersistenceErrorV1> {
    i64::try_from(value).map_err(|_| invalid_input())
}
fn optional_signed(value: Option<u64>) -> Result<Option<i64>, AiInferencePersistenceErrorV1> {
    value.map(signed).transpose()
}
fn optional_i16(value: Option<i32>) -> Result<Option<i16>, AiInferencePersistenceErrorV1> {
    value
        .map(|value| i16::try_from(value).map_err(|_| invalid_input()))
        .transpose()
}
fn optional_i32(value: Option<u32>) -> Result<Option<i32>, AiInferencePersistenceErrorV1> {
    value
        .map(|value| i32::try_from(value).map_err(|_| invalid_input()))
        .transpose()
}
fn positive_u64(value: i64) -> Result<u64, AiInferencePersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(AiInferencePersistenceErrorV1::InvalidRow)
}
fn positive_u32(value: i32) -> Result<u32, AiInferencePersistenceErrorV1> {
    u32::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(AiInferencePersistenceErrorV1::InvalidRow)
}
fn nonnegative_u32(value: i32) -> Result<u32, AiInferencePersistenceErrorV1> {
    u32::try_from(value).map_err(|_| AiInferencePersistenceErrorV1::InvalidRow)
}
fn invalid_input() -> AiInferencePersistenceErrorV1 {
    AiInferencePersistenceErrorV1::InvalidInput
}
fn row_error(_: sqlx::Error) -> AiInferencePersistenceErrorV1 {
    AiInferencePersistenceErrorV1::InvalidRow
}
fn storage_error(_: sqlx::Error) -> AiInferencePersistenceErrorV1 {
    AiInferencePersistenceErrorV1::StorageUnavailable
}
