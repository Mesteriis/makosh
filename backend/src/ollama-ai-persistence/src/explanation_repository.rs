use makosh_ai_contracts::{
    decode_provider_explanation_result_v1, encode_provider_explanation_result_v1,
};
use makosh_ollama_ai_core::{OllamaAiRunStateV1, OllamaExplanationRunV1};
use sqlx::{Row, postgres::PgRow};

use crate::{
    OllamaAiPersistenceErrorV1, OllamaAiPersistenceV1, OllamaExplanationPersistenceOutcomeV1,
    OllamaExplanationTransitionV1, PersistedOllamaExplanationRunV1,
    explanation_model::{
        validate_explanation_accepted, validate_explanation_run, validate_explanation_transition,
    },
    model::validate_owner,
};

impl OllamaAiPersistenceV1 {
    pub async fn accept_explanation_run(
        &self,
        logical_owner_id: &str,
        run: OllamaExplanationRunV1,
    ) -> Result<OllamaExplanationPersistenceOutcomeV1, OllamaAiPersistenceErrorV1> {
        validate_explanation_accepted(logical_owner_id, &run)?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.ollama_ai_explanation_runs (
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
            .load_explanation_run(logical_owner_id, run.request_id)
            .await?
            .ok_or(OllamaAiPersistenceErrorV1::InvalidRow)?;
        if !inserted
            && (persisted.run.request_digest != run.request_digest
                || persisted.run.settings_revision != run.settings_revision)
        {
            return Err(OllamaAiPersistenceErrorV1::RequestConflict);
        }
        Ok(OllamaExplanationPersistenceOutcomeV1 {
            persisted,
            replayed: !inserted,
        })
    }

    pub async fn load_explanation_run(
        &self,
        logical_owner_id: &str,
        request_id: [u8; 16],
    ) -> Result<Option<PersistedOllamaExplanationRunV1>, OllamaAiPersistenceErrorV1> {
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

    pub async fn persist_explanation_transition(
        &self,
        transition: OllamaExplanationTransitionV1,
    ) -> Result<PersistedOllamaExplanationRunV1, OllamaAiPersistenceErrorV1> {
        validate_owner(&transition.logical_owner_id)
            .then_some(())
            .ok_or(OllamaAiPersistenceErrorV1::InvalidInput)?;
        validate_explanation_run(&transition.next_run)?;
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let row = sqlx::query(SELECT_RUN_FOR_UPDATE)
            .bind(&transition.logical_owner_id)
            .bind(transition.next_run.request_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(storage_error)?
            .ok_or(OllamaAiPersistenceErrorV1::RevisionConflict)?;
        let current = persisted_from_row(&row)?;
        validate_explanation_transition(&current, &transition)?;
        let result = transition.next_run.terminal_result.as_ref();
        let result_exact_bytes = result
            .map(encode_provider_explanation_result_v1)
            .transpose()
            .map_err(|_| OllamaAiPersistenceErrorV1::InvalidInput)?;
        let updated = sqlx::query(
            "UPDATE makosh_data.ollama_ai_explanation_runs SET
               state_revision=$4, run_state=$5, selected_model_revision_sha256=$6,
               result_exact_bytes=$7, result_terminal_status=$8
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
        .bind(result_exact_bytes)
        .bind(optional_i16(result.map(|value| value.terminal_status))?)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
        if updated.rows_affected() != 1 {
            return Err(OllamaAiPersistenceErrorV1::RevisionConflict);
        }
        transaction.commit().await.map_err(storage_error)?;
        Ok(PersistedOllamaExplanationRunV1 {
            logical_owner_id: transition.logical_owner_id,
            run: transition.next_run,
        })
    }
}

const SELECT_RUN: &str = "SELECT logical_owner_id, request_id, request_digest, settings_revision,
 state_revision, run_state, selected_model_revision_sha256, result_exact_bytes,
 result_terminal_status
 FROM makosh_data.ollama_ai_explanation_runs WHERE logical_owner_id=$1 AND request_id=$2";
const SELECT_RUN_FOR_UPDATE: &str =
    "SELECT logical_owner_id, request_id, request_digest, settings_revision,
 state_revision, run_state, selected_model_revision_sha256, result_exact_bytes,
 result_terminal_status
 FROM makosh_data.ollama_ai_explanation_runs WHERE logical_owner_id=$1 AND request_id=$2 FOR UPDATE";

fn persisted_from_row(
    row: &PgRow,
) -> Result<PersistedOllamaExplanationRunV1, OllamaAiPersistenceErrorV1> {
    let request_id = id16(row.try_get("request_id").map_err(row_error)?)?;
    let status: Option<i16> = row.try_get("result_terminal_status").map_err(row_error)?;
    let result_bytes: Option<Vec<u8>> = row.try_get("result_exact_bytes").map_err(row_error)?;
    let result = match (status, result_bytes) {
        (Some(status), Some(bytes)) => {
            let result = decode_provider_explanation_result_v1(&bytes)
                .map_err(|_| OllamaAiPersistenceErrorV1::InvalidRow)?;
            (result.request_id.as_slice() == request_id.as_slice()
                && result.terminal_status == i32::from(status))
            .then_some(result)
            .ok_or(OllamaAiPersistenceErrorV1::InvalidRow)
            .map(Some)?
        }
        (None, None) => None,
        _ => return Err(OllamaAiPersistenceErrorV1::InvalidRow),
    };
    let model: Option<Vec<u8>> = row
        .try_get("selected_model_revision_sha256")
        .map_err(row_error)?;
    let persisted = PersistedOllamaExplanationRunV1 {
        logical_owner_id: row.try_get("logical_owner_id").map_err(row_error)?,
        run: OllamaExplanationRunV1 {
            request_id,
            request_digest: id32(row.try_get("request_digest").map_err(row_error)?)?,
            settings_revision: positive_u64(row.try_get("settings_revision").map_err(row_error)?)?,
            selected_model_digest: model.map(id32).transpose()?,
            revision: positive_u64(row.try_get("state_revision").map_err(row_error)?)?,
            state: run_state(row.try_get("run_state").map_err(row_error)?)?,
            terminal_result: result,
        },
    };
    validate_explanation_run(&persisted.run).map_err(|_| OllamaAiPersistenceErrorV1::InvalidRow)?;
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

fn optional_i16(value: Option<i32>) -> Result<Option<i16>, OllamaAiPersistenceErrorV1> {
    value
        .map(|item| i16::try_from(item).map_err(|_| OllamaAiPersistenceErrorV1::InvalidInput))
        .transpose()
}

fn positive_u64(value: i64) -> Result<u64, OllamaAiPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|item| *item > 0)
        .ok_or(OllamaAiPersistenceErrorV1::InvalidRow)
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
