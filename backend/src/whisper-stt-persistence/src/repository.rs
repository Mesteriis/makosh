use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool, Row,
    postgres::{PgConnectOptions, PgPoolOptions, PgRow},
};

use crate::model::{
    PersistedWhisperSttRunV1, WhisperSttPersistenceErrorV1, WhisperSttPersistenceOutcomeV1,
    WhisperSttReadyMetadataV1, WhisperSttRunIdentityV1, WhisperSttRunStateV1,
    WhisperSttTransitionV1, valid_owner, validate_accepted, validate_transition,
};

#[derive(Clone)]
pub struct WhisperSttPersistenceV1 {
    pool: PgPool,
}

impl WhisperSttPersistenceV1 {
    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, WhisperSttPersistenceErrorV1> {
        if database_id.is_empty()
            || database_id != binding.identity().database_id()
            || pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || binding.access().runtime_principal().is_empty()
        {
            return Err(WhisperSttPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| WhisperSttPersistenceErrorV1::StorageUnavailable)?;
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

    pub async fn verify_storage_ready(&self) -> Result<(), WhisperSttPersistenceErrorV1> {
        sqlx::query("SELECT 1 FROM makosh_data.whisper_stt_runs LIMIT 0")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(storage_error)
    }

    pub async fn accept_run(
        &self,
        run: PersistedWhisperSttRunV1,
    ) -> Result<WhisperSttPersistenceOutcomeV1, WhisperSttPersistenceErrorV1> {
        validate_accepted(&run)?;
        let identity = &run.identity;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.whisper_stt_runs (
               logical_owner_id, request_id, request_digest, source_reference_id,
               source_declared_bytes, source_sha256, model_revision_sha256,
               provider_settings_revision, provider_policy_revision,
               state_revision, run_state
             ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
             ON CONFLICT (logical_owner_id, request_id) DO NOTHING",
        )
        .bind(&identity.logical_owner_id)
        .bind(identity.request_id.as_slice())
        .bind(identity.request_digest.as_slice())
        .bind(identity.source_reference_id.as_slice())
        .bind(signed(identity.source_declared_bytes)?)
        .bind(identity.source_sha256.as_slice())
        .bind(identity.model_revision_sha256.as_slice())
        .bind(signed(identity.provider_settings_revision)?)
        .bind(i32::try_from(identity.provider_policy_revision).map_err(|_| invalid_input())?)
        .bind(signed(run.revision)?)
        .bind(state_code(run.state))
        .execute(&self.pool)
        .await
        .map_err(storage_error)?
        .rows_affected()
            == 1;
        let persisted = self
            .load_run(&identity.logical_owner_id, identity.request_id)
            .await?
            .ok_or(WhisperSttPersistenceErrorV1::InvalidRow)?;
        if persisted.identity != run.identity {
            return Err(WhisperSttPersistenceErrorV1::RequestConflict);
        }
        Ok(WhisperSttPersistenceOutcomeV1 {
            persisted,
            replayed: !inserted,
        })
    }

    pub async fn load_run(
        &self,
        logical_owner_id: &str,
        request_id: [u8; 16],
    ) -> Result<Option<PersistedWhisperSttRunV1>, WhisperSttPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || request_id == [0; 16] {
            return Err(WhisperSttPersistenceErrorV1::InvalidInput);
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

    pub async fn persist_transition(
        &self,
        transition: WhisperSttTransitionV1,
    ) -> Result<PersistedWhisperSttRunV1, WhisperSttPersistenceErrorV1> {
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let identity = &transition.next.identity;
        let current = sqlx::query(SELECT_RUN_FOR_UPDATE)
            .bind(&identity.logical_owner_id)
            .bind(identity.request_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(storage_error)?
            .ok_or(WhisperSttPersistenceErrorV1::RevisionConflict)
            .and_then(|row| persisted_from_row(&row))?;
        validate_transition(&current, &transition)?;
        let ready = transition.next.ready.as_ref();
        let updated = sqlx::query(
            "UPDATE makosh_data.whisper_stt_runs SET
               state_revision=$4, run_state=$5,
               transcript_reference_id=$6, transcript_declared_bytes=$7,
               transcript_sha256=$8, detected_language=$9, segment_count=$10,
               completeness=$11, confidence_basis_points=$12, reject_code=$13
             WHERE logical_owner_id=$1 AND request_id=$2 AND state_revision=$3",
        )
        .bind(&identity.logical_owner_id)
        .bind(identity.request_id.as_slice())
        .bind(signed(transition.current_revision)?)
        .bind(signed(transition.next.revision)?)
        .bind(state_code(transition.next.state))
        .bind(ready.map(|value| value.transcript_reference_id.as_slice()))
        .bind(optional_signed(
            ready.map(|value| value.transcript_declared_bytes),
        )?)
        .bind(ready.map(|value| value.transcript_sha256.as_slice()))
        .bind(optional_i16(ready.map(|value| value.detected_language))?)
        .bind(optional_i32(ready.map(|value| value.segment_count))?)
        .bind(optional_i16(ready.map(|value| value.completeness))?)
        .bind(optional_i32(
            ready.map(|value| value.confidence_basis_points),
        )?)
        .bind(optional_i16(transition.next.reject_code)?)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
        if updated.rows_affected() != 1 {
            return Err(WhisperSttPersistenceErrorV1::RevisionConflict);
        }
        transaction.commit().await.map_err(storage_error)?;
        Ok(transition.next)
    }
}

const SELECT_RUN: &str = "
 SELECT logical_owner_id, request_id, request_digest, source_reference_id,
        source_declared_bytes, source_sha256, model_revision_sha256,
        provider_settings_revision, provider_policy_revision, state_revision, run_state,
        transcript_reference_id, transcript_declared_bytes, transcript_sha256,
        detected_language, segment_count, completeness, confidence_basis_points, reject_code
 FROM makosh_data.whisper_stt_runs
 WHERE logical_owner_id=$1 AND request_id=$2";

const SELECT_RUN_FOR_UPDATE: &str = "
 SELECT logical_owner_id, request_id, request_digest, source_reference_id,
        source_declared_bytes, source_sha256, model_revision_sha256,
        provider_settings_revision, provider_policy_revision, state_revision, run_state,
        transcript_reference_id, transcript_declared_bytes, transcript_sha256,
        detected_language, segment_count, completeness, confidence_basis_points, reject_code
 FROM makosh_data.whisper_stt_runs
 WHERE logical_owner_id=$1 AND request_id=$2
 FOR UPDATE";

fn persisted_from_row(
    row: &PgRow,
) -> Result<PersistedWhisperSttRunV1, WhisperSttPersistenceErrorV1> {
    let state = state_from_code(row.try_get("run_state").map_err(row_error)?)?;
    let transcript_reference: Option<Vec<u8>> =
        row.try_get("transcript_reference_id").map_err(row_error)?;
    let ready = transcript_reference
        .map(|reference| {
            Ok(WhisperSttReadyMetadataV1 {
                transcript_reference_id: id16(reference)?,
                transcript_declared_bytes: unsigned_i64(
                    row.try_get("transcript_declared_bytes")
                        .map_err(row_error)?,
                )?,
                transcript_sha256: id32(row.try_get("transcript_sha256").map_err(row_error)?)?,
                detected_language: unsigned_i16(
                    row.try_get("detected_language").map_err(row_error)?,
                ),
                segment_count: unsigned_i32(row.try_get("segment_count").map_err(row_error)?)?,
                completeness: unsigned_i16(row.try_get("completeness").map_err(row_error)?),
                confidence_basis_points: unsigned_i32(
                    row.try_get("confidence_basis_points").map_err(row_error)?,
                )?,
            })
        })
        .transpose()?;
    let reject_code: Option<i16> = row.try_get("reject_code").map_err(row_error)?;
    let value = PersistedWhisperSttRunV1 {
        identity: WhisperSttRunIdentityV1 {
            logical_owner_id: row.try_get("logical_owner_id").map_err(row_error)?,
            request_id: id16(row.try_get("request_id").map_err(row_error)?)?,
            request_digest: id32(row.try_get("request_digest").map_err(row_error)?)?,
            source_reference_id: id16(row.try_get("source_reference_id").map_err(row_error)?)?,
            source_declared_bytes: unsigned_i64(
                row.try_get("source_declared_bytes").map_err(row_error)?,
            )?,
            source_sha256: id32(row.try_get("source_sha256").map_err(row_error)?)?,
            model_revision_sha256: id32(row.try_get("model_revision_sha256").map_err(row_error)?)?,
            provider_settings_revision: unsigned_i64(
                row.try_get("provider_settings_revision")
                    .map_err(row_error)?,
            )?,
            provider_policy_revision: unsigned_i32(
                row.try_get("provider_policy_revision").map_err(row_error)?,
            )?,
        },
        revision: positive_u64(row.try_get("state_revision").map_err(row_error)?)?,
        state,
        ready,
        reject_code: reject_code.map(unsigned_i16),
    };
    crate::model::validate_run(&value).map_err(|_| WhisperSttPersistenceErrorV1::InvalidRow)?;
    Ok(value)
}

fn state_code(value: WhisperSttRunStateV1) -> i16 {
    match value {
        WhisperSttRunStateV1::Accepted => 1,
        WhisperSttRunStateV1::Executing => 2,
        WhisperSttRunStateV1::Ready => 3,
        WhisperSttRunStateV1::Rejected => 4,
        WhisperSttRunStateV1::Uncertain => 5,
    }
}

fn state_from_code(value: i16) -> Result<WhisperSttRunStateV1, WhisperSttPersistenceErrorV1> {
    match value {
        1 => Ok(WhisperSttRunStateV1::Accepted),
        2 => Ok(WhisperSttRunStateV1::Executing),
        3 => Ok(WhisperSttRunStateV1::Ready),
        4 => Ok(WhisperSttRunStateV1::Rejected),
        5 => Ok(WhisperSttRunStateV1::Uncertain),
        _ => Err(WhisperSttPersistenceErrorV1::InvalidRow),
    }
}

fn id16(value: Vec<u8>) -> Result<[u8; 16], WhisperSttPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| WhisperSttPersistenceErrorV1::InvalidRow)
}

fn id32(value: Vec<u8>) -> Result<[u8; 32], WhisperSttPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| WhisperSttPersistenceErrorV1::InvalidRow)
}

fn signed(value: u64) -> Result<i64, WhisperSttPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| invalid_input())
}

fn optional_signed(value: Option<u64>) -> Result<Option<i64>, WhisperSttPersistenceErrorV1> {
    value.map(signed).transpose()
}

fn optional_i16(value: Option<u32>) -> Result<Option<i16>, WhisperSttPersistenceErrorV1> {
    value
        .map(|value| i16::try_from(value).map_err(|_| invalid_input()))
        .transpose()
}

fn optional_i32(value: Option<u32>) -> Result<Option<i32>, WhisperSttPersistenceErrorV1> {
    value
        .map(|value| i32::try_from(value).map_err(|_| invalid_input()))
        .transpose()
}

fn positive_u64(value: i64) -> Result<u64, WhisperSttPersistenceErrorV1> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(WhisperSttPersistenceErrorV1::InvalidRow)
}

fn unsigned_i64(value: i64) -> Result<u64, WhisperSttPersistenceErrorV1> {
    u64::try_from(value).map_err(|_| WhisperSttPersistenceErrorV1::InvalidRow)
}

fn unsigned_i32(value: i32) -> Result<u32, WhisperSttPersistenceErrorV1> {
    u32::try_from(value).map_err(|_| WhisperSttPersistenceErrorV1::InvalidRow)
}

fn unsigned_i16(value: i16) -> u32 {
    u32::from(value as u16)
}

fn invalid_input() -> WhisperSttPersistenceErrorV1 {
    WhisperSttPersistenceErrorV1::InvalidInput
}

fn row_error(_: sqlx::Error) -> WhisperSttPersistenceErrorV1 {
    WhisperSttPersistenceErrorV1::InvalidRow
}

fn storage_error(_: sqlx::Error) -> WhisperSttPersistenceErrorV1 {
    WhisperSttPersistenceErrorV1::StorageUnavailable
}
