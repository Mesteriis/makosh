use makosh_ai_contracts::wire::AiProviderSummaryGenerationResultV1;
use makosh_ollama_ai_core::{OllamaAiRunStateV1, OllamaSummaryRunV1};
use sqlx::{Row, postgres::PgRow};

use crate::{
    OllamaAiPersistenceErrorV1, OllamaAiPersistenceV1, OllamaSummaryPersistenceOutcomeV1,
    OllamaSummaryTransitionV1, PersistedOllamaSummaryRunV1,
    model::validate_owner,
    summary_model::{validate_summary_accepted, validate_summary_run, validate_summary_transition},
};

impl OllamaAiPersistenceV1 {
    pub async fn accept_summary_run(
        &self,
        logical_owner_id: &str,
        run: OllamaSummaryRunV1,
    ) -> Result<OllamaSummaryPersistenceOutcomeV1, OllamaAiPersistenceErrorV1> {
        validate_summary_accepted(logical_owner_id, &run)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.ollama_ai_summary_runs (
               logical_owner_id, request_id, request_digest, settings_revision,
               state_revision, run_state
             ) VALUES ($1,$2,$3,$4,$5,$6)
             ON CONFLICT (logical_owner_id, request_id) DO NOTHING",
        )
        .bind(logical_owner_id)
        .bind(run.request_id.as_slice())
        .bind(run.request_digest.as_slice())
        .bind(signed(run.settings_revision)?)
        .bind(signed(run.revision)?)
        .bind(run_state_code(run.state))
        .execute(&self.pool)
        .await
        .map_err(storage_error)?
        .rows_affected()
            == 1;
        let persisted = self
            .load_summary_run(logical_owner_id, run.request_id)
            .await?
            .ok_or(OllamaAiPersistenceErrorV1::InvalidRow)?;
        if !inserted
            && (persisted.run.request_digest != run.request_digest
                || persisted.run.settings_revision != run.settings_revision)
        {
            return Err(OllamaAiPersistenceErrorV1::RequestConflict);
        }
        Ok(OllamaSummaryPersistenceOutcomeV1 {
            persisted,
            replayed: !inserted,
        })
    }

    pub async fn load_summary_run(
        &self,
        logical_owner_id: &str,
        request_id: [u8; 16],
    ) -> Result<Option<PersistedOllamaSummaryRunV1>, OllamaAiPersistenceErrorV1> {
        if !validate_owner(logical_owner_id) || request_id == [0; 16] {
            return Err(OllamaAiPersistenceErrorV1::InvalidInput);
        }
        sqlx::query(SELECT_RUN)
            .bind(logical_owner_id)
            .bind(request_id.as_slice())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_error)?
            .map(|row| persisted_from_row(&row))
            .transpose()
    }

    pub async fn persist_summary_transition(
        &self,
        transition: OllamaSummaryTransitionV1,
    ) -> Result<PersistedOllamaSummaryRunV1, OllamaAiPersistenceErrorV1> {
        validate_summary_run(&transition.next_run)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let row = sqlx::query(SELECT_RUN_FOR_UPDATE)
            .bind(&transition.logical_owner_id)
            .bind(transition.next_run.request_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(storage_error)?
            .ok_or(OllamaAiPersistenceErrorV1::RevisionConflict)?;
        let current = persisted_from_row(&row)?;
        validate_summary_transition(&current, &transition)?;
        let result = transition.next_run.terminal_result.as_ref();
        let updated = sqlx::query(
            "UPDATE makosh_data.ollama_ai_summary_runs SET
               state_revision=$4, run_state=$5, selected_model_revision_sha256=$6,
               result_summary_utf8=$7, result_resolved_language=$8,
               result_resolved_length=$9, result_model_revision_sha256=$10,
               result_input_tokens=$11, result_output_tokens=$12,
               result_terminal_status=$13, result_completeness=$14,
               result_confidence_basis_points=$15, result_provider_settings_revision=$16
             WHERE logical_owner_id=$1 AND request_id=$2 AND state_revision=$3",
        )
        .bind(&transition.logical_owner_id)
        .bind(transition.next_run.request_id.as_slice())
        .bind(signed(transition.current_revision)?)
        .bind(signed(transition.next_run.revision)?)
        .bind(run_state_code(transition.next_run.state))
        .bind(
            transition
                .next_run
                .selected_model_digest
                .as_ref()
                .map(<[u8; 32]>::as_slice),
        )
        .bind(result.map(|value| value.summary_utf8.as_slice()))
        .bind(optional_i16(result.map(|value| value.resolved_language))?)
        .bind(optional_i16(result.map(|value| value.resolved_length))?)
        .bind(result.and_then(|value| {
            (!value.model_revision_sha256.is_empty())
                .then_some(value.model_revision_sha256.as_slice())
        }))
        .bind(optional_i32(result.map(|value| value.input_tokens))?)
        .bind(optional_i32(result.map(|value| value.output_tokens))?)
        .bind(optional_i16(result.map(|value| value.terminal_status))?)
        .bind(optional_i16(result.map(|value| value.completeness))?)
        .bind(optional_i32(
            result.map(|value| value.confidence_basis_points),
        )?)
        .bind(optional_signed(
            result.map(|value| value.provider_settings_revision),
        )?)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
        if updated.rows_affected() != 1 {
            return Err(OllamaAiPersistenceErrorV1::RevisionConflict);
        }
        transaction.commit().await.map_err(storage_error)?;
        Ok(PersistedOllamaSummaryRunV1 {
            logical_owner_id: transition.logical_owner_id,
            run: transition.next_run,
        })
    }
}

const SELECT_RUN: &str = "SELECT logical_owner_id, request_id, request_digest, settings_revision,
 state_revision, run_state, selected_model_revision_sha256, result_summary_utf8,
 result_resolved_language, result_resolved_length, result_model_revision_sha256,
 result_input_tokens, result_output_tokens, result_terminal_status, result_completeness,
 result_confidence_basis_points, result_provider_settings_revision
 FROM makosh_data.ollama_ai_summary_runs WHERE logical_owner_id=$1 AND request_id=$2";
const SELECT_RUN_FOR_UPDATE: &str =
    "SELECT logical_owner_id, request_id, request_digest, settings_revision,
 state_revision, run_state, selected_model_revision_sha256, result_summary_utf8,
 result_resolved_language, result_resolved_length, result_model_revision_sha256,
 result_input_tokens, result_output_tokens, result_terminal_status, result_completeness,
 result_confidence_basis_points, result_provider_settings_revision
 FROM makosh_data.ollama_ai_summary_runs WHERE logical_owner_id=$1 AND request_id=$2 FOR UPDATE";

fn persisted_from_row(
    row: &PgRow,
) -> Result<PersistedOllamaSummaryRunV1, OllamaAiPersistenceErrorV1> {
    let request_id = id16(row.try_get("request_id").map_err(row_error)?)?;
    let status: Option<i16> = row.try_get("result_terminal_status").map_err(row_error)?;
    let result = status
        .map(|status| {
            Ok(AiProviderSummaryGenerationResultV1 {
                request_id: request_id.to_vec(),
                summary_utf8: row.try_get("result_summary_utf8").map_err(row_error)?,
                resolved_language: i32::from(
                    row.try_get::<i16, _>("result_resolved_language")
                        .map_err(row_error)?,
                ),
                resolved_length: i32::from(
                    row.try_get::<i16, _>("result_resolved_length")
                        .map_err(row_error)?,
                ),
                model_revision_sha256: row
                    .try_get::<Option<Vec<u8>>, _>("result_model_revision_sha256")
                    .map_err(row_error)?
                    .unwrap_or_default(),
                input_tokens: unsigned_i32(row.try_get("result_input_tokens").map_err(row_error)?)?,
                output_tokens: unsigned_i32(
                    row.try_get("result_output_tokens").map_err(row_error)?,
                )?,
                terminal_status: i32::from(status),
                completeness: i32::from(
                    row.try_get::<i16, _>("result_completeness")
                        .map_err(row_error)?,
                ),
                confidence_basis_points: unsigned_i32(
                    row.try_get("result_confidence_basis_points")
                        .map_err(row_error)?,
                )?,
                provider_settings_revision: unsigned_i64(
                    row.try_get("result_provider_settings_revision")
                        .map_err(row_error)?,
                )?,
            })
        })
        .transpose()?;
    let model: Option<Vec<u8>> = row
        .try_get("selected_model_revision_sha256")
        .map_err(row_error)?;
    let persisted = PersistedOllamaSummaryRunV1 {
        logical_owner_id: row.try_get("logical_owner_id").map_err(row_error)?,
        run: OllamaSummaryRunV1 {
            request_id,
            request_digest: id32(row.try_get("request_digest").map_err(row_error)?)?,
            settings_revision: positive_u64(row.try_get("settings_revision").map_err(row_error)?)?,
            selected_model_digest: model.map(id32).transpose()?,
            revision: positive_u64(row.try_get("state_revision").map_err(row_error)?)?,
            state: run_state(row.try_get("run_state").map_err(row_error)?)?,
            terminal_result: result,
        },
    };
    validate_summary_run(&persisted.run).map_err(|_| OllamaAiPersistenceErrorV1::InvalidRow)?;
    Ok(persisted)
}

fn run_state_code(state: OllamaAiRunStateV1) -> i16 {
    match state {
        OllamaAiRunStateV1::Accepted => 1,
        OllamaAiRunStateV1::Executing => 2,
        OllamaAiRunStateV1::Ready => 3,
        OllamaAiRunStateV1::Rejected => 4,
        OllamaAiRunStateV1::Uncertain => 5,
    }
}
fn run_state(value: i16) -> Result<OllamaAiRunStateV1, OllamaAiPersistenceErrorV1> {
    match value {
        1 => Ok(OllamaAiRunStateV1::Accepted),
        2 => Ok(OllamaAiRunStateV1::Executing),
        3 => Ok(OllamaAiRunStateV1::Ready),
        4 => Ok(OllamaAiRunStateV1::Rejected),
        5 => Ok(OllamaAiRunStateV1::Uncertain),
        _ => Err(OllamaAiPersistenceErrorV1::InvalidRow),
    }
}
fn signed(value: u64) -> Result<i64, OllamaAiPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| OllamaAiPersistenceErrorV1::InvalidInput)
}
fn optional_signed(value: Option<u64>) -> Result<Option<i64>, OllamaAiPersistenceErrorV1> {
    value.map(signed).transpose()
}
fn optional_i16(value: Option<i32>) -> Result<Option<i16>, OllamaAiPersistenceErrorV1> {
    value
        .map(|v| i16::try_from(v).map_err(|_| OllamaAiPersistenceErrorV1::InvalidInput))
        .transpose()
}
fn optional_i32(value: Option<u32>) -> Result<Option<i32>, OllamaAiPersistenceErrorV1> {
    value
        .map(|v| i32::try_from(v).map_err(|_| OllamaAiPersistenceErrorV1::InvalidInput))
        .transpose()
}
fn positive_u64(value: i64) -> Result<u64, OllamaAiPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|v| *v > 0)
        .ok_or(OllamaAiPersistenceErrorV1::InvalidRow)
}
fn unsigned_i32(value: i32) -> Result<u32, OllamaAiPersistenceErrorV1> {
    u32::try_from(value).map_err(|_| OllamaAiPersistenceErrorV1::InvalidRow)
}
fn unsigned_i64(value: i64) -> Result<u64, OllamaAiPersistenceErrorV1> {
    u64::try_from(value).map_err(|_| OllamaAiPersistenceErrorV1::InvalidRow)
}
fn id16(value: Vec<u8>) -> Result<[u8; 16], OllamaAiPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| OllamaAiPersistenceErrorV1::InvalidRow)
}
fn id32(value: Vec<u8>) -> Result<[u8; 32], OllamaAiPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| OllamaAiPersistenceErrorV1::InvalidRow)
}
fn storage_error(_: sqlx::Error) -> OllamaAiPersistenceErrorV1 {
    OllamaAiPersistenceErrorV1::StorageUnavailable
}
fn row_error(_: sqlx::Error) -> OllamaAiPersistenceErrorV1 {
    OllamaAiPersistenceErrorV1::InvalidRow
}
