use makosh_speech_to_text_core::{
    SpeechAudioFormatV1, SpeechLanguageV1, SpeechToTextRejectionV1, SpeechToTextRunStateV1,
    SpeechToTextRunV1, SpeechTranscriptCompletenessV1,
};
use makosh_storage_protocol::StorageBindingV1;
use sqlx::{
    PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions, PgRow},
};

use crate::model::{
    PersistedSpeechToTextRequestV1, PersistedSpeechToTextRunV1,
    PersistedSpeechTranscriptArtifactV1, SPEECH_TO_TEXT_RECOVERY_LIMIT_V1,
    SpeechToTextPersistenceErrorV1, SpeechToTextPersistenceOutcomeV1, SpeechToTextTransitionV1,
    persisted_request, request_matches, terminal_parts, valid_owner, validate_accepted,
    validate_transition,
};

#[derive(Clone)]
pub struct SpeechToTextPersistenceV1 {
    pool: PgPool,
}

impl SpeechToTextPersistenceV1 {
    async fn begin_owner_transaction(
        &self,
        logical_owner_id: &str,
    ) -> Result<Transaction<'_, Postgres>, SpeechToTextPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) {
            return Err(SpeechToTextPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        sqlx::query("SELECT set_config('makosh.logical_owner_id', $1, true)")
            .bind(logical_owner_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage_error)?;
        Ok(transaction)
    }

    pub async fn connect_runtime(
        binding: &StorageBindingV1,
        database_id: &str,
        pgbouncer_host: &str,
        pgbouncer_port: u32,
        password: &str,
    ) -> Result<Self, SpeechToTextPersistenceErrorV1> {
        if database_id.is_empty()
            || database_id != binding.identity().database_id()
            || pgbouncer_host.is_empty()
            || pgbouncer_port == 0
            || binding.access().runtime_principal().is_empty()
        {
            return Err(SpeechToTextPersistenceErrorV1::StorageUnavailable);
        }
        let port = u16::try_from(pgbouncer_port)
            .map_err(|_| SpeechToTextPersistenceErrorV1::StorageUnavailable)?;
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
            .map_err(|_| SpeechToTextPersistenceErrorV1::StorageUnavailable)?;
        Ok(Self { pool })
    }

    pub async fn verify_storage_ready(&self) -> Result<(), SpeechToTextPersistenceErrorV1> {
        sqlx::query("SELECT 1 FROM makosh_data.speech_to_text_runs LIMIT 0")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(storage_error)
    }

    pub async fn accept_run(
        &self,
        run: SpeechToTextRunV1,
    ) -> Result<SpeechToTextPersistenceOutcomeV1, SpeechToTextPersistenceErrorV1> {
        validate_accepted(&run)?;
        let request = &run.request;
        let mut transaction = self
            .begin_owner_transaction(&request.logical_owner_id)
            .await?;
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.speech_to_text_runs (
               logical_owner_id, request_id, request_digest, source_reference_id,
               source_declared_bytes, source_sha256, audio_format, duration_millis,
               requested_language, consent_receipt_id, consent_policy_revision,
               maximum_transcript_bytes, maximum_segments, state_revision, run_state
             ) VALUES (
               $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
             ) ON CONFLICT (logical_owner_id, request_id) DO NOTHING",
        )
        .bind(&request.logical_owner_id)
        .bind(request.request_id.as_slice())
        .bind(request.request_digest.as_slice())
        .bind(request.source.reference_id.as_slice())
        .bind(signed(request.source.declared_bytes)?)
        .bind(request.source.sha256.as_slice())
        .bind(audio_format_code(request.audio_format))
        .bind(signed(request.duration_millis)?)
        .bind(language_code(request.requested_language))
        .bind(request.consent_receipt_id.as_slice())
        .bind(i32::try_from(request.consent_policy_revision).map_err(|_| invalid_input())?)
        .bind(i32::try_from(request.maximum_transcript_bytes).map_err(|_| invalid_input())?)
        .bind(i32::try_from(request.maximum_segments).map_err(|_| invalid_input())?)
        .bind(signed(run.revision)?)
        .bind(run_state_code(run.state))
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?
        .rows_affected()
            == 1;
        let persisted = load_run_in_transaction(
            &mut transaction,
            &request.logical_owner_id,
            request.request_id,
        )
        .await?
        .ok_or(SpeechToTextPersistenceErrorV1::InvalidRow)?;
        if !request_matches(&persisted.request, request) {
            return Err(SpeechToTextPersistenceErrorV1::RequestConflict);
        }
        transaction.commit().await.map_err(storage_error)?;
        Ok(SpeechToTextPersistenceOutcomeV1 {
            persisted,
            replayed: !inserted,
        })
    }

    pub async fn load_run(
        &self,
        logical_owner_id: &str,
        request_id: [u8; 16],
    ) -> Result<Option<PersistedSpeechToTextRunV1>, SpeechToTextPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || request_id == [0; 16] {
            return Err(SpeechToTextPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let persisted =
            load_run_in_transaction(&mut transaction, logical_owner_id, request_id).await?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(persisted)
    }

    pub async fn load_recoverable_runs(
        &self,
        logical_owner_id: &str,
        limit: u32,
    ) -> Result<Vec<PersistedSpeechToTextRunV1>, SpeechToTextPersistenceErrorV1> {
        if !valid_owner(logical_owner_id)
            || !(1..=SPEECH_TO_TEXT_RECOVERY_LIMIT_V1).contains(&limit)
        {
            return Err(SpeechToTextPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.begin_owner_transaction(logical_owner_id).await?;
        let persisted = sqlx::query(SELECT_RECOVERABLE)
            .bind(logical_owner_id)
            .bind(i64::from(limit))
            .fetch_all(&mut *transaction)
            .await
            .map_err(storage_error)?
            .iter()
            .map(persisted_from_row)
            .collect::<Result<Vec<_>, _>>()?;
        transaction.commit().await.map_err(storage_error)?;
        Ok(persisted)
    }

    pub async fn persist_transition(
        &self,
        transition: SpeechToTextTransitionV1,
    ) -> Result<PersistedSpeechToTextRunV1, SpeechToTextPersistenceErrorV1> {
        let request = &transition.next_run.request;
        let mut transaction = self
            .begin_owner_transaction(&request.logical_owner_id)
            .await?;
        let current_row = sqlx::query(SELECT_RUN_FOR_UPDATE)
            .bind(&request.logical_owner_id)
            .bind(request.request_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(storage_error)?
            .ok_or(SpeechToTextPersistenceErrorV1::RevisionConflict)?;
        let current = persisted_from_row(&current_row)?;
        validate_transition(&current, &transition)?;
        let (artifact, rejection) = terminal_parts(&transition.next_run)?;
        let updated = sqlx::query(
            "UPDATE makosh_data.speech_to_text_runs SET
               state_revision = $4, run_state = $5,
               transcript_reference_id = $6, transcript_declared_bytes = $7,
               transcript_sha256 = $8, detected_language = $9, segment_count = $10,
               completeness = $11, confidence_basis_points = $12,
               provider_contract_schema_sha256 = $13, model_revision_sha256 = $14,
               provider_settings_revision = $15, provider_policy_revision = $16,
               rejection_code = $17
             WHERE logical_owner_id = $1 AND request_id = $2 AND state_revision = $3",
        )
        .bind(&request.logical_owner_id)
        .bind(request.request_id.as_slice())
        .bind(signed(transition.current_revision)?)
        .bind(signed(transition.next_run.revision)?)
        .bind(run_state_code(transition.next_run.state))
        .bind(artifact.as_ref().map(|value| value.reference_id.as_slice()))
        .bind(optional_signed(
            artifact.as_ref().map(|value| value.declared_bytes),
        )?)
        .bind(artifact.as_ref().map(|value| value.sha256.as_slice()))
        .bind(
            artifact
                .as_ref()
                .map(|value| language_code(value.detected_language)),
        )
        .bind(optional_i32(
            artifact.as_ref().map(|value| value.segment_count),
        )?)
        .bind(
            artifact
                .as_ref()
                .map(|value| completeness_code(value.completeness)),
        )
        .bind(optional_i32(
            artifact.as_ref().map(|value| value.confidence_basis_points),
        )?)
        .bind(
            artifact
                .as_ref()
                .map(|value| value.provider_contract_schema_sha256.as_slice()),
        )
        .bind(
            artifact
                .as_ref()
                .map(|value| value.model_revision_sha256.as_slice()),
        )
        .bind(optional_signed(
            artifact
                .as_ref()
                .map(|value| value.provider_settings_revision),
        )?)
        .bind(optional_i32(
            artifact
                .as_ref()
                .map(|value| value.provider_policy_revision),
        )?)
        .bind(rejection.map(rejection_code))
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
        if updated.rows_affected() != 1 {
            return Err(SpeechToTextPersistenceErrorV1::RevisionConflict);
        }
        transaction.commit().await.map_err(storage_error)?;
        Ok(PersistedSpeechToTextRunV1 {
            request: persisted_request(request),
            state: transition.next_run.state,
            revision: transition.next_run.revision,
            artifact,
            rejection,
        })
    }
}

#[cfg(test)]
const SELECT_COLUMNS: &str = "
 logical_owner_id, request_id, request_digest, source_reference_id,
 source_declared_bytes, source_sha256, audio_format, duration_millis,
 requested_language, consent_receipt_id, consent_policy_revision,
 maximum_transcript_bytes, maximum_segments, state_revision, run_state,
 transcript_reference_id, transcript_declared_bytes, transcript_sha256,
 detected_language, segment_count, completeness, confidence_basis_points,
 provider_contract_schema_sha256, model_revision_sha256,
 provider_settings_revision, provider_policy_revision, rejection_code";

const SELECT_RUN: &str = concat!(
    "SELECT ",
    "logical_owner_id, request_id, request_digest, source_reference_id, source_declared_bytes, source_sha256, audio_format, duration_millis, requested_language, consent_receipt_id, consent_policy_revision, maximum_transcript_bytes, maximum_segments, state_revision, run_state, transcript_reference_id, transcript_declared_bytes, transcript_sha256, detected_language, segment_count, completeness, confidence_basis_points, provider_contract_schema_sha256, model_revision_sha256, provider_settings_revision, provider_policy_revision, rejection_code ",
    "FROM makosh_data.speech_to_text_runs WHERE logical_owner_id = $1 AND request_id = $2"
);
const SELECT_RUN_FOR_UPDATE: &str = concat!(
    "SELECT ",
    "logical_owner_id, request_id, request_digest, source_reference_id, source_declared_bytes, source_sha256, audio_format, duration_millis, requested_language, consent_receipt_id, consent_policy_revision, maximum_transcript_bytes, maximum_segments, state_revision, run_state, transcript_reference_id, transcript_declared_bytes, transcript_sha256, detected_language, segment_count, completeness, confidence_basis_points, provider_contract_schema_sha256, model_revision_sha256, provider_settings_revision, provider_policy_revision, rejection_code ",
    "FROM makosh_data.speech_to_text_runs WHERE logical_owner_id = $1 AND request_id = $2 FOR UPDATE"
);
const SELECT_RECOVERABLE: &str = concat!(
    "SELECT ",
    "logical_owner_id, request_id, request_digest, source_reference_id, source_declared_bytes, source_sha256, audio_format, duration_millis, requested_language, consent_receipt_id, consent_policy_revision, maximum_transcript_bytes, maximum_segments, state_revision, run_state, transcript_reference_id, transcript_declared_bytes, transcript_sha256, detected_language, segment_count, completeness, confidence_basis_points, provider_contract_schema_sha256, model_revision_sha256, provider_settings_revision, provider_policy_revision, rejection_code ",
    "FROM makosh_data.speech_to_text_runs WHERE logical_owner_id = $1 AND run_state IN (1, 2) ORDER BY state_revision, request_id LIMIT $2"
);

async fn load_run_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    logical_owner_id: &str,
    request_id: [u8; 16],
) -> Result<Option<PersistedSpeechToTextRunV1>, SpeechToTextPersistenceErrorV1> {
    sqlx::query(SELECT_RUN)
        .bind(logical_owner_id)
        .bind(request_id.as_slice())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_error)?
        .map(|row| persisted_from_row(&row))
        .transpose()
}

fn persisted_from_row(
    row: &PgRow,
) -> Result<PersistedSpeechToTextRunV1, SpeechToTextPersistenceErrorV1> {
    let state = run_state_from_code(row.try_get("run_state").map_err(row_error)?)?;
    let artifact = if state == SpeechToTextRunStateV1::Ready {
        Some(PersistedSpeechTranscriptArtifactV1 {
            reference_id: required_id16(row, "transcript_reference_id")?,
            declared_bytes: required_unsigned(row, "transcript_declared_bytes")?,
            sha256: required_id32(row, "transcript_sha256")?,
            detected_language: language_from_code(required_i16(row, "detected_language")?)?,
            segment_count: required_u32(row, "segment_count")?,
            completeness: completeness_from_code(required_i16(row, "completeness")?)?,
            confidence_basis_points: required_u32(row, "confidence_basis_points")?,
            provider_contract_schema_sha256: required_id32(row, "provider_contract_schema_sha256")?,
            model_revision_sha256: required_id32(row, "model_revision_sha256")?,
            provider_settings_revision: required_unsigned(row, "provider_settings_revision")?,
            provider_policy_revision: required_u32(row, "provider_policy_revision")?,
        })
    } else {
        None
    };
    let rejection = if state == SpeechToTextRunStateV1::Rejected {
        Some(rejection_from_code(required_i16(row, "rejection_code")?)?)
    } else {
        None
    };
    if matches!(
        state,
        SpeechToTextRunStateV1::Accepted | SpeechToTextRunStateV1::Executing
    ) && (row
        .try_get::<Option<Vec<u8>>, _>("transcript_reference_id")
        .map_err(row_error)?
        .is_some()
        || row
            .try_get::<Option<i16>, _>("rejection_code")
            .map_err(row_error)?
            .is_some())
    {
        return Err(SpeechToTextPersistenceErrorV1::InvalidRow);
    }
    Ok(PersistedSpeechToTextRunV1 {
        request: PersistedSpeechToTextRequestV1 {
            request_id: id16(row.try_get("request_id").map_err(row_error)?)?,
            logical_owner_id: row.try_get("logical_owner_id").map_err(row_error)?,
            source_reference_id: id16(row.try_get("source_reference_id").map_err(row_error)?)?,
            source_declared_bytes: unsigned(
                row.try_get("source_declared_bytes").map_err(row_error)?,
            )?,
            source_sha256: id32(row.try_get("source_sha256").map_err(row_error)?)?,
            audio_format: audio_format_from_code(row.try_get("audio_format").map_err(row_error)?)?,
            duration_millis: unsigned(row.try_get("duration_millis").map_err(row_error)?)?,
            requested_language: language_from_code(
                row.try_get("requested_language").map_err(row_error)?,
            )?,
            consent_receipt_id: id16(row.try_get("consent_receipt_id").map_err(row_error)?)?,
            consent_policy_revision: positive_u32(
                row.try_get("consent_policy_revision").map_err(row_error)?,
            )?,
            maximum_transcript_bytes: positive_u32(
                row.try_get("maximum_transcript_bytes").map_err(row_error)?,
            )?,
            maximum_segments: positive_u32(row.try_get("maximum_segments").map_err(row_error)?)?,
            request_digest: id32(row.try_get("request_digest").map_err(row_error)?)?,
        },
        state,
        revision: unsigned(row.try_get("state_revision").map_err(row_error)?)?,
        artifact,
        rejection,
    })
}

fn required_id16(row: &PgRow, name: &str) -> Result<[u8; 16], SpeechToTextPersistenceErrorV1> {
    id16(
        row.try_get::<Option<Vec<u8>>, _>(name)
            .map_err(row_error)?
            .ok_or(SpeechToTextPersistenceErrorV1::InvalidRow)?,
    )
}
fn required_id32(row: &PgRow, name: &str) -> Result<[u8; 32], SpeechToTextPersistenceErrorV1> {
    id32(
        row.try_get::<Option<Vec<u8>>, _>(name)
            .map_err(row_error)?
            .ok_or(SpeechToTextPersistenceErrorV1::InvalidRow)?,
    )
}
fn required_i16(row: &PgRow, name: &str) -> Result<i16, SpeechToTextPersistenceErrorV1> {
    row.try_get::<Option<i16>, _>(name)
        .map_err(row_error)?
        .ok_or(SpeechToTextPersistenceErrorV1::InvalidRow)
}
fn required_u32(row: &PgRow, name: &str) -> Result<u32, SpeechToTextPersistenceErrorV1> {
    positive_or_zero_u32(
        row.try_get::<Option<i32>, _>(name)
            .map_err(row_error)?
            .ok_or(SpeechToTextPersistenceErrorV1::InvalidRow)?,
    )
}
fn required_unsigned(row: &PgRow, name: &str) -> Result<u64, SpeechToTextPersistenceErrorV1> {
    unsigned(
        row.try_get::<Option<i64>, _>(name)
            .map_err(row_error)?
            .ok_or(SpeechToTextPersistenceErrorV1::InvalidRow)?,
    )
}

fn audio_format_code(value: SpeechAudioFormatV1) -> i16 {
    match value {
        SpeechAudioFormatV1::WavPcmS16LeMono16Khz => 1,
    }
}
fn audio_format_from_code(
    value: i16,
) -> Result<SpeechAudioFormatV1, SpeechToTextPersistenceErrorV1> {
    match value {
        1 => Ok(SpeechAudioFormatV1::WavPcmS16LeMono16Khz),
        _ => Err(SpeechToTextPersistenceErrorV1::InvalidRow),
    }
}
fn language_code(value: SpeechLanguageV1) -> i16 {
    match value {
        SpeechLanguageV1::Auto => 1,
        SpeechLanguageV1::English => 2,
        SpeechLanguageV1::Russian => 3,
        SpeechLanguageV1::Spanish => 4,
    }
}
fn language_from_code(value: i16) -> Result<SpeechLanguageV1, SpeechToTextPersistenceErrorV1> {
    match value {
        1 => Ok(SpeechLanguageV1::Auto),
        2 => Ok(SpeechLanguageV1::English),
        3 => Ok(SpeechLanguageV1::Russian),
        4 => Ok(SpeechLanguageV1::Spanish),
        _ => Err(SpeechToTextPersistenceErrorV1::InvalidRow),
    }
}
fn run_state_code(value: SpeechToTextRunStateV1) -> i16 {
    match value {
        SpeechToTextRunStateV1::Accepted => 1,
        SpeechToTextRunStateV1::Executing => 2,
        SpeechToTextRunStateV1::Ready => 3,
        SpeechToTextRunStateV1::Rejected => 4,
    }
}
fn run_state_from_code(
    value: i16,
) -> Result<SpeechToTextRunStateV1, SpeechToTextPersistenceErrorV1> {
    match value {
        1 => Ok(SpeechToTextRunStateV1::Accepted),
        2 => Ok(SpeechToTextRunStateV1::Executing),
        3 => Ok(SpeechToTextRunStateV1::Ready),
        4 => Ok(SpeechToTextRunStateV1::Rejected),
        _ => Err(SpeechToTextPersistenceErrorV1::InvalidRow),
    }
}
fn completeness_code(value: SpeechTranscriptCompletenessV1) -> i16 {
    match value {
        SpeechTranscriptCompletenessV1::Complete => 1,
        SpeechTranscriptCompletenessV1::Partial => 2,
    }
}
fn completeness_from_code(
    value: i16,
) -> Result<SpeechTranscriptCompletenessV1, SpeechToTextPersistenceErrorV1> {
    match value {
        1 => Ok(SpeechTranscriptCompletenessV1::Complete),
        2 => Ok(SpeechTranscriptCompletenessV1::Partial),
        _ => Err(SpeechToTextPersistenceErrorV1::InvalidRow),
    }
}
fn rejection_code(value: SpeechToTextRejectionV1) -> i16 {
    match value {
        SpeechToTextRejectionV1::InvalidRequest => 1,
        SpeechToTextRejectionV1::ConsentRejected => 2,
        SpeechToTextRejectionV1::UnsupportedAudio => 3,
        SpeechToTextRejectionV1::ProviderUnavailable => 4,
        SpeechToTextRejectionV1::ProviderRejected => 5,
        SpeechToTextRejectionV1::Policy => 6,
    }
}
fn rejection_from_code(
    value: i16,
) -> Result<SpeechToTextRejectionV1, SpeechToTextPersistenceErrorV1> {
    match value {
        1 => Ok(SpeechToTextRejectionV1::InvalidRequest),
        2 => Ok(SpeechToTextRejectionV1::ConsentRejected),
        3 => Ok(SpeechToTextRejectionV1::UnsupportedAudio),
        4 => Ok(SpeechToTextRejectionV1::ProviderUnavailable),
        5 => Ok(SpeechToTextRejectionV1::ProviderRejected),
        6 => Ok(SpeechToTextRejectionV1::Policy),
        _ => Err(SpeechToTextPersistenceErrorV1::InvalidRow),
    }
}

fn signed(value: u64) -> Result<i64, SpeechToTextPersistenceErrorV1> {
    i64::try_from(value).map_err(|_| invalid_input())
}
fn optional_signed(value: Option<u64>) -> Result<Option<i64>, SpeechToTextPersistenceErrorV1> {
    value.map(signed).transpose()
}
fn optional_i32(value: Option<u32>) -> Result<Option<i32>, SpeechToTextPersistenceErrorV1> {
    value
        .map(|value| i32::try_from(value).map_err(|_| invalid_input()))
        .transpose()
}
fn unsigned(value: i64) -> Result<u64, SpeechToTextPersistenceErrorV1> {
    u64::try_from(value).map_err(|_| SpeechToTextPersistenceErrorV1::InvalidRow)
}
fn positive_u32(value: i32) -> Result<u32, SpeechToTextPersistenceErrorV1> {
    let value = positive_or_zero_u32(value)?;
    if value == 0 {
        return Err(SpeechToTextPersistenceErrorV1::InvalidRow);
    }
    Ok(value)
}
fn positive_or_zero_u32(value: i32) -> Result<u32, SpeechToTextPersistenceErrorV1> {
    u32::try_from(value).map_err(|_| SpeechToTextPersistenceErrorV1::InvalidRow)
}
fn id16(value: Vec<u8>) -> Result<[u8; 16], SpeechToTextPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| SpeechToTextPersistenceErrorV1::InvalidRow)
}
fn id32(value: Vec<u8>) -> Result<[u8; 32], SpeechToTextPersistenceErrorV1> {
    value
        .try_into()
        .map_err(|_| SpeechToTextPersistenceErrorV1::InvalidRow)
}
fn invalid_input() -> SpeechToTextPersistenceErrorV1 {
    SpeechToTextPersistenceErrorV1::InvalidInput
}
fn storage_error(_: sqlx::Error) -> SpeechToTextPersistenceErrorV1 {
    SpeechToTextPersistenceErrorV1::StorageUnavailable
}
fn row_error(_: sqlx::Error) -> SpeechToTextPersistenceErrorV1 {
    SpeechToTextPersistenceErrorV1::InvalidRow
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enum_codes_are_exact_and_provider_neutral() {
        assert_eq!(language_code(SpeechLanguageV1::Auto), 1);
        assert_eq!(run_state_from_code(4), Ok(SpeechToTextRunStateV1::Rejected));
        assert_eq!(
            rejection_from_code(rejection_code(SpeechToTextRejectionV1::ProviderRejected)),
            Ok(SpeechToTextRejectionV1::ProviderRejected)
        );
    }

    #[test]
    fn select_surface_never_contains_private_payload_or_custody_proof() {
        for forbidden in [
            "audio_bytes",
            "transcript_text",
            "custody_proof",
            "provider_name",
        ] {
            assert!(!SELECT_COLUMNS.contains(forbidden), "forbidden {forbidden}");
        }
    }
}
